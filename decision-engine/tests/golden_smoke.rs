use decision_engine::domain::{CashEvent, CashState, InventoryState, ItemPolicy};
use decision_engine::report::json::to_json;
use decision_engine::report::types::{
    Alert, AlertCode, AlertSeverity, DailyCashPoint, DailyInventoryPoint, KpiReport, ScenarioInfo,
    SimulationReport,
};
use decision_engine::sim::cashflow::{cash_additive_holds, cash_one_day};
use decision_engine::sim::inventory::{inv_conservation_holds, inventory_one_day, inventory_step_spec};

#[test]
fn inventory_one_day_matches_spec_behavior() {
    let r = inventory_one_day(10, 5, 20);
    assert_eq!(r.sold, 15);
    assert_eq!(r.stockout, 5);
    assert_eq!(r.next_on_hand, 0);
    assert!(inv_conservation_holds(10, 5, 20));
}

#[test]
fn inventory_step_spec_returns_stockout() {
    let st = InventoryState {
        on_hand: 3,
        on_order: 9,
    };
    let policy = ItemPolicy {
        item_id: "A001".to_string(),
        reorder_point: 5,
        order_up_to: 12,
        lead_time_days: 2,
    };
    let out = inventory_step_spec(0, st, 8, 1, &policy);
    assert_eq!(out.next.on_hand, 0);
    assert_eq!(out.next.on_order, 9);
    assert_eq!(out.stockout, 4);
    assert!(out.new_orders.is_empty());
}

#[test]
fn cashflow_additivity_holds() {
    let st = CashState { cash: 100 };
    let today = 7;
    let a = vec![
        CashEvent {
            due: today,
            amount: 20,
            category: "sales".to_string(),
        },
        CashEvent {
            due: today + 1,
            amount: 1000,
            category: "ignore".to_string(),
        },
    ];
    let b = vec![CashEvent {
        due: today,
        amount: -35,
        category: "purchase".to_string(),
    }];

    let one_day = cash_one_day(today, st, &a);
    assert_eq!(one_day.cash, 120);
    assert!(cash_additive_holds(today, st, &a, &b));
}

#[test]
fn report_json_matches_v01_shape() {
    let report = SimulationReport {
        schema_version: "v0.1".to_string(),
        generated_at: "2026-03-05T13:30:00+09:00".to_string(),
        scenario: ScenarioInfo {
            id: "baseline".to_string(),
            name: "ベースライン".to_string(),
            description: Some("テスト用シナリオ".to_string()),
        },
        horizon_days: 7,
        currency: "JPY".to_string(),
        kpi: KpiReport {
            min_cash: 920000,
            first_cash_shortfall_date: None,
            total_stockout_qty: 6,
            stockout_rate: 0.021,
            days_on_hand_avg: 12.4,
        },
        alerts: vec![Alert {
            code: AlertCode::StockoutRateBreach,
            severity: AlertSeverity::Warn,
            date: "2026-03-08".to_string(),
            item_id: Some("A001".to_string()),
            message: "欠品率の閾値を超過しました".to_string(),
        }],
        cash_series: vec![DailyCashPoint {
            date: "2026-03-05".to_string(),
            cash: 1000000,
            inflow: 120000,
            outflow: 85000,
        }],
        inventory_series: vec![DailyInventoryPoint {
            date: "2026-03-05".to_string(),
            item_id: "A001".to_string(),
            on_hand: 12,
            on_order: 20,
            demand: 8,
            sold: 8,
            stockout: 0,
        }],
    };
    let json = to_json(&report);
    assert!(json.contains("\"schema_version\":\"v0.1\""));
    assert!(json.contains("\"scenario\""));
    assert!(json.contains("\"alerts\""));
    assert!(json.contains("\"cash_series\""));
    assert!(json.contains("\"inventory_series\""));
    assert!(json.contains("\"inflow\""));
    assert!(json.contains("\"outflow\""));
    assert!(json.contains("\"on_order\""));
}
