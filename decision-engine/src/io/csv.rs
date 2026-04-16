use crate::domain::date::Date;
use crate::domain::money::{Qty, Yen};

/// 商品マスタの正規テーブル名です。
pub const ITEMS: &str = "items";
/// 日次売上の正規テーブル名です。
pub const SALES_DAILY: &str = "sales_daily";
/// 日次在庫スナップショットの正規テーブル名です。
pub const INVENTORY_DAILY: &str = "inventory_daily";
/// スタッフ制約の正規テーブル名です。
pub const STAFF: &str = "staff";
/// 日次資金イベントの正規テーブル名です。
pub const CASHFLOW_DAILY: &str = "cashflow_daily";

/// `items` テーブルの解析済み行です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemRow {
    pub item_id: String,
    pub lead_time_days: Date,
    pub unit_cost: Yen,
    pub unit_price: Yen,
    pub safety_stock: Qty,
}

/// `sales_daily` テーブルの解析済み行です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SalesDailyRow {
    pub date: Date,
    pub item_id: String,
    pub qty: Qty,
    pub unit_price: Yen,
}

/// `inventory_daily` テーブルの解析済み行です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InventoryDailyRow {
    pub date: Date,
    pub item_id: String,
    pub on_hand: Qty,
    pub on_order: Qty,
}

/// `cashflow_daily` テーブルの解析済み行です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CashflowDailyRow {
    pub date: Date,
    pub category: String,
    pub amount: Yen,
    pub direction: String,
}

/// CSV 行レベル検証のエラー種別です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CsvValidationError {
    /// `item_id` が空、または空白のみです。
    EmptyItemId,
    /// `direction` が `in` または `out` ではありません。
    InvalidDirection(String),
}

/// `items` の 1 行を検証します。
///
/// # 規則
/// - `item_id` は空にできません。
pub fn validate_item_row(row: &ItemRow) -> Result<(), CsvValidationError> {
    if row.item_id.trim().is_empty() {
        return Err(CsvValidationError::EmptyItemId);
    }
    Ok(())
}

/// `cashflow_daily` の 1 行を検証します。
///
/// # 規則
/// - `direction` は `"in"` または `"out"` でなければなりません。
pub fn validate_cashflow_row(row: &CashflowDailyRow) -> Result<(), CsvValidationError> {
    match row.direction.as_str() {
        "in" | "out" => Ok(()),
        other => Err(CsvValidationError::InvalidDirection(other.to_string())),
    }
}
