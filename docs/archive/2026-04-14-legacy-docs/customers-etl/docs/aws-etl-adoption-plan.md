# AWS への ETL 導入計画書（バージョン 0.1）

## 1. 文書の目的

本計画書は、既存のRust製CSV整形処理をベースに、低コストかつ運用可能なAWS ETLパイプラインを段階的に構築するための実行計画を定義する。

バージョン 1 の最終到達点は以下とする。

- S3上の生CSVを入力として処理を開始できる
- Rustの前処理ロジックをローカルとAWSの両方で同じコードパスで実行できる
- `formatted.csv`、`format_issues.csv`、`customer_segment_summary.csv`、`run_summary.json` をS3へ出力できる
- 各実行を `run_id` 単位で追跡できる
- EventBridgeで定期実行できる
- CloudWatch Logsから障害原因を追跡できる
- 再実行時に過去成果物を壊さず安全にやり直せる

---

## 2. 現状整理

現時点のRustプロジェクトは、ETLのコアとなる整形処理をすでに持っている。

- `src/formatter.rs` に `format_dataset` があり、入力文字列から整形済み行と集計レポートを返せる
- `src/report.rs` に、列別統計・行スキップ・不正行・フィールド失敗の集計ロジックがある
- `src/lib.rs` は `FormatRun` と `format_dataset` を再公開しており、ライブラリ化の土台がある
- `src/main.rs` は入力・出力ファイルパスを定数で固定しており、任意パス実行やクラウド実行に未対応である
- モノレポ移行後のローカル fixture は `data/customers/raw/raw_customers_5000.csv` を入力とし、`formatted.csv`、`format_issues.csv`、`customer_segment_summary.csv`、`run_summary.json` を標準成果物とする

このため、既存コードを全面書き換えするのではなく、**コア整形ロジックは維持しつつ、実行設定・出力契約・運用機能を追加する** 方針で進める。

---

## 3. 計画上の重要方針

### 3.1 実行方式

元計画では「ローカルとAWSで同じ実行ファイルを使う」とされていたが、開発環境がWindows、Lambda実行環境がLinuxであるため、**完全に同一のバイナリを両環境で使う** ことは現実的ではない。

そのため、本計画では以下を正式方針とする。

- 同一のRustコードベースを使う
- 同一の入出力契約と設定仕様を使う
- コア整形処理は共通ライブラリとして維持する
- ローカルCLI実行とLambda実行は、必要に応じて薄いエントリポイントを分ける

### 3.2 データ正本

- バージョン 1 では **S3 を入力・出力の正本** とする
- PostgreSQL はローカル学習・検証用のオプション機能とし、AWS必須依存にはしない

### 3.3 再実行安全性

- すべての実行に一意な `run_id` を付与する
- 出力は `run_id` 単位のプレフィックスへ保存し、過去成果物を上書きしない
- 手動再実行は「同じ入力・新しい `run_id`」で行う

### 3.4 コスト方針

- 常時起動のEC2やRDSは導入しない
- バージョン 1 の AWS 構成は **S3 + Lambda + EventBridge + CloudWatch Logs/Alarm** に限定する
- ECS Fargate は Lambda で性能不足が確認された場合のみ検討する

---

## 4. バージョン 1 の対象範囲

### 対象

- 1ファイル単位のCSV整形バッチ
- S3からの入力取得
- S3への成果物保存
- 実行メタデータのJSON保存
- CloudWatch Logsによるログ確認
- EventBridgeによる定期実行
- 手動再実行手順の整備
- PostgreSQLへの任意ロード機能

### 対象外

- 複数ファイル並列実行
- ワークフローエンジン導入（Step Functions、Airflow等）
- RDS本番導入
- リアルタイム処理
- 入力ファイル到着イベントに応じた自動起動
- ダッシュボードUI構築

---

## 5. 目標アーキテクチャ

### 5.1 構成要素

| コンポーネント | 役割 |
| --- | --- |
| S3 | 生CSV、整形済みCSV、問題ログ、実行メタデータの保管 |
| Rust ETL コア | CSV 整形、列別集計、問題抽出 |
| Lambda | S3から入力取得、ETL実行、S3へ成果物保存 |
| EventBridge | 定期起動 |
| CloudWatch Logs | アプリケーションログ、障害追跡 |
| CloudWatch Alarms | 基本障害監視 |
| PostgreSQL（任意） | 学習・検証用ロード先、`CustomerID` ベースのupsert検証 |

### 5.2 処理フロー

1. 生CSVをS3の `raw/` 配下へ配置する
2. EventBridgeまたは手動起動によりLambdaを実行する
3. Lambdaは対象S3キーと `run_id` を受け取る
4. Lambdaは入力CSVを `/tmp/input.csv` にダウンロードする
5. 共通Rustロジックで整形を実行する
6. `/tmp/` に `formatted.csv`、`format_issues.csv`、`run_summary.json` を生成する
7. 生成物を `processed/` 配下の `run_id` プレフィックスへアップロードする
8. 成功・失敗をCloudWatch Logsと `run_summary.json` で確認できるようにする

---

## 6. データ契約

### 6.1 S3キー設計

```text
raw/customers/date=YYYY-MM-DD/source.csv
processed/customers/date=YYYY-MM-DD/run_id=<RUN_ID>/formatted.csv
processed/customers/date=YYYY-MM-DD/run_id=<RUN_ID>/format_issues.csv
processed/customers/date=YYYY-MM-DD/run_id=<RUN_ID>/run_summary.json
processed/customers/date=YYYY-MM-DD/run_id=<RUN_ID>/_SUCCESS
processed/customers/date=YYYY-MM-DD/latest.json
```

補足:

- `_SUCCESS` は正常終了時のみ作成する
- `latest.json` は最新成功実行の `run_id` とS3キーを示す任意マニフェストとする
- 再実行時は `run_id` を変えるため、過去ファイルを壊さない

### 6.2 Lambdaイベント契約

手動起動・EventBridge起動とも、最終的にLambdaへ渡す入力契約は以下に統一する。

```json
{
  "job_name": "customers_formatter",
  "run_id": "20260412T010000Z_customers",
  "input_bucket": "example-etl-bucket",
  "input_key": "raw/customers/date=2026-04-12/source.csv",
  "output_bucket": "example-etl-bucket",
  "output_prefix": "processed/customers/date=2026-04-12/run_id=20260412T010000Z_customers/"
}
```

### 6.3 `run_summary.json` の標準構造

```json
{
  "job_name": "customers_formatter",
  "run_id": "20260412T010000Z_customers",
  "status": "succeeded",
  "started_at": "2026-04-12T01:00:00Z",
  "finished_at": "2026-04-12T01:00:08Z",
  "input": {
    "bucket": "example-etl-bucket",
    "key": "raw/customers/date=2026-04-12/source.csv"
  },
  "output": {
    "bucket": "example-etl-bucket",
    "prefix": "processed/customers/date=2026-04-12/run_id=20260412T010000Z_customers/"
  },
  "counts": {
    "data_rows_seen": 5000,
    "rows_written": 5000,
    "rows_with_failures": 237,
    "skipped_rows": 12,
    "malformed_rows": 3
  },
  "artifacts": {
    "formatted_csv": "formatted.csv",
    "format_issues_csv": "format_issues.csv"
  },
  "executor": {
    "mode": "lambda",
    "request_id": "aws-request-id"
  },
  "error": null
}
```

失敗時は `status` を `failed` にし、`error` へメッセージを入れる。

### 6.4 実行時設定

| 設定名 | 用途 | 例 |
| --- | --- | --- |
| `--input` | ローカル入力ファイル | `data/customers/raw/raw_customers_5000.csv` |
| `--output-dir` | ローカル出力先ディレクトリ | `./out/2026-04-12T010000Z` |
| `--run-id` | 実行識別子 | `20260412T010000Z_local` |
| `--load-postgres` | DBロード有無 | `true` / `false` |
| `DATABASE_URL` | PostgreSQL接続先 | `postgres://...` |
| `INPUT_BUCKET` | Lambda入力S3バケット | `example-etl-bucket` |
| `OUTPUT_BUCKET` | Lambda出力S3バケット | `example-etl-bucket` |
| `LOG_LEVEL` | ログ粒度 | `info` |

---

## 7. Rustプロジェクトの具体的変更方針

### 7.1 変更対象

| ファイル/モジュール | 変更内容 |
| --- | --- |
| `src/main.rs` | 固定パス定数を廃止し、CLI設定を受けて実行するオーケストレータへ変更 |
| `src/lib.rs` | 共通利用する型と実行関数を再公開 |
| `src/report.rs` | `run_summary.json` に必要なシリアライズ可能な要約構造を追加 |
| `src/formatter.rs` | コア整形ロジックは基本維持、必要に応じてメタデータ生成用情報を追加 |
| `src/config.rs`（新規） | CLI引数・環境変数の解釈 |
| `src/output.rs`（新規） | `formatted.csv`、`format_issues.csv`、`run_summary.json` の書き出し |
| `src/db.rs`（新規・任意） | PostgreSQLロードと `etl_job_runs` 更新 |
| `src/bin/lambda.rs` または同等の薄いアダプタ（新規） | Lambdaイベント受信、S3入出力、共通コア呼び出し |

### 7.2 実装方針

- `format_dataset` は純粋関数として維持する
- ファイルI/OとS3 I/Oはコア処理の外に出す
- 失敗しても何が起きたか追えるように、ログと `run_summary.json` を必ず残す
- ローカルCLIとLambdaは共通の出力フォーマットを使う

### 7.3 追加推奨依存関係

- CLI引数処理
- JSONシリアライズ
- 構造化ログ
- AWS S3 / Lambda実行用SDK
- PostgreSQL接続ライブラリ

依存関係の導入はフェーズごとに最小限に行い、一度に広げすぎない。

---

## 8. PostgreSQLの最小設計（任意機能）

バージョン 1 では DB 連携は必須ではないが、ローカル検証用として以下の最小設計を採用する。

### 8.1 テーブル

#### `etl_job_runs`

- `run_id` : 実行識別子、主キー
- `job_name` : ジョブ名
- `input_uri` : 入力ファイル識別子
- `output_prefix` : 出力先識別子
- `status` : `running` / `succeeded` / `failed`
- `started_at` : 開始時刻
- `finished_at` : 終了時刻
- `data_rows_seen` : データ行数
- `rows_written` : 出力行数
- `rows_with_failures` : フィールド失敗を含む行数
- `skipped_rows` : スキップ行数
- `malformed_rows` : 不正行数
- `error_message` : 障害時メッセージ

#### `customers_cleaned`

- 整形済みCSVの19列を保持する
- `CustomerID` を主キーにする
- `source_run_id` を保持して、どの実行が書いたデータか追跡できるようにする
- `updated_at` を保持する

### 8.2 書き込み方式

- `CustomerID` をキーに `INSERT ... ON CONFLICT DO UPDATE` を使う
- `--load-postgres=false` の場合はDB処理を完全にスキップする
- DBロード失敗時の扱いはフェーズ2で明確化する

バージョン 1 の推奨仕様:

- コアETL成功を優先する
- DBロードはオプションであり、無効化時でもジョブ全体は成功扱いにできる
- DBロードを有効にした場合は、DBエラーを `run_summary.json` とログへ残す

---

## 9. フェーズ別実行計画

### フェーズ 0: 設計凍結

#### 目的

実装前に、入出力契約・成果物名・`run_id` 規則・AWS構成を固定する。

#### 作業

- 本計画書を実装基準としてレビューする
- S3キー命名規則を確定する
- `run_summary.json` の項目を確定する
- ローカルCLI契約とLambdaイベント契約を確定する
- PostgreSQLを任意機能とする境界を明確化する

#### 完了条件

- 主要な命名規則が変更不要な状態になる
- 実装者が迷わず着手できる

### フェーズ 1: Rust バッチの再構成

#### 目的

現在の固定パス実装を、任意入力・任意出力先で動く再利用可能なバッチへ変える。

#### 作業

- `src/main.rs` から固定定数を除去する
- CLI引数または環境変数から入力・出力・`run_id` を受け取る
- `formatted.csv`、`format_issues.csv`、`run_summary.json` を出力する
- 実行開始・終了・件数を標準出力にも出す
- 任意の作業ディレクトリから実行できることを確認する
- 既存の `format_dataset` が壊れていないことを `cargo test` で確認する

#### 成果物

- 任意入力ファイルを処理できるローカルCLI
- 標準化された3つの成果物

#### 完了条件

- 以下のようなコマンドで実行できる

```bash
cargo run -p customers-etl -- --input data/customers/raw/raw_customers_5000.csv --output-dir out/customers-etl/2026-04-12T010000Z --run-id 20260412T010000Z_local
```

- 出力先に `formatted.csv`、`format_issues.csv`、`customer_segment_summary.csv`、`run_summary.json` が生成される

### フェーズ 2: ローカル運用機能の追加

#### 目的

AWSに上げる前に、実行追跡と任意DBロードまで含めた運用可能なバッチにする。

#### 作業

- `etl_job_runs` のレコード更新処理を追加する
- `customers_cleaned` へのupsert処理を追加する
- `--load-postgres` フラグでDBロードON/OFFを切り替える
- DB接続失敗・upsert失敗の挙動をログとメタデータに残す
- ローカルで「入力 → 整形 → CSV出力 → 任意DBロード」を通しで確認する

#### 成果物

- ローカル完結の検証フロー
- DB有効/無効の両パスの動作確認記録

#### 完了条件

- DB無効時にETLが成功する
- DB有効時に `CustomerID` ベースのupsertが動作する
- `etl_job_runs` で実行結果を追跡できる

### フェーズ 3: AWS ストレージ・権限・実行契約の確定

#### 目的

最小限の AWS サービスだけでバージョン 1 を成立させるための準備を行う。

#### 作業

- S3バケットとプレフィックスを作成する
- Lambda実行ロールを作成する
- 必要最小限のIAMポリシーを定義する
- CloudWatch Logs の出力先を確認する
- 生CSVをS3へ手動アップロードする
- 手動で「ダウンロード → ローカル実行 → 再アップロード」を行い、S3キー設計が妥当か検証する

#### 必要権限

- 入力 `raw/` プレフィックスの `GetObject`
- 出力 `processed/` プレフィックスの `PutObject`
- CloudWatch Logs の書き込み

#### 成果物

- S3上のファイル配置ルール
- Lambdaが必要とする権限一覧
- 手動検証結果

#### 完了条件

- 入出力S3キーに迷いがない
- IAM権限が過不足なく定義できている

### フェーズ 4: Lambda への実装・デプロイ

#### 目的

Rust ETLをAWS Lambdaで実行し、S3からS3へ成果物を戻せるようにする。

#### 作業

- Lambda用の薄いアダプタを実装する
- Lambdaはイベントから `input_bucket` / `input_key` / `output_prefix` / `run_id` を受け取る
- 入力CSVをS3から `/tmp/input.csv` に取得する
- 共通ETLコアで処理する
- 成果物をS3へアップロードする
- 正常終了時のみ `_SUCCESS` を作成する
- 失敗時も `run_summary.json` を可能な範囲で更新する
- CloudWatch Logsに `run_id`、入力S3キー、件数、失敗理由を出力する

#### 初期設定値

- Lambdaメモリ: 512 MB から開始
- タイムアウト: 60 秒から開始
- 同時実行数: 1で開始してよい

#### 成果物

- AWS上で動作する初版ETL
- 手動Lambda実行結果

#### 完了条件

- 正常系で3成果物と `_SUCCESS` がS3へ保存される
- 異常系でCloudWatch Logsから原因を追える
- `run_summary.json` に失敗内容が残る

### フェーズ 5: 定期実行・監視・運用手順整備

#### 目的

「一度動く」状態から、「繰り返し安全に運用できる」状態へ進める。

#### 作業

- EventBridgeスケジュールを設定する
- 日次実行時のイベントペイロードを固定する
- CloudWatch Alarm を設定する
- 再実行手順を文書化する
- ログ確認手順を文書化する
- 出力確認手順を文書化する
- 失敗時の一次切り分け手順を文書化する

#### 最低限入れる監視

- Lambda `Errors` が発生した場合の通知
- Lambda実行時間がタイムアウト近傍に達した場合の通知

#### 成果物

- スケジュール設定
- CloudWatch Alarm
- 簡易ランブック

#### 完了条件

- 定期実行で日次ジョブが起動する
- 失敗時に確認すべき場所が1ページで分かる
- 手動再実行が運用者1名で実施できる

---

## 10. 運用設計

### 10.1 ログ方針

CloudWatch Logsには最低限以下を出力する。

- `run_id`
- `job_name`
- `input_bucket`
- `input_key`
- `output_prefix`
- 開始時刻 / 終了時刻
- 行数集計
- 失敗時の例外メッセージ

### 10.2 再実行方針

- 同じ `input_key` を使ってよい
- `run_id` は必ず新しく採番する
- 再実行結果は新しい `processed/.../run_id=<RUN_ID>/` に出す
- `latest.json` を更新するのは成功実行のみとする

### 10.3 ランブックに必ず含める項目

- 手動起動の方法
- ログの見方
- 出力S3キーの見方
- `format_issues.csv` の確認方法
- `run_summary.json` の見方
- 失敗時の一次切り分け手順

---

## 11. 受け入れ条件

バージョン 1 完了判定は以下とする。

- ローカルCLIが任意入力・任意出力先で動作する
- 出力ファイル名が `formatted.csv`、`format_issues.csv`、`run_summary.json` に統一される
- LambdaがS3入力を処理し、S3へ成果物を返せる
- `run_id` 単位で実行結果を追跡できる
- EventBridgeで日次起動できる
- CloudWatch Logsだけで障害原因の一次特定ができる
- 同じ入力ファイルを安全に再実行できる
- PostgreSQL機能を無効化してもETL本体は成功する

---

## 12. 判断ゲートとリスク

### 12.1 Lambda 継続判断

以下のいずれかに該当した場合は、バージョン 1 完了後に ECS Fargate への移行を検討する。

- Lambdaタイムアウトに近い実行が継続する
- `/tmp` 容量やメモリ制約が処理の前提を壊す
- 単一実行で扱うCSVサイズがLambda運用に不向きになる

### 12.2 主なリスク

| リスク | 内容 | 対応 |
| --- | --- | --- |
| 実行契約の曖昧さ | ローカルとAWSで引数仕様がずれる | CLI 契約と Lambda 契約をフェーズ 0 で固定する |
| 失敗追跡不足 | CSV出力だけでは失敗が見えない | `run_summary.json` とCloudWatch Logsを必須化する |
| 再実行時の上書き | 同じ出力先へ書くと事故が起きる | `run_id` プレフィックスを必須にする |
| DB依存の増大 | DB障害でETL全体が止まる | PostgreSQLを任意機能に留める |
| Lambda移植コスト | Windows開発とLinux実行の差異 | 共通コア + 薄い実行アダプタに分離する |

---

## 13. 想定工数

| フェーズ | 目安 |
| --- | --- |
| フェーズ 0 | 0.5日 |
| フェーズ 1 | 2日 |
| フェーズ 2 | 1.5日 |
| フェーズ 3 | 1日 |
| フェーズ 4 | 2日 |
| フェーズ 5 | 1日 |
| 合計 | 約8日 |

---

## 14. 最終的なバージョン 1 の姿

バージョン 1 完了時点では、以下の状態になっていることを目標とする。

- 生CSVをS3へ置けば、定期または手動でRust ETLを起動できる
- 整形済みCSV、問題ログ、実行メタデータがS3に揃う
- どの実行が成功し、どの実行が失敗したかを `run_id` 単位で追える
- 失敗時はCloudWatch Logsで一次原因を特定できる
- 再実行しても過去成果物を壊さない
- PostgreSQL連携は任意で切り替えられる

この構成は、低コスト・低複雑性・運用現実性のバランスが良く、現行のRustコードベースを活かしながら段階的にクラウド運用へ移行する最初の到達点として妥当である。
