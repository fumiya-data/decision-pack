# Docker ローカル運用の足場を追加

## 実施内容

- `Dockerfile` を追加し、Rust workspace の主要 CLI / API binary を `decision-pack:local` image にまとめた
- `docker-compose.yml` を追加し、PostgreSQL、migration、API、ETL、purchase-insights、decision-engine report、Python reporting を service 化した
- `reporting/python/Dockerfile` を追加し、reporting を Python container から実行できるようにした
- `.dockerignore` を追加し、`target/`, `out/`, full 規模生成データなどを Docker build context から除外した
- `docs/plans/docker-learning-roadmap.md` を追加し、Docker 習熟の順序と完了条件を定義した
- `docs/operations/docker-local-runbook.md` を追加し、Docker Compose によるローカル smoke 運用手順を定義した
- Docker smoke 検証中に見つかった `customers-etl` の任意日付永続化を修正し、不正日付は DB では `NULL` として扱うようにした
- Docker smoke 検証中に見つかった `commerce-etl` サンプル生成を修正し、空の `customer_id` を注文生成に使わないようにした
- reporting Docker image に日本語フォントを追加し、Matplotlib の日本語 glyph 警告を解消した

## 検証結果

- `docker compose config`
- `docker compose build app-api reporting-smoke`
- `docker compose up -d postgres`
- `docker compose run --rm migrate`
- `docker compose run --rm customers-etl-smoke`
- `docker compose run --rm commerce-sample-smoke`
- `docker compose run --rm commerce-etl-smoke`
- `docker compose run --rm purchase-insights-smoke`
- `docker compose up -d app-api`
- `Invoke-RestMethod http://localhost:8080/health`
- `Invoke-RestMethod "http://localhost:8080/api/v1/customers?limit=3"`
- `Invoke-RestMethod "http://localhost:8080/api/v1/items?limit=3"`
- `POST /api/v1/simulations`
- `GET /api/v1/simulations/{run_id}/report`
- `docker compose logs --tail 20 app-api` で DB credential が redaction 済みであることを確認
- `docker compose run --rm decision-engine-report`
- `docker compose run --rm reporting-smoke`

## Rust 検証

- `cargo fmt --all --check`
- `cargo test -p customers-etl`
- `cargo test -p commerce-etl`

## 補足

無料運用前提の AWS 方針は、Docker ローカル運用を一通り確認した後に別文書として固める。
