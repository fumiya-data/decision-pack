# Decision Pack

Decision Pack は、顧客データ、購入履歴、在庫、需要予測、意思決定支援を一貫して扱うことを目的とした、開発中の個人プロジェクトです。
当プロジェクトは、最終的には、デスクトップ GUI を薄いクライアントとして保ちつつ、API とジョブ基盤を介してローカル実行と AWS 実行の両方に対応できる構成になることを目指しています。

現状においては、ディレクトリ構造、モジュール責務、データ契約、Lean 仕様は再編途中にあり、実装よりも先に `docs/` 配下の文書を正本として更新しているところです。現時点では、文書に記載された方針が最新であり、コードはそれに追従する方向です。

## 最初に読む文書

1. `docs/index.md`
2. `docs/plans/master-plan.md`
3. `docs/specs/product-scope.md`
4. `docs/architecture/integrated-architecture.md`
5. `docs/architecture/directory-layout.md`

## 主要ディレクトリ

- `customers-etl/`: 顧客データ前処理
- `commerce-etl/`: 商品、受注、在庫取込
- `purchase-insights/`: 購入傾向分析と需要予測
- `decision-engine/`: 在庫・資金の意思決定計算
- `app-api/`: GUI 向け API 境界
- `desktop-ui/`: Iced ベースのデスクトップ GUI
- `reporting/`: レポート成果物生成
- `spec/`: Lean による形式仕様と検証
- `data/`: fixture、raw sample、expected output

## 文書の扱い

- 現行文書の正本は `docs/` 配下です。
- 進捗記録は `docs/progress/` に時刻付きファイルで残します。
- 当プロジェクトでは、元々個別に実験的に作成していたモジュールを一部組み込んでいるため、個別モジュール時代に作成した旧文書が存在し、これらは `docs/archive/` 配下に隔離されています。