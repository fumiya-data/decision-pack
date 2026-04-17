# Decision Pack 文書一覧

このディレクトリを Decision Pack の現行文書の正本とします。

## 1. 最初に読む文書

1. `plans/master-plan.md`
2. `specs/product-scope.md`
3. `architecture/integrated-architecture.md`
4. `architecture/directory-layout.md`

## 2. 計画

- `plans/master-plan.md`
  - プロジェクト全体の到達点、優先順位、段階的な進め方
- `plans/roadmap.md`
  - フェーズ単位の実装順と完了条件
- `plans/existing-module-migration-plan.md`
  - 既存モジュールを新しい責務構造へ移すための移行方針

## 3. 仕様

- `specs/product-scope.md`
  - 何を作るか、何を作らないか、誰のためのものか
- `specs/functional-requirements.md`
  - 顧客、購入履歴、在庫、シミュレーション、GUI、API の機能要件
- `specs/domain-model.md`
  - ドメイン概念、主要エンティティ、集約の境界
- `specs/data-contracts.md`
  - CSV、DB、API、レポート JSON の契約
- `specs/non-functional-requirements.md`
  - 性能、運用、監査、変更容易性などの非機能要件

## 4. アーキテクチャ

- `architecture/integrated-architecture.md`
  - 全体の論理構成
- `architecture/module-responsibilities-and-boundaries.md`
  - モジュール責務と境界の図
- `architecture/directory-layout.md`
  - 最終的に採用するディレクトリ構造
- `architecture/aws-runtime-architecture.md`
  - AWS 上での実行構成

## 5. 意思決定記録

- `decisions/adr-0001-casing-and-document-layout.md`
  - 命名規則と文書配置ルール
- `decisions/adr-0002-module-boundaries.md`
  - モジュール境界の固定方針
- `decisions/adr-0003-postgresql-migration-strategy.md`
  - PostgreSQL の migration 方式
- `decisions/adr-0004-app-api-technology-stack.md`
  - `app-api` の実装技術
- `decisions/adr-0005-decision-engine-report-json-boundary.md`
  - GUI と reporting が依存するレポート JSON 境界
- `decisions/adr-0006-replenishment-constraints-in-decision-engine.md`
  - MOQ と lot size の適用位置
- `decisions/adr-0007-multi-source-ingestion-via-adapters-and-staging.md`
  - 複数データソース統合取込の基本方式

## 6. 記録

- `progress/2026-04-17-2137-phase-1-implementation-complete.md`
  - ETL、需要予測、シミュレーション、API、GUI の実装フェーズ1完了
- `progress/2026-04-17-2112-initial-implementation-started.md`
  - `db/migrations` と `app-api` の初期実装着手
- `progress/2026-04-17-2102-specs-expanded.md`
  - `docs/specs/` の仕様書詳細化
- `progress/2026-04-17-2045-no-orm-policy.md`
  - ORM 非採用と手書き SQL 方針の明文化
- `progress/2026-04-17-2038-backlog-decisions.md`
  - 未決事項 9 件の初回判断と ADR 反映
- `progress/2026-04-16-1903-structure-and-spec-update.md`
  - 2026-04-16 19:03 時点のディレクトリ再構築と Lean 仕様更新
- `progress/2026-04-14-1215-current-status.md`
  - 2026-04-14 12:15 時点の整理と次の着手点
- `backlog/open-items.md`
  - 未決事項と後回し項目

## 7. 文書配置ルール

- 新しい仕様書、計画書、設計書、進捗、運用手順、ADR は `docs/` 配下に置く。
- 文書ファイル名と文書ディレクトリ名は英語の `kebab-case` に統一すること。
- `docs/progress/` の進捗記録は `yyyy-mm-dd-hhmm-<summary>.md` 形式とし、同日に複数回更新する場合も更新ごとに別ファイルを作ること。
- 本文、コメント、説明は日本語を正本とし、完成後に必要に応じ英語版を追加すること。
- 言語仕様に制約があるコードファイルは各言語の標準規約に従うこと。
- 過去文書は `archive/` 配下へ退避し、現行文書とは分離すること。

## 8. アーカイブ

- アーカイブ一覧は `archive/index.md` で管理すること。
- 旧文書は `archive/2026-04-14-legacy-docs/` 配下へ退避すること。
- 旧文書は参照用であり、現行方針や現状実装を保証するものではないこと。
