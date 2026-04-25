# Docker ローカル運用 runbook

## 1. 目的

この runbook は、Decision Pack を Docker Compose でローカル運用するための手順を固定する。
対象は PostgreSQL、migration、顧客 ETL、商取引サンプル生成、商取引 ETL、purchase-insights、API、decision-engine report、Python reporting である。

通常の host 実行手順は `docs/operations/local-operations-runbook.md` を正本とする。
この文書は、同じ流れを Docker で再現するための運用手順である。

## 2. 前提

- Docker Desktop が起動している
- `docker --version` が成功する
- `docker compose version` が成功する
- リポジトリルートをカレントディレクトリにする

```powershell
Set-Location C:\Users\kinbo\decision-pack
```

## 3. Compose service 一覧

| service | 種別 | 役割 |
| --- | --- | --- |
| `postgres` | 常駐 | ローカル PostgreSQL |
| `migrate` | one-shot | `db/migrations` を DB に適用 |
| `app-api` | 常駐 | GUI / API client 向け API |
| `customers-etl-smoke` | one-shot | 5,000 件顧客 CSV を整形し DB へ保存 |
| `commerce-sample-smoke` | one-shot | 商取引 smoke サンプル CSV を生成 |
| `commerce-etl-smoke` | one-shot | 商品、受注、明細、在庫を DB へ保存 |
| `purchase-insights-smoke` | one-shot | 購入傾向分析と需要予測を DB へ保存 |
| `decision-engine-report` | one-shot | schema 互換の report JSON を生成 |
| `reporting-smoke` | one-shot | report JSON から図表と要約を生成 |

## 4. 初回 build

Rust の実行 image と Python reporting image を作る。

```powershell
docker compose build app-api reporting-smoke
```

Rust コードを変更した場合は、再度 build する。

```powershell
docker compose build app-api
```

Python reporting を変更した場合は、次を実行する。

```powershell
docker compose build reporting-smoke
```

## 5. DB 起動

```powershell
docker compose up -d postgres
docker compose ps
```

`postgres` が `healthy` になるまで待つ。
host から直接接続したい場合、既定の公開ポートはローカル PostgreSQL と衝突しにくい `55432` である。
container 間通信では常に `postgres:5432` を使う。

ログを見る場合:

```powershell
docker compose logs postgres
```

## 6. Migration

```powershell
docker compose run --rm migrate
```

初回は migration が applied になり、2 回目以降は skipped になる。

## 7. Smoke ETL と分析

次の順番で実行する。

```powershell
docker compose run --rm customers-etl-smoke
docker compose run --rm commerce-sample-smoke
docker compose run --rm commerce-etl-smoke
docker compose run --rm purchase-insights-smoke
```

期待する主な成果物:

- `out/customers-etl/docker-smoke/formatted.csv`
- `out/customers-etl/docker-smoke/format_issues.csv`
- `out/customers-etl/docker-smoke/customer_segment_summary.csv`
- `out/commerce-docker-smoke/items.csv`
- `out/commerce-docker-smoke/orders.csv`
- `out/commerce-docker-smoke/order_items.csv`
- `out/commerce-docker-smoke/inventory.csv`

## 8. API 起動

```powershell
docker compose up -d app-api
```

health check:

```powershell
Invoke-RestMethod http://localhost:8080/health
```

DB-backed endpoint:

```powershell
Invoke-RestMethod "http://localhost:8080/api/v1/customers?limit=3"
Invoke-RestMethod "http://localhost:8080/api/v1/items?limit=3"
```

シミュレーション作成:

```powershell
Invoke-RestMethod `
  -Method Post `
  -Uri http://localhost:8080/api/v1/simulations `
  -ContentType "application/json" `
  -Body '{"scenario_id":"baseline-docker","scenario_name":"Baseline Docker"}'
```

API log:

```powershell
docker compose logs app-api
```

## 9. Report 生成

`decision-engine` の最小例から report JSON を生成する。

```powershell
docker compose run --rm decision-engine-report
```

Python reporting で図表と要約を生成する。

```powershell
docker compose run --rm reporting-smoke
```

期待する成果物:

- `out/simulation_report_v0.1.json`
- `out/reporting-docker-smoke/cash_balance.png`
- `out/reporting-docker-smoke/daily_stockout.png`
- `out/reporting-docker-smoke/summary.txt`

## 10. 停止と reset

API と DB を止める。

```powershell
docker compose down
```

DB volume も含めて完全に初期化する場合だけ、次を使う。

```powershell
docker compose down -v
```

`down -v` を使うと `postgres-data` volume が削除され、migration と ETL を最初から流し直す必要がある。

## 11. よくある確認

Compose 定義の構文確認:

```powershell
docker compose config
```

起動中 container の確認:

```powershell
docker compose ps
```

Rust image の中で shell を開く:

```powershell
docker compose run --rm app-api sh
```

DB だけ作り直す:

```powershell
docker compose down -v
docker compose up -d postgres
docker compose run --rm migrate
```

## 12. 注意

- `docker-compose.yml` の DB credential はローカル Docker 専用であり、本番や共有環境では使わない。
- container 内の DB host は `localhost` ではなく `postgres` である。
- host から API にアクセスするときは `localhost:8080` を使う。
- `out/` は host と container の共有ディレクトリであり、Git には載せない。
