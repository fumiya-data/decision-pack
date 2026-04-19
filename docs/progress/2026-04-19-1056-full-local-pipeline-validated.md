# full local pipeline validated

## 実施内容
- `customers-etl` で `data/customers/raw/raw_customers_50000_multilingual.csv` を整形し、`out/customers-50000/` に `formatted.csv`、`format_issues.csv`、`customer_segment_summary.csv`、`run_summary.json` を出力した。
- `customers-etl` の PostgreSQL 永続化を実行し、`customers` に `50,160` 件、`customer_load_issues` に `245` 件、`etl_job_runs` に `customers-etl` 実行履歴を保存した。
- `commerce-etl --generate-sample` で `100` 商品、`150,000` 注文、`382,949` 明細、`100` 在庫行を生成し、`out/commerce-50000/` に出力した。
- `commerce-etl` で `items`、`orders`、`order_items`、`inventory_balance` を PostgreSQL に投入した。
- `purchase-insights --evaluate` を実行し、推薦評価と永続化を行った。
- `app-api` を実データ接続で起動し、顧客、購買履歴、次回購入候補、在庫、シミュレーション作成、レポート取得の API を確認した。
- `desktop-ui` をビルドし、実データ接続中の `app-api` と組み合わせて起動確認を行った。

## 実行結果
- `customers`: `50,160`
- `customer_load_issues`: `245`
- `items`: `100`
- `orders`: `150,000`
- `order_items`: `382,949`
- `inventory_balance`: `100`
- `customer_item_next_buy_score`: `250,800`
- `item_demand_forecast`: `100`
- `simulation_runs`: `1`
- `simulation_item_results`: `100`

## 推薦評価
- `eligible_customers`: `50,000`
- `customers_with_predictions`: `50,000`
- model
  - `hit@3 = 0.6179`
  - `hit@5 = 0.7578`
  - `recall@5 = 0.4481`
  - `recall@10 = 0.5617`
  - `ndcg@5 = 0.3743`
  - `ndcg@10 = 0.4236`
- popularity baseline
  - `hit@3 = 0.1737`
  - `hit@5 = 0.2831`
  - `recall@5 = 0.1209`
  - `recall@10 = 0.2409`
  - `ndcg@5 = 0.0921`
  - `ndcg@10 = 0.1417`

## 実装で補強した点
- `customers-etl` の氏名整形で、英語・日本語・中国語に加えて、ヒンディー語の結合文字を許可するよう修正した。
- `customers-etl` の PostgreSQL 永続化で、必須列欠損の整形済み行があっても ETL 全体を失敗させず、保存対象から除外して先に進めるようにした。
- `customer_load_issues` への行単位 issue 保存で、`column_name` が `NOT NULL` 制約を満たすよう `__row__` を使うようにした。

## 補足
- `formatted.csv` には整形結果として必須列欠損を含む行も残るため、PostgreSQL への保存時に `80` 行を除外した。
- full 規模の raw データはローカル検証専用であり、Git/GitHub には反映しない方針を維持する。
