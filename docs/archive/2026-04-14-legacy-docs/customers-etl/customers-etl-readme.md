# customers_etl

`customers_etl` は、汚れた顧客エクスポートを整形し、意思決定支援側へ渡す安定したバッチ成果物を生成するコンポーネントです。

## 出力物

- `formatted.csv`
- `format_issues.csv`
- `customer_segment_summary.csv`
- `run_summary.json`

## 実行方法

ワークスペースのルートで次を実行します。

```powershell
cargo run -p customers-etl -- --input data/customers/raw/dirty_customers_5000.csv --output-dir out/customers-etl/dev --run-id dev
```

意思決定支援側へ渡すのは集約済みの顧客データのみです。個別の顧客 ID は `engine` へ持ち込みません。
