# Docker 習熟ロードマップ

## 1. 目的

このロードマップは、Decision Pack を題材にして Docker を実務的に習熟するための順序を固定する。
抽象的な Docker 学習ではなく、PostgreSQL、migration、ETL、分析、API、reporting を Docker でローカル運用できることを第一目標にする。

AWS 実装へ進む前に、同じコンテナイメージと同じ環境変数でローカル再現できる状態を作る。

## 2. 到達目標

最初の到達点は次の状態である。

- Docker Desktop と Docker Compose を使える
- PostgreSQL を Compose service として起動できる
- `db-migrate` を one-shot container として実行できる
- `customers-etl`, `commerce-etl`, `purchase-insights` を one-shot job として順番に実行できる
- `app-api` を container として起動し、host から `http://localhost:8080/health` を確認できる
- `decision-engine` と Python reporting を container から実行できる
- volume、network、environment、healthcheck、logs、reset の意味を説明できる

## 3. 学習順

### Step 1: Docker の基本語彙

理解すること:

- image は実行可能なひな形
- container は image から起動したプロセス
- volume は container を消しても残すデータ領域
- network は service 間通信の名前解決境界
- environment は container に渡す設定値
- healthcheck は service が使える状態かどうかの判定

Decision Pack で対応するもの:

- image: `decision-pack:local`, `decision-pack-reporting:local`
- container: `app-api`, `migrate`, ETL jobs
- volume: `postgres-data`, `./out:/app/out`
- network: Compose が作る既定 network
- environment: `DATABASE_URL`, `APP_API_BIND_ADDR`
- healthcheck: `postgres`, `app-api`

### Step 2: Compose ファイルを読む

読む対象:

- `docker-compose.yml`
- `.dockerignore`
- `Dockerfile`
- `reporting/python/Dockerfile`

確認する観点:

- `postgres` が DB を持つ
- `migrate` が DB schema を作る
- `app-api` が `postgres` service 名で DB に接続する
- ETL と分析は常駐ではなく one-shot job として動く
- `out/` は host と container で共有される

### Step 3: DB と migration を動かす

実行すること:

```powershell
docker compose up -d postgres
docker compose run --rm migrate
```

理解すること:

- `postgres` は常駐 service
- `migrate` は完了したら終了する job
- `postgres://postgres:postgres@postgres:5432/decision_pack_app` の host 名 `postgres` は Compose network 内の service 名である

### Step 4: ETL job を順番に流す

実行する順番:

1. `customers-etl-smoke`
2. `commerce-sample-smoke`
3. `commerce-etl-smoke`
4. `purchase-insights-smoke`

理解すること:

- DB に保存する job と、CSV だけを生成する job がある
- `out/` に書いた成果物を次の container が読む
- job は順番が重要で、外部キー制約により customers と items が先に必要である

### Step 5: API を container として起動する

実行すること:

```powershell
docker compose up -d app-api
Invoke-RestMethod http://localhost:8080/health
```

理解すること:

- container 内では `0.0.0.0:8080` に bind する
- host からは published port の `localhost:8080` にアクセスする
- DB URL は log に credential を出さない

### Step 6: レポート生成を container で動かす

実行すること:

```powershell
docker compose run --rm decision-engine-report
docker compose run --rm reporting-smoke
```

理解すること:

- Rust の `decision-engine` が JSON を生成する
- Python reporting が同じ `out/` volume から JSON を読み、画像と要約を生成する
- Rust と Python は別 image に分ける

### Step 7: 運用コマンドに慣れる

覚えるコマンド:

```powershell
docker compose ps
docker compose logs app-api
docker compose logs postgres
docker compose down
docker compose down -v
docker compose build app-api
```

使い分け:

- `down` は container と network を止める
- `down -v` は DB volume も消すため、ローカル DB を初期化したいときだけ使う
- `build` は Rust / Python の image を作り直す

## 4. 完了条件

Docker ローカル運用の最初の完了条件は次の通り。

- `docker compose config` が成功する
- `docker compose up -d postgres` が成功する
- `docker compose run --rm migrate` が成功する
- smoke ETL と purchase-insights が最後まで成功する
- `docker compose up -d app-api` 後に `/health` が成功する
- `decision-engine-report` と `reporting-smoke` が `out/` に成果物を生成する
- reset 手順として `docker compose down -v` の意味を理解している

## 5. AWS への接続

Docker 習熟後に AWS 方針へ進む。
最初は無料運用前提のため、EKS や NAT Gateway のような固定費が出やすい構成には進まない。

Docker 側で固めた成果は、次の AWS 検討に使う。

- `app-api` image を ECR へ push できるか
- migration を ECS task / EC2 one-shot / CI のどこで実行するか
- DB を RDS PostgreSQL に分離するか
- secrets を Parameter Store Standard tier で扱えるか
- CloudWatch logs を最小量に抑えられるか
