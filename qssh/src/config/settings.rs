use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};

// ── ~/.qsshrc 程序设置 ──────────────────────────────────

/// 嵌入式终端连接时的加载动画样式（`~/.qsshrc`: `connect_animation`）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConnectAnimation {
    /// 单行旋转字符 spinner
    Spinner,
    /// donut.c 风格 ASCII 甜甜圈（默认）
    #[default]
    Donut,
}

impl ConnectAnimation {
    /// 配置值（写入 ~/.qsshrc / WebUI 选项）
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Spinner => "spinner",
            Self::Donut => "donut",
        }
    }

    /// 解析配置值；未知值返回 None，由调用方决定回退
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "spinner" => Some(Self::Spinner),
            "donut" => Some(Self::Donut),
            _ => None,
        }
    }
}

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
    /// 嵌入式终端连接加载动画（"spinner" / "donut"）
    #[serde(default = "default_connect_animation")]
    pub connect_animation: String,
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
fn default_connect_animation() -> String {
    ConnectAnimation::default().as_str().to_string()
}

impl Default for QsshSettings {
    fn default() -> Self {
        Self {
            default_port: default_port(),
            ping_timeout_secs: default_timeout(),
            upload_concurrency: default_concurrency(),
            connect_animation: default_connect_animation(),
            ssh_config_path: None,
        }
    }
}

impl QsshSettings {
    /// 解析后的连接加载动画样式（未知值回退默认）
    pub fn connect_animation_style(&self) -> ConnectAnimation {
        ConnectAnimation::parse(&self.connect_animation).unwrap_or_default()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connect_animation_parse_known_values() {
        assert_eq!(
            ConnectAnimation::parse("spinner"),
            Some(ConnectAnimation::Spinner)
        );
        assert_eq!(
            ConnectAnimation::parse(" donut "),
            Some(ConnectAnimation::Donut)
        );
        assert_eq!(ConnectAnimation::parse("DO NOT"), None);
        assert_eq!(ConnectAnimation::parse(""), None);
    }

    #[test]
    fn connect_animation_str_roundtrip() {
        for style in [ConnectAnimation::Spinner, ConnectAnimation::Donut] {
            assert_eq!(ConnectAnimation::parse(style.as_str()), Some(style));
        }
    }

    /// 旧版 ~/.qsshrc 缺少 connect_animation 字段时回退默认（甜甜圈）
    #[test]
    fn settings_without_connect_animation_falls_back_to_donut() {
        let json = r#"{
            "default_port": 22,
            "ping_timeout_secs": 3,
            "upload_concurrency": 3,
            "ssh_config_path": null
        }"#;
        let settings: QsshSettings = serde_json::from_str(json).unwrap();
        assert_eq!(settings.connect_animation_style(), ConnectAnimation::Donut);

        // 手写非法值同样回退默认，不破坏其它字段
        let settings = QsshSettings {
            connect_animation: "circle".to_string(),
            ..QsshSettings::default()
        };
        assert_eq!(settings.connect_animation_style(), ConnectAnimation::Donut);
    }
}
