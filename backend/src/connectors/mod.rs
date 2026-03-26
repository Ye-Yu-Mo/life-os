pub mod feishu;
pub mod telegram;
pub mod wechat_bridge;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectorKind {
    Telegram,
    Feishu,
    WechatBridge,
}

impl ConnectorKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Telegram => "telegram",
            Self::Feishu => "feishu",
            Self::WechatBridge => "wechat_bridge",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectorRuntimeMode {
    Polling,
    Webhook,
    Bridge,
}

impl ConnectorRuntimeMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Polling => "polling",
            Self::Webhook => "webhook",
            Self::Bridge => "bridge",
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::connectors::{
        ConnectorKind, ConnectorRuntimeMode, feishu::FeishuConnectorConfig,
        wechat_bridge::WechatBridgeConnectorConfig,
    };

    #[test]
    fn reserves_connector_kinds_for_feishu_and_wechat_bridge() {
        assert_eq!(ConnectorKind::Feishu.as_str(), "feishu");
        assert_eq!(ConnectorKind::WechatBridge.as_str(), "wechat_bridge");
    }

    #[test]
    fn reserves_connector_runtime_modes_for_future_connectors() {
        assert_eq!(ConnectorRuntimeMode::Webhook.as_str(), "webhook");
        assert_eq!(ConnectorRuntimeMode::Bridge.as_str(), "bridge");
    }

    #[test]
    fn exposes_placeholder_connector_configs_for_feishu_and_wechat_bridge() {
        let feishu = FeishuConnectorConfig::disabled();
        let wechat_bridge = WechatBridgeConnectorConfig::disabled();

        assert!(!feishu.enabled);
        assert!(!wechat_bridge.enabled);
    }
}
