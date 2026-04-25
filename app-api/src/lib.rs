//! `app-api` は GUI とバックエンド処理をつなぐ HTTP API です。
//!
//! このクレートは初期実装として、ヘルスチェックと主要読み取り系 API の
//! 最小雛形を提供します。重い処理は持たず、DB とジョブ実行基盤の境界に徹します。

pub mod config;
mod error;
mod handlers;
mod models;
mod simulation;

use axum::{
    Router,
    routing::{get, post},
};
use config::AppConfig;
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub pool: PgPool,
}

pub fn build_app(state: AppState) -> Router {
    Router::new()
        .route("/", get(handlers::root))
        .route("/health", get(handlers::health))
        .route("/api/v1/customers", get(handlers::list_customers))
        .route(
            "/api/v1/customers/{customer_id}",
            get(handlers::get_customer),
        )
        .route(
            "/api/v1/customers/{customer_id}/purchases",
            get(handlers::list_customer_purchases),
        )
        .route(
            "/api/v1/customers/{customer_id}/next-buy",
            get(handlers::list_customer_next_buy),
        )
        .route("/api/v1/items", get(handlers::list_items))
        .route("/api/v1/items/{item_id}", get(handlers::get_item))
        .route(
            "/api/v1/items/{item_id}/inventory",
            get(handlers::get_item_inventory),
        )
        .route("/api/v1/items/{item_id}/risk", get(handlers::get_item_risk))
        .route("/api/v1/simulations", get(handlers::list_simulations))
        .route("/api/v1/simulations", post(handlers::create_simulation))
        .route(
            "/api/v1/simulations/{run_id}",
            get(handlers::get_simulation),
        )
        .route(
            "/api/v1/simulations/{run_id}/report",
            get(handlers::get_simulation_report),
        )
        .with_state(state)
}
