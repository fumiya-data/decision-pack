# データ契約

## 1. 文書の目的

この文書は、CSV、DB、API、レポート JSON の契約を定義する。
ここで言う契約とは、モジュール間で受け渡すときに壊してはならない最小の意味単位と互換性条件を指す。

## 2. 契約設計の原則

- 契約はモジュール境界ごとに分ける
- 顧客個票と品目別集約値を混ぜない
- GUI は API 契約とレポート JSON 契約だけを見る
- DB の schema の意味は migration SQL と仕様書で管理する
- 互換性を壊す変更は新バージョンまたは明示的移行として扱う

## 3. CSV 契約

### 3.1 顧客CSV

#### 入力契約

- 入力は顧客マスタ相当の行である
- 列名や表記ゆれを含む可能性がある
- 顧客識別に必要な列が最低限存在することを期待する

#### 出力契約

- 正規化後の出力は `customers` と `customer_load_issues` に分かれる
- `customers` には業務利用可能な値だけを保持する
- `customer_load_issues` には欠損、解釈不能、補正不能などの問題を残す

### 3.2 商品・受注CSV

#### 入力契約

- 入力は `items`, `orders`, `order_items`, `inventory_balance` 相当のデータで構成される
- 同一ソースから来なくてもよいが、最終的に共通の識別子で結びつく必要がある
- 評価用標準サンプルは `50,000` 顧客、`100` 商品、`150,000` 注文を前提とする
- 詳細は `specs/sample-dataset-spec.md` に従う

#### 出力契約

- 出力は PostgreSQL テーブル群である
- 入力ソースごとの差異は adapter と staging で吸収し、正規化後テーブルへ反映する

## 4. DB 契約

### 4.1 最小テーブル

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

### 4.2 ジョブ管理系テーブル

- `etl_job_runs`

### 4.3 テーブル群の役割

- `customers`, `customer_load_issues`
  - 顧客整形結果と品質問題
- `items`, `orders`, `order_items`, `inventory_balance`, `inventory_movements`
  - 業務データ基盤
- `customer_item_next_buy_score`, `item_demand_forecast`
  - 分析成果
- `simulation_runs`, `simulation_item_results`
  - 意思決定支援結果
- `etl_job_runs`
  - 取込と分析ジョブの追跡

### 4.4 DB 互換性の原則

- schema 変更は migration で管理する
- 既存列の意味を破壊する変更は避ける
- 互換性破壊が必要な場合は、移行順序と影響範囲を明記する

## 5. API 契約

### 5.1 読み取り系

- `GET /customers`
  - 顧客一覧を返す
- `GET /customers/{customer_id}`
  - 顧客詳細を返す
- `GET /customers/{customer_id}/purchases`
  - 顧客の購入履歴を返す
- `GET /customers/{customer_id}/next-buy`
  - 顧客ごとの次回購入候補を返す
- `GET /items`
  - 品目一覧を返す
- `GET /items/{item_id}`
  - 品目詳細を返す
- `GET /items/{item_id}/inventory`
  - 品目の在庫情報を返す
- `GET /items/{item_id}/risk`
  - 品目の在庫リスク情報を返す
- `GET /simulations`
  - シミュレーション実行一覧を返す
- `GET /simulations/{run_id}`
  - 実行単位の結果概要を返す
- `GET /simulations/{run_id}/report`
  - GUI や reporting が利用するレポート成果を返す

### 5.2 実行系

- `POST /jobs/customers-etl`
  - 顧客取込ジョブを起動する
- `POST /jobs/commerce-etl`
  - 商品・受注・在庫取込ジョブを起動する
- `POST /jobs/purchase-insights`
  - 分析ジョブを起動する
- `POST /simulations`
  - シミュレーション実行を起動する

### 5.3 API 応答の意味

- 長時間ジョブは完了結果を同期で返さない
- 実行要求の成功時は `job_id` または `run_id` を返す
- GUI は返却された識別子を用いて状態確認と結果取得を行う

## 6. `decision-engine` 入力契約

### 6.1 受け取るもの

- `inventory_balance`
- `item_demand_forecast`
- `item_policy`
- `cashflow` 系データ

### 6.2 受け取らないもの

- 顧客個票
- 顧客別推薦結果そのもの
- 生の `order_item` 群

### 6.3 意味

- `decision-engine` は計算専用であり、一覧検索や個票分析の責務を持たない
- 顧客分析結果は、`item_id` 単位へ集約された需要予測として渡される

## 7. レポート JSON 契約

### 7.1 ベース契約

- ベースは `simulation_report_v0.1`
- 今後の拡張では互換性を壊さない方針を取る

### 7.2 含むべき情報

- 実行要約
- 品目別結果
- アラート
- KPI
- 成果物参照

### 7.3 既定で含めない情報

- 日次トレース全量
- 内部中間値
- デバッグ専用の計算過程

### 7.4 利用者

- GUI はこの JSON を用いて結果表示を行う
- reporting はこの JSON を用いて図表とテキスト成果物を生成する

## 8. 契約変更時の扱い

- 契約変更は文書、migration、実装の順で同期させる
- GUI と reporting に影響する変更は、事前に JSON 契約の影響範囲を整理する
- DB 契約の変更は migration と合わせて記録する
