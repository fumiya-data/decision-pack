# 未決事項

## 現在の状態

- 2026-04-17 時点で、初回の未決事項 9 件はすべて方針決定済み
- 新たな未決事項が発生した場合は、この文書へ追記する

## 決定済み事項

1. PostgreSQL の migration 方式
   - 生 SQL + migration runner を採用する
   - 参照: `decisions/adr-0003-postgresql-migration-strategy.md`

2. `purchase-insights` の MVP アルゴリズムの重み
   - 検証データで調整する
   - 参照: `specs/functional-requirements.md`, `architecture/integrated-architecture.md`

3. `decision-engine` のレポート JSON の拡張範囲
   - GUI 表示に必要な実行要約と品目別結果まで拡張する
   - 参照: `decisions/adr-0005-decision-engine-report-json-boundary.md`

4. `app-api` の実装技術
   - Rust + Axum + SQLx を採用する
   - 参照: `decisions/adr-0004-app-api-technology-stack.md`

5. 在庫リスクの閾値
   - 品目カテゴリ別に設定する
   - 参照: `specs/functional-requirements.md`

6. 補充提案に MOQ や lot size を入れる段階
   - `decision-engine` の補充提案ロジックで適用する
   - 参照: `decisions/adr-0006-replenishment-constraints-in-decision-engine.md`

7. GUI での顧客画面と在庫画面の導線
   - `顧客`, `在庫`, `シミュレーション` の並列タブを基本導線とする
   - 参照: `specs/functional-requirements.md`, `architecture/integrated-architecture.md`

8. 高度な推薦アルゴリズムへの移行条件
   - データ量・履歴期間の条件と、業務指標または検証指標の条件を両方満たしたときに検討する
   - 参照: `specs/non-functional-requirements.md`, `architecture/integrated-architecture.md`

9. 複数データソースからの統合取込
   - ソースごとの adapter と共通 staging を採用する
   - 参照: `decisions/adr-0007-multi-source-ingestion-via-adapters-and-staging.md`
