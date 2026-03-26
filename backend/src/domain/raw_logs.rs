use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputChannel {
    Web,
    Mobile,
    Cli,
    Api,
    Import,
    Telegram,
    Feishu,
    WechatBridge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceType {
    Manual,
    Imported,
    Synced,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseStatus {
    Pending,
    Parsed,
    Partial,
    Failed,
    NeedsReview,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawLog {
    pub id: String,
    pub user_id: String,
    pub raw_text: String,
    pub input_channel: InputChannel,
    pub source_type: SourceType,
    pub context_date: Option<String>,
    pub timezone: Option<String>,
    pub parse_status: ParseStatus,
    pub parser_version: Option<String>,
    pub parse_error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateRawLog {
    pub user_id: String,
    pub raw_text: String,
    pub input_channel: InputChannel,
    pub source_type: SourceType,
    pub context_date: Option<String>,
    pub timezone: Option<String>,
}
