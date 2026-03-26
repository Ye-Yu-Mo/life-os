use chrono::DateTime;

use crate::config::TelegramConfig;
use crate::domain::raw_logs::{CreateRawLog, InputChannel, SourceType};
use crate::error::AppError;

pub use crate::config::ConnectorRuntimeMode as TelegramCallbackMode;
pub use crate::config::TelegramConfig as TelegramConnectorConfig;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TelegramIncomingMessage {
    pub message_id: i64,
    pub chat_id: i64,
    pub user_id: String,
    pub text: String,
    pub sent_at: String,
    pub timezone: Option<String>,
}

pub fn map_message_to_raw_log(
    config: &TelegramConfig,
    message: TelegramIncomingMessage,
) -> Result<CreateRawLog, AppError> {
    if !config.enabled {
        return Err(AppError::Validation(
            "telegram connector is disabled".to_string(),
        ));
    }

    if !config.allowlist_chat_ids.is_empty() && !config.allowlist_chat_ids.contains(&message.chat_id)
    {
        return Err(AppError::Validation(format!(
            "telegram chat_id {} is not in allowlist",
            message.chat_id
        )));
    }

    let context_date = DateTime::parse_from_rfc3339(&message.sent_at)
        .ok()
        .map(|value| value.date_naive().to_string());

    Ok(CreateRawLog {
        user_id: message.user_id,
        raw_text: message.text,
        input_channel: InputChannel::Telegram,
        source_type: SourceType::Synced,
        context_date,
        timezone: message.timezone,
    })
}

#[cfg(test)]
mod tests {
    use crate::connectors::telegram::{
        TelegramCallbackMode, TelegramConnectorConfig, TelegramIncomingMessage,
        map_message_to_raw_log,
    };
    use crate::domain::raw_logs::{InputChannel, SourceType};

    #[test]
    fn telegram_message_maps_to_synced_raw_log_input() {
        let config = TelegramConnectorConfig {
            enabled: true,
            bot_token: Some("test-bot-token".to_string()),
            allowlist_chat_ids: vec![10001],
            callback_mode: TelegramCallbackMode::Webhook,
            webhook_base_url: Some("https://example.com".to_string()),
        };

        let mapped = map_message_to_raw_log(
            &config,
            TelegramIncomingMessage {
                message_id: 42,
                chat_id: 10001,
                user_id: "550e8400-e29b-41d4-a716-446655440001".to_string(),
                text: "今天 9:40 起床".to_string(),
                sent_at: "2026-03-26T09:40:00+08:00".to_string(),
                timezone: Some("Asia/Shanghai".to_string()),
            },
        )
        .expect("message should map");

        assert_eq!(mapped.input_channel, InputChannel::Telegram);
        assert_eq!(mapped.source_type, SourceType::Synced);
        assert_eq!(mapped.raw_text, "今天 9:40 起床");
        assert_eq!(mapped.timezone.as_deref(), Some("Asia/Shanghai"));
    }

    #[test]
    fn telegram_mapping_rejects_messages_from_disallowed_chat() {
        let config = TelegramConnectorConfig {
            enabled: true,
            bot_token: Some("test-bot-token".to_string()),
            allowlist_chat_ids: vec![10001],
            callback_mode: TelegramCallbackMode::Webhook,
            webhook_base_url: Some("https://example.com".to_string()),
        };

        let error = map_message_to_raw_log(
            &config,
            TelegramIncomingMessage {
                message_id: 42,
                chat_id: 20002,
                user_id: "550e8400-e29b-41d4-a716-446655440001".to_string(),
                text: "今天 9:40 起床".to_string(),
                sent_at: "2026-03-26T09:40:00+08:00".to_string(),
                timezone: Some("Asia/Shanghai".to_string()),
            },
        )
        .expect_err("disallowed chat should fail");

        assert!(
            error.to_string().contains("allowlist"),
            "error should mention allowlist rejection"
        );
    }
}
