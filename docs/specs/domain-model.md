# ドメインモデル

## 1. 主要ドメイン

- 顧客ドメイン
- 商品・販売ドメイン
- 在庫ドメイン
- 購入傾向分析ドメイン
- 意思決定支援ドメイン

## 2. 主要エンティティ

### 顧客ドメイン

- `customer`
- `customer_load_issue`

### 商品・販売ドメイン

- `item`
- `order`
- `order_item`

### 在庫ドメイン

- `inventory_balance`
- `inventory_movement`
- `item_policy`

### 購入傾向分析ドメイン

- `customer_item_next_buy_score`
- `item_demand_forecast`

### 意思決定支援ドメイン

- `simulation_run`
- `simulation_item_result`
- `simulation_report`

## 3. 境界

- 顧客個票は分析系までで扱う
- `decision-engine` は顧客個票を持たない
- `decision-engine` が受けるのは `item_id` 単位の需要予測と在庫情報

## 4. 識別子の方針

- DB と API の識別子は `snake_case` の英語名を使う
- 個票の `customer_id` は分析系と業務DBで使う
- `decision-engine` は `item_id`, `scenario_id`, `run_id` を中心に扱う
