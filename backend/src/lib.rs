pub mod config;
pub mod domain;
pub mod error;
pub mod http;
pub mod repository;
pub mod service;
pub mod validation;

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

pub async fn create_db_pool(config: &config::Config) -> Result<PgPool, error::AppError> {
    let pool = PgPoolOptions::new()
        .max_connections(config.database.max_connections)
        .connect(&config.database.url)
        .await?;

    Ok(pool)
}

pub async fn run_migrations(pool: &PgPool) -> Result<(), error::AppError> {
    sqlx::migrate!("./migrations").run(pool).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    use crate::domain::raw_logs::{CreateRawLog, RawLog};
    use crate::config::Config;
    use crate::error::AppError;
    use crate::repository::raw_logs::RawLogRepository;
    use crate::service::raw_logs::RawLogService;

    struct FakeRawLogRepository;

    #[async_trait]
    impl RawLogRepository for FakeRawLogRepository {
        async fn create(&self, _input: CreateRawLog) -> Result<RawLog, AppError> {
            Err(AppError::Internal)
        }

        async fn list(&self) -> Result<Vec<RawLog>, AppError> {
            Ok(vec![])
        }

        async fn get_by_id(&self, _id: &str) -> Result<Option<RawLog>, AppError> {
            Ok(None)
        }
    }

    #[test]
    fn loads_config_from_env() {
        let config = Config::from_env_map([
            ("APP_HOST", "127.0.0.1"),
            ("APP_PORT", "4100"),
            ("DATABASE_URL", "postgres://postgres:postgres@localhost:5432/life_os"),
            ("DATABASE_MAX_CONNECTIONS", "7"),
        ])
        .expect("config should load");

        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 4100);
        assert_eq!(
            config.database.url,
            "postgres://postgres:postgres@localhost:5432/life_os"
        );
        assert_eq!(config.database.max_connections, 7);
    }

    #[test]
    fn missing_database_url_returns_config_error() {
        let error = Config::from_env_map([("APP_HOST", "127.0.0.1"), ("APP_PORT", "4100")])
            .expect_err("missing database url should fail");

        match error {
            AppError::Config(message) => {
                assert!(message.contains("DATABASE_URL"));
            }
            other => panic!("expected config error, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn health_route_returns_ok() {
        let app = crate::http::build_router(Arc::new(RawLogService::new(Arc::new(
            FakeRawLogRepository,
        ))));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("request should succeed");

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn app_error_converts_to_json_http_response() {
        let response = axum::response::IntoResponse::into_response(AppError::Config(
            "missing config".to_string(),
        ));

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(
            response.headers().get(axum::http::header::CONTENT_TYPE),
            Some(&axum::http::HeaderValue::from_static("application/json"))
        );
    }
}
