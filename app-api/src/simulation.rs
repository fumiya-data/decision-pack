use crate::{
    error::ApiError,
    models::{CreateSimulationRequest, SimulationDetail},
};
use decision_engine::report::json::to_json;
use decision_engine::sim::runner::{ItemRunInput, ScenarioRunInput, run_scenario};
use serde_json::Value;
use sqlx::{FromRow, PgPool};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, FromRow)]
struct PreparedItemRow {
    item_id: String,
    lead_time_days: i32,
    moq: Option<i32>,
    lot_size: Option<i32>,
    on_hand: i32,
    on_order: i32,
    forecast_qty: i32,
    avg_price: Option<f64>,
}

pub async fn run_and_store(
    pool: &PgPool,
    request: CreateSimulationRequest,
) -> Result<SimulationDetail, ApiError> {
    let horizon_days = request.horizon_days.unwrap_or(30).clamp(7, 180) as usize;
    let initial_cash = request.initial_cash.unwrap_or(1_000_000);
    let scenario_name = request
        .scenario_name
        .unwrap_or_else(|| "需要予測ベース在庫シミュレーション".to_string());
    let scenario_id = request.scenario_id.unwrap_or_else(|| {
        scenario_name
            .chars()
            .map(|ch| {
                if ch.is_ascii_alphanumeric() {
                    ch.to_ascii_lowercase()
                } else {
                    '-'
                }
            })
            .collect::<String>()
            .trim_matches('-')
            .to_string()
    });
    let currency = request.currency.unwrap_or_else(|| "JPY".to_string());
    let run_id = format!(
        "sim-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|err| ApiError::Internal(err.to_string()))?
            .as_secs()
    );

    sqlx::query(
        r#"
        INSERT INTO simulation_runs (
            run_id, scenario_id, scenario_name, status, requested_at, started_at
        ) VALUES ($1, $2, $3, 'running', NOW(), NOW())
        "#,
    )
    .bind(&run_id)
    .bind(&scenario_id)
    .bind(&scenario_name)
    .execute(pool)
    .await?;

    let prepared_items = load_items_for_simulation(pool).await?;
    if prepared_items.is_empty() {
        return Err(ApiError::BadRequest(
            "シミュレーション対象の品目が存在しません".to_string(),
        ));
    }

    let scenario = build_input(
        &scenario_id,
        &scenario_name,
        request.scenario_description,
        &currency,
        initial_cash,
        horizon_days,
        &prepared_items,
    )?;
    let output = run_scenario(&scenario);
    let report_json = serde_json::from_str::<Value>(&to_json(&output.report))
        .map_err(|err| ApiError::Internal(format!("report serialization failed: {err}")))?;

    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM simulation_item_results WHERE run_id = $1")
        .bind(&run_id)
        .execute(tx.as_mut())
        .await?;

    for summary in &output.item_summaries {
        sqlx::query(
            r#"
            INSERT INTO simulation_item_results (
                run_id,
                item_id,
                risk_level,
                recommended_reorder_qty,
                expected_stockout_qty,
                expected_days_on_hand
            ) VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(&run_id)
        .bind(&summary.item_id)
        .bind(classify_risk(summary.stockout_rate))
        .bind(summary.recommended_reorder_qty as i32)
        .bind(summary.total_stockout_qty as i32)
        .bind(summary.avg_on_hand)
        .execute(tx.as_mut())
        .await?;
    }

    sqlx::query(
        r#"
        UPDATE simulation_runs
        SET
            status = 'succeeded',
            completed_at = NOW(),
            report_schema_version = $2,
            report_json = $3
        WHERE run_id = $1
        "#,
    )
    .bind(&run_id)
    .bind(&output.report.schema_version)
    .bind(report_json)
    .execute(tx.as_mut())
    .await?;

    tx.commit().await?;

    Ok(SimulationDetail {
        run_id,
        scenario_id,
        scenario_name,
        status: "succeeded".to_string(),
        requested_at: output.report.generated_at.clone(),
        started_at: Some(output.report.generated_at.clone()),
        completed_at: Some(output.report.generated_at),
        report_schema_version: Some(output.report.schema_version),
        report_uri: None,
        report_available: true,
    })
}

async fn load_items_for_simulation(pool: &PgPool) -> Result<Vec<PreparedItemRow>, ApiError> {
    let rows = sqlx::query_as::<_, PreparedItemRow>(
        r#"
        SELECT
            i.item_id,
            i.lead_time_days,
            i.moq,
            i.lot_size,
            COALESCE(b.on_hand, 0) AS on_hand,
            COALESCE(b.on_order, 0) AS on_order,
            COALESCE(f.expected_qty, 0) AS forecast_qty,
            p.avg_price
        FROM items i
        LEFT JOIN inventory_balance b ON b.item_id = i.item_id
        LEFT JOIN LATERAL (
            SELECT expected_qty
            FROM item_demand_forecast f
            WHERE f.item_id = i.item_id
            ORDER BY f.as_of DESC, f.forecast_date DESC
            LIMIT 1
        ) f ON TRUE
        LEFT JOIN LATERAL (
            SELECT AVG(oi.unit_price)::double precision AS avg_price
            FROM order_items oi
            WHERE oi.item_id = i.item_id
              AND oi.unit_price IS NOT NULL
        ) p ON TRUE
        WHERE i.is_active = TRUE
        ORDER BY i.item_id
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

fn build_input(
    scenario_id: &str,
    scenario_name: &str,
    scenario_description: Option<String>,
    currency: &str,
    initial_cash: i64,
    horizon_days: usize,
    items: &[PreparedItemRow],
) -> Result<ScenarioRunInput, ApiError> {
    if horizon_days == 0 {
        return Err(ApiError::BadRequest(
            "horizon_days は 1 以上で指定してください".to_string(),
        ));
    }

    let days = (0..horizon_days)
        .map(|day| format!("D+{:02}", day + 1))
        .collect::<Vec<_>>();

    let inputs = items
        .iter()
        .map(|item| {
            let daily_demand = spread_quantity(item.forecast_qty.max(0) as u32, horizon_days);
            let mut arrivals = vec![0; horizon_days];
            if item.on_order > 0 {
                let arrival_day = item.lead_time_days.max(1) as usize - 1;
                if arrival_day < horizon_days {
                    arrivals[arrival_day] = item.on_order as u32;
                }
            }

            let avg_daily = ((item.forecast_qty.max(0) as f64) / horizon_days as f64).ceil() as u32;
            let lead_time = item.lead_time_days.max(1) as u32;
            let base_reorder = avg_daily.saturating_mul(lead_time.max(1));
            let moq = item.moq.unwrap_or(0).max(0) as u32;
            let lot_size = item.lot_size.unwrap_or(0).max(0) as u32;
            let reorder_point = base_reorder.max(moq);
            let safety_window = avg_daily.saturating_mul(7);
            let replenishment_batch = safety_window.max(lot_size.max(moq));
            let order_up_to = reorder_point.saturating_add(replenishment_batch);
            let sales_price = item.avg_price.unwrap_or(1000.0).round().max(1.0) as i64;
            let purchase_cost = ((sales_price as f64) * 0.6).round().max(1.0) as i64;

            ItemRunInput {
                item_id: item.item_id.clone(),
                opening_on_hand: item.on_hand.max(0) as u32,
                opening_on_order: item.on_order.max(0) as u32,
                demand_by_day: daily_demand,
                arrivals_by_day: arrivals,
                reorder_point,
                order_up_to,
                lead_time_days: lead_time,
                sales_unit_price: sales_price,
                purchase_unit_cost: purchase_cost,
            }
        })
        .collect::<Vec<_>>();

    Ok(ScenarioRunInput {
        scenario_id: scenario_id.to_string(),
        scenario_name: scenario_name.to_string(),
        scenario_description,
        currency: currency.to_string(),
        initial_cash,
        days,
        items: inputs,
    })
}

fn spread_quantity(total: u32, horizon_days: usize) -> Vec<u32> {
    if horizon_days == 0 {
        return Vec::new();
    }
    let base = total / horizon_days as u32;
    let remainder = total % horizon_days as u32;
    (0..horizon_days)
        .map(|idx| base + u32::from((idx as u32) < remainder))
        .collect()
}

fn classify_risk(stockout_rate: f64) -> &'static str {
    if stockout_rate >= 0.15 {
        "high"
    } else if stockout_rate >= 0.05 {
        "medium"
    } else {
        "low"
    }
}
