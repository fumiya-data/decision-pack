# データ契約

## 1. CSV 契約

### 顧客CSV

- 入力: 顧客マスタ相当の行
- 出力:
  - `customers`
  - `customer_load_issues`

### 商品・受注CSV

- 入力:
  - `items`
  - `orders`
  - `order_items`
  - `inventory_balance`
- 出力:
  - PostgreSQL テーブル群

## 2. DB 契約

### 最小テーブル

- `customers`
- `customer_load_issues`
- `items`
- `inventory_balance`
- `inventory_movements`
- `orders`
- `order_items`
- `customer_item_next_buy_score`
- `item_demand_forecast`
- `simulation_runs`
- `simulation_item_results`

### ジョブ管理系テーブル

- `etl_job_runs`

## 3. API 契約

### 読み取り系

- `GET /customers`
- `GET /customers/{customer_id}`
- `GET /customers/{customer_id}/purchases`
- `GET /customers/{customer_id}/next-buy`
- `GET /items`
- `GET /items/{item_id}`
- `GET /items/{item_id}/inventory`
- `GET /items/{item_id}/risk`
- `GET /simulations`
- `GET /simulations/{run_id}`
- `GET /simulations/{run_id}/report`

### 実行系

- `POST /jobs/customers-etl`
- `POST /jobs/commerce-etl`
- `POST /jobs/purchase-insights`
- `POST /simulations`

## 4. レポート JSON 契約

- ベースは `simulation_report_v0.1`
- 今後の拡張では互換性を壊さない方針を取る
- UI と reporting はこの JSON 契約に依存する
