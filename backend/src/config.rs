use crate::error::AppError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub telegram: TelegramConfig,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TelegramConfig {
    pub enabled: bool,
    pub bot_token: Option<String>,
    pub allowlist_chat_ids: Vec<i64>,
    pub callback_mode: TelegramCallbackMode,
    pub webhook_base_url: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TelegramCallbackMode {
    Polling,
    Webhook,
}

impl Config {
    pub fn from_env() -> Result<Self, AppError> {
        Self::from_env_map(std::env::vars())
    }

    pub fn from_env_map<I, K, V>(vars: I) -> Result<Self, AppError>
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        let env = vars
            .into_iter()
            .map(|(key, value)| (key.as_ref().to_string(), value.as_ref().to_string()))
            .collect::<std::collections::HashMap<_, _>>();

        let host = env
            .get("APP_HOST")
            .cloned()
            .unwrap_or_else(|| "127.0.0.1".to_string());
        let port = env
            .get("APP_PORT")
            .map(|value| value.parse::<u16>())
            .transpose()
            .map_err(|error| AppError::Config(format!("invalid APP_PORT: {error}")))?
            .unwrap_or(3000);
        let database_url = env
            .get("DATABASE_URL")
            .cloned()
            .ok_or_else(|| AppError::Config("missing DATABASE_URL".to_string()))?;
        let max_connections = env
            .get("DATABASE_MAX_CONNECTIONS")
            .map(|value| value.parse::<u32>())
            .transpose()
            .map_err(|error| AppError::Config(format!("invalid DATABASE_MAX_CONNECTIONS: {error}")))?
            .unwrap_or(5);
        let telegram_enabled = env
            .get("TELEGRAM_ENABLED")
            .map(|value| parse_bool(value, "TELEGRAM_ENABLED"))
            .transpose()?
            .unwrap_or(false);
        let telegram_bot_token = env
            .get("TELEGRAM_BOT_TOKEN")
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        let telegram_allowlist_chat_ids = env
            .get("TELEGRAM_ALLOWLIST_CHAT_IDS")
            .map(|value| parse_allowlist_chat_ids(value))
            .transpose()?
            .unwrap_or_default();
        let telegram_callback_mode = env
            .get("TELEGRAM_CALLBACK_MODE")
            .map(|value| parse_callback_mode(value))
            .transpose()?
            .unwrap_or(TelegramCallbackMode::Polling);
        let telegram_webhook_base_url = env
            .get("TELEGRAM_WEBHOOK_BASE_URL")
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());

        Ok(Self {
            server: ServerConfig { host, port },
            database: DatabaseConfig {
                url: database_url,
                max_connections,
            },
            telegram: TelegramConfig {
                enabled: telegram_enabled,
                bot_token: telegram_bot_token,
                allowlist_chat_ids: telegram_allowlist_chat_ids,
                callback_mode: telegram_callback_mode,
                webhook_base_url: telegram_webhook_base_url,
            },
        })
    }
}

fn parse_bool(value: &str, key: &str) -> Result<bool, AppError> {
    match value.trim() {
        "true" => Ok(true),
        "false" => Ok(false),
        other => Err(AppError::Config(format!("invalid {key}: {other}"))),
    }
}

fn parse_allowlist_chat_ids(value: &str) -> Result<Vec<i64>, AppError> {
    value
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| {
            value.parse::<i64>().map_err(|error| {
                AppError::Config(format!("invalid TELEGRAM_ALLOWLIST_CHAT_IDS: {error}"))
            })
        })
        .collect()
}

fn parse_callback_mode(value: &str) -> Result<TelegramCallbackMode, AppError> {
    match value.trim() {
        "polling" => Ok(TelegramCallbackMode::Polling),
        "webhook" => Ok(TelegramCallbackMode::Webhook),
        other => Err(AppError::Config(format!(
            "invalid TELEGRAM_CALLBACK_MODE: {other}"
        ))),
    }
}
