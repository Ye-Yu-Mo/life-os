use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::domain::raw_logs::{CreateRawLog, RawLog};
use crate::error::AppError;

#[async_trait]
pub trait RawLogRepository: Send + Sync {
    async fn create(&self, input: CreateRawLog) -> Result<RawLog, AppError>;
    async fn create_many(&self, inputs: Vec<CreateRawLog>) -> Result<Vec<RawLog>, AppError>;
    async fn list(&self) -> Result<Vec<RawLog>, AppError>;
    async fn get_by_id(&self, id: &str) -> Result<Option<RawLog>, AppError>;
}

pub struct PgRawLogRepository {
    pool: PgPool,
}

impl PgRawLogRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(Debug, FromRow)]
struct RawLogRecord {
    id: String,
    user_id: String,
    raw_text: String,
    input_channel: String,
    source_type: String,
    context_date: Option<String>,
    timezone: Option<String>,
    parse_status: String,
    parser_version: Option<String>,
    parse_error: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<RawLogRecord> for RawLog {
    type Error = AppError;

    fn try_from(value: RawLogRecord) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id,
            user_id: value.user_id,
            raw_text: value.raw_text,
            input_channel: parse_input_channel(&value.input_channel)?,
            source_type: parse_source_type(&value.source_type)?,
            context_date: value.context_date,
            timezone: value.timezone,
            parse_status: parse_status(&value.parse_status)?,
            parser_version: value.parser_version,
            parse_error: value.parse_error,
            created_at: value.created_at,
            updated_at: value.updated_at,
        })
    }
}

#[async_trait]
impl RawLogRepository for PgRawLogRepository {
    async fn create(&self, input: CreateRawLog) -> Result<RawLog, AppError> {
        let mut created = self.create_many(vec![input]).await?;
        created
            .pop()
            .ok_or_else(|| AppError::InternalState("batch insert returned no rows".to_string()))
    }

    async fn create_many(&self, inputs: Vec<CreateRawLog>) -> Result<Vec<RawLog>, AppError> {
        let mut transaction = self.pool.begin().await?;
        let mut created = Vec::with_capacity(inputs.len());

        for input in inputs {
            let record = insert_raw_log(&mut transaction, input).await?;
            created.push(record.try_into()?);
        }

        transaction.commit().await?;
        Ok(created)
    }

    async fn list(&self) -> Result<Vec<RawLog>, AppError> {
        let records = sqlx::query_as::<_, RawLogRecord>(
            r#"
            SELECT
                id::text AS id,
                user_id::text AS user_id,
                raw_text,
                input_channel,
                source_type,
                context_date::text AS context_date,
                timezone,
                parse_status,
                parser_version,
                parse_error,
                created_at,
                updated_at
            FROM raw_logs
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        records.into_iter().map(TryInto::try_into).collect()
    }

    async fn get_by_id(&self, id: &str) -> Result<Option<RawLog>, AppError> {
        let record = sqlx::query_as::<_, RawLogRecord>(
            r#"
            SELECT
                id::text AS id,
                user_id::text AS user_id,
                raw_text,
                input_channel,
                source_type,
                context_date::text AS context_date,
                timezone,
                parse_status,
                parser_version,
                parse_error,
                created_at,
                updated_at
            FROM raw_logs
            WHERE id = $1
            "#,
        )
        .bind(parse_uuid(id)?)
        .fetch_optional(&self.pool)
        .await?;

        record.map(TryInto::try_into).transpose()
    }
}

async fn insert_raw_log(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    input: CreateRawLog,
) -> Result<RawLogRecord, AppError> {
    let record = sqlx::query_as::<_, RawLogRecord>(
        r#"
        INSERT INTO raw_logs (
            id,
            user_id,
            raw_text,
            input_channel,
            source_type,
            context_date,
            timezone,
            parse_status,
            parser_version,
            parse_error
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NULL, NULL)
        RETURNING
            id::text AS id,
            user_id::text AS user_id,
            raw_text,
            input_channel,
            source_type,
            context_date::text AS context_date,
            timezone,
            parse_status,
            parser_version,
            parse_error,
            created_at,
            updated_at
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(parse_uuid(&input.user_id)?)
    .bind(input.raw_text)
    .bind(format_input_channel(input.input_channel))
    .bind(format_source_type(input.source_type))
    .bind(parse_context_date(input.context_date.as_deref())?)
    .bind(input.timezone)
    .bind(format_parse_status(crate::domain::raw_logs::ParseStatus::Pending))
    .fetch_one(transaction.as_mut())
    .await?;

    Ok(record)
}

fn parse_uuid(value: &str) -> Result<Uuid, AppError> {
    Uuid::parse_str(value).map_err(|error| AppError::Validation(format!("invalid uuid: {error}")))
}

fn parse_context_date(value: Option<&str>) -> Result<Option<NaiveDate>, AppError> {
    value
        .map(|raw| {
            NaiveDate::parse_from_str(raw, "%Y-%m-%d")
                .map_err(|error| AppError::Validation(format!("invalid context_date: {error}")))
        })
        .transpose()
}

fn parse_input_channel(value: &str) -> Result<crate::domain::raw_logs::InputChannel, AppError> {
    match value {
        "web" => Ok(crate::domain::raw_logs::InputChannel::Web),
        "mobile" => Ok(crate::domain::raw_logs::InputChannel::Mobile),
        "cli" => Ok(crate::domain::raw_logs::InputChannel::Cli),
        "api" => Ok(crate::domain::raw_logs::InputChannel::Api),
        "import" => Ok(crate::domain::raw_logs::InputChannel::Import),
        other => Err(AppError::InternalState(format!(
            "unknown input_channel value: {other}"
        ))),
    }
}

fn parse_source_type(value: &str) -> Result<crate::domain::raw_logs::SourceType, AppError> {
    match value {
        "manual" => Ok(crate::domain::raw_logs::SourceType::Manual),
        "imported" => Ok(crate::domain::raw_logs::SourceType::Imported),
        "synced" => Ok(crate::domain::raw_logs::SourceType::Synced),
        other => Err(AppError::InternalState(format!(
            "unknown source_type value: {other}"
        ))),
    }
}

fn parse_status(value: &str) -> Result<crate::domain::raw_logs::ParseStatus, AppError> {
    match value {
        "pending" => Ok(crate::domain::raw_logs::ParseStatus::Pending),
        "parsed" => Ok(crate::domain::raw_logs::ParseStatus::Parsed),
        "partial" => Ok(crate::domain::raw_logs::ParseStatus::Partial),
        "failed" => Ok(crate::domain::raw_logs::ParseStatus::Failed),
        "needs_review" => Ok(crate::domain::raw_logs::ParseStatus::NeedsReview),
        other => Err(AppError::InternalState(format!(
            "unknown parse_status value: {other}"
        ))),
    }
}

fn format_input_channel(value: crate::domain::raw_logs::InputChannel) -> &'static str {
    match value {
        crate::domain::raw_logs::InputChannel::Web => "web",
        crate::domain::raw_logs::InputChannel::Mobile => "mobile",
        crate::domain::raw_logs::InputChannel::Cli => "cli",
        crate::domain::raw_logs::InputChannel::Api => "api",
        crate::domain::raw_logs::InputChannel::Import => "import",
    }
}

fn format_source_type(value: crate::domain::raw_logs::SourceType) -> &'static str {
    match value {
        crate::domain::raw_logs::SourceType::Manual => "manual",
        crate::domain::raw_logs::SourceType::Imported => "imported",
        crate::domain::raw_logs::SourceType::Synced => "synced",
    }
}

fn format_parse_status(value: crate::domain::raw_logs::ParseStatus) -> &'static str {
    match value {
        crate::domain::raw_logs::ParseStatus::Pending => "pending",
        crate::domain::raw_logs::ParseStatus::Parsed => "parsed",
        crate::domain::raw_logs::ParseStatus::Partial => "partial",
        crate::domain::raw_logs::ParseStatus::Failed => "failed",
        crate::domain::raw_logs::ParseStatus::NeedsReview => "needs_review",
    }
}
