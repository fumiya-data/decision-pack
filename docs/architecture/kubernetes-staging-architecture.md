# Kubernetes staging 実行アーキテクチャ

## 1. 基本方針

- GUI は薄いクライアントに保つ
- ETL、分析、シミュレーションは Kubernetes Job または CronJob として実行する
- GUI は Kubernetes 内部の個別サービスを直接叩かず、`app-api` を通す
- staging は無料運用を優先し、固定費が発生しやすい managed cloud 構成は後回しにする
- Docker Compose で確認した container image と環境変数を Kubernetes manifest へ移す

## 2. 構成要素

- `Deployment`
  - `app-api` を常時起動する
- `Service`
  - GUI または `kubectl port-forward` から `app-api` へ到達する入口
- `Job`
  - migration、ETL、分析、シミュレーション、reporting の一回実行
- `CronJob`
  - 夜間ジョブや定期更新
- `ConfigMap`
  - 入力パス、出力パス、ログ設定、実行モードなどの非機密設定
- `Secret`
  - DB 接続文字列などの機密設定
- `PersistentVolume`
  - PostgreSQL データ、raw データ、成果物の保持
- `kubectl logs`
  - staging 段階のログ確認

## 3. フロー

1. GUI が `app-api` を呼ぶ
2. `app-api` が PostgreSQL または成果物 volume を参照する
3. 重い処理が必要なら `app-api` が Kubernetes Job を起動する
4. 定期処理は CronJob として実行する
5. 結果は PostgreSQL と成果物 volume に保存する
6. GUI は `run_id` または `job_id` で状態確認する

## 4. 守るべきこと

- GUI から PostgreSQL へ直接接続しない
- GUI から成果物 volume を直接参照しない
- GUI から Kubernetes API や個別 Job を直接呼ばない
- Kubernetes staging で固めるのは container 境界、環境変数、永続化、ジョブ起動方式であり、本番 managed cloud の最終構成ではない
