//! Agent 配置：`agent.json` 加载 / 保存 / 默认值
//!
//! 路径：`~/.config/quick-ssh/agent.json`（Windows: `%APPDATA%\quick-ssh\agent.json`）。
//! 与 Dashboard 配置目录一致。

use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::permissions::PermissionLevel;
use super::provider::{ProviderConfig, ProviderKind};

/// Agent 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AgentConfig {
    /// Provider 类型
    pub provider: String,
    /// OpenAI 兼容 base_url
    pub base_url: String,
    /// 模型名
    pub model: String,
    /// 权限级别
    pub permission: String,
    /// 请求超时（秒）
    pub timeout_secs: u64,
    /// 显式配置的 API Key（可为空，空时回退到环境变量 / opencode auth.json）
    pub api_key: String,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            provider: ProviderKind::OpenAI.as_str().to_string(),
            base_url: "https://api.deepseek.com/v1".to_string(),
            model: "deepseek-chat".to_string(),
            permission: PermissionLevel::AskBeforeExecute.as_str().to_string(),
            timeout_secs: 60,
            api_key: String::new(),
        }
    }
}

impl AgentConfig {
    /// 转换为运行时 [`ProviderConfig`]
    ///
    /// opencode / opencode-go 的 `base_url` / `model` 若未配置，自动填入 opencode
    /// 官方默认值（`https://opencode.ai/zen/go/v1` + `deepseek-v4-flash`），
    /// 复用 opencode 的 API key 即可直接使用。
    pub fn to_provider_config(&self) -> ProviderConfig {
        let kind = ProviderKind::from_str(&self.provider).unwrap_or_default();
        let (base_url, model) = if kind == ProviderKind::OpenCodeGo {
            let base_url = if self.base_url.trim().is_empty() {
                super::provider::OPENCODE_GO_BASE_URL.to_string()
            } else {
                self.base_url.clone()
            };
            let model = if self.model.trim().is_empty() {
                super::provider::OPENCODE_GO_MODEL.to_string()
            } else {
                self.model.clone()
            };
            (base_url, model)
        } else {
            (self.base_url.clone(), self.model.clone())
        };
        ProviderConfig {
            kind,
            base_url,
            model,
            timeout_secs: self.timeout_secs,
            api_key: if self.api_key.trim().is_empty() {
                None
            } else {
                Some(self.api_key.trim().to_string())
            },
        }
    }

    /// 当前权限级别
    pub fn permission_level(&self) -> PermissionLevel {
        PermissionLevel::from_str(&self.permission).unwrap_or_default()
    }
}

/// 配置文件路径
pub fn config_path() -> PathBuf {
    let base = dirs::config_dir().unwrap_or_else(|| {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".config")
    });
    base.join("quick-ssh").join("agent.json")
}

/// 加载配置（不存在或损坏时回退默认）
pub fn load_agent_config() -> AgentConfig {
    let path = config_path();
    if let Ok(content) = std::fs::read_to_string(&path) {
        if let Ok(config) = serde_json::from_str(&content) {
            return config;
        }
    }
    AgentConfig::default()
}

/// 保存配置到磁盘
/// 保存 Agent 配置到 `~/.config/quick-ssh/agent.json`（设置面板使用）
pub fn save_agent_config(config: &AgentConfig) -> Result<()> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(config)?;
    std::fs::write(&path, content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_ask_before_execute() {
        let config = AgentConfig::default();
        assert_eq!(config.permission_level(), PermissionLevel::AskBeforeExecute);
        assert_eq!(config.to_provider_config().kind, ProviderKind::OpenAI);
    }

    #[test]
    fn config_serde_roundtrip() {
        let config = AgentConfig {
            provider: "ollama".to_string(),
            base_url: "http://localhost:11434".to_string(),
            model: "qwen2.5:7b".to_string(),
            permission: "auto_safe".to_string(),
            timeout_secs: 30,
            api_key: "sk-test-123".to_string(),
        };
        let json = serde_json::to_string(&config).unwrap();
        let back: AgentConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.provider, "ollama");
        assert_eq!(back.permission_level(), PermissionLevel::AutoSafe);
        assert_eq!(back.api_key, "sk-test-123");
        // api_key 透传到 ProviderConfig
        let pc = back.to_provider_config();
        assert_eq!(pc.api_key.as_deref(), Some("sk-test-123"));
    }

    #[test]
    fn api_key_blank_maps_to_none() {
        let config = AgentConfig {
            api_key: "  ".to_string(),
            ..AgentConfig::default()
        };
        assert_eq!(config.to_provider_config().api_key, None);
    }

    #[test]
    fn invalid_permission_falls_back_to_default() {
        let config = AgentConfig {
            permission: "bogus".to_string(),
            ..AgentConfig::default()
        };
        assert_eq!(config.permission_level(), PermissionLevel::AskBeforeExecute);
    }

    #[test]
    fn opencode_go_fills_defaults_when_blank() {
        let config = AgentConfig {
            provider: "opencode".to_string(),
            base_url: String::new(),
            model: String::new(),
            ..AgentConfig::default()
        };
        let pc = config.to_provider_config();
        assert_eq!(pc.kind, ProviderKind::OpenCodeGo);
        assert_eq!(pc.base_url, super::super::provider::OPENCODE_GO_BASE_URL);
        assert_eq!(pc.model, super::super::provider::OPENCODE_GO_MODEL);
    }

    #[test]
    fn opencode_go_keeps_custom_values() {
        let config = AgentConfig {
            provider: "opencode".to_string(),
            base_url: "https://custom.example/v1".to_string(),
            model: "custom-model".to_string(),
            ..AgentConfig::default()
        };
        let pc = config.to_provider_config();
        assert_eq!(pc.kind, ProviderKind::OpenCodeGo);
        assert_eq!(pc.base_url, "https://custom.example/v1");
        assert_eq!(pc.model, "custom-model");
    }
}
