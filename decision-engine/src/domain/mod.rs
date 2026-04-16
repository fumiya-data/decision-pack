//! モジュール間で共有する業務ドメインのデータ構造です。

pub mod date;
pub mod kpi;
pub mod money;

use crate::domain::date::Date;
use crate::domain::money::{Qty, Yen};

/// ある時点における 1 品目の在庫状態です。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InventoryState {
    /// 現在利用できる実在庫です。
    pub on_hand: Qty,
    /// すでに発注済みで未入荷の数量です。
    pub on_order: Qty,
}

/// ある時点の資金状態です。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CashState {
    /// 現在の現金残高（円）です。
    pub cash: Yen,
}

/// 1 品目の補充方針です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemPolicy {
    /// 安定した品目識別子です。
    pub item_id: String,
    /// 再発注を判断するしきい値です。
    pub reorder_point: Qty,
    /// 発注後に目指す在庫水準です。
    pub order_up_to: Qty,
    /// 想定リードタイム（日数）です。
    pub lead_time_days: Date,
}

/// 予定されている入荷です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Delivery {
    /// 品目識別子です。
    pub item_id: String,
    /// 入荷予定日です。
    pub due: Date,
    /// 予定入荷数量です。
    pub qty: Qty,
}

/// 1 ステップ分の在庫計算結果です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InventoryStepResult {
    /// 次時点の在庫状態です。
    pub next: InventoryState,
    /// このステップで新たに生成された発注です。
    pub new_orders: Vec<Delivery>,
    /// このステップで発生した欠品数量です。
    pub stockout: Qty,
}

/// ある日に予定された 1 件の資金イベントです。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CashEvent {
    /// イベント発生日です。
    pub due: Date,
    /// 符号付き金額です。正は入金、負は出金を表します。
    pub amount: Yen,
    /// `sales` や `purchase` のようなカテゴリ名です。
    pub category: String,
}
