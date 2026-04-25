use app_api::{AppState, build_app, config::AppConfig};
use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Method, Request, StatusCode},
};
use serde_json::{Value, json};
use sqlx::{Executor, PgPool, postgres::PgPoolOptions};
use std::sync::LazyLock;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;
use tower::ServiceExt;

const MIGRATION_SQL: &str = include_str!("../../db/migrations/202604172120_initial_schema.sql");
static DB_TEST_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

struct TestIds {
    customer_id: String,
    item_id: String,
    order_id: String,
    scenario_id: String,
}

#[tokio::test]
async fn db_backed_read_endpoints_return_seeded_rows() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = DB_TEST_LOCK.lock().await;
    let Some((pool, app)) = test_app().await? else {
        return Ok(());
    };
    let ids = TestIds::new()?;
    cleanup_rows(&pool, &ids).await?;
    seed_rows(&pool, &ids).await?;

    let (status, customer) = request_json(
        app.clone(),
        Method::GET,
        &format!("/api/v1/customers/{}", ids.customer_id),
        None,
    )
    .await?;
    assert_eq!(status, StatusCode::OK, "response body: {customer}");
    assert_eq!(customer["customer_id"], ids.customer_id);

    let (status, purchases) = request_json(
        app.clone(),
        Method::GET,
        &format!("/api/v1/customers/{}/purchases", ids.customer_id),
        None,
    )
    .await?;
    assert_eq!(status, StatusCode::OK, "response body: {purchases}");
    assert_eq!(purchases[0]["item_id"], ids.item_id);

    let (status, next_buy) = request_json(
        app.clone(),
        Method::GET,
        &format!("/api/v1/customers/{}/next-buy", ids.customer_id),
        None,
    )
    .await?;
    assert_eq!(status, StatusCode::OK, "response body: {next_buy}");
    assert_eq!(next_buy[0]["item_id"], ids.item_id);

    let (status, items) = request_json(
        app.clone(),
        Method::GET,
        &format!("/api/v1/items?q={}", ids.item_id),
        None,
    )
    .await?;
    assert_eq!(status, StatusCode::OK, "response body: {items}");
    assert_eq!(items[0]["item_id"], ids.item_id);

    let (status, inventory) = request_json(
        app,
        Method::GET,
        &format!("/api/v1/items/{}/inventory", ids.item_id),
        None,
    )
    .await?;
    assert_eq!(status, StatusCode::OK, "response body: {inventory}");
    assert_eq!(inventory["on_hand"], 25);

    cleanup_rows(&pool, &ids).await?;
    Ok(())
}

#[tokio::test]
async fn post_simulation_persists_run_results_and_report() -> Result<(), Box<dyn std::error::Error>>
{
    let _guard = DB_TEST_LOCK.lock().await;
    let Some((pool, app)) = test_app().await? else {
        return Ok(());
    };
    let ids = TestIds::new()?;
    cleanup_rows(&pool, &ids).await?;
    seed_rows(&pool, &ids).await?;

    let (status, simulation) = request_json(
        app.clone(),
        Method::POST,
        "/api/v1/simulations",
        Some(json!({
            "scenario_id": ids.scenario_id,
            "scenario_name": "Integration Test Baseline",
            "horizon_days": 7,
            "initial_cash": 1000000,
            "currency": "JPY"
        })),
    )
    .await?;
    assert_eq!(status, StatusCode::OK, "response body: {simulation}");
    assert_eq!(simulation["status"], "succeeded");
    assert_eq!(simulation["report_available"], true);

    let run_id = simulation["run_id"]
        .as_str()
        .expect("run_id should be a string");
    assert!(run_id.starts_with("sim-"));

    let result_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM simulation_item_results WHERE run_id = $1 AND item_id = $2",
    )
    .bind(run_id)
    .bind(&ids.item_id)
    .fetch_one(&pool)
    .await?;
    assert_eq!(result_count, 1);

    let (status, envelope) = request_json(
        app,
        Method::GET,
        &format!("/api/v1/simulations/{run_id}/report"),
        None,
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(envelope["run_id"], run_id);
    assert_eq!(envelope["report"]["schema_version"], "v0.1");
    assert_eq!(envelope["report"]["horizon_days"], 7);

    let first_date = envelope["report"]["cash_series"][0]["date"]
        .as_str()
        .expect("cash series date should be a string");
    assert!(
        is_iso_date(first_date),
        "API report dates should be schema-compatible ISO dates, got {first_date}"
    );

    cleanup_rows(&pool, &ids).await?;
    Ok(())
}

async fn test_app() -> Result<Option<(PgPool, Router)>, Box<dyn std::error::Error>> {
    let Some(database_url) = std::env::var("APP_API_TEST_DATABASE_URL")
        .ok()
        .filter(|value| !value.trim().is_empty())
    else {
        eprintln!("skipping app-api integration test: APP_API_TEST_DATABASE_URL is not set");
        return Ok(None);
    };

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;
    pool.execute(MIGRATION_SQL).await?;

    let app = build_app(AppState {
        config: AppConfig {
            service_name: "app-api-integration-test".to_string(),
            bind_addr: "127.0.0.1:0".to_string(),
            database_url,
        },
        pool: pool.clone(),
    });

    Ok(Some((pool, app)))
}

async fn request_json(
    app: Router,
    method: Method,
    uri: &str,
    body: Option<Value>,
) -> Result<(StatusCode, Value), Box<dyn std::error::Error>> {
    let mut builder = Request::builder().method(method).uri(uri);
    let request_body = if let Some(value) = body {
        builder = builder.header("content-type", "application/json");
        Body::from(value.to_string())
    } else {
        Body::empty()
    };

    let response = app.oneshot(builder.body(request_body)?).await?;
    let status = response.status();
    let bytes = to_bytes(response.into_body(), 1024 * 1024).await?;
    let value = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes)?
    };

    Ok((status, value))
}

impl TestIds {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let suffix = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        Ok(Self {
            customer_id: format!("TEST_CUST_{suffix}"),
            item_id: format!("TEST_ITEM_{suffix}"),
            order_id: format!("TEST_ORDER_{suffix}"),
            scenario_id: format!("test-scenario-{suffix}"),
        })
    }
}

async fn seed_rows(pool: &PgPool, ids: &TestIds) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO customers (
            customer_id, full_name, email, country, status, tier
        ) VALUES ($1, 'Integration Test Customer', 'it@example.test', 'Japan', 'active', 'gold')
        "#,
    )
    .bind(&ids.customer_id)
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO items (
            item_id, item_name, category, uom, is_active, lead_time_days, moq, lot_size
        ) VALUES ($1, 'Integration Test Item', 'test', 'ea', TRUE, 2, 5, 10)
        "#,
    )
    .bind(&ids.item_id)
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO inventory_balance (item_id, on_hand, on_order, reserved_qty)
        VALUES ($1, 25, 5, 2)
        "#,
    )
    .bind(&ids.item_id)
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO orders (
            order_id, customer_id, ordered_at, status, currency, total_amount
        ) VALUES ($1, $2, NOW(), 'completed', 'JPY', 1200)
        "#,
    )
    .bind(&ids.order_id)
    .bind(&ids.customer_id)
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO order_items (
            order_id, line_no, item_id, quantity, unit_price, line_amount
        ) VALUES ($1, 1, $2, 3, 400, 1200)
        "#,
    )
    .bind(&ids.order_id)
    .bind(&ids.item_id)
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO customer_item_next_buy_score (
            customer_id, item_id, score, rank, as_of
        ) VALUES ($1, $2, 0.9, 1, NOW())
        "#,
    )
    .bind(&ids.customer_id)
    .bind(&ids.item_id)
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO item_demand_forecast (
            forecast_date, item_id, expected_qty, low_qty, high_qty, as_of, source_run_id
        ) VALUES (CURRENT_DATE, $1, 14, 10, 18, NOW(), 'app-api-integration-test')
        "#,
    )
    .bind(&ids.item_id)
    .execute(pool)
    .await?;

    Ok(())
}

async fn cleanup_rows(pool: &PgPool, ids: &TestIds) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM simulation_runs WHERE scenario_id = $1")
        .bind(&ids.scenario_id)
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM customer_item_next_buy_score WHERE customer_id = $1 OR item_id = $2")
        .bind(&ids.customer_id)
        .bind(&ids.item_id)
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM item_demand_forecast WHERE item_id = $1")
        .bind(&ids.item_id)
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM order_items WHERE order_id = $1")
        .bind(&ids.order_id)
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM orders WHERE order_id = $1 OR customer_id = $2")
        .bind(&ids.order_id)
        .bind(&ids.customer_id)
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM inventory_balance WHERE item_id = $1")
        .bind(&ids.item_id)
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM items WHERE item_id = $1")
        .bind(&ids.item_id)
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM customers WHERE customer_id = $1")
        .bind(&ids.customer_id)
        .execute(pool)
        .await?;

    Ok(())
}

fn is_iso_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    value.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes
            .iter()
            .enumerate()
            .filter(|(idx, _)| *idx != 4 && *idx != 7)
            .all(|(_, byte)| byte.is_ascii_digit())
}
