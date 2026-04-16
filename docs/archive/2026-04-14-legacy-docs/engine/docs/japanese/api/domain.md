# domain モジュール

`domain` はシミュレーション全体で共有する基本型を定義します。

## 主な型

- `Date = u32`
  - 日次粒度の日付表現
- `Yen = i64`
  - 金額（正: 入金、負: 出金）
- `Qty = u32`
  - 非負数量

## 業務構造体

- `InventoryState { on_hand, on_order }`
- `CashState { cash }`
- `ItemPolicy { item_id, reorder_point, order_up_to, lead_time_days }`
- `Delivery { item_id, due, qty }`
- `InventoryStepResult { next, new_orders, stockout }`
- `CashEvent { due, amount, category }`

## Lean 仕様との対応

- `CashState` / `InventoryState` / `Delivery` / `InventoryStepResult` は `spec/DecisionSpec/Common.lean` の同名概念に対応します。
