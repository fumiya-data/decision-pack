import DecisionSpec.Common

namespace DecisionSpec

/-- 在庫切れ数量（欠品数量）を返すKPI関数。需要が可用在庫以下なら0、超過分を欠品とする。 -/
def StockoutQty (demand available : Qty) : Qty :=
  if _ : demand ≤ available then 0 else (demand - available)

/-- 1日分の在庫処理結果。販売数量sold、欠品数量stockout、翌日在庫nextOnHandを持つ。 -/
structure InventoryDayResult where
  sold : Qty
  stockout : Qty
  nextOnHand : Qty
deriving Repr

/-- 1日分の在庫更新。可用在庫を計算し、販売・欠品・翌日在庫を同時に算出する。 -/
def InventoryOneDay (onHand arrivals demand : Qty) : InventoryDayResult := by
  let available := onHand + arrivals
  let sold := Nat.min demand available
  let stockout := demand - sold
  let nextOnHand := available - sold
  exact {sold := sold, stockout := stockout, nextOnHand := nextOnHand}

/-- 欠品数量は `demand - min(demand, available)` と一致する。 -/
theorem stockout_qty_eq_sub_min
  (demand available : Qty) :
  StockoutQty demand available = demand - Nat.min demand available := by
  by_cases h : demand ≤ available
  · simp [StockoutQty, h, Nat.min_eq_left h]
  · have h' : available ≤ demand := Nat.le_of_not_ge h
    simp [StockoutQty, h, Nat.min_eq_right h']

/-- 保存則の不変条件。翌日在庫と販売数量の和は、可用在庫に等しい。 -/
theorem inv_conservation
  (onHand arrivals demand : Qty) :
  let s := InventoryOneDay onHand arrivals demand
  s.nextOnHand + s.sold = onHand + arrivals := by
  simp [InventoryOneDay, Nat.sub_add_cancel, Nat.min_le_right]

/-- `InventoryStepSpec` は `onOrder` を変更しない。 -/
theorem inventory_step_preserves_on_order
  (today : Date)
  (st : InventoryState)
  (salesQty arrivalsQty : Qty)
  (policy : ItemPolicy) :
  (InventoryStepSpec today st salesQty arrivalsQty policy).next.onOrder = st.onOrder := by
  simp [InventoryStepSpec]

/-- `InventoryStepSpec` は現時点では新規発注を生成しない。 -/
theorem inventory_step_emits_no_orders
  (today : Date)
  (st : InventoryState)
  (salesQty arrivalsQty : Qty)
  (policy : ItemPolicy) :
  (InventoryStepSpec today st salesQty arrivalsQty policy).newOrders = [] := by
  simp [InventoryStepSpec]

/-- `InventoryStepSpec` の在庫更新は `InventoryOneDay` と整合する。 -/
theorem inventory_step_matches_inventory_one_day
  (today : Date)
  (st : InventoryState)
  (salesQty arrivalsQty : Qty)
  (policy : ItemPolicy) :
  let day := InventoryOneDay st.onHand arrivalsQty salesQty
  (InventoryStepSpec today st salesQty arrivalsQty policy).next.onHand = day.nextOnHand ∧
  (InventoryStepSpec today st salesQty arrivalsQty policy).stockout = day.stockout := by
  simp [InventoryStepSpec, InventoryOneDay]

end DecisionSpec