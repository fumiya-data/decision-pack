use crate::algorithm::{CandidateScore, DemandForecastRow};
use chrono::{DateTime, Utc};
use sqlx::{FromRow, postgres::PgPoolOptions};

#[derive(Debug, Clone, FromRow)]
pub struct CustomerRecord {
    pub customer_id: String,
    pub country: Option<String>,
    pub status: Option<String>,
    pub tier: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
pub struct ItemRecord {
    pub item_id: String,
    pub item_name: String,
    pub category: String,
    pub is_active: bool,
}

#[derive(Debug, Clone, FromRow)]
pub struct OrderItemHistoryRow {
    pub customer_id: String,
    pub order_id: String,
    pub ordered_at: Option<DateTime<Utc>>,
    pub item_id: String,
    pub quantity: i32,
}

#[derive(Debug, Clone, Copy)]
pub struct PersistSummary {
    pub scores_inserted: usize,
    pub forecasts_inserted: usize,
}

pub async fn load_customers(
    database_url: &str,
) -> Result<Vec<CustomerRecord>, Box<dyn std::error::Error>> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;
    let rows = sqlx::query_as::<_, CustomerRecord>(
        r#"
        SELECT customer_id, country, status, tier
        FROM customers
        ORDER BY customer_id
        "#,
    )
    .fetch_all(&pool)
    .await?;
    Ok(rows)
}

pub async fn load_items(database_url: &str) -> Result<Vec<ItemRecord>, Box<dyn std::error::Error>> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;
    let rows = sqlx::query_as::<_, ItemRecord>(
        r#"
        SELECT item_id, item_name, category, is_active
        FROM items
        ORDER BY item_id
        "#,
    )
    .fetch_all(&pool)
    .await?;
    Ok(rows)
}

pub async fn load_order_history(
    database_url: &str,
) -> Result<Vec<OrderItemHistoryRow>, Box<dyn std::error::Error>> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;
    let rows = sqlx::query_as::<_, OrderItemHistoryRow>(
        r#"
        SELECT o.customer_id, o.order_id, o.ordered_at, oi.item_id, oi.quantity
        FROM orders o
        JOIN order_items oi ON oi.order_id = o.order_id
        ORDER BY o.customer_id, o.ordered_at, o.order_id, oi.line_no
        "#,
    )
    .fetch_all(&pool)
    .await?;
    Ok(rows)
}

pub async fn persist_results(
    database_url: &str,
    run_id: &str,
    scores: &[CandidateScore],
    forecasts: &[DemandForecastRow],
) -> Result<PersistSummary, Box<dyn std::error::Error>> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;
    let mut tx = pool.begin().await?;
    let as_of = Utc::now();

    for score in scores {
        sqlx::query(
            r#"
            INSERT INTO customer_item_next_buy_score (
                customer_id, item_id, score, rank, as_of
            ) VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (customer_id, item_id, as_of) DO UPDATE
            SET score = EXCLUDED.score, rank = EXCLUDED.rank
            "#,
        )
        .bind(&score.customer_id)
        .bind(&score.item_id)
        .bind(score.score)
        .bind(score.rank)
        .bind(as_of)
        .execute(tx.as_mut())
        .await?;
    }

    for forecast in forecasts {
        sqlx::query(
            r#"
            INSERT INTO item_demand_forecast (
                forecast_date, item_id, expected_qty, low_qty, high_qty, as_of, source_run_id
            ) VALUES (CURRENT_DATE, $1, $2, $3, $4, $5, $6)
            ON CONFLICT (forecast_date, item_id, as_of) DO UPDATE
            SET
                expected_qty = EXCLUDED.expected_qty,
                low_qty = EXCLUDED.low_qty,
                high_qty = EXCLUDED.high_qty,
                source_run_id = EXCLUDED.source_run_id
            "#,
        )
        .bind(&forecast.item_id)
        .bind(forecast.expected_qty)
        .bind(forecast.low_qty)
        .bind(forecast.high_qty)
        .bind(as_of)
        .bind(run_id)
        .execute(tx.as_mut())
        .await?;
    }

    sqlx::query(
        r#"
        INSERT INTO etl_job_runs (
            job_id, job_kind, status, requested_at, started_at, completed_at, artifact_uri
        ) VALUES ($1, 'purchase-insights', 'succeeded', NOW(), NOW(), NOW(), $2)
        ON CONFLICT (job_id) DO UPDATE
        SET
            job_kind = EXCLUDED.job_kind,
            status = EXCLUDED.status,
            artifact_uri = EXCLUDED.artifact_uri,
            completed_at = NOW()
        "#,
    )
    .bind(run_id)
    .bind(format!(
        "scores={},forecasts={}",
        scores.len(),
        forecasts.len()
    ))
    .execute(tx.as_mut())
    .await?;

    tx.commit().await?;
    Ok(PersistSummary {
        scores_inserted: scores.len(),
        forecasts_inserted: forecasts.len(),
    })
}
