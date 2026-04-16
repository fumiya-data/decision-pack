use crate::domain::date::Date;
use crate::domain::money::Qty;
use crate::domain::{Delivery, InventoryState, InventoryStepResult, ItemPolicy};

/// 1 日分の在庫計算結果です。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InventoryDayResult {
    /// 利用可能在庫から販売できた数量です。
    pub sold: Qty,
    /// 満たせなかった需要数量です。
    pub stockout: Qty,
    /// 翌日に繰り越される手持在庫です。
    pub next_on_hand: Qty,
}

/// 需要量と利用可能量から欠品数量を返します。
///
/// 飽和減算を使うため、結果が負になることはありません。
pub fn stockout_qty(demand: Qty, available: Qty) -> Qty {
    demand.saturating_sub(available)
}

/// 1 日分の在庫フローを計算します。
///
/// 手順:
/// 1. `available = on_hand + arrivals`
/// 2. `sold = min(demand, available)`
/// 3. `stockout = demand - sold`
/// 4. `next_on_hand = available - sold`
pub fn inventory_one_day(on_hand: Qty, arrivals: Qty, demand: Qty) -> InventoryDayResult {
    let available = on_hand.saturating_add(arrivals);
    let sold = demand.min(available);
    let stockout = demand.saturating_sub(sold);
    let next_on_hand = available.saturating_sub(sold);
    InventoryDayResult {
        sold,
        stockout,
        next_on_hand,
    }
}

/// 保存則 `next_on_hand + sold == on_hand + arrivals` を検査します。
pub fn inv_conservation_holds(on_hand: Qty, arrivals: Qty, demand: Qty) -> bool {
    let s = inventory_one_day(on_hand, arrivals, demand);
    s.next_on_hand.saturating_add(s.sold) == on_hand.saturating_add(arrivals)
}

/// 仕様に整合した 1 日分の在庫ステップです。
///
/// 現在の振る舞いは Lean の `InventoryStepSpec` をそのまま反映します。
/// - `on_hand` を更新する
/// - `on_order` は変更しない
/// - `stockout` を計算する
/// - 新規発注はまだ生成しない
pub fn inventory_step_spec(
    _today: Date,
    st: InventoryState,
    sales_qty: Qty,
    arrivals_qty: Qty,
    _policy: &ItemPolicy,
) -> InventoryStepResult {
    let day = inventory_one_day(st.on_hand, arrivals_qty, sales_qty);
    InventoryStepResult {
        next: InventoryState {
            on_hand: day.next_on_hand,
            on_order: st.on_order,
        },
        new_orders: Vec::<Delivery>::new(),
        stockout: day.stockout,
    }
}
