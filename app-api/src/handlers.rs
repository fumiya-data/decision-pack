use crate::{
    AppState,
    error::ApiError,
    models::{
        CreateSimulationRequest, CustomerDetail, CustomerNextBuy, CustomerPurchase,
        CustomerSummary, HealthResponse, ItemDetail, ItemInventory, ItemListQuery, ItemRisk,
        ItemSummary, Pagination, RootResponse, SimulationDetail, SimulationReportEnvelope,
        SimulationSummary,
    },
    simulation,
};
use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde_json::Value;
use sqlx::PgPool;

fn pool(state: &AppState) -> Result<&PgPool, ApiError> {
    state.pool.as_ref().ok_or(ApiError::DatabaseUnavailable)
}

pub async fn root(State(state): State<AppState>) -> Json<RootResponse> {
    Json(RootResponse {
        service: state.config.service_name,
        status: "ok",
        docs_path: "docs/index.md",
    })
}

pub async fn health(State(state): State<AppState>) -> Result<Json<HealthResponse>, ApiError> {
    let database = if let Some(pool) = &state.pool {
        sqlx::query_scalar::<_, i32>("SELECT 1")
            .fetch_one(pool)
            .await?;
        "connected"
    } else {
        "disabled"
    };

    Ok(Json(HealthResponse {
        service: state.config.service_name,
        status: "ok",
        database,
    }))
}

pub async fn list_customers(
    State(state): State<AppState>,
    Query(pagination): Query<Pagination>,
) -> Result<Json<Vec<CustomerSummary>>, ApiError> {
    let customers = sqlx::query_as::<_, CustomerSummary>(
        r#"
        SELECT customer_id, full_name, email, status, tier, country
        FROM customers
        ORDER BY customer_id
        LIMIT $1 OFFSET $2
        "#,
    )
    .bind(pagination.limit())
    .bind(pagination.offset())
    .fetch_all(pool(&state)?)
    .await?;

    Ok(Json(customers))
}

pub async fn get_customer(
    State(state): State<AppState>,
    Path(customer_id): Path<String>,
) -> Result<Json<CustomerDetail>, ApiError> {
    let customer = sqlx::query_as::<_, CustomerDetail>(
        r#"
        SELECT
            customer_id,
            full_name,
            email,
            phone,
            address_line,
            city,
            region,
            postal_code,
            country,
            birth_date::text AS birth_date,
            signup_date::text AS signup_date,
            last_purchase_date::text AS last_purchase_date,
            status,
            tier,
            preferred_language,
            marketing_opt_in,
            total_spend::double precision AS total_spend,
            order_count,
            notes
        FROM customers
        WHERE customer_id = $1
        "#,
    )
    .bind(customer_id)
    .fetch_optional(pool(&state)?)
    .await?
    .ok_or(ApiError::NotFound("customer"))?;

    Ok(Json(customer))
}

pub async fn list_customer_purchases(
    State(state): State<AppState>,
    Path(customer_id): Path<String>,
    Query(pagination): Query<Pagination>,
) -> Result<Json<Vec<CustomerPurchase>>, ApiError> {
    let purchases = sqlx::query_as::<_, CustomerPurchase>(
        r#"
        SELECT
            o.order_id,
            o.ordered_at::text AS ordered_at,
            o.status AS order_status,
            oi.item_id,
            i.item_name,
            oi.quantity,
            oi.unit_price::double precision AS unit_price,
            oi.line_amount::double precision AS line_amount
        FROM orders o
        JOIN order_items oi ON oi.order_id = o.order_id
        JOIN items i ON i.item_id = oi.item_id
        WHERE o.customer_id = $1
        ORDER BY o.ordered_at DESC, oi.line_no ASC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(customer_id)
    .bind(pagination.limit())
    .bind(pagination.offset())
    .fetch_all(pool(&state)?)
    .await?;

    Ok(Json(purchases))
}

pub async fn list_customer_next_buy(
    State(state): State<AppState>,
    Path(customer_id): Path<String>,
    Query(pagination): Query<Pagination>,
) -> Result<Json<Vec<CustomerNextBuy>>, ApiError> {
    let next_buy = sqlx::query_as::<_, CustomerNextBuy>(
        r#"
        SELECT
            s.item_id,
            i.item_name,
            s.score,
            s.rank,
            s.as_of::text AS as_of
        FROM customer_item_next_buy_score s
        JOIN items i ON i.item_id = s.item_id
        WHERE s.customer_id = $1
        ORDER BY s.rank ASC, s.score DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(customer_id)
    .bind(pagination.limit())
    .bind(pagination.offset())
    .fetch_all(pool(&state)?)
    .await?;

    Ok(Json(next_buy))
}

pub async fn list_items(
    State(state): State<AppState>,
    Query(query): Query<ItemListQuery>,
) -> Result<Json<Vec<ItemSummary>>, ApiError> {
    let search = query.q.clone();
    let category = query.category.clone();
    let items = sqlx::query_as::<_, ItemSummary>(
        r#"
        SELECT
            i.item_id,
            i.item_name,
            i.category,
            i.is_active,
            b.on_hand,
            b.on_order,
            b.reserved_qty,
            b.updated_at::text AS updated_at
        FROM items i
        LEFT JOIN inventory_balance b ON b.item_id = i.item_id
        WHERE ($1::text IS NULL OR i.item_id ILIKE '%' || $1 || '%' OR i.item_name ILIKE '%' || $1 || '%')
          AND ($2::text IS NULL OR i.category = $2)
        ORDER BY i.item_id
        LIMIT $3 OFFSET $4
        "#,
    )
    .bind(search)
    .bind(category)
    .bind(query.limit())
    .bind(query.offset())
    .fetch_all(pool(&state)?)
    .await?;

    Ok(Json(items))
}

pub async fn get_item(
    State(state): State<AppState>,
    Path(item_id): Path<String>,
) -> Result<Json<ItemDetail>, ApiError> {
    let item = sqlx::query_as::<_, ItemDetail>(
        r#"
        SELECT
            item_id,
            item_name,
            category,
            uom,
            is_active,
            lead_time_days,
            moq,
            lot_size,
            updated_at::text AS updated_at
        FROM items
        WHERE item_id = $1
        "#,
    )
    .bind(item_id)
    .fetch_optional(pool(&state)?)
    .await?
    .ok_or(ApiError::NotFound("item"))?;

    Ok(Json(item))
}

pub async fn get_item_inventory(
    State(state): State<AppState>,
    Path(item_id): Path<String>,
) -> Result<Json<ItemInventory>, ApiError> {
    let inventory = sqlx::query_as::<_, ItemInventory>(
        r#"
        SELECT
            item_id,
            on_hand,
            on_order,
            reserved_qty,
            updated_at::text AS updated_at
        FROM inventory_balance
        WHERE item_id = $1
        "#,
    )
    .bind(item_id)
    .fetch_optional(pool(&state)?)
    .await?
    .ok_or(ApiError::NotFound("inventory"))?;

    Ok(Json(inventory))
}

pub async fn get_item_risk(
    State(state): State<AppState>,
    Path(item_id): Path<String>,
) -> Result<Json<ItemRisk>, ApiError> {
    let risk = sqlx::query_as::<_, ItemRisk>(
        r#"
        SELECT
            sr.run_id,
            sr.scenario_id,
            sr.scenario_name,
            sir.risk_level,
            sir.recommended_reorder_qty,
            sir.expected_stockout_qty,
            sir.expected_days_on_hand,
            sr.requested_at::text AS requested_at
        FROM simulation_item_results sir
        JOIN simulation_runs sr ON sr.run_id = sir.run_id
        WHERE sir.item_id = $1
        ORDER BY sr.requested_at DESC
        LIMIT 1
        "#,
    )
    .bind(item_id)
    .fetch_optional(pool(&state)?)
    .await?
    .ok_or(ApiError::NotFound("item risk"))?;

    Ok(Json(risk))
}

pub async fn list_simulations(
    State(state): State<AppState>,
    Query(pagination): Query<Pagination>,
) -> Result<Json<Vec<SimulationSummary>>, ApiError> {
    let runs = sqlx::query_as::<_, SimulationSummary>(
        r#"
        SELECT
            run_id,
            scenario_id,
            scenario_name,
            status,
            requested_at::text AS requested_at,
            completed_at::text AS completed_at,
            report_schema_version,
            report_uri
        FROM simulation_runs
        ORDER BY requested_at DESC
        LIMIT $1 OFFSET $2
        "#,
    )
    .bind(pagination.limit())
    .bind(pagination.offset())
    .fetch_all(pool(&state)?)
    .await?;

    Ok(Json(runs))
}

pub async fn create_simulation(
    State(state): State<AppState>,
    Json(request): Json<CreateSimulationRequest>,
) -> Result<Json<SimulationDetail>, ApiError> {
    let simulation = simulation::run_and_store(pool(&state)?, request).await?;
    Ok(Json(simulation))
}

pub async fn get_simulation(
    State(state): State<AppState>,
    Path(run_id): Path<String>,
) -> Result<Json<SimulationDetail>, ApiError> {
    let run = sqlx::query_as::<_, SimulationDetail>(
        r#"
        SELECT
            run_id,
            scenario_id,
            scenario_name,
            status,
            requested_at::text AS requested_at,
            started_at::text AS started_at,
            completed_at::text AS completed_at,
            report_schema_version,
            report_uri,
            (report_json IS NOT NULL OR report_uri IS NOT NULL) AS report_available
        FROM simulation_runs
        WHERE run_id = $1
        "#,
    )
    .bind(run_id)
    .fetch_optional(pool(&state)?)
    .await?
    .ok_or(ApiError::NotFound("simulation"))?;

    Ok(Json(run))
}

pub async fn get_simulation_report(
    State(state): State<AppState>,
    Path(run_id): Path<String>,
) -> Result<Json<SimulationReportEnvelope>, ApiError> {
    let report = sqlx::query_scalar::<_, Option<Value>>(
        r#"
        SELECT report_json
        FROM simulation_runs
        WHERE run_id = $1
        "#,
    )
    .bind(&run_id)
    .fetch_optional(pool(&state)?)
    .await?
    .flatten()
    .ok_or(ApiError::NotFound("simulation report"))?;

    Ok(Json(SimulationReportEnvelope { run_id, report }))
}
