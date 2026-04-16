use crate::domain::money::{Qty, Yen};

/// `simulation_report_v0.1` に含まれるシナリオ情報です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioInfo {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
}

/// `simulation_report_v0.1` に含まれる KPI 要約です。
#[derive(Debug, Clone, PartialEq)]
pub struct KpiReport {
    pub min_cash: Yen,
    pub first_cash_shortfall_date: Option<String>,
    pub total_stockout_qty: Qty,
    pub stockout_rate: f64,
    pub days_on_hand_avg: f64,
}

/// レポート出力用のアラートコード列挙です。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertCode {
    CashThresholdBreach,
    CashShortfall,
    StockoutRateBreach,
    OrderBudgetBreach,
}

impl AlertCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CashThresholdBreach => "cash_threshold_breach",
            Self::CashShortfall => "cash_shortfall",
            Self::StockoutRateBreach => "stockout_rate_breach",
            Self::OrderBudgetBreach => "order_budget_breach",
        }
    }
}

/// レポート出力用の重要度列挙です。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertSeverity {
    Info,
    Warn,
    Critical,
}

impl AlertSeverity {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warn => "warn",
            Self::Critical => "critical",
        }
    }
}

/// `simulation_report_v0.1` のアラート 1 件です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Alert {
    pub code: AlertCode,
    pub severity: AlertSeverity,
    pub date: String,
    pub item_id: Option<String>,
    pub message: String,
}

/// 1 日分の資金残高点です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DailyCashPoint {
    pub date: String,
    pub cash: Yen,
    pub inflow: Yen,
    pub outflow: Yen,
}

/// 1 日・1 品目の在庫点です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DailyInventoryPoint {
    pub date: String,
    pub item_id: String,
    pub on_hand: Qty,
    pub on_order: Qty,
    pub demand: Qty,
    pub sold: Qty,
    pub stockout: Qty,
}

/// UI とエクスポート層へ渡す完全なシミュレーション出力です。
#[derive(Debug, Clone, PartialEq)]
pub struct SimulationReport {
    pub schema_version: String,
    pub generated_at: String,
    pub scenario: ScenarioInfo,
    pub horizon_days: u32,
    pub currency: String,
    pub kpi: KpiReport,
    pub alerts: Vec<Alert>,
    pub cash_series: Vec<DailyCashPoint>,
    pub inventory_series: Vec<DailyInventoryPoint>,
}
