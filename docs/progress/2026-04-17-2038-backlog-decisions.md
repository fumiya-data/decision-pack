# 2026-04-17 20:38 未決事項の初回判断反映

## 完了済み

- `docs/backlog/open-items.md` の 9 件について初回判断を反映した
- ADR 化すべき判断を `adr-0003` から `adr-0007` として追加した
- `purchase-insights` の重み調整方針を仕様へ反映した
- 在庫リスクのカテゴリ別閾値と MOQ/lot size の適用位置を仕様へ反映した
- GUI の並列タブ導線を仕様へ反映した
- 高度な推薦アルゴリズムへの移行条件を非機能要件へ反映した
- `decision-engine` のレポート JSON 拡張範囲を契約へ反映した
- PostgreSQL migration 方式の決定に合わせて `db/migrations` をディレクトリ構造へ反映した

## 次にやること

1. `db/migrations/` の実ディレクトリと初期 migration 雛形を作る
2. `app-api` の Axum + SQLx 雛形を切る
3. `purchase-insights` の検証データと重み調整手順を具体化する
4. `decision-engine` のレポート JSON 拡張項目を具体的なフィールドに落とす
