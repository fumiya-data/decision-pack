use crate::config::ImportConfig;
use crate::csv_input::{InventoryRow, ItemRow, OrderItemRow, OrderRow};
use sqlx::{Postgres, Transaction, postgres::PgPoolOptions};

#[derive(Debug, Default, Clone, Copy)]
pub struct PersistSummary {
    pub items_upserted: usize,
    pub orders_upserted: usize,
    pub order_items_upserted: usize,
    pub inventory_upserted: usize,
}

pub async fn persist_all(
    config: &ImportConfig,
    items: &[ItemRow],
    orders: &[OrderRow],
    order_items: &[OrderItemRow],
    inventory: &[InventoryRow],
) -> Result<PersistSummary, Box<dyn std::error::Error>> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await?;
    let mut tx = pool.begin().await?;

    for row in items {
        upsert_item(&mut tx, row).await?;
    }
    for row in orders {
        upsert_order(&mut tx, row).await?;
    }
    for row in order_items {
        upsert_order_item(&mut tx, row).await?;
    }
    for row in inventory {
        upsert_inventory(&mut tx, row).await?;
    }

    upsert_job_run(
        config,
        &mut tx,
        items.len(),
        orders.len(),
        order_items.len(),
        inventory.len(),
    )
    .await?;
    tx.commit().await?;

    Ok(PersistSummary {
        items_upserted: items.len(),
        orders_upserted: orders.len(),
        order_items_upserted: order_items.len(),
        inventory_upserted: inventory.len(),
    })
}

async fn upsert_item(tx: &mut Transaction<'_, Postgres>, row: &ItemRow) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO items (
            item_id, item_name, category, uom, is_active, lead_time_days, moq, lot_size
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        ON CONFLICT (item_id) DO UPDATE
        SET
            item_name = EXCLUDED.item_name,
            category = EXCLUDED.category,
            uom = EXCLUDED.uom,
            is_active = EXCLUDED.is_active,
            lead_time_days = EXCLUDED.lead_time_days,
            moq = EXCLUDED.moq,
            lot_size = EXCLUDED.lot_size,
            updated_at = NOW()
        "#,
    )
    .bind(&row.item_id)
    .bind(&row.item_name)
    .bind(&row.category)
    .bind(row.uom.as_deref())
    .bind(row.is_active.unwrap_or(true))
    .bind(row.lead_time_days.unwrap_or(0))
    .bind(row.moq)
    .bind(row.lot_size)
    .execute(tx.as_mut())
    .await?;
    Ok(())
}

async fn upsert_order(
    tx: &mut Transaction<'_, Postgres>,
    row: &OrderRow,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO orders (
            order_id, customer_id, ordered_at, status, currency, total_amount
        ) VALUES ($1, $2, $3::timestamptz, $4, $5, $6)
        ON CONFLICT (order_id) DO UPDATE
        SET
            customer_id = EXCLUDED.customer_id,
            ordered_at = EXCLUDED.ordered_at,
            status = EXCLUDED.status,
            currency = EXCLUDED.currency,
            total_amount = EXCLUDED.total_amount
        "#,
    )
    .bind(&row.order_id)
    .bind(&row.customer_id)
    .bind(&row.ordered_at)
    .bind(&row.status)
    .bind(row.currency.as_deref().unwrap_or("JPY"))
    .bind(row.total_amount)
    .execute(tx.as_mut())
    .await?;
    Ok(())
}

async fn upsert_order_item(
    tx: &mut Transaction<'_, Postgres>,
    row: &OrderItemRow,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO order_items (
            order_id, line_no, item_id, quantity, unit_price, line_amount
        ) VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (order_id, line_no) DO UPDATE
        SET
            item_id = EXCLUDED.item_id,
            quantity = EXCLUDED.quantity,
            unit_price = EXCLUDED.unit_price,
            line_amount = EXCLUDED.line_amount
        "#,
    )
    .bind(&row.order_id)
    .bind(row.line_no)
    .bind(&row.item_id)
    .bind(row.quantity)
    .bind(row.unit_price)
    .bind(row.line_amount)
    .execute(tx.as_mut())
    .await?;
    Ok(())
}

async fn upsert_inventory(
    tx: &mut Transaction<'_, Postgres>,
    row: &InventoryRow,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO inventory_balance (
            item_id, on_hand, on_order, reserved_qty
        ) VALUES ($1, $2, $3, $4)
        ON CONFLICT (item_id) DO UPDATE
        SET
            on_hand = EXCLUDED.on_hand,
            on_order = EXCLUDED.on_order,
            reserved_qty = EXCLUDED.reserved_qty,
            updated_at = NOW()
        "#,
    )
    .bind(&row.item_id)
    .bind(row.on_hand)
    .bind(row.on_order.unwrap_or(0))
    .bind(row.reserved_qty.unwrap_or(0))
    .execute(tx.as_mut())
    .await?;
    Ok(())
}

async fn upsert_job_run(
    config: &ImportConfig,
    tx: &mut Transaction<'_, Postgres>,
    items: usize,
    orders: usize,
    order_items: usize,
    inventory: usize,
) -> Result<(), sqlx::Error> {
    let source_uri = [
        config
            .items_csv
            .as_ref()
            .map(|p| format!("items={}", p.display())),
        config
            .orders_csv
            .as_ref()
            .map(|p| format!("orders={}", p.display())),
        config
            .order_items_csv
            .as_ref()
            .map(|p| format!("order_items={}", p.display())),
        config
            .inventory_csv
            .as_ref()
            .map(|p| format!("inventory={}", p.display())),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join(";");

    let artifact_uri = format!(
        "items={},orders={},order_items={},inventory={}",
        items, orders, order_items, inventory
    );

    sqlx::query(
        r#"
        INSERT INTO etl_job_runs (
            job_id, job_kind, status, requested_at, started_at, completed_at, source_uri, artifact_uri
        ) VALUES ($1, 'commerce-etl', 'succeeded', NOW(), NOW(), NOW(), $2, $3)
        ON CONFLICT (job_id) DO UPDATE
        SET
            job_kind = EXCLUDED.job_kind,
            status = EXCLUDED.status,
            source_uri = EXCLUDED.source_uri,
            artifact_uri = EXCLUDED.artifact_uri,
            completed_at = NOW()
        "#,
    )
    .bind(&config.run_id)
    .bind(source_uri)
    .bind(artifact_uri)
    .execute(tx.as_mut())
    .await?;
    Ok(())
}
