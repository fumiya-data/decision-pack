# decision-report（Python）

`simulation_report_v0.1` JSON から図表と要約テキストを生成する Python 補助パッケージです。

## インストール（editable）

```bash
cd reporting/python
python -m pip install -e .
```

## CLI の使い方

```bash
decision-report \
  --input ../samples/simulation_report_v0.1.sample.json \
  --out-dir ./out
```

生成物:
- `cash_balance.png`
- `daily_stockout.png`
- `summary.txt`
