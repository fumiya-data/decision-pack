# ADR-0003 PostgreSQL migration 方式

## 状態

採用済み

## 背景

このプロジェクトでは `customers-etl`、`commerce-etl`、`purchase-insights`、`app-api` が共通の PostgreSQL を扱う。
スキーマ変更をコード実装や ORM 定義に埋め込むと、変更履歴の追跡、再現、レビューが難しくなる。
個人開発でも、環境再構築と将来の AWS 移行に耐える migration 方式を先に固定したい。

## 決定事項

- PostgreSQL の schema 変更は、生 SQL の migration ファイルで管理する
- migration の適用は migration runner で行う
- migration ファイルは `db/migrations/` 配下で順序付きに管理する
- ORM 定義やアプリケーションコードを schema の正本にはしない
- schema の意味は migration SQL と仕様書で管理する

## 影響

- DB 変更は必ず SQL ファイルとしてレビューできる
- ローカルと AWS の両方で同じ migration を再利用できる
- DB の初期化や再構築が容易になる
