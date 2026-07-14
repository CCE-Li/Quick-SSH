use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};

// ── ~/.qsshrc 程序设置 ──────────────────────────────────

/// 用户自定义设置（预留，后续版本启用）
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QsshSettings {
    /// 默认 SSH 端口
    #[serde(default = "default_port")]
    pub default_port: u16,
    /// TCP 超时秒数
    #[serde(default = "default_timeout")]
    pub ping_timeout_secs: u64,
    /// 上传并发数
    #[serde(default = "default_concurrency")]
    pub upload_concurrency: usize,
    /// 自定义 SSH 配置路径
    pub ssh_config_path: Option<PathBuf>,
}

#[allow(dead_code)]
fn default_port() -> u16 {
    22
}
#[allow(dead_code)]
fn default_timeout() -> u64 {
    3
}
#[allow(dead_code)]
fn default_concurrency() -> usize {
    3
}

impl Default for QsshSettings {
    fn default() -> Self {
        Self {
            default_port: default_port(),
            ping_timeout_secs: default_timeout(),
            upload_concurrency: default_concurrency(),
            ssh_config_path: None,
        }
    }
}

/// 加载 ~/.qsshrc（如果存在）
#[allow(dead_code)]
pub fn load_settings() -> Result<QsshSettings> {
    let config_path = dirs::home_dir()
        .map(|p| p.join(".qsshrc"))
        .unwrap_or_default();

    if config_path.exists() {
        let content = std::fs::read_to_string(&config_path)?;
        let settings: QsshSettings = serde_json::from_str(&content)?;
        return Ok(settings);
    }

    Ok(QsshSettings::default())
}

/// 保存设置到 ~/.qsshrc
#[allow(dead_code)]
pub fn save_settings(settings: &QsshSettings) -> Result<()> {
    let config_path = dirs::home_dir()
        .map(|p| p.join(".qsshrc"))
        .unwrap_or_default();

    let content = serde_json::to_string_pretty(settings)?;
    std::fs::write(&config_path, content)?;
    Ok(())
}
