use std::sync::Arc;

use crate::domain::raw_logs::{CreateRawLog, RawLog};
use crate::error::AppError;
use crate::repository::raw_logs::RawLogRepository;
use crate::validation::logs::{validate_context_date, validate_raw_text};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportRawLogsResult {
    pub total_count: usize,
    pub success_count: usize,
    pub failure_count: usize,
    pub errors: Vec<String>,
}

pub struct RawLogService {
    repository: Arc<dyn RawLogRepository>,
}

impl RawLogService {
    pub fn new(repository: Arc<dyn RawLogRepository>) -> Self {
        Self { repository }
    }

    pub async fn create(&self, input: CreateRawLog) -> Result<RawLog, AppError> {
        self.validate_input(&input)?;
        self.repository.create(input).await
    }

    pub async fn create_connector_input(&self, input: CreateRawLog) -> Result<RawLog, AppError> {
        self.validate_input(&input)?;
        self.repository.create(input).await
    }

    pub async fn list(&self) -> Result<Vec<RawLog>, AppError> {
        self.repository.list().await
    }

    pub async fn get_by_id(&self, id: &str) -> Result<Option<RawLog>, AppError> {
        self.repository.get_by_id(id).await
    }

    pub async fn import(&self, inputs: Vec<CreateRawLog>) -> Result<ImportRawLogsResult, AppError> {
        if inputs.is_empty() {
            return Err(AppError::Validation(
                "import records cannot be empty".to_string(),
            ));
        }

        for (index, input) in inputs.iter().enumerate() {
            self.validate_input(input)
                .map_err(|error| annotate_import_validation_error(index, error))?;
        }

        let total_count = inputs.len();
        self.repository
            .create_many(inputs)
            .await
            .map_err(annotate_import_persistence_error)?;

        Ok(ImportRawLogsResult {
            total_count,
            success_count: total_count,
            failure_count: 0,
            errors: vec![],
        })
    }

    fn validate_input(&self, input: &CreateRawLog) -> Result<(), AppError> {
        validate_raw_text(&input.raw_text)?;
        validate_context_date(input.context_date.as_deref())?;
        Ok(())
    }
}

fn annotate_import_validation_error(index: usize, error: AppError) -> AppError {
    match error {
        AppError::Validation(message) => {
            AppError::Validation(format!("record {}: {}", index + 1, message))
        }
        other => other,
    }
}

fn annotate_import_persistence_error(error: AppError) -> AppError {
    match error {
        AppError::InternalState(message) => AppError::InternalState(format!(
            "batch import failed and no records were persisted: {message}"
        )),
        AppError::Database(error) => AppError::InternalState(format!(
            "batch import failed and no records were persisted: {error}"
        )),
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;
    use chrono::{TimeZone, Utc};

    use crate::domain::raw_logs::{CreateRawLog, InputChannel, ParseStatus, RawLog, SourceType};
    use crate::error::AppError;
    use crate::repository::raw_logs::RawLogRepository;
    use crate::service::raw_logs::RawLogService;

    #[derive(Default)]
    struct FakeRawLogRepository {
        created_inputs: Mutex<Vec<CreateRawLog>>,
        list_response: Mutex<Vec<RawLog>>,
        get_response: Mutex<Option<RawLog>>,
        batch_error_message: Mutex<Option<String>>,
    }

    #[async_trait]
    impl RawLogRepository for FakeRawLogRepository {
        async fn create(&self, input: CreateRawLog) -> Result<RawLog, AppError> {
            self.created_inputs
                .lock()
                .expect("mutex should not be poisoned")
                .push(input.clone());

            Ok(sample_raw_log_from_input(input))
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
                created.push(sample_raw_log_from_input(input));
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
    }

    fn sample_create_raw_log() -> CreateRawLog {
        CreateRawLog {
            user_id: "user-1".to_string(),
            raw_text: "今天 9:40 起床".to_string(),
            input_channel: InputChannel::Web,
            source_type: SourceType::Manual,
            context_date: Some("2026-03-26".to_string()),
            timezone: Some("Asia/Shanghai".to_string()),
        }
    }

    fn sample_raw_log_from_input(input: CreateRawLog) -> RawLog {
        RawLog {
            id: "log-1".to_string(),
            user_id: input.user_id,
            raw_text: input.raw_text,
            input_channel: input.input_channel,
            source_type: input.source_type,
            context_date: input.context_date,
            timezone: input.timezone,
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

    #[tokio::test]
    async fn create_delegates_to_repository_and_returns_pending_raw_log() {
        let repository = Arc::new(FakeRawLogRepository::default());
        let service = RawLogService::new(repository.clone());

        let created = service
            .create(sample_create_raw_log())
            .await
            .expect("create should succeed");

        let stored_inputs = repository
            .created_inputs
            .lock()
            .expect("mutex should not be poisoned");

        assert_eq!(stored_inputs.len(), 1);
        assert_eq!(created.parse_status, ParseStatus::Pending);
        assert_eq!(created.raw_text, "今天 9:40 起床");
    }

    #[tokio::test]
    async fn list_returns_repository_results() {
        let repository = Arc::new(FakeRawLogRepository::default());
        repository
            .list_response
            .lock()
            .expect("mutex should not be poisoned")
            .push(sample_raw_log_from_input(sample_create_raw_log()));

        let service = RawLogService::new(repository);
        let logs = service.list().await.expect("list should succeed");

        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].id, "log-1");
    }

    #[tokio::test]
    async fn get_by_id_returns_repository_result() {
        let repository = Arc::new(FakeRawLogRepository::default());
        repository
            .get_response
            .lock()
            .expect("mutex should not be poisoned")
            .replace(sample_raw_log_from_input(sample_create_raw_log()));

        let service = RawLogService::new(repository);
        let log = service
            .get_by_id("log-1")
            .await
            .expect("get_by_id should succeed")
            .expect("raw log should exist");

        assert_eq!(log.id, "log-1");
        assert_eq!(log.parse_status, ParseStatus::Pending);
    }

    #[tokio::test]
    async fn create_rejects_empty_raw_text_before_repository_call() {
        let repository = Arc::new(FakeRawLogRepository::default());
        let service = RawLogService::new(repository.clone());

        let error = service
            .create(CreateRawLog {
                raw_text: "".to_string(),
                ..sample_create_raw_log()
            })
            .await
            .expect_err("empty raw_text should fail");

        match error {
            AppError::Validation(message) => assert!(message.contains("raw_text")),
            other => panic!("expected validation error, got {other:?}"),
        }

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
    async fn create_rejects_invalid_context_date_before_repository_call() {
        let repository = Arc::new(FakeRawLogRepository::default());
        let service = RawLogService::new(repository.clone());

        let error = service
            .create(CreateRawLog {
                context_date: Some("2026-99-99".to_string()),
                ..sample_create_raw_log()
            })
            .await
            .expect_err("invalid context_date should fail");

        match error {
            AppError::Validation(message) => assert!(message.contains("context_date")),
            other => panic!("expected validation error, got {other:?}"),
        }

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
    async fn import_creates_all_records_when_batch_is_valid() {
        let repository = Arc::new(FakeRawLogRepository::default());
        let service = RawLogService::new(repository.clone());

        let result = service
            .import(vec![
                sample_create_raw_log(),
                CreateRawLog {
                    raw_text: "晚上跑步 35 分钟".to_string(),
                    input_channel: InputChannel::Import,
                    source_type: SourceType::Imported,
                    ..sample_create_raw_log()
                },
            ])
            .await
            .expect("import should succeed");

        assert_eq!(result.total_count, 2);
        assert_eq!(result.success_count, 2);
        assert_eq!(result.failure_count, 0);
        assert!(result.errors.is_empty());
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
    async fn import_rejects_invalid_batch_before_repository_call() {
        let repository = Arc::new(FakeRawLogRepository::default());
        let service = RawLogService::new(repository.clone());

        let error = service
            .import(vec![
                sample_create_raw_log(),
                CreateRawLog {
                    raw_text: "".to_string(),
                    input_channel: InputChannel::Import,
                    source_type: SourceType::Imported,
                    ..sample_create_raw_log()
                },
            ])
            .await
            .expect_err("import should fail");

        match error {
            AppError::Validation(message) => {
                assert!(message.contains("record 2"));
                assert!(message.contains("raw_text"));
            }
            other => panic!("expected validation error, got {other:?}"),
        }

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
    async fn create_connector_input_reuses_service_validation_and_persists_synced_log() {
        let repository = Arc::new(FakeRawLogRepository::default());
        let service = RawLogService::new(repository.clone());

        let created = service
            .create_connector_input(CreateRawLog {
                user_id: "user-1".to_string(),
                raw_text: "来自 Telegram 的同步消息".to_string(),
                input_channel: InputChannel::Telegram,
                source_type: SourceType::Synced,
                context_date: Some("2026-03-26".to_string()),
                timezone: Some("Asia/Shanghai".to_string()),
            })
            .await
            .expect("connector input should persist");

        assert_eq!(created.input_channel, InputChannel::Telegram);
        assert_eq!(created.source_type, SourceType::Synced);
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
    async fn import_explains_that_failed_batch_did_not_persist_any_records() {
        let repository = Arc::new(FakeRawLogRepository::default());
        repository
            .batch_error_message
            .lock()
            .expect("mutex should not be poisoned")
            .replace("simulated batch failure".to_string());
        let service = RawLogService::new(repository.clone());

        let error = service
            .import(vec![
                CreateRawLog {
                    input_channel: InputChannel::Import,
                    source_type: SourceType::Imported,
                    ..sample_create_raw_log()
                },
                CreateRawLog {
                    raw_text: "晚上跑步 35 分钟".to_string(),
                    input_channel: InputChannel::Import,
                    source_type: SourceType::Imported,
                    ..sample_create_raw_log()
                },
            ])
            .await
            .expect_err("batch failure should surface");

        match error {
            AppError::InternalState(message) => {
                assert!(message.contains("no records were persisted"));
            }
            other => panic!("expected internal state error, got {other:?}"),
        }

        assert_eq!(
            repository
                .created_inputs
                .lock()
                .expect("mutex should not be poisoned")
                .len(),
            0
        );
    }

    #[test]
    fn raw_logs_migration_contains_required_columns() {
        let migration = std::fs::read_to_string("migrations/202603260001_create_raw_logs.sql")
            .expect("migration file should exist");

        for column in [
            "id",
            "user_id",
            "raw_text",
            "input_channel",
            "source_type",
            "context_date",
            "timezone",
            "parse_status",
            "parser_version",
            "parse_error",
            "created_at",
            "updated_at",
        ] {
            assert!(
                migration.contains(column),
                "migration should contain required column: {column}"
            );
        }
    }
}
