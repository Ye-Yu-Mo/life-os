use backend::error::AppError;
use backend::http::dto::logs::{CreateRawLogRequest, RawLogResponse};
use backend::validation::logs::{validate_context_date, validate_raw_text};
use clap::Parser;
use serde::Deserialize;
use std::process::ExitCode;

const DEFAULT_API_BASE_URL: &str = "http://127.0.0.1:3000";

#[derive(Debug, Parser, PartialEq, Eq)]
#[command(name = "logs-cli")]
#[command(about = "Submit a raw log to the backend from the command line")]
struct CliArgs {
    #[arg(long)]
    user_id: String,
    #[arg(long)]
    context_date: Option<String>,
    #[arg(long)]
    timezone: Option<String>,
    raw_text: String,
}

fn build_create_raw_log_request(args: CliArgs) -> Result<CreateRawLogRequest, AppError> {
    let raw_text = args.raw_text.trim().to_string();
    let context_date = args.context_date.map(|value| value.trim().to_string());
    let timezone = args.timezone.map(|value| value.trim().to_string());

    validate_raw_text(&raw_text)?;
    validate_context_date(context_date.as_deref())?;

    Ok(CreateRawLogRequest {
        user_id: args.user_id,
        raw_text,
        input_channel: "cli".to_string(),
        source_type: "manual".to_string(),
        context_date,
        timezone,
    })
}

fn parse_args_from<I, T>(args: I) -> CliArgs
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    CliArgs::parse_from(args)
}

fn api_base_url() -> String {
    resolve_api_base_url(std::env::var("LIFE_OS_API_BASE_URL").ok())
}

fn resolve_api_base_url(value: Option<String>) -> String {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| DEFAULT_API_BASE_URL.to_string())
}

async fn submit_raw_log(
    client: &reqwest::Client,
    base_url: &str,
    request: &CreateRawLogRequest,
) -> Result<RawLogResponse, AppError> {
    let endpoint = format!("{}/logs", base_url.trim_end_matches('/'));
    let response = client
        .post(&endpoint)
        .json(request)
        .send()
        .await
        .map_err(|error| AppError::InternalState(format!("failed to reach backend: {error}")))?;

    if response.status().is_success() {
        return response.json::<RawLogResponse>().await.map_err(|error| {
            AppError::InternalState(format!("invalid backend response: {error}"))
        });
    }

    let status = response.status();
    let error_text = response.text().await.map_err(|error| {
        AppError::InternalState(format!("failed to read error response: {error}"))
    })?;
    let message = parse_error_message(status, &error_text);

    match status {
        reqwest::StatusCode::BAD_REQUEST => Err(AppError::Validation(message)),
        reqwest::StatusCode::NOT_FOUND => Err(AppError::NotFound(message)),
        _ => Err(AppError::InternalState(message)),
    }
}

#[tokio::main]
async fn main() -> ExitCode {
    let args = parse_args_from(std::env::args_os());
    let request = match build_create_raw_log_request(args) {
        Ok(request) => request,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::FAILURE;
        }
    };

    let client = reqwest::Client::new();
    match submit_raw_log(&client, &api_base_url(), &request).await {
        Ok(raw_log) => {
            println!("created raw_log {}", raw_log.id);
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

#[derive(Debug, Deserialize)]
struct CliErrorBody {
    error: CliErrorMessage,
}

#[derive(Debug, Deserialize)]
struct CliErrorMessage {
    message: String,
}

fn parse_error_message(status: reqwest::StatusCode, error_text: &str) -> String {
    match serde_json::from_str::<CliErrorBody>(error_text) {
        Ok(error_body) => error_body.error.message,
        Err(_) => {
            let body = error_text.trim();
            if body.is_empty() {
                format!("backend returned {}", status)
            } else {
                format!("backend returned {}: {}", status, body)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use axum::{Json, Router, routing::post};
    use chrono::{TimeZone, Utc};
    use serde_json::json;
    use tokio::net::TcpListener;

    use super::{
        RawLogResponse, build_create_raw_log_request, parse_args_from, parse_error_message,
        resolve_api_base_url, submit_raw_log,
    };

    #[test]
    fn parses_required_cli_arguments() {
        let args = parse_args_from([
            "logs-cli",
            "--user-id",
            "550e8400-e29b-41d4-a716-446655440001",
            "今天 9:40 起床",
        ]);

        assert_eq!(args.user_id, "550e8400-e29b-41d4-a716-446655440001");
        assert_eq!(args.raw_text, "今天 9:40 起床");
        assert_eq!(args.context_date, None);
    }

    #[test]
    fn maps_cli_arguments_to_create_raw_log_request() {
        let args = parse_args_from([
            "logs-cli",
            "--user-id",
            "550e8400-e29b-41d4-a716-446655440001",
            "--context-date",
            "2026-03-26",
            "--timezone",
            "Asia/Shanghai",
            "今天 9:40 起床",
        ]);

        let request = build_create_raw_log_request(args).expect("mapping should succeed");

        assert_eq!(request.user_id, "550e8400-e29b-41d4-a716-446655440001");
        assert_eq!(request.raw_text, "今天 9:40 起床");
        assert_eq!(request.input_channel, "cli");
        assert_eq!(request.source_type, "manual");
        assert_eq!(request.context_date.as_deref(), Some("2026-03-26"));
        assert_eq!(request.timezone.as_deref(), Some("Asia/Shanghai"));
    }

    #[test]
    fn rejects_empty_raw_text_after_trim() {
        let args = parse_args_from([
            "logs-cli",
            "--user-id",
            "550e8400-e29b-41d4-a716-446655440001",
            "   ",
        ]);

        let error = build_create_raw_log_request(args).expect_err("blank text should fail");

        match error {
            backend::error::AppError::Validation(message) => {
                assert!(message.contains("raw_text"));
            }
            other => panic!("expected validation error, got {other:?}"),
        }
    }

    #[test]
    fn rejects_invalid_context_date() {
        let args = parse_args_from([
            "logs-cli",
            "--user-id",
            "550e8400-e29b-41d4-a716-446655440001",
            "--context-date",
            "2026-99-99",
            "今天 9:40 起床",
        ]);

        let error =
            build_create_raw_log_request(args).expect_err("invalid context date should fail");

        match error {
            backend::error::AppError::Validation(message) => {
                assert!(message.contains("context_date"));
            }
            other => panic!("expected validation error, got {other:?}"),
        }
    }

    #[test]
    fn uses_default_api_base_url_when_value_is_missing() {
        assert_eq!(resolve_api_base_url(None), "http://127.0.0.1:3000");
    }

    #[test]
    fn uses_env_api_base_url_when_value_is_present() {
        assert_eq!(
            resolve_api_base_url(Some("http://127.0.0.1:4010".to_string())),
            "http://127.0.0.1:4010"
        );
    }

    #[tokio::test]
    async fn submit_raw_log_posts_request_to_backend() {
        let app = Router::new().route(
            "/logs",
            post(
                |Json(request): Json<backend::http::dto::logs::CreateRawLogRequest>| async move {
                    assert_eq!(request.user_id, "550e8400-e29b-41d4-a716-446655440001");
                    assert_eq!(request.raw_text, "今天 9:40 起床");
                    assert_eq!(request.input_channel, "cli");

                    Json(sample_raw_log_response())
                },
            ),
        );
        let base_url = spawn_test_server(app).await;
        let client = reqwest::Client::new();
        let request = build_create_raw_log_request(parse_args_from([
            "logs-cli",
            "--user-id",
            "550e8400-e29b-41d4-a716-446655440001",
            "--context-date",
            "2026-03-26",
            "今天 9:40 起床",
        ]))
        .expect("request should build");

        let response = submit_raw_log(&client, &base_url, &request)
            .await
            .expect("submission should succeed");

        assert_eq!(response.id, "550e8400-e29b-41d4-a716-446655440000");
    }

    #[tokio::test]
    async fn submit_raw_log_returns_validation_error_from_backend() {
        let app = Router::new().route(
            "/logs",
            post(|| async {
                (
                    axum::http::StatusCode::BAD_REQUEST,
                    Json(json!({
                        "error": {
                            "code": "validation_error",
                            "message": "raw_text must not be blank"
                        }
                    })),
                )
            }),
        );
        let base_url = spawn_test_server(app).await;
        let client = reqwest::Client::new();
        let request = build_create_raw_log_request(parse_args_from([
            "logs-cli",
            "--user-id",
            "550e8400-e29b-41d4-a716-446655440001",
            "今天 9:40 起床",
        ]))
        .expect("request should build");

        let error = submit_raw_log(&client, &base_url, &request)
            .await
            .expect_err("submission should fail");

        match error {
            backend::error::AppError::Validation(message) => {
                assert!(message.contains("raw_text"));
            }
            other => panic!("expected validation error, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn submit_raw_log_falls_back_to_plain_text_error_body() {
        let app = Router::new().route(
            "/logs",
            post(|| async { (axum::http::StatusCode::NOT_FOUND, "Not Found") }),
        );
        let base_url = spawn_test_server(app).await;
        let client = reqwest::Client::new();
        let request = build_create_raw_log_request(parse_args_from([
            "logs-cli",
            "--user-id",
            "550e8400-e29b-41d4-a716-446655440001",
            "今天 9:40 起床",
        ]))
        .expect("request should build");

        let error = submit_raw_log(&client, &base_url, &request)
            .await
            .expect_err("submission should fail");

        match error {
            backend::error::AppError::NotFound(message) => {
                assert!(message.contains("404"));
                assert!(message.contains("Not Found"));
            }
            other => panic!("expected not found error, got {other:?}"),
        }
    }

    #[test]
    fn parse_error_message_uses_plain_text_fallback() {
        let message = parse_error_message(reqwest::StatusCode::NOT_FOUND, "Not Found");

        assert!(message.contains("404"));
        assert!(message.contains("Not Found"));
    }

    fn sample_raw_log_response() -> RawLogResponse {
        RawLogResponse {
            id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            user_id: "550e8400-e29b-41d4-a716-446655440001".to_string(),
            raw_text: "今天 9:40 起床".to_string(),
            input_channel: "cli".to_string(),
            source_type: "manual".to_string(),
            context_date: Some("2026-03-26".to_string()),
            timezone: Some("Asia/Shanghai".to_string()),
            parse_status: "pending".to_string(),
            parser_version: None,
            parse_error: None,
            created_at: Utc
                .with_ymd_and_hms(2026, 3, 26, 2, 0, 0)
                .single()
                .expect("time should be valid")
                .to_rfc3339(),
            updated_at: Utc
                .with_ymd_and_hms(2026, 3, 26, 2, 0, 0)
                .single()
                .expect("time should be valid")
                .to_rfc3339(),
        }
    }

    async fn spawn_test_server(app: Router) -> String {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("listener should bind");
        let address = listener.local_addr().expect("address should exist");

        tokio::spawn(async move {
            axum::serve(listener, app).await.expect("server should run");
        });

        format!("http://{address}")
    }
}
