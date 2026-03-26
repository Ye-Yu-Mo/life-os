use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    Config(String),
    #[error("{0}")]
    Validation(String),
    #[error("ai decode error at {stage} ({encoding}): {message}")]
    AiDecode {
        stage: &'static str,
        encoding: &'static str,
        message: String,
    },
    #[error("ai schema error at {stage} ({schema}): {message}")]
    AiSchema {
        stage: &'static str,
        schema: &'static str,
        message: String,
    },
    #[error("ai retry exhausted at {stage} after {attempts} attempts: {message}")]
    AiRetryExhausted {
        stage: &'static str,
        attempts: usize,
        message: String,
    },
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    InternalState(String),
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),
    #[error("internal server error")]
    Internal,
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    error: ErrorMessage,
}

#[derive(Debug, Serialize)]
struct ErrorMessage {
    code: &'static str,
    message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            Self::Config(message) => (StatusCode::INTERNAL_SERVER_ERROR, "config_error", message),
            Self::Validation(message) => (StatusCode::BAD_REQUEST, "validation_error", message),
            Self::AiDecode { message, .. } => (StatusCode::BAD_REQUEST, "ai_decode_error", message),
            Self::AiSchema { message, .. } => (StatusCode::BAD_REQUEST, "ai_schema_error", message),
            Self::AiRetryExhausted { message, .. } => {
                (StatusCode::BAD_REQUEST, "ai_retry_exhausted", message)
            }
            Self::NotFound(message) => (StatusCode::NOT_FOUND, "not_found", message),
            Self::InternalState(message) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_state_error",
                message,
            ),
            Self::Database(error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "database_error",
                error.to_string(),
            ),
            Self::Migration(error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "migration_error",
                error.to_string(),
            ),
            Self::Internal => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                "internal server error".to_string(),
            ),
        };

        (
            status,
            Json(ErrorBody {
                error: ErrorMessage { code, message },
            }),
        )
            .into_response()
    }
}
