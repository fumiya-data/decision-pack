# io.csv

CSV 入力スキーマの行型と軽量バリデーションを提供します。

## テーブル名定数

- `ITEMS`
- `SALES_DAILY`
- `INVENTORY_DAILY`
- `STAFF`
- `CASHFLOW_DAILY`

## 行型

- `ItemRow`
- `SalesDailyRow`
- `InventoryDailyRow`
- `CashflowDailyRow`

## バリデーション関数

## `validate_item_row(row)`

- `item_id` が空でないことを検証
- エラー: `CsvValidationError::EmptyItemId`

## `validate_cashflow_row(row)`

- `direction` が `"in"` または `"out"` であることを検証
- エラー: `CsvValidationError::InvalidDirection`

## 注意

現状は「行単位の基本検証」のみです。  
ファイル読み込み、型変換、主キー重複チェックは今後実装予定です。
