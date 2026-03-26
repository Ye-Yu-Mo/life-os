use serde::{Deserialize, Serialize};

use crate::domain::raw_logs::{CreateRawLog, InputChannel, RawLog, SourceType};

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateRawLogRequest {
    pub user_id: String,
    pub raw_text: String,
    pub input_channel: String,
    pub source_type: String,
    pub context_date: Option<String>,
    pub timezone: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RawLogResponse {
    pub id: String,
    pub user_id: String,
    pub raw_text: String,
    pub input_channel: String,
    pub source_type: String,
    pub context_date: Option<String>,
    pub timezone: Option<String>,
    pub parse_status: String,
    pub parser_version: Option<String>,
    pub parse_error: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ImportRawLogsRequest {
    pub format: String,
    pub records: Option<Vec<CreateRawLogRequest>>,
    pub content: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ImportRawLogsResponse {
    pub total_count: usize,
    pub success_count: usize,
    pub failure_count: usize,
    pub errors: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct CsvRawLogRecord {
    user_id: String,
    raw_text: String,
    input_channel: String,
    source_type: String,
    context_date: Option<String>,
    timezone: Option<String>,
}

impl TryFrom<CreateRawLogRequest> for CreateRawLog {
    type Error = crate::error::AppError;

    fn try_from(value: CreateRawLogRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            user_id: value.user_id,
            raw_text: value.raw_text,
            input_channel: parse_input_channel(&value.input_channel)?,
            source_type: parse_source_type(&value.source_type)?,
            context_date: value.context_date,
            timezone: value.timezone,
        })
    }
}

impl From<RawLog> for RawLogResponse {
    fn from(value: RawLog) -> Self {
        Self {
            id: value.id,
            user_id: value.user_id,
            raw_text: value.raw_text,
            input_channel: format_input_channel(value.input_channel).to_string(),
            source_type: format_source_type(value.source_type).to_string(),
            context_date: value.context_date,
            timezone: value.timezone,
            parse_status: format_parse_status(value.parse_status).to_string(),
            parser_version: value.parser_version,
            parse_error: value.parse_error,
            created_at: value.created_at.to_rfc3339(),
            updated_at: value.updated_at.to_rfc3339(),
        }
    }
}

impl ImportRawLogsRequest {
    pub fn try_into_create_raw_logs(self) -> Result<Vec<CreateRawLog>, crate::error::AppError> {
        match self.format.as_str() {
            "json" => self.parse_json_records(),
            "csv" => self.parse_csv_records(),
            other => Err(crate::error::AppError::Validation(format!(
                "invalid import format: {other}"
            ))),
        }
    }

    fn parse_json_records(self) -> Result<Vec<CreateRawLog>, crate::error::AppError> {
        let records = self.records.ok_or_else(|| {
            crate::error::AppError::Validation("json import requires records".to_string())
        })?;

        if records.is_empty() {
            return Err(crate::error::AppError::Validation(
                "import records cannot be empty".to_string(),
            ));
        }

        records.into_iter().map(TryInto::try_into).collect()
    }

    fn parse_csv_records(self) -> Result<Vec<CreateRawLog>, crate::error::AppError> {
        let content = self.content.ok_or_else(|| {
            crate::error::AppError::Validation("csv import requires content".to_string())
        })?;

        let mut reader = csv::Reader::from_reader(content.as_bytes());
        let mut records = Vec::new();

        for row in reader.deserialize::<CsvRawLogRecord>() {
            let record = row.map_err(|error| {
                crate::error::AppError::Validation(format!("invalid csv import: {error}"))
            })?;

            records.push(CreateRawLogRequest {
                user_id: record.user_id,
                raw_text: record.raw_text,
                input_channel: record.input_channel,
                source_type: record.source_type,
                context_date: record.context_date,
                timezone: record.timezone,
            }
            .try_into()?);
        }

        if records.is_empty() {
            return Err(crate::error::AppError::Validation(
                "import records cannot be empty".to_string(),
            ));
        }

        Ok(records)
    }
}

fn parse_input_channel(value: &str) -> Result<InputChannel, crate::error::AppError> {
    match value {
        "web" => Ok(InputChannel::Web),
        "mobile" => Ok(InputChannel::Mobile),
        "cli" => Ok(InputChannel::Cli),
        "api" => Ok(InputChannel::Api),
        "import" => Ok(InputChannel::Import),
        other => Err(crate::error::AppError::Validation(format!(
            "invalid input_channel: {other}"
        ))),
    }
}

fn parse_source_type(value: &str) -> Result<SourceType, crate::error::AppError> {
    match value {
        "manual" => Ok(SourceType::Manual),
        "imported" => Ok(SourceType::Imported),
        "synced" => Ok(SourceType::Synced),
        other => Err(crate::error::AppError::Validation(format!(
            "invalid source_type: {other}"
        ))),
    }
}

fn format_input_channel(value: InputChannel) -> &'static str {
    match value {
        InputChannel::Web => "web",
        InputChannel::Mobile => "mobile",
        InputChannel::Cli => "cli",
        InputChannel::Api => "api",
        InputChannel::Import => "import",
    }
}

fn format_source_type(value: SourceType) -> &'static str {
    match value {
        SourceType::Manual => "manual",
        SourceType::Imported => "imported",
        SourceType::Synced => "synced",
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
