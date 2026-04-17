# 2026-04-17 21:12 初期実装の着手

## 完了済み

- `db/migrations/` と `db/seeds/` を追加した
- PostgreSQL の初期 schema を `db/migrations/202604172120_initial_schema.sql` として追加した
- `app-api` を Axum + SQLx 前提の実行可能な雛形へ置き換えた
- `app-api` にヘルスチェックと主要読み取り系 API の土台を追加した
- `cargo check -p app-api` と `cargo check --workspace` が成功することを確認した

## 次にやること

1. `customers-etl` から PostgreSQL へ顧客データを保存する経路を実装する
2. `commerce-etl` の入力契約に対応する fixture と取込処理を実装する
3. `app-api` の読み取り系 API をテスト可能な形へ整える
4. `purchase-insights` の MVP 入出力型と最初の計算処理を追加する
