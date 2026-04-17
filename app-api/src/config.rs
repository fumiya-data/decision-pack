use std::env;

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub service_name: String,
    pub bind_addr: String,
    pub database_url: Option<String>,
}

impl AppConfig {
    pub fn from_env() -> Self {
        Self {
            service_name: env::var("APP_API_SERVICE_NAME")
                .unwrap_or_else(|_| "decision-pack-app-api".to_string()),
            bind_addr: env::var("APP_API_BIND_ADDR")
                .unwrap_or_else(|_| "127.0.0.1:8080".to_string()),
            database_url: env::var("DATABASE_URL")
                .ok()
                .filter(|v| !v.trim().is_empty()),
        }
    }
}
