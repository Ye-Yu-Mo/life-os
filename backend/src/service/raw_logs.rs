use std::sync::Arc;

use crate::domain::raw_logs::{CreateRawLog, RawLog};
use crate::error::AppError;
use crate::repository::raw_logs::RawLogRepository;

pub struct RawLogService {
    repository: Arc<dyn RawLogRepository>,
}

impl RawLogService {
    pub fn new(repository: Arc<dyn RawLogRepository>) -> Self {
        Self { repository }
    }

    pub async fn create(&self, input: CreateRawLog) -> Result<RawLog, AppError> {
        self.repository.create(input).await
    }

    pub async fn list(&self) -> Result<Vec<RawLog>, AppError> {
        self.repository.list().await
    }

    pub async fn get_by_id(&self, id: &str) -> Result<Option<RawLog>, AppError> {
        self.repository.get_by_id(id).await
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
