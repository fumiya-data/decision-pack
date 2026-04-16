# Decision Pack 進捗記録（2026-03-05）

## 1. プロジェクトの前提

- 対象: 小規模事業者向けの意思決定支援ソフトウェア
- 中核目標:
  - キャッシュフロー予測
  - 在庫最適化
- 技術スタック:
  - 仕様: Lean（`spec/`）
  - エンジン: Rust（`engine/`）
  - レポート: Python（`reporting/`）
  - UI: Rust + Iced（`ui/`）

## 2. 完了済みマイルストーン

### 2.1 仕様（Lean）

- 在庫と資金の仕様ファイルを見直し、整合を取り直した。
- プレースホルダが残っていた不変条件まわりの証明を補完した。
- 日本語の文書コメントを整理した。
- 在庫結果名を含む命名と整合性の揺れを解消した。

対象ファイル:

- `spec/DecisionSpec/Common.lean`
- `spec/DecisionSpec/Inventory.lean`
- `spec/DecisionSpec/Cashflow.lean`

### 2.2 プロダクト・計画文書

- `spec` 配下で製品要件 v0.1 を整理した。
- README と TODO を日本語基準で整理した。
- 見出しと文書体裁を統一した。

対象ファイル:

- `spec/README_ja.md`
- `spec/TODO_ja.md`
- `engine/TODO_ja.md`

### 2.3 エンジン（Rust）

- コアとなるドメイン型を実装した。
- Lean の意味に合わせた決定論的な在庫・資金更新を実装した。
- レポート出力を `simulation_report_v0.1` スキーマへ合わせた。
- 生成レポート JSON を整形出力できるようにした。
- ゴールデンスモークテストを追加し、通過状態にした。
- JSON Schema 検証スクリプトと CI ワークフローを追加した。

対象ファイル:

- `engine/src/domain/*`
- `engine/src/sim/*`
- `engine/src/report/types.rs`
- `engine/src/report/json.rs`
- `engine/tests/golden_smoke.rs`
- `engine/schemas/simulation_report_v0.1.schema.json`
- `engine/scripts/validate_report_schema.py`
- `engine/.github/workflows/ci.yml`

### 2.4 レポート層（Python）

- レポート用サブプロジェクトを立ち上げた。
- 正式スキーマとサンプル JSON を追加した。
- 図表成果物を生成する Python CLI を実装した。
- CLI の主な成果物:
  - `cash_balance.png`
  - `daily_stockout.png`
  - `summary.txt`

対象ファイル:

- `reporting/schemas/simulation_report_v0.1.schema.json`
- `reporting/samples/simulation_report_v0.1.sample.json`
- `reporting/python/src/decision_report/*`
- `reporting/README_ja.md`

### 2.5 UI（Iced）

- `ui/` 配下に Cargo プロジェクトを初期化した。
- Iced ベースの MVP ウィンドウを実装した。
- JSON 読込とステータス・エラー表示を追加した。
- KPI と計算済みサマリの表示を追加した。
- アラート表示を追加した。
- 成果物操作を追加した。
  - Python CLI（`uv run decision-report`）による成果物生成
  - UI から生成済み PNG/TXT を開く操作

対象ファイル:

- `ui/src/main.rs`
- `ui/Cargo.toml`
- `ui/docs/MVP_UI_v0.md`
- `ui/TODO_iced.md`

## 3. 現在の統合状態

- エンジンは `engine/out/simulation_report_v0.1.json` を生成できる。
- 生成 JSON はスキーマ検証を通過する。
- レポート CLI はエンジン出力 JSON から成果物を生成できる。
- UI は JSON を読み込み、成果物生成と成果物オープンを操作できる。

## 4. 運用メモ

- 当時は `spec` `engine` `ui` `reporting` がディレクトリ単位で分かれていた。
- ローカル git 操作に関する ownership / safe-directory 問題へ対応した。
- `uv` 利用時は環境によってキャッシュ権限調整が必要になる場合がある。

## 5. 次に進めるべき事項

1. `engine` に CSV 取り込みを実装する（`csv` / `serde` / `chrono` 系の経路）。
2. シナリオ実行ループと比較出力を追加する。
3. モンテカルロを補助関数レベルから反復実行まで拡張する。
4. UI の状態管理を強化する（読込状態、ボタン活性制御、結果パネル）。
5. `reporting/python` 向けの CI（lint + schema + 描画スモーク）を追加する。

## 6. 基盤完了とみなせる理由

基盤フェーズは、次の理由から完了と判断できる。

- `spec` `engine` `reporting` `ui` の縦断経路が成立している。
- スキーマ契約が存在し、検証も通っている。
- 利用者は GUI からレポートを読み込み、意思決定に使う成果物を確認できる。
