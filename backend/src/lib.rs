pub mod config;
pub mod connectors;
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

        async fn create_many(&self, _inputs: Vec<CreateRawLog>) -> Result<Vec<RawLog>, AppError> {
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
        assert_eq!(
            config.ai.model_payload_encoding,
            crate::config::ModelPayloadEncoding::Toon
        );
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

    #[test]
    fn loads_ai_model_payload_encoding_from_env() {
        let config = Config::from_env_map([
            ("DATABASE_URL", "postgres://postgres:postgres@localhost:5432/life_os"),
            ("AI_MODEL_PAYLOAD_ENCODING", "json"),
        ])
        .expect("config should load");

        assert_eq!(
            config.ai.model_payload_encoding,
            crate::config::ModelPayloadEncoding::Json
        );
    }

    #[test]
    fn rejects_invalid_ai_model_payload_encoding() {
        let error = Config::from_env_map([
            ("DATABASE_URL", "postgres://postgres:postgres@localhost:5432/life_os"),
            ("AI_MODEL_PAYLOAD_ENCODING", "yaml"),
        ])
        .expect_err("invalid encoding should fail");

        match error {
            AppError::Config(message) => {
                assert!(message.contains("AI_MODEL_PAYLOAD_ENCODING"));
            }
            other => panic!("expected config error, got {other:?}"),
        }
    }

    #[test]
    fn loads_telegram_connector_config_from_env() {
        let config = Config::from_env_map([
            ("APP_HOST", "127.0.0.1"),
            ("APP_PORT", "4100"),
            ("DATABASE_URL", "postgres://postgres:postgres@localhost:5432/life_os"),
            ("DATABASE_MAX_CONNECTIONS", "7"),
            ("TELEGRAM_ENABLED", "true"),
            ("TELEGRAM_BOT_TOKEN", "test-bot-token"),
            ("TELEGRAM_ALLOWLIST_CHAT_IDS", "10001,10002"),
            ("TELEGRAM_CALLBACK_MODE", "webhook"),
            ("TELEGRAM_WEBHOOK_BASE_URL", "https://example.com"),
        ])
        .expect("config should load");

        assert!(config.telegram.enabled);
        assert_eq!(config.telegram.bot_token.as_deref(), Some("test-bot-token"));
        assert_eq!(config.telegram.allowlist_chat_ids, vec![10001, 10002]);
        assert_eq!(
            config.telegram.callback_mode,
            crate::config::ConnectorRuntimeMode::Webhook
        );
        assert_eq!(
            config.telegram.webhook_base_url.as_deref(),
            Some("https://example.com")
        );
    }

    #[test]
    fn loads_reserved_connector_configs_for_feishu_and_wechat_bridge() {
        let config = Config::from_env_map([
            ("APP_HOST", "127.0.0.1"),
            ("APP_PORT", "4100"),
            ("DATABASE_URL", "postgres://postgres:postgres@localhost:5432/life_os"),
            ("DATABASE_MAX_CONNECTIONS", "7"),
            ("FEISHU_ENABLED", "false"),
            ("FEISHU_APP_ID", "cli_aabbcc"),
            ("FEISHU_APP_SECRET", "secret-value"),
            ("FEISHU_CALLBACK_MODE", "webhook"),
            ("WECHAT_BRIDGE_ENABLED", "true"),
            ("WECHAT_BRIDGE_ENDPOINT", "http://127.0.0.1:8787"),
            ("WECHAT_BRIDGE_SHARED_SECRET", "shared-secret"),
        ])
        .expect("config should load");

        assert!(!config.feishu.enabled);
        assert_eq!(config.feishu.app_id.as_deref(), Some("cli_aabbcc"));
        assert_eq!(
            config.feishu.callback_mode,
            crate::config::ConnectorRuntimeMode::Webhook
        );

        assert!(config.wechat_bridge.enabled);
        assert_eq!(
            config.wechat_bridge.endpoint.as_deref(),
            Some("http://127.0.0.1:8787")
        );
        assert_eq!(
            config.wechat_bridge.shared_secret.as_deref(),
            Some("shared-secret")
        );
    }

    #[test]
    fn backend_env_example_contains_telegram_connector_variables() {
        let env_example = std::fs::read_to_string(".env.example")
            .expect(".env.example should exist");

        for expected_key in [
            "TELEGRAM_ENABLED",
            "TELEGRAM_BOT_TOKEN",
            "TELEGRAM_ALLOWLIST_CHAT_IDS",
            "TELEGRAM_CALLBACK_MODE",
            "TELEGRAM_WEBHOOK_BASE_URL",
        ] {
            assert!(
                env_example.contains(expected_key),
                ".env.example should contain {expected_key}"
            );
        }
    }

    #[test]
    fn backend_env_example_contains_reserved_feishu_and_wechat_bridge_variables() {
        let env_example = std::fs::read_to_string(".env.example")
            .expect(".env.example should exist");

        for expected_key in [
            "FEISHU_ENABLED",
            "FEISHU_APP_ID",
            "FEISHU_APP_SECRET",
            "FEISHU_CALLBACK_MODE",
            "WECHAT_BRIDGE_ENABLED",
            "WECHAT_BRIDGE_ENDPOINT",
            "WECHAT_BRIDGE_SHARED_SECRET",
        ] {
            assert!(
                env_example.contains(expected_key),
                ".env.example should contain {expected_key}"
            );
        }
    }

    #[test]
    fn readme_mentions_telegram_connector_setup() {
        let readme =
            std::fs::read_to_string("../README.md").expect("README should exist");

        assert!(
            readme.contains("Telegram"),
            "README should mention Telegram connector setup"
        );
        assert!(
            readme.contains("TELEGRAM_BOT_TOKEN"),
            "README should document Telegram bot token configuration"
        );
    }

    #[test]
    fn readme_mentions_reserved_feishu_and_wechat_bridge_connectors() {
        let readme =
            std::fs::read_to_string("../README.md").expect("README should exist");

        assert!(readme.contains("Feishu"), "README should mention Feishu connector");
        assert!(
            readme.contains("WeChat Bridge"),
            "README should mention WeChat Bridge reservation"
        );
        assert!(
            readme.contains("FEISHU_APP_ID"),
            "README should mention Feishu config"
        );
        assert!(
            readme.contains("WECHAT_BRIDGE_ENDPOINT"),
            "README should mention WeChat Bridge config"
        );
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
