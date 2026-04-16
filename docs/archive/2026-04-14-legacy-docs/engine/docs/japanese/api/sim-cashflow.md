# sim.cashflow

資金の日次更新ロジックを提供します。

## API

## `cash_one_day(today, st, events) -> CashState`

当日 `today` のイベントだけを合計し、キャッシュ残高を更新します。

- `event.due == today` のものだけ対象
- `amount` は符号付き（入金は正、出金は負）

## `cash_additive_holds(today, st, a, b) -> bool`

加法性（連結不変条件）を確認します。

- `cash_one_day(today, st, a ++ b)`
- `cash_one_day(today, cash_one_day(today, st, a), b)`
- 上記2つが一致するかを返す

## Lean 仕様との対応

- `spec/DecisionSpec/Cashflow.lean` の `CashOneDay`
- `cash_additive` 定理
