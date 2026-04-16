# sim.inventory

在庫の日次更新ロジックを提供します。

## API

## `stockout_qty(demand, available) -> Qty`

- 需要と可用在庫から欠品数量を返す
- `saturating_sub` を使うため負値にはならない

## `inventory_one_day(on_hand, arrivals, demand) -> InventoryDayResult`

1日分の在庫処理を実行します。

1. `available = on_hand + arrivals`
2. `sold = min(demand, available)`
3. `stockout = demand - sold`
4. `next_on_hand = available - sold`

## `inv_conservation_holds(on_hand, arrivals, demand) -> bool`

保存則を検証します。

- `next_on_hand + sold == on_hand + arrivals`

## `inventory_step_spec(today, st, sales_qty, arrivals_qty, policy) -> InventoryStepResult`

Lean の `InventoryStepSpec` に揃えた日次ステップです。

- `next.on_hand` を更新
- `next.on_order` は現状維持
- `stockout` を返却
- `new_orders` は現在未実装（空）

## Lean 仕様との対応

- `spec/DecisionSpec/Inventory.lean`
- `spec/DecisionSpec/Common.lean` の `InventoryStepSpec`
