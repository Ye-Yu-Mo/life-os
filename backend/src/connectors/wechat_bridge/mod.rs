#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WechatBridgeConnectorConfig {
    pub enabled: bool,
}

impl WechatBridgeConnectorConfig {
    pub fn disabled() -> Self {
        Self { enabled: false }
    }
}
