use std::sync::Arc;

use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};

use crate::error::AppError;
use crate::http::dto::logs::{CreateRawLogRequest, RawLogResponse};
use crate::service::raw_logs::RawLogService;

#[derive(Clone)]
pub struct LogsApiState {
    pub raw_log_service: Arc<RawLogService>,
}

pub fn router(state: LogsApiState) -> Router {
    Router::new()
        .route("/logs", post(create_log).get(list_logs))
        .route("/logs/{id}", get(get_log))
        .with_state(state)
}

async fn create_log(
    State(state): State<LogsApiState>,
    Json(request): Json<CreateRawLogRequest>,
) -> Result<Json<RawLogResponse>, AppError> {
    let raw_log = state.raw_log_service.create(request.try_into()?).await?;
    Ok(Json(raw_log.into()))
}

async fn list_logs(
    State(state): State<LogsApiState>,
) -> Result<Json<Vec<RawLogResponse>>, AppError> {
    let logs = state.raw_log_service.list().await?;
    Ok(Json(logs.into_iter().map(Into::into).collect()))
}

async fn get_log(
    State(state): State<LogsApiState>,
    Path(id): Path<String>,
) -> Result<Json<RawLogResponse>, AppError> {
    let raw_log = state
        .raw_log_service
        .get_by_id(&id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("raw_log not found: {id}")))?;

    Ok(Json(raw_log.into()))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use chrono::{TimeZone, Utc};
    use serde_json::json;
    use tower::ServiceExt;

    use super::*;
    use crate::domain::raw_logs::{CreateRawLog, InputChannel, ParseStatus, RawLog, SourceType};
    use crate::repository::raw_logs::RawLogRepository;

    #[derive(Default)]
    struct FakeRawLogRepository {
        created_inputs: std::sync::Mutex<Vec<CreateRawLog>>,
        list_response: std::sync::Mutex<Vec<RawLog>>,
        get_response: std::sync::Mutex<Option<RawLog>>,
    }

    #[async_trait]
    impl RawLogRepository for FakeRawLogRepository {
        async fn create(&self, input: CreateRawLog) -> Result<RawLog, AppError> {
            self.created_inputs
                .lock()
                .expect("mutex should not be poisoned")
                .push(input.clone());
            Ok(sample_raw_log())
        }

        async fn list(&self) -> Result<Vec<RawLog>, AppError> {
            Ok(self
                .list_response
                .lock()
                .expect("mutex should not be poisoned")
                .clone())
        }

        async fn get_by_id(&self, _id: &str) -> Result<Option<RawLog>, AppError> {
            Ok(self
                .get_response
                .lock()
                .expect("mutex should not be poisoned")
                .clone())
        }
    }

    fn sample_raw_log() -> RawLog {
        RawLog {
            id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            user_id: "550e8400-e29b-41d4-a716-446655440001".to_string(),
            raw_text: "今天 9:40 起床".to_string(),
            input_channel: InputChannel::Web,
            source_type: SourceType::Manual,
            context_date: Some("2026-03-26".to_string()),
            timezone: Some("Asia/Shanghai".to_string()),
            parse_status: ParseStatus::Pending,
            parser_version: None,
            parse_error: None,
            created_at: Utc
                .with_ymd_and_hms(2026, 3, 26, 2, 0, 0)
                .single()
                .expect("time should be valid"),
            updated_at: Utc
                .with_ymd_and_hms(2026, 3, 26, 2, 0, 0)
                .single()
                .expect("time should be valid"),
        }
    }

    fn build_test_router(repository: Arc<FakeRawLogRepository>) -> Router {
        let service = Arc::new(RawLogService::new(repository));
        router(LogsApiState {
            raw_log_service: service,
        })
    }

    #[tokio::test]
    async fn post_logs_creates_raw_log_and_returns_created_payload() {
        let repository = Arc::new(FakeRawLogRepository::default());
        let app = build_test_router(repository.clone());

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/logs")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "user_id": "550e8400-e29b-41d4-a716-446655440001",
                            "raw_text": "今天 9:40 起床",
                            "input_channel": "web",
                            "source_type": "manual",
                            "context_date": "2026-03-26",
                            "timezone": "Asia/Shanghai"
                        })
                        .to_string(),
                    ))
                    .expect("request should build"),
            )
            .await
            .expect("request should succeed");

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            repository
                .created_inputs
                .lock()
                .expect("mutex should not be poisoned")
                .len(),
            1
        );
    }

    #[tokio::test]
    async fn post_logs_rejects_empty_raw_text_with_bad_request() {
        let repository = Arc::new(FakeRawLogRepository::default());
        let app = build_test_router(repository);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/logs")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "user_id": "550e8400-e29b-41d4-a716-446655440001",
                            "raw_text": "",
                            "input_channel": "web",
                            "source_type": "manual"
                        })
                        .to_string(),
                    ))
                    .expect("request should build"),
            )
            .await
            .expect("request should succeed");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn get_logs_returns_raw_log_list() {
        let repository = Arc::new(FakeRawLogRepository::default());
        repository
            .list_response
            .lock()
            .expect("mutex should not be poisoned")
            .push(sample_raw_log());

        let app = build_test_router(repository);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/logs")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("request should succeed");

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn get_log_by_id_returns_detail() {
        let repository = Arc::new(FakeRawLogRepository::default());
        repository
            .get_response
            .lock()
            .expect("mutex should not be poisoned")
            .replace(sample_raw_log());

        let app = build_test_router(repository);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/logs/550e8400-e29b-41d4-a716-446655440000")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("request should succeed");

        assert_eq!(response.status(), StatusCode::OK);
    }
}
