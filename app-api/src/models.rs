use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;

#[derive(Debug, Deserialize)]
pub struct Pagination {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

impl Pagination {
    pub fn limit(&self) -> i64 {
        self.limit.unwrap_or(50).clamp(1, 200) as i64
    }

    pub fn offset(&self) -> i64 {
        self.offset.unwrap_or(0) as i64
    }
}

#[derive(Debug, Deserialize)]
pub struct ItemListQuery {
    pub q: Option<String>,
    pub category: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

impl ItemListQuery {
    pub fn limit(&self) -> i64 {
        self.limit.unwrap_or(50).clamp(1, 200) as i64
    }

    pub fn offset(&self) -> i64 {
        self.offset.unwrap_or(0) as i64
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateSimulationRequest {
    pub scenario_id: Option<String>,
    pub scenario_name: Option<String>,
    pub scenario_description: Option<String>,
    pub horizon_days: Option<u32>,
    pub initial_cash: Option<i64>,
    pub currency: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RootResponse {
    pub service: String,
    pub status: &'static str,
    pub docs_path: &'static str,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub service: String,
    pub status: &'static str,
    pub database: &'static str,
}

#[derive(Debug, Serialize, FromRow)]
pub struct CustomerSummary {
    pub customer_id: String,
    pub full_name: String,
    pub email: Option<String>,
    pub status: Option<String>,
    pub tier: Option<String>,
    pub country: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct CustomerDetail {
    pub customer_id: String,
    pub full_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address_line: Option<String>,
    pub city: Option<String>,
    pub region: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
    pub birth_date: Option<String>,
    pub signup_date: Option<String>,
    pub last_purchase_date: Option<String>,
    pub status: Option<String>,
    pub tier: Option<String>,
    pub preferred_language: Option<String>,
    pub marketing_opt_in: Option<bool>,
    pub total_spend: Option<f64>,
    pub order_count: Option<i32>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct CustomerPurchase {
    pub order_id: String,
    pub ordered_at: String,
    pub order_status: String,
    pub item_id: String,
    pub item_name: String,
    pub quantity: i32,
    pub unit_price: Option<f64>,
    pub line_amount: Option<f64>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct CustomerNextBuy {
    pub item_id: String,
    pub item_name: String,
    pub score: f64,
    pub rank: i32,
    pub as_of: String,
}

#[derive(Debug, Serialize, FromRow)]
pub struct ItemSummary {
    pub item_id: String,
    pub item_name: String,
    pub category: String,
    pub is_active: bool,
    pub on_hand: Option<i32>,
    pub on_order: Option<i32>,
    pub reserved_qty: Option<i32>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct ItemDetail {
    pub item_id: String,
    pub item_name: String,
    pub category: String,
    pub uom: Option<String>,
    pub is_active: bool,
    pub lead_time_days: i32,
    pub moq: Option<i32>,
    pub lot_size: Option<i32>,
    pub updated_at: String,
}

#[derive(Debug, Serialize, FromRow)]
pub struct ItemInventory {
    pub item_id: String,
    pub on_hand: i32,
    pub on_order: i32,
    pub reserved_qty: i32,
    pub updated_at: String,
}

#[derive(Debug, Serialize, FromRow)]
pub struct ItemRisk {
    pub run_id: String,
    pub scenario_id: String,
    pub scenario_name: String,
    pub risk_level: Option<String>,
    pub recommended_reorder_qty: Option<i32>,
    pub expected_stockout_qty: Option<i32>,
    pub expected_days_on_hand: Option<f64>,
    pub requested_at: String,
}

#[derive(Debug, Serialize, FromRow)]
pub struct SimulationSummary {
    pub run_id: String,
    pub scenario_id: String,
    pub scenario_name: String,
    pub status: String,
    pub requested_at: String,
    pub completed_at: Option<String>,
    pub report_schema_version: Option<String>,
    pub report_uri: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct SimulationDetail {
    pub run_id: String,
    pub scenario_id: String,
    pub scenario_name: String,
    pub status: String,
    pub requested_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub report_schema_version: Option<String>,
    pub report_uri: Option<String>,
    pub report_available: bool,
}

#[derive(Debug, Serialize)]
pub struct SimulationReportEnvelope {
    pub run_id: String,
    pub report: Value,
}
