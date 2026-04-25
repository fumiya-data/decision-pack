# ローカル運用 runbook 追加

## 実施内容

- `docs/operations/local-operations-runbook.md` を追加した
- ローカルでの PostgreSQL 作成、migration 適用、顧客 ETL、商取引サンプル生成、商取引 ETL、購入傾向分析、API、GUI、reporting の実行順を文書化した
- smoke 規模と full 規模の 2 系統の実行手順を分けて記載した
- `decision-engine` 単体の report JSON 生成と schema 検証手順を追記した
- Python reporting の仮想環境作成と成果物生成手順を追記した
- `docs/index.md` に運用手順セクションを追加し、新しい runbook と本進捗記録をリンクした

## 確認結果

- `cargo fmt --all --check` が成功した
- `cargo test --workspace` が成功した
- `lake build` が成功した
- `out/simulation_report_v0.1.json` が `decision-engine/schemas/simulation_report_v0.1.schema.json` に適合することを確認した
- `reporting/python/.venv` の Python を使い、reporting CLI で図表と要約を生成できることを確認した

## 補足

- system Python では `matplotlib` が見つからなかったため、reporting の確認には既存の `reporting/python/.venv` を使った
- 現時点では migration runner が未実装であるため、runbook では `psql` による直接適用を正本手順として記載した
