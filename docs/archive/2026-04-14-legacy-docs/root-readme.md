# Decision Pack モノレポ

Decision Pack は、在庫・資金の意思決定支援、顧客データ前処理、レポート生成、形式仕様を 1 つのリポジトリで扱うモノレポです。

## ディレクトリ構成

- `engine/`: 在庫と資金のシミュレーションを担う Rust エンジン
- `ui/`: レポート読込と成果物表示を行う iced デスクトップ UI
- `customers_etl/`: 顧客データを整形・集約する Rust 製 ETL
- `reporting/`: `engine` が出力した JSON から図表を生成する Python レイヤー
- `spec/`: Lean による形式仕様
- `data/`: 追跡対象の入力サンプルと期待成果物

## 設計メモ

- モジュール責務と境界: `docs/モジュール責務と境界.md`

## よく使うコマンド

```powershell
cargo test --workspace
cargo run -p customers-etl -- --input data/customers/raw/raw_customers_5000.csv --output-dir out/customers-etl/dev --run-id dev
cargo run -p engine
```
