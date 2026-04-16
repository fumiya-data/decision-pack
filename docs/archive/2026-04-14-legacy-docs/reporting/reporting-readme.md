# レポート層の計画（v0.1）

このディレクトリは「Pythonレポート先行」の実装基盤です。

## ディレクトリ構成案

- `reporting/schemas/`
  - レポートJSONの正式スキーマ
- `reporting/samples/`
  - サンプル入力JSON
- `reporting/python/`
  - PythonライブラリとCLI

この構成にする理由:

1. UI（Iced）から独立して、先にアウトプット品質を検証できる
2. スキーマを固定してからUIに渡せる
3. Python側で図の試行錯誤がしやすい

## JSON スキーマ

- ファイル: `schemas/simulation_report_v0.1.schema.json`
- 目的: `engine` と `python` と `ui` のデータ契約を固定する
- 主要セクション:
  - `scenario`
  - `kpi`
  - `alerts`
  - `cash_series`
  - `inventory_series`

## Python レポート CLI

- 実装: `python/src/decision_report/`
- エントリポイント: `decision-report`
- 入力: `simulation_report_v0.1` JSON
- 出力:
  - `cash_balance.png`
  - `daily_stockout.png`
  - `summary.txt`

## 実行手順

```bash
cd reporting/python
python -m pip install -e .
decision-report --input ../samples/simulation_report_v0.1.sample.json --out-dir ./out
```

## 次の実装タスク

1. `engine` 側で `simulation_report_v0.1` 構造の直接出力を実装
2. スキーマ検証をCIに追加
3. UI側で `out` 画像/JSONの読み込み表示を実装
