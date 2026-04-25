# ローカル運用 runbook

## 1. 目的

この runbook は、Decision Pack をローカル環境で一通り動かすための運用手順を固定する。
対象は、PostgreSQL の作成、migration、顧客 ETL、商取引 ETL、サンプル生成、購入傾向分析、API、GUI、reporting である。

この文書は、開発者が同じ順序で再実行できることを目的とする。AWS 化や自動ジョブ化の前段階では、この手順を正本として扱う。

## 2. 前提

- Windows PowerShell での実行を基準にする
- リポジトリルートをカレントディレクトリにする
- Rust toolchain と `cargo` が利用できる
- PostgreSQL と `psql` / `createdb` が利用できる
- Python reporting を使う場合は Python 3.10 以上を利用する
- Lean 仕様を検証する場合は `lake` が利用できる

```powershell
Set-Location C:\Users\kinbo\decision-pack
```

## 3. 共通環境変数

ローカルの標準 DB 名は `decision_pack_app` とする。

```powershell
$env:DATABASE_URL = "postgres://postgres:postgres@localhost/decision_pack_app"
```

`app-api` を別ポートで起動したい場合だけ、次を上書きする。

```powershell
$env:APP_API_BIND_ADDR = "127.0.0.1:8080"
```

## 4. PostgreSQL 作成

DB がまだ存在しない場合は作成する。

```powershell
createdb -U postgres decision_pack_app
```

すでに DB が存在する場合、この手順は失敗してよい。その場合は次の migration 適用へ進む。

## 5. Migration 適用

現時点では migration runner は未実装である。そのため、初期 migration は `psql` で直接適用する。

```powershell
psql $env:DATABASE_URL -f db\migrations\202604172120_initial_schema.sql
```

この migration は `CREATE TABLE IF NOT EXISTS` と `CREATE INDEX IF NOT EXISTS` を使うため、同じローカル DB に再適用できる。

## 6. Smoke 規模の実行

短時間で全体導線を確認する場合は、リポジトリ内の `5,000` 件顧客サンプルを使う。

### 6.1 顧客 ETL

```powershell
cargo run -p customers-etl -- `
  --input data\customers\raw\raw_customers_5000.csv `
  --output-dir out\customers-etl\local-smoke `
  --run-id customers-local-smoke `
  --database-url $env:DATABASE_URL
```

期待する成果物:

- `out/customers-etl/local-smoke/formatted.csv`
- `out/customers-etl/local-smoke/format_issues.csv`
- `out/customers-etl/local-smoke/customer_segment_summary.csv`
- `out/customers-etl/local-smoke/run_summary.json`
- DB の `customers`, `customer_load_issues`, `etl_job_runs`

### 6.2 商取引サンプル生成

```powershell
cargo run -p commerce-etl -- `
  --generate-sample `
  --customers-csv out\customers-etl\local-smoke\formatted.csv `
  --output-dir out\commerce-local-smoke `
  --customer-count 5000 `
  --item-count 100 `
  --order-count 15000 `
  --seed 20260417
```

期待する成果物:

- `out/commerce-local-smoke/items.csv`
- `out/commerce-local-smoke/orders.csv`
- `out/commerce-local-smoke/order_items.csv`
- `out/commerce-local-smoke/inventory.csv`
- `out/commerce-local-smoke/sample_metadata.json`

### 6.3 商取引 ETL

```powershell
cargo run -p commerce-etl -- `
  --items-csv out\commerce-local-smoke\items.csv `
  --orders-csv out\commerce-local-smoke\orders.csv `
  --order-items-csv out\commerce-local-smoke\order_items.csv `
  --inventory-csv out\commerce-local-smoke\inventory.csv `
  --database-url $env:DATABASE_URL `
  --run-id commerce-local-smoke
```

取り込み順は、外部キー制約に合わせて `items`, `orders`, `order_items`, `inventory_balance` の順にする。

### 6.4 購入傾向分析

評価と永続化を同時に行う。

```powershell
cargo run -p purchase-insights -- `
  --database-url $env:DATABASE_URL `
  --run-id insights-local-smoke `
  --evaluate
```

評価だけを確認して DB に保存しない場合は、次のようにする。

```powershell
cargo run -p purchase-insights -- `
  --database-url $env:DATABASE_URL `
  --run-id insights-eval-only `
  --evaluate `
  --skip-persist
```

期待する DB 出力:

- `customer_item_next_buy_score`
- `item_demand_forecast`
- `etl_job_runs`

### 6.5 API 起動

```powershell
cargo run -p app-api
```

別の PowerShell から API を確認する。

```powershell
Invoke-RestMethod http://127.0.0.1:8080/health
Invoke-RestMethod "http://127.0.0.1:8080/api/v1/customers?limit=3"
Invoke-RestMethod "http://127.0.0.1:8080/api/v1/items?limit=3"
```

シミュレーションを API から作成する。

```powershell
Invoke-RestMethod `
  -Method Post `
  -Uri http://127.0.0.1:8080/api/v1/simulations `
  -ContentType "application/json" `
  -Body '{"scenario_id":"baseline-local","scenario_name":"Baseline Local"}'
```

返却された `run_id` を使って結果とレポートを確認する。

```powershell
$runId = "<returned-run-id>"
Invoke-RestMethod "http://127.0.0.1:8080/api/v1/simulations/$runId"
Invoke-RestMethod "http://127.0.0.1:8080/api/v1/simulations/$runId/report"
```

### 6.6 GUI 起動

`app-api` を起動したまま、別の PowerShell で GUI を起動する。

```powershell
cargo run -p desktop-ui
```

GUI の API 入力欄は既定で `http://127.0.0.1:8080` を指す。

確認観点:

- 顧客タブで顧客一覧が表示される
- 顧客を選ぶと購入履歴と次回購入候補が表示される
- 在庫タブで品目と在庫が表示される
- シミュレーションタブで実行履歴とレポート概要が表示される

## 7. Full 規模のローカル実行

`50,000` 顧客、`100` 商品、`150,000` 注文の full サンプルはローカル検証専用である。生成データは Git に載せない。

### 7.1 多言語 raw 顧客サンプル生成

```powershell
cargo run -p customers-etl -- `
  --generate-raw-sample `
  --output-raw data\customers\raw\raw_customers_50000_multilingual.csv `
  --target-formatted-count 50000 `
  --invalid-row-count 240 `
  --seed 20260419
```

### 7.2 顧客 ETL

```powershell
cargo run -p customers-etl -- `
  --input data\customers\raw\raw_customers_50000_multilingual.csv `
  --output-dir out\customers-50000 `
  --run-id customers-full-local `
  --database-url $env:DATABASE_URL
```

### 7.3 商取引 full サンプル生成

```powershell
cargo run -p commerce-etl -- `
  --generate-sample `
  --customers-csv out\customers-50000\formatted.csv `
  --output-dir out\commerce-50000 `
  --customer-count 50000 `
  --item-count 100 `
  --order-count 150000 `
  --seed 20260417
```

### 7.4 商取引 ETL

```powershell
cargo run -p commerce-etl -- `
  --items-csv out\commerce-50000\items.csv `
  --orders-csv out\commerce-50000\orders.csv `
  --order-items-csv out\commerce-50000\order_items.csv `
  --inventory-csv out\commerce-50000\inventory.csv `
  --database-url $env:DATABASE_URL `
  --run-id commerce-full-local
```

### 7.5 購入傾向分析

```powershell
cargo run -p purchase-insights -- `
  --database-url $env:DATABASE_URL `
  --run-id insights-full-local `
  --evaluate
```

その後は smoke 規模と同じく、`app-api`, `desktop-ui`, reporting を確認する。

## 8. decision-engine 単体レポート確認

API 経由ではなく、`decision-engine` の最小実行例から report JSON を生成する場合は次を実行する。

```powershell
cargo run -p decision-engine
```

期待する成果物:

- `out/simulation_report_v0.1.json`

schema 互換性を確認する。

```powershell
python decision-engine\scripts\validate_report_schema.py `
  --schema decision-engine\schemas\simulation_report_v0.1.schema.json `
  --input out\simulation_report_v0.1.json
```

## 9. Reporting

Python reporting の仮想環境を用意する。

```powershell
py -3 -m venv reporting\python\.venv
reporting\python\.venv\Scripts\python.exe -m pip install -e reporting\python
```

report JSON から図表と要約を生成する。

```powershell
reporting\python\.venv\Scripts\python.exe -m decision_report.cli `
  --input out\simulation_report_v0.1.json `
  --out-dir out\reporting-local-smoke
```

期待する成果物:

- `out/reporting-local-smoke/cash_balance.png`
- `out/reporting-local-smoke/daily_stockout.png`
- `out/reporting-local-smoke/summary.txt`

## 10. 検証コマンド

コード変更後は、最低限次を実行する。

```powershell
cargo fmt --all --check
cargo test --workspace
```

`app-api` の DB-backed 統合テストを実 DB に対して実行する場合は、専用の環境変数を設定する。

```powershell
$env:APP_API_TEST_DATABASE_URL = $env:DATABASE_URL
cargo test -p app-api --test api_integration -- --nocapture
```

`APP_API_TEST_DATABASE_URL` が未設定の場合、統合テストはローカル DB に接続せず skip する。

Lean 仕様も確認する場合:

```powershell
Push-Location spec
lake build
Pop-Location
```

report JSON 契約を確認する場合:

```powershell
python decision-engine\scripts\validate_report_schema.py `
  --schema decision-engine\schemas\simulation_report_v0.1.schema.json `
  --input out\simulation_report_v0.1.json
```

## 11. 成果物と Git 管理

次はローカル生成物であり、通常は Git に載せない。

- `out/`
- `target/`
- `reporting/python/.venv/`
- full 規模の `*50000*` / `*150000*` データ

full 規模データは、生成器、seed、metadata、仕様書で再現可能にする。

## 12. よくある失敗と確認点

### DB 接続に失敗する

- `$env:DATABASE_URL` が設定されているか確認する
- PostgreSQL が起動しているか確認する
- `decision_pack_app` DB が存在するか確認する
- migration が適用済みか確認する

### `commerce-etl` が外部キー制約で失敗する

- `customers-etl` を先に実行しているか確認する
- `items` を取り込む前に `order_items` を取り込んでいないか確認する
- 生成した commerce sample が、同じ顧客 CSV から作られているか確認する

### `purchase-insights` の出力が空になる

- `orders` と `order_items` が DB に存在するか確認する
- `items.is_active = true` の商品があるか確認する
- `customers` と `orders.customer_id` が一致しているか確認する

### GUI に何も表示されない

- `app-api` が起動しているか確認する
- GUI の API 入力欄が `http://127.0.0.1:8080` を指しているか確認する
- `/health` が `database: connected` を返すか確認する

### reporting で `matplotlib` が見つからない

- system Python ではなく `reporting/python/.venv` の Python を使う
- `pip install -e reporting\python` を実行済みか確認する
