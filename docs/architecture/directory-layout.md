# ディレクトリ構造

## 1. 採用方針

- 役割ディレクトリは英語 `kebab-case`
- コードファイルは各言語の標準規約に従う
- 文書の正本は `docs/` 配下に集約する
- `docs/progress/` の進捗記録は時刻付きファイル名で更新単位に分ける

## 2. 目標構造

```text
decision-pack/
  docs/
    index.md
    plans/
    specs/
    architecture/
    decisions/
    progress/
    backlog/
    archive/
  db/
    migrations/
    seeds/
  customers-etl/
  commerce-etl/
  purchase-insights/
  decision-engine/
  app-api/
  desktop-ui/
  reporting/
  spec/
  data/
```

## 3. 旧構造からの読み替え

- 旧 `customers_etl` は `customers-etl` へ再配置済み
- 旧 `engine` は `decision-engine` へ再配置済み
- 旧 `ui` は `desktop-ui` へ再配置済み

## 4. 役割

- `customers-etl`: 顧客データ整形
- `commerce-etl`: 商品、受注、在庫取込
- `purchase-insights`: 購入傾向推定
- `decision-engine`: 在庫・資金の意思決定計算
- `app-api`: GUI 向け API
- `desktop-ui`: GUI
- `reporting`: 図表生成
- `spec`: 形式仕様
- `db`: PostgreSQL の migration と seed 管理
