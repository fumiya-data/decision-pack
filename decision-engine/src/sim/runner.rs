use chrono::Utc;

use crate::domain::date::Date;
use crate::domain::kpi::KpiSummary;
use crate::domain::money::{Qty, Yen};
use crate::domain::{CashEvent, CashState};
use crate::report::types::{
    Alert, AlertCode, AlertSeverity, DailyCashPoint, DailyInventoryPoint, KpiReport, ScenarioInfo,
    SimulationReport,
};
use crate::sim::cashflow::cash_one_day;
use crate::sim::inventory::inventory_one_day;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemRunInput {
    pub item_id: String,
    pub opening_on_hand: Qty,
    pub opening_on_order: Qty,
    pub demand_by_day: Vec<Qty>,
    pub arrivals_by_day: Vec<Qty>,
    pub reorder_point: Qty,
    pub order_up_to: Qty,
    pub moq: Qty,
    pub lot_size: Qty,
    pub lead_time_days: Date,
    pub sales_unit_price: Yen,
    pub purchase_unit_cost: Yen,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioRunInput {
    pub scenario_id: String,
    pub scenario_name: String,
    pub scenario_description: Option<String>,
    pub currency: String,
    pub initial_cash: Yen,
    pub days: Vec<String>,
    pub items: Vec<ItemRunInput>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ItemRunSummary {
    pub item_id: String,
    pub total_stockout_qty: Qty,
    pub stockout_rate: f64,
    pub avg_on_hand: f64,
    pub recommended_reorder_qty: Qty,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScenarioRunOutput {
    pub report: SimulationReport,
    pub item_summaries: Vec<ItemRunSummary>,
}

pub fn run_scenario(input: &ScenarioRunInput) -> ScenarioRunOutput {
    let horizon_days = input.days.len();
    let mut alerts = Vec::new();
    let mut cash_series = Vec::with_capacity(horizon_days);
    let mut inventory_series = Vec::with_capacity(horizon_days.saturating_mul(input.items.len()));
    let mut cash_state = CashState {
        cash: input.initial_cash,
    };
    let mut kpi = KpiSummary::default();
    let mut first_cash_shortfall_date = None;

    let mut items = input
        .items
        .iter()
        .map(|item| ItemRuntime::new(item, horizon_days))
        .collect::<Vec<_>>();

    for (day_idx, date) in input.days.iter().enumerate() {
        let today = day_idx as Date;
        let mut inflow: Yen = 0;
        let mut outflow: Yen = 0;

        for item in &mut items {
            let day = item.step(today, horizon_days);
            inflow += day.inflow;
            outflow += day.outflow;

            if day.stockout > 0 {
                alerts.push(Alert {
                    code: AlertCode::StockoutRateBreach,
                    severity: AlertSeverity::Warn,
                    date: date.clone(),
                    item_id: Some(item.input.item_id.clone()),
                    message: format!(
                        "{} で {} 個の欠品が発生しました",
                        item.input.item_id, day.stockout
                    ),
                });
            }

            inventory_series.push(DailyInventoryPoint {
                date: date.clone(),
                item_id: item.input.item_id.clone(),
                on_hand: item.on_hand,
                on_order: item.on_order,
                demand: day.demand,
                sold: day.sold,
                stockout: day.stockout,
            });
            item.total_stockout = item.total_stockout.saturating_add(day.stockout);
            item.total_demand = item.total_demand.saturating_add(day.demand);
            item.total_on_hand += item.on_hand as f64;
            item.total_reorder = item.total_reorder.saturating_add(day.reorder_qty);
        }

        cash_state = cash_one_day(
            today,
            cash_state,
            &[
                CashEvent {
                    due: today,
                    amount: inflow,
                    category: "sales".to_string(),
                },
                CashEvent {
                    due: today,
                    amount: -outflow,
                    category: "purchase".to_string(),
                },
            ],
        );
        kpi.observe_cash(cash_state.cash);
        if cash_state.cash < 0 && first_cash_shortfall_date.is_none() {
            first_cash_shortfall_date = Some(date.clone());
        }

        cash_series.push(DailyCashPoint {
            date: date.clone(),
            cash: cash_state.cash,
            inflow,
            outflow,
        });
    }

    let item_summaries = items
        .iter()
        .map(|item| ItemRunSummary {
            item_id: item.input.item_id.clone(),
            total_stockout_qty: item.total_stockout,
            stockout_rate: if item.total_demand == 0 {
                0.0
            } else {
                item.total_stockout as f64 / item.total_demand as f64
            },
            avg_on_hand: if horizon_days == 0 {
                0.0
            } else {
                item.total_on_hand / horizon_days as f64
            },
            recommended_reorder_qty: item.total_reorder,
        })
        .collect::<Vec<_>>();

    for item in &item_summaries {
        kpi.add_stockout(item.total_stockout_qty);
    }

    let total_demand = items.iter().map(|item| item.total_demand).sum::<Qty>();
    let stockout_rate = if total_demand == 0 {
        0.0
    } else {
        kpi.total_stockout as f64 / total_demand as f64
    };
    if stockout_rate > 0.05 {
        alerts.push(Alert {
            code: AlertCode::StockoutRateBreach,
            severity: AlertSeverity::Warn,
            date: input.days.last().cloned().unwrap_or_default(),
            item_id: None,
            message: format!(
                "全体の欠品率が閾値を超過しました: {:.2}%",
                stockout_rate * 100.0
            ),
        });
    }
    if let Some(date) = &first_cash_shortfall_date {
        alerts.push(Alert {
            code: AlertCode::CashShortfall,
            severity: AlertSeverity::Critical,
            date: date.clone(),
            item_id: None,
            message: "資金残高がマイナスになりました".to_string(),
        });
    }

    ScenarioRunOutput {
        report: SimulationReport {
            schema_version: "v0.1".to_string(),
            generated_at: Utc::now().to_rfc3339(),
            scenario: ScenarioInfo {
                id: input.scenario_id.clone(),
                name: input.scenario_name.clone(),
                description: input.scenario_description.clone(),
            },
            horizon_days: horizon_days as u32,
            currency: input.currency.clone(),
            kpi: KpiReport {
                min_cash: if kpi.min_cash == i64::MAX {
                    input.initial_cash
                } else {
                    kpi.min_cash
                },
                first_cash_shortfall_date,
                total_stockout_qty: kpi.total_stockout,
                stockout_rate,
                days_on_hand_avg: if inventory_series.is_empty() {
                    0.0
                } else {
                    inventory_series
                        .iter()
                        .map(|row| row.on_hand as f64)
                        .sum::<f64>()
                        / inventory_series.len() as f64
                },
            },
            alerts,
            cash_series,
            inventory_series,
        },
        item_summaries,
    }
}

#[derive(Debug, Clone)]
struct ItemRuntime<'a> {
    input: &'a ItemRunInput,
    on_hand: Qty,
    on_order: Qty,
    future_arrivals: Vec<Qty>,
    total_stockout: Qty,
    total_demand: Qty,
    total_on_hand: f64,
    total_reorder: Qty,
}

impl<'a> ItemRuntime<'a> {
    fn new(input: &'a ItemRunInput, horizon_days: usize) -> Self {
        let mut future_arrivals = vec![0; horizon_days];
        for (idx, qty) in input
            .arrivals_by_day
            .iter()
            .copied()
            .enumerate()
            .take(horizon_days)
        {
            future_arrivals[idx] = qty;
        }

        Self {
            input,
            on_hand: input.opening_on_hand,
            on_order: input.opening_on_order,
            future_arrivals,
            total_stockout: 0,
            total_demand: 0,
            total_on_hand: 0.0,
            total_reorder: 0,
        }
    }

    fn step(&mut self, today: Date, horizon_days: usize) -> ItemDayResult {
        let idx = today as usize;
        let arrivals = self.future_arrivals.get(idx).copied().unwrap_or(0);
        self.on_order = self.on_order.saturating_sub(arrivals);
        let demand = self.input.demand_by_day.get(idx).copied().unwrap_or(0);
        let day = inventory_one_day(self.on_hand, arrivals, demand);
        self.on_hand = day.next_on_hand;

        let mut reorder_qty = 0;
        if self.on_hand <= self.input.reorder_point {
            let target = self.input.order_up_to.max(self.input.reorder_point);
            reorder_qty = apply_replenishment_constraints(
                target.saturating_sub(self.on_hand),
                self.input.moq,
                self.input.lot_size,
            );
            if reorder_qty > 0 {
                self.on_order = self.on_order.saturating_add(reorder_qty);
                let due_idx = idx.saturating_add(self.input.lead_time_days as usize);
                if due_idx < horizon_days {
                    self.future_arrivals[due_idx] =
                        self.future_arrivals[due_idx].saturating_add(reorder_qty);
                }
            }
        }

        ItemDayResult {
            demand,
            sold: day.sold,
            stockout: day.stockout,
            inflow: day.sold as Yen * self.input.sales_unit_price,
            outflow: reorder_qty as Yen * self.input.purchase_unit_cost,
            reorder_qty,
        }
    }
}

fn apply_replenishment_constraints(base_qty: Qty, moq: Qty, lot_size: Qty) -> Qty {
    if base_qty == 0 {
        return 0;
    }

    let constrained = base_qty.max(moq);
    if lot_size == 0 {
        return constrained;
    }

    let remainder = constrained % lot_size;
    if remainder == 0 {
        constrained
    } else {
        constrained.saturating_add(lot_size - remainder)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ItemDayResult {
    demand: Qty,
    sold: Qty,
    stockout: Qty,
    inflow: Yen,
    outflow: Yen,
    reorder_qty: Qty,
}

#[cfg(test)]
mod tests {
    use super::{ItemRunInput, ScenarioRunInput, run_scenario};

    #[test]
    fn run_scenario_builds_report_and_item_summaries() {
        let output = run_scenario(&ScenarioRunInput {
            scenario_id: "baseline".into(),
            scenario_name: "ベースライン".into(),
            scenario_description: Some("単体テスト".into()),
            currency: "JPY".into(),
            initial_cash: 100_000,
            days: vec![
                "2026-04-01".into(),
                "2026-04-02".into(),
                "2026-04-03".into(),
            ],
            items: vec![ItemRunInput {
                item_id: "A001".into(),
                opening_on_hand: 3,
                opening_on_order: 0,
                demand_by_day: vec![2, 2, 2],
                arrivals_by_day: vec![0, 0, 0],
                reorder_point: 1,
                order_up_to: 5,
                moq: 0,
                lot_size: 0,
                lead_time_days: 1,
                sales_unit_price: 1000,
                purchase_unit_cost: 500,
            }],
        });

        assert_eq!(output.report.horizon_days, 3);
        assert_eq!(output.report.inventory_series.len(), 3);
        assert_eq!(output.item_summaries.len(), 1);
        assert!(output.report.kpi.total_stockout_qty <= 6);
    }

    #[test]
    fn run_scenario_applies_moq_and_lot_size_to_replenishment() {
        let output = run_scenario(&ScenarioRunInput {
            scenario_id: "constraints".into(),
            scenario_name: "補充制約".into(),
            scenario_description: None,
            currency: "JPY".into(),
            initial_cash: 100_000,
            days: vec!["2026-04-01".into(), "2026-04-02".into()],
            items: vec![ItemRunInput {
                item_id: "A001".into(),
                opening_on_hand: 1,
                opening_on_order: 0,
                demand_by_day: vec![1, 0],
                arrivals_by_day: vec![0, 0],
                reorder_point: 1,
                order_up_to: 7,
                moq: 8,
                lot_size: 5,
                lead_time_days: 1,
                sales_unit_price: 1000,
                purchase_unit_cost: 100,
            }],
        });

        assert_eq!(output.item_summaries[0].recommended_reorder_qty, 10);
        assert_eq!(output.report.cash_series[0].outflow, 1000);
    }

    #[test]
    fn replenishment_constraints_leave_zero_order_at_zero() {
        assert_eq!(super::apply_replenishment_constraints(0, 10, 5), 0);
    }

    #[test]
    fn replenishment_constraints_apply_moq_without_lot_size() {
        assert_eq!(super::apply_replenishment_constraints(3, 8, 0), 8);
    }
}
