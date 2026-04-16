# AWS 実行アーキテクチャ

## 1. 基本方針

- GUI は薄いクライアントに保つ
- ETL、分析、シミュレーションは AWS 側のジョブとして実行する
- GUI は AWS の個別サービスを直接叩かず、`app-api` を通す

## 2. 構成要素

- `S3`
  - raw データと成果物の保管
- `RDS for PostgreSQL`
  - 業務データと分析結果の保管
- `API Gateway`
  - GUI の入口
- `Lambda`
  - 軽量 API ハンドラ
- `Step Functions`
  - ジョブのオーケストレーション
- `ECS/Fargate`
  - ETL、分析、シミュレーションの実行
- `EventBridge Scheduler`
  - 夜間ジョブや定期更新
- `CloudWatch`
  - ログと監視

## 3. フロー

1. GUI が `app-api` を呼ぶ
2. `app-api` が PostgreSQL または S3 を参照する
3. 重い処理が必要なら `Step Functions` でジョブを起動する
4. ジョブは `ECS/Fargate` 上で実行する
5. 結果は PostgreSQL と S3 に保存する
6. GUI は `run_id` または `job_id` で状態確認する

## 4. 守るべきこと

- GUI から RDS へ直接接続しない
- GUI から S3 を直接参照しない
- GUI から ECS や Step Functions を直接呼ばない
