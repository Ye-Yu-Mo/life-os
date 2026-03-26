pub mod dto;
pub mod routes;

use std::sync::Arc;

use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;

use crate::service::raw_logs::RawLogService;

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: &'static str,
}

pub fn build_router(raw_log_service: Arc<RawLogService>) -> Router {
    Router::new()
        .route("/health", get(health))
        .merge(routes::logs::router(routes::logs::LogsApiState {
            raw_log_service,
        }))
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}
