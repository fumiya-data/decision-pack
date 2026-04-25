# Decision Pack

Decision Pack は、顧客データ、購入履歴、在庫、需要予測、意思決定支援を一貫して扱うことを目的とした、開発中の個人プロジェクトです。

現在は、ローカル実行基盤と Docker Compose によるローカル運用手順の整備が完了し、AWS に持ち上げるための準備段階にあります。
PostgreSQL、migration、ETL、購入傾向分析、API、デスクトップ GUI、reporting は、host 実行または Docker Compose で一通り確認できる状態です。
次のマイルストーンは、無料運用を前提にした AWS staging 構成の設計と実装です。

## 最初に読む文書

1. [docs/index.md](docs/index.md)
2. [docs/plans/master-plan.md](docs/plans/master-plan.md)
3. [docs/specs/product-scope.md](docs/specs/product-scope.md)
4. [docs/architecture/integrated-architecture.md](docs/architecture/integrated-architecture.md)
5. [docs/architecture/directory-layout.md](docs/architecture/directory-layout.md)
6. [docs/operations/local-operations-runbook.md](docs/operations/local-operations-runbook.md)
7. [docs/operations/docker-local-runbook.md](docs/operations/docker-local-runbook.md)

## 主要ディレクトリ

- [customers-etl/](customers-etl/): 顧客データの整形、検証、PostgreSQL への保存
- [commerce-etl/](commerce-etl/): 商品、受注、受注明細、在庫のサンプル生成と取込
- [purchase-insights/](purchase-insights/): 購入傾向分析、次回購入候補、需要予測
- [decision-engine/](decision-engine/): 在庫・資金シミュレーションとレポート JSON 生成
- [app-api/](app-api/): GUI と外部クライアント向けの API 境界
- [desktop-ui/](desktop-ui/): Iced ベースのデスクトップ GUI
- [reporting/](reporting/): シミュレーションレポートの図表と要約生成
- [db-migrate/](db-migrate/): PostgreSQL migration runner
- [db/](db/): SQL migration
- [data/](data/): fixture、raw sample、expected output
- [spec/](spec/): Lean による形式仕様と検証
- [docs/](docs/): 現行仕様、設計、ADR、運用手順、進捗記録

## 文書の扱い

このプロジェクトは、`docs/` を正本とする仕様駆動開発を採用しています。
実装は、先に合意・記録した仕様、設計、ADR、runbook に追従させます。
現在方針は [docs/index.md](docs/index.md) から確認できます。

仕様駆動開発として、文書は次の役割を持ちます。

- [docs/specs/](docs/specs/) では、製品、データ契約、機能要件、非機能要件、評価用サンプル仕様を定義します。
- [docs/architecture/](docs/architecture/) では、モジュール責務、ディレクトリ構成、AWS を含む実行構成の設計を定義します。
- [docs/decisions/](docs/decisions/) では、開発途中に行った技術判断を ADR として固定します。
- [docs/operations/](docs/operations/) では、ローカル運用、Docker 運用、将来の AWS 運用の実行手順を固定します。
- [docs/progress/](docs/progress/) では、完了したタスク、確認事項等を時刻付きで記録します。
- [docs/backlog/](docs/backlog/) では、未決事項を管理します。

変更したい場合は、仕様や責務に変更があるなら `docs/` をまず更新し、その後で実装を整合させます。
実装中に仕様の穴や運用上の問題を発見した場合は、修正内容を `docs/progress/` や該当の runbook に記録します。
古い文書は、現行方針と混ざらないように [docs/archive/](docs/archive/) へと隔離します。

## ローカル運用

通常の host 実行は [docs/operations/local-operations-runbook.md](docs/operations/local-operations-runbook.md) を参照します。
Docker Compose での実行は [docs/operations/docker-local-runbook.md](docs/operations/docker-local-runbook.md) を参照します。

Docker では、PostgreSQL、migration、ETL、purchase-insights、app-api、decision-engine report、Python reporting をローカル環境で再現できるようにしています。
この Docker ローカル運用を AWS 移行前の実行基盤の基準にしています。
