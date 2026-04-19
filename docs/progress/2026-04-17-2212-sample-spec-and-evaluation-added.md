# 2026-04-17 22:12 サンプル仕様と推薦評価追加

## 実施内容

- 評価用標準サンプルの件数を `50,000` 顧客、`100` 商品、`150,000` 注文に固定した。
- `docs/specs/sample-dataset-spec.md` を追加し、件数、分布、再購入条件、カテゴリ構成、生成成果物を定義した。
- `docs/specs/recommendation-evaluation-spec.md` を追加し、`leave-last-order-out` を用いた推薦評価方式と `Hit@3`, `Hit@5`, `Recall@5`, `Recall@10`, `NDCG@5`, `NDCG@10` を定義した。
- `commerce-etl` に `--generate-sample` モードを追加し、整形済み顧客 CSV から `items.csv`, `orders.csv`, `order_items.csv`, `inventory.csv`, `sample_metadata.json` を生成できるようにした。
- `purchase-insights` に `--evaluate` と `--skip-persist` を追加し、オフライン推薦評価と人気順ベースライン比較を実行できるようにした。

## 確認結果

- `cargo test -p commerce-etl` が成功した。
- `cargo test -p purchase-insights` が成功した。

## 補足

- 現在のリポジトリにある整形済み顧客 CSV は `5,000` 件版であり、`50,000` 件版の full サンプル生成には `50,000` 件の整形済み顧客 CSV が別途必要である。
