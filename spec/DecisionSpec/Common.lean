namespace DecisionSpec
/- 目的: Lean4で意思決定仕様を記述し、Rust実装の参照基準とする。 -/

/-- Dateは日付を抽象化した日数表現。ここではNatを用いる。Rust側では実日付型で扱う。 -/
abbrev Date := Nat

/-- Yenは金額（円）を表す型。収支の符号を扱うためIntを用いる。 -/
abbrev Yen := Int

/-- Qtyは物品数量を表す型。0以上を保証するためNatを用いる。 -/
abbrev Qty := Nat

/-- 在庫状態。手持在庫onHandと発注残onOrderを持つ。 -/
structure InventoryState where
  onHand : Qty
  onOrder : Qty
deriving Repr

/-- 現金状態。現在の保有現金cash（円）を持つ。 -/
structure CashState where
  cash : Yen
deriving Repr

/-- 日次売上観測。商品ID、売上数量、単価を持つ。 -/
structure SalesObs where
  itemID : String
  qtySold : Qty
  unitPrice : Yen
deriving Repr

/-- 商品別発注方針。発注点reorderPoint、発注上限orderUpTo、調達リードタイムleadTimeDaysを持つ。 -/
structure ItemPolicy where
  itemID : String
  reorderPoint : Qty
  orderUpTo : Qty
  leadTimeDays : Nat
deriving Repr

/-- 入荷予定。商品ID、納期due、入荷数量qtyを持つ。 -/
structure Delivery where
  itemID : String
  due : Date
  qty : Qty
deriving Repr

/-- 在庫ステップ結果。次状態next、新規発注newOrders、欠品数量stockoutを返す。 -/
structure InventoryStepResult where
  next : InventoryState
  newOrders : List Delivery
  stockout : Qty

/-- 入荷予定の妥当性条件。入荷数量は1以上。 -/
def ValidDelivery (d : Delivery) : Prop :=
  d.qty >= 1

/-- 1日分の在庫更新仕様。入荷反映後に販売を充当し、次在庫と欠品数量を計算する。
発注ロジックは暫定で未実装（newOrdersは空）。 -/
def InventoryStepSpec
  (_today : Date)
  (st : InventoryState)
  (salesQty : Qty)
  (arrivalsQty : Qty)
  (_policy : ItemPolicy)
  : InventoryStepResult :=
  let available := st.onHand + arrivalsQty
  let sold := Nat.min salesQty available
  {
    next := {
      onHand := available - sold
      onOrder := st.onOrder
    }
    newOrders := []
    stockout := salesQty - sold
  }
end DecisionSpec
