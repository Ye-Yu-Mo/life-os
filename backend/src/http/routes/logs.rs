use std::sync::Arc;

use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};

use crate::error::AppError;
use crate::http::dto::logs::{
    CreateRawLogRequest, ImportRawLogsRequest, ImportRawLogsResponse, RawLogResponse,
};
use crate::service::raw_logs::RawLogService;

#[derive(Clone)]
pub struct LogsApiState {
    pub raw_log_service: Arc<RawLogService>,
}

pub fn router(state: LogsApiState) -> Router {
    Router::new()
        .route("/logs", post(create_log).get(list_logs))
        .route("/logs/import", post(import_logs))
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

async fn import_logs(
    State(state): State<LogsApiState>,
    Json(request): Json<ImportRawLogsRequest>,
) -> Result<Json<ImportRawLogsResponse>, AppError> {
    let result = state
        .raw_log_service
        .import(request.try_into_create_raw_logs()?)
        .await?;

    Ok(Json(ImportRawLogsResponse {
        total_count: result.total_count,
        success_count: result.success_count,
        failure_count: result.failure_count,
        errors: result.errors,
    }))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode};
    use chrono::{TimeZone, Utc};
    use serde_json::{Value, json};
    use tower::ServiceExt;

    use super::*;
    use crate::domain::raw_logs::{CreateRawLog, InputChannel, ParseStatus, RawLog, SourceType};
    use crate::repository::raw_logs::{RawLogRepository, UpdateRawLogParseState};

    #[derive(Default)]
    struct FakeRawLogRepository {
        created_inputs: std::sync::Mutex<Vec<CreateRawLog>>,
        list_response: std::sync::Mutex<Vec<RawLog>>,
        get_response: std::sync::Mutex<Option<RawLog>>,
        batch_error_message: std::sync::Mutex<Option<String>>,
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

        async fn create_many(&self, inputs: Vec<CreateRawLog>) -> Result<Vec<RawLog>, AppError> {
            if let Some(message) = self
                .batch_error_message
                .lock()
                .expect("mutex should not be poisoned")
                .take()
            {
                return Err(AppError::InternalState(message));
            }

            let mut created = Vec::with_capacity(inputs.len());

            for input in inputs {
                self.created_inputs
                    .lock()
                    .expect("mutex should not be poisoned")
                    .push(input.clone());
                created.push(RawLog {
                    raw_text: input.raw_text,
                    input_channel: input.input_channel,
                    source_type: input.source_type,
                    context_date: input.context_date,
                    timezone: input.timezone,
                    ..sample_raw_log()
                });
            }

            Ok(created)
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

        async fn update_parse_state(
            &self,
            _input: UpdateRawLogParseState,
        ) -> Result<RawLog, AppError> {
            Err(AppError::InternalState(
                "not used in http tests".to_string(),
            ))
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

    #[tokio::test]
    async fn post_logs_import_accepts_json_batch_and_returns_summary() {
        let repository = Arc::new(FakeRawLogRepository::default());
        let app = build_test_router(repository.clone());

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/logs/import")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "format": "json",
                            "records": [
                                {
                                    "user_id": "550e8400-e29b-41d4-a716-446655440001",
                                    "raw_text": "今天 9:40 起床",
                                    "input_channel": "import",
                                    "source_type": "imported",
                                    "context_date": "2026-03-26",
                                    "timezone": "Asia/Shanghai"
                                },
                                {
                                    "user_id": "550e8400-e29b-41d4-a716-446655440001",
                                    "raw_text": "晚上跑步 35 分钟",
                                    "input_channel": "import",
                                    "source_type": "imported",
                                    "context_date": "2026-03-26",
                                    "timezone": "Asia/Shanghai"
                                }
                            ]
                        })
                        .to_string(),
                    ))
                    .expect("request should build"),
            )
            .await
            .expect("request should succeed");

        let body = read_json(response).await;

        assert_eq!(body["total_count"], 2);
        assert_eq!(body["success_count"], 2);
        assert_eq!(body["failure_count"], 0);
        assert_eq!(body["errors"], json!([]));
        assert_eq!(
            repository
                .created_inputs
                .lock()
                .expect("mutex should not be poisoned")
                .len(),
            2
        );
    }

    #[tokio::test]
    async fn post_logs_import_accepts_csv_batch_and_returns_summary() {
        let repository = Arc::new(FakeRawLogRepository::default());
        let app = build_test_router(repository.clone());

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/logs/import")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "format": "csv",
                            "content": "user_id,raw_text,input_channel,source_type,context_date,timezone\n550e8400-e29b-41d4-a716-446655440001,今天 9:40 起床,import,imported,2026-03-26,Asia/Shanghai\n550e8400-e29b-41d4-a716-446655440001,晚上跑步 35 分钟,import,imported,2026-03-26,Asia/Shanghai\n"
                        })
                        .to_string(),
                    ))
                    .expect("request should build"),
            )
            .await
            .expect("request should succeed");

        let body = read_json(response).await;

        assert_eq!(body["total_count"], 2);
        assert_eq!(body["success_count"], 2);
        assert_eq!(body["failure_count"], 0);
        assert_eq!(body["errors"], json!([]));
        assert_eq!(
            repository
                .created_inputs
                .lock()
                .expect("mutex should not be poisoned")
                .len(),
            2
        );
    }

    #[tokio::test]
    async fn post_logs_import_rejects_unknown_format() {
        let repository = Arc::new(FakeRawLogRepository::default());
        let app = build_test_router(repository);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/logs/import")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "format": "xml",
                            "content": "<logs />"
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
    async fn post_logs_import_rejects_invalid_row_without_partial_write() {
        let repository = Arc::new(FakeRawLogRepository::default());
        let app = build_test_router(repository.clone());

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/logs/import")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "format": "json",
                            "records": [
                                {
                                    "user_id": "550e8400-e29b-41d4-a716-446655440001",
                                    "raw_text": "今天 9:40 起床",
                                    "input_channel": "import",
                                    "source_type": "imported",
                                    "context_date": "2026-03-26",
                                    "timezone": "Asia/Shanghai"
                                },
                                {
                                    "user_id": "550e8400-e29b-41d4-a716-446655440001",
                                    "raw_text": "",
                                    "input_channel": "import",
                                    "source_type": "imported",
                                    "context_date": "2026-03-26",
                                    "timezone": "Asia/Shanghai"
                                }
                            ]
                        })
                        .to_string(),
                    ))
                    .expect("request should build"),
            )
            .await
            .expect("request should succeed");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            repository
                .created_inputs
                .lock()
                .expect("mutex should not be poisoned")
                .len(),
            0
        );
    }

    #[tokio::test]
    async fn post_logs_import_returns_explainable_batch_failure_message() {
        let repository = Arc::new(FakeRawLogRepository::default());
        repository
            .batch_error_message
            .lock()
            .expect("mutex should not be poisoned")
            .replace("simulated batch failure".to_string());
        let app = build_test_router(repository);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/logs/import")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "format": "json",
                            "records": [
                                {
                                    "user_id": "550e8400-e29b-41d4-a716-446655440001",
                                    "raw_text": "今天 9:40 起床",
                                    "input_channel": "import",
                                    "source_type": "imported",
                                    "context_date": "2026-03-26",
                                    "timezone": "Asia/Shanghai"
                                }
                            ]
                        })
                        .to_string(),
                    ))
                    .expect("request should build"),
            )
            .await
            .expect("request should succeed");

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let body = read_json_with_status(response, StatusCode::INTERNAL_SERVER_ERROR).await;
        assert!(
            body["error"]["message"]
                .as_str()
                .expect("error message should be string")
                .contains("no records were persisted")
        );
    }

    async fn read_json(response: axum::response::Response) -> Value {
        read_json_with_status(response, StatusCode::OK).await
    }

    async fn read_json_with_status(
        response: axum::response::Response,
        expected_status: StatusCode,
    ) -> Value {
        let status = response.status();
        assert_eq!(status, expected_status);

        let bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body should read");

        serde_json::from_slice(&bytes).expect("body should be valid json")
    }
}
