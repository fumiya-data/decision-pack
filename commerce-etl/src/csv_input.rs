use csv::{ReaderBuilder, Trim};
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct ItemRow {
    pub item_id: String,
    pub item_name: String,
    pub category: String,
    pub uom: Option<String>,
    pub is_active: Option<bool>,
    pub lead_time_days: Option<i32>,
    pub moq: Option<i32>,
    pub lot_size: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OrderRow {
    pub order_id: String,
    pub customer_id: String,
    pub ordered_at: String,
    pub status: String,
    pub currency: Option<String>,
    pub total_amount: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OrderItemRow {
    pub order_id: String,
    pub line_no: i32,
    pub item_id: String,
    pub quantity: i32,
    pub unit_price: Option<f64>,
    pub line_amount: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InventoryRow {
    pub item_id: String,
    pub on_hand: i32,
    pub on_order: Option<i32>,
    pub reserved_qty: Option<i32>,
}

pub fn read_csv<T>(path: &Path) -> Result<Vec<T>, Box<dyn std::error::Error>>
where
    T: for<'de> Deserialize<'de>,
{
    let mut reader = ReaderBuilder::new().trim(Trim::All).from_path(path)?;
    let mut rows = Vec::new();
    for row in reader.deserialize() {
        rows.push(row?);
    }
    Ok(rows)
}
