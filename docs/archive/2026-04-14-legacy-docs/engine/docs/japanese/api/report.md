# report

シミュレーション結果を UI や外部出力で扱いやすい形にまとめます。

## 型

- `DailyCashPoint { date, cash }`
- `DailyInventoryPoint { date, item_id, on_hand, stockout }`
- `SimulationReport { cash_series, inventory_series }`

## JSON 変換

## `json::to_json(report) -> String`

- `SimulationReport` を JSON 文字列へ変換
- 依存を増やさない軽量実装
- 現在はプロトタイプ用途（将来は正式スキーマ化予定）

## 使い方

1. `SimulationReport` を構築
2. `to_json` を呼ぶ
3. UI やファイル保存へ渡す
