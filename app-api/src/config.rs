use std::{env, fmt};

const LOCAL_DEFAULT_DATABASE_URL: &str = "postgres://postgres:postgres@localhost/decision_pack_app";

#[derive(Clone)]
pub struct AppConfig {
    pub service_name: String,
    pub bind_addr: String,
    pub database_url: String,
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
                .or_else(|| env::var("APP_API_DATABASE_URL").ok())
                .filter(|v| !v.trim().is_empty())
                .unwrap_or_else(|| LOCAL_DEFAULT_DATABASE_URL.to_string()),
        }
    }

    pub fn redacted_database_url(&self) -> String {
        redact_database_url(&self.database_url)
    }
}

impl fmt::Debug for AppConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AppConfig")
            .field("service_name", &self.service_name)
            .field("bind_addr", &self.bind_addr)
            .field("database_url", &self.redacted_database_url())
            .finish()
    }
}

pub fn redact_database_url(value: &str) -> String {
    let Some(scheme_idx) = value.find("://") else {
        return value.to_string();
    };
    let auth_start = scheme_idx + 3;
    let rest = &value[auth_start..];
    let Some(at_idx) = rest.find('@') else {
        return value.to_string();
    };

    format!("{}***@{}", &value[..auth_start], &rest[at_idx + 1..])
}

#[cfg(test)]
mod tests {
    use super::{AppConfig, redact_database_url};

    #[test]
    fn redacts_credentials_from_database_url() {
        let input = "postgres://postgres:postgres@localhost/decision_pack_app";
        assert_eq!(
            redact_database_url(input),
            "postgres://***@localhost/decision_pack_app"
        );
    }

    #[test]
    fn leaves_urls_without_authority_credentials_unchanged() {
        let input = "postgres://localhost/decision_pack_app";
        assert_eq!(redact_database_url(input), input);
    }

    #[test]
    fn debug_output_uses_redacted_database_url() {
        let config = AppConfig {
            service_name: "svc".to_string(),
            bind_addr: "127.0.0.1:0".to_string(),
            database_url: "postgres://user:secret@localhost/db".to_string(),
        };

        let output = format!("{config:?}");
        assert!(output.contains("postgres://***@localhost/db"));
        assert!(!output.contains("secret"));
        assert!(!output.contains("user:secret"));
    }
}
