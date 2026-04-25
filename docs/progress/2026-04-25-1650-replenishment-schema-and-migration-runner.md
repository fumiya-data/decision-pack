# replenishment schema and migration runner

## 実施内容

- `decision-engine` の `ItemRunInput` に `moq` と `lot_size` を追加した
- 補充数量の MOQ 適用と lot size 丸めを `decision-engine` の補充ロジックで実行するように変更した
- `app-api` では MOQ と lot size を需要由来の `reorder_point` / `order_up_to` に織り込まず、補充制約として `decision-engine` へ渡すように変更した
- `decision-engine/tests/report_schema_sync.rs` を追加し、`decision-engine/schemas` と `reporting/schemas` の `simulation_report_v0.1` が同期していることを `cargo test` で確認できるようにした
- `db-migrate` crate を追加し、`db/migrations/` 配下の SQL migration をファイル名順に適用できるようにした
- `db-migrate` は `schema_migrations` に version、name、checksum、applied_at を記録し、適用済み migration の再実行を skip する
- `docs/operations/local-operations-runbook.md` の migration 手順を `db-migrate` 標準に更新した

## 確認結果

- `cargo fmt --all --check` が成功した
- `cargo test --workspace` が成功した
- `lake build` が成功した
- ローカル DB に対して `cargo run -p db-migrate -- --database-url $env:DATABASE_URL` を実行し、初回は `202604172120_initial_schema.sql` が applied になった
- 同じ migration runner を再実行し、適用済み migration が skipped になることを確認した
- `APP_API_TEST_DATABASE_URL` を使った `app-api` DB-backed 統合テストが成功した

## 補足

- ADR-0006 の方針に合わせ、需要予測と補充制約を分けたまま `decision-engine` 側で発注数量制約を適用する形にした
- schema の完全な single source 化はまだ行っていないが、少なくとも重複ファイルの差分はテストで検出できる
