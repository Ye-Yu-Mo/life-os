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

            records.push(
                CreateRawLogRequest {
                    user_id: record.user_id,
                    raw_text: record.raw_text,
                    input_channel: record.input_channel,
                    source_type: record.source_type,
                    context_date: record.context_date,
                    timezone: record.timezone,
                }
                .try_into()?,
            );
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
        "telegram" => Ok(InputChannel::Telegram),
        "feishu" => Ok(InputChannel::Feishu),
        "wechat_bridge" => Ok(InputChannel::WechatBridge),
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
        InputChannel::Telegram => "telegram",
        InputChannel::Feishu => "feishu",
        InputChannel::WechatBridge => "wechat_bridge",
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

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::{CreateRawLogRequest, RawLogResponse};
    use crate::domain::raw_logs::{CreateRawLog, InputChannel, ParseStatus, RawLog, SourceType};

    #[test]
    fn create_raw_log_request_accepts_expanded_input_channels() {
        for input_channel in [
            "mobile",
            "cli",
            "import",
            "telegram",
            "feishu",
            "wechat_bridge",
        ] {
            let request = CreateRawLogRequest {
                user_id: "550e8400-e29b-41d4-a716-446655440001".to_string(),
                raw_text: "今天 9:40 起床".to_string(),
                input_channel: input_channel.to_string(),
                source_type: "manual".to_string(),
                context_date: Some("2026-03-26".to_string()),
                timezone: Some("Asia/Shanghai".to_string()),
            };

            let created: CreateRawLog = request
                .try_into()
                .expect("expanded input channel should parse");

            match input_channel {
                "mobile" => assert_eq!(created.input_channel, InputChannel::Mobile),
                "cli" => assert_eq!(created.input_channel, InputChannel::Cli),
                "import" => assert_eq!(created.input_channel, InputChannel::Import),
                "telegram" => assert_eq!(created.input_channel, InputChannel::Telegram),
                "feishu" => assert_eq!(created.input_channel, InputChannel::Feishu),
                "wechat_bridge" => {
                    assert_eq!(created.input_channel, InputChannel::WechatBridge)
                }
                _ => unreachable!("unexpected input channel"),
            }
        }
    }

    #[test]
    fn raw_log_response_formats_expanded_input_channels() {
        let created_at = Utc
            .with_ymd_and_hms(2026, 3, 26, 2, 0, 0)
            .single()
            .expect("time should be valid");

        for (input_channel, expected) in [
            (InputChannel::Telegram, "telegram"),
            (InputChannel::Feishu, "feishu"),
            (InputChannel::WechatBridge, "wechat_bridge"),
        ] {
            let response = RawLogResponse::from(RawLog {
                id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
                user_id: "550e8400-e29b-41d4-a716-446655440001".to_string(),
                raw_text: "今天 9:40 起床".to_string(),
                input_channel,
                source_type: SourceType::Manual,
                context_date: Some("2026-03-26".to_string()),
                timezone: Some("Asia/Shanghai".to_string()),
                parse_status: ParseStatus::Pending,
                parser_version: None,
                parse_error: None,
                created_at,
                updated_at: created_at,
            });

            assert_eq!(response.input_channel, expected);
        }
    }
}
