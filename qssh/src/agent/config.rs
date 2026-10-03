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
    /// `provider` 命中内置预设（openai / deepseek / gemini / doubao / commandcode /
    /// opencode / ollama / custom，含别名）时，空缺的 `base_url` / `model` 自动填入
    /// 预设默认值（端点与模型 id 取自 models.dev 注册表）；显式配置始终优先。
    pub fn to_provider_config(&self) -> ProviderConfig {
        let preset = super::provider::provider_preset(&self.provider);
        let kind = ProviderKind::from_str(&self.provider).unwrap_or_default();
        let base_url = if self.base_url.trim().is_empty() {
            preset
                .map(|preset| preset.base_url.to_string())
                .unwrap_or_default()
        } else {
            self.base_url.clone()
        };
        let model = if self.model.trim().is_empty() {
            preset
                .map(|preset| preset.model.to_string())
                .unwrap_or_default()
        } else {
            self.model.clone()
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
            api_key_envs: preset.map(|preset| preset.api_key_envs).unwrap_or(&[]),
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

    /// 内置供应商预设：空缺的 base_url / model 自动填入权威默认值
    #[test]
    fn provider_presets_fill_blank_endpoint_and_model() {
        let cases = [
            (
                "deepseek",
                "https://api.deepseek.com",
                "deepseek-v4-flash",
                "DEEPSEEK_API_KEY",
            ),
            (
                "gemini",
                "https://generativelanguage.googleapis.com/v1beta/openai",
                "gemini-flash-latest",
                "GEMINI_API_KEY",
            ),
            (
                "doubao",
                "https://ark.cn-beijing.volces.com/api/v3",
                "doubao-seed-2-1-pro-260628",
                "ARK_API_KEY",
            ),
            (
                "openai",
                "https://api.openai.com/v1",
                "gpt-5.4",
                "OPENAI_API_KEY",
            ),
        ];
        for (provider, base_url, model, env) in cases {
            let config = AgentConfig {
                provider: provider.to_string(),
                base_url: String::new(),
                model: String::new(),
                ..AgentConfig::default()
            };
            let pc = config.to_provider_config();
            assert_eq!(pc.base_url, base_url, "{provider} 端点默认值");
            assert_eq!(pc.model, model, "{provider} 模型默认值");
            assert_eq!(
                pc.api_key_envs.first().copied(),
                Some(env),
                "{provider} 首选密钥变量"
            );
        }
    }

    /// 别名归一化：google→gemini、volcengine→doubao
    #[test]
    fn provider_aliases_fill_preset_defaults() {
        let gemini = AgentConfig {
            provider: "google".to_string(),
            base_url: String::new(),
            model: String::new(),
            ..AgentConfig::default()
        };
        assert_eq!(
            gemini.to_provider_config().base_url,
            "https://generativelanguage.googleapis.com/v1beta/openai"
        );

        let doubao = AgentConfig {
            provider: "volcengine".to_string(),
            base_url: String::new(),
            model: String::new(),
            ..AgentConfig::default()
        };
        let pc = doubao.to_provider_config();
        assert!(pc.base_url.contains("volces.com"), "{}", pc.base_url);
    }

    /// 无默认端点的供应商（commandcode）保留用户填写值，不注入默认
    #[test]
    fn providers_without_default_endpoint_keep_user_values() {
        let blank = AgentConfig {
            provider: "commandcode".to_string(),
            base_url: String::new(),
            model: String::new(),
            ..AgentConfig::default()
        };
        let pc = blank.to_provider_config();
        assert!(pc.base_url.is_empty());
        assert_eq!(pc.api_key_envs, ["COMMANDCODE_API_KEY"].as_slice());

        let custom = AgentConfig {
            provider: "commandcode".to_string(),
            base_url: "https://gateway.example/v1".to_string(),
            model: "my-model".to_string(),
            ..AgentConfig::default()
        };
        let pc = custom.to_provider_config();
        assert_eq!(pc.base_url, "https://gateway.example/v1");
        assert_eq!(pc.model, "my-model");
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
