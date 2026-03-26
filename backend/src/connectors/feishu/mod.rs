#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeishuConnectorConfig {
    pub enabled: bool,
}

impl FeishuConnectorConfig {
    pub fn disabled() -> Self {
        Self { enabled: false }
    }
}
