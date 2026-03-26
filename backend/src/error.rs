use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    Config(String),
    #[error("{0}")]
    Validation(String),
    #[error("{0}")]
    InternalState(String),
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
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
            Self::Internal => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                "internal server error".to_string(),
            ),
        };

        (status, Json(ErrorBody { error: ErrorMessage { code, message } })).into_response()
    }
}
