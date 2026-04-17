use app_api::{AppState, build_app, config::AppConfig};
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("app_api=info,axum=info")),
        )
        .init();

    let config = AppConfig::from_env();
    let pool = if let Some(database_url) = &config.database_url {
        Some(
            PgPoolOptions::new()
                .max_connections(5)
                .connect(database_url)
                .await?,
        )
    } else {
        None
    };

    let listener = TcpListener::bind(&config.bind_addr).await?;
    info!(
        service = %config.service_name,
        bind_addr = %config.bind_addr,
        database_enabled = pool.is_some(),
        "starting app-api"
    );

    let app = build_app(AppState { config, pool });
    axum::serve(listener, app).await?;
    Ok(())
}
