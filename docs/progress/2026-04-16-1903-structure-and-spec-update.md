# 2026-04-16 19:03 ディレクトリ再構築と Lean 仕様更新

## 完了済み

- 既存 `customers_etl` を `customers-etl` へ再配置した
- 既存 `engine` を `decision-engine` へ再配置した
- 既存 `ui` を `desktop-ui` へ再配置した
- `commerce-etl`、`purchase-insights`、`app-api` の雛形ディレクトリを追加した
- Rust workspace を新しいモジュール名に合わせて更新した
- `decision-engine` と `desktop-ui` の package 名を新名称に合わせた
- `spec/` の既存 Lean 検証を維持しつつ、`Kpi` と在庫ステップ整合の検証を追加した
- `cargo test --workspace` と `lake build` が成功することを確認した

## 追加した Lean 検証

- `DecisionSpec.Inventory`
  - `stockout_qty_eq_sub_min`
  - `inventory_step_preserves_on_order`
  - `inventory_step_emits_no_orders`
  - `inventory_step_matches_inventory_one_day`
- `DecisionSpec.Kpi`
  - `observe_cash_never_increases`
  - `add_stockout_monotone`

## 次にやること

1. `commerce-etl` の入力契約と fixture を具体化する
2. `purchase-insights` の MVP 入出力と特徴量を確定する
3. `app-api` のエンドポイント雛形を切る
4. `decision-engine` の package 名変更に合わせて関連文書の表記差分が残っていないかを継続確認する