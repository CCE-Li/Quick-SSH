use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};

// ── 渐进式 SSH 配置类型定义 ──────────────────────────────

/// SSH 配置指令 — 仅解析 qssh 关心的字段，其余保持 Raw
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SshDirective {
    HostName(String),
    User(String),
    Port(u16),
    IdentityFile(PathBuf),
    /// 未识别的指令，以 (Key, Value) 原始文本保留
    Unknown(String, String),
}

impl SshDirective {
    /// 渲染为 SSH 配置文件的一行
    pub fn to_line(&self) -> String {
        match self {
            SshDirective::HostName(v) => format!("    HostName {}", v),
            SshDirective::User(v) => format!("    User {}", v),
            SshDirective::Port(v) => format!("    Port {}", v),
            SshDirective::IdentityFile(v) => {
                format!("    IdentityFile {}", v.display())
            }
            SshDirective::Unknown(k, v) => format!("    {} {}", k, v),
        }
    }
}

/// 一个 Host 块
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostBlock {
    /// 别名（Host 值）
    pub alias: String,
    /// 已解析的指令
    pub directives: Vec<SshDirective>,
    /// 原始文本，用于忠实重建
    pub raw_text: String,
}

impl HostBlock {
    /// 将 HostBlock 渲染回 SSH 配置文件格式
    pub fn render(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("Host {}\n", self.alias));
        for d in &self.directives {
            out.push_str(&d.to_line());
            out.push('\n');
        }
        out
    }

    /// 获取 HostName
    pub fn hostname(&self) -> Option<&str> {
        self.directives.iter().find_map(|d| {
            if let SshDirective::HostName(v) = d {
                Some(v.as_str())
            } else {
                None
            }
        })
    }

    /// 获取 User
    pub fn user(&self) -> Option<&str> {
        self.directives.iter().find_map(|d| {
            if let SshDirective::User(v) = d {
                Some(v.as_str())
            } else {
                None
            }
        })
    }

    /// 获取 Port
    pub fn port(&self) -> u16 {
        self.directives
            .iter()
            .find_map(|d| {
                if let SshDirective::Port(v) = d {
                    Some(*v)
                } else {
                    None
                }
            })
            .unwrap_or(22)
    }

    /// 获取 IdentityFile
    pub fn identity_file(&self) -> Option<&PathBuf> {
        self.directives.iter().find_map(|d| {
            if let SshDirective::IdentityFile(v) = d {
                Some(v)
            } else {
                None
            }
        })
    }
}

/// 完整的 SSH 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshConfig {
    pub hosts: Vec<HostBlock>,
    /// 非 Host 段（如全局配置、Match 块等）
    pub preamble: String,
}

// ── 查询与工具函数 ───────────────────────────────────────

/// 查找别名对应的 HostBlock
pub fn find_host<'a>(config: &'a SshConfig, alias: &str) -> Option<&'a HostBlock> {
    config.hosts.iter().find(|h| h.alias == alias)
}

/// 可变查找
#[allow(dead_code)]
pub fn find_host_mut<'a>(
    config: &'a mut SshConfig,
    alias: &str,
) -> Option<&'a mut HostBlock> {
    config.hosts.iter_mut().find(|h| h.alias == alias)
}

/// 获取所有别名
#[allow(dead_code)]
pub fn list_aliases(config: &SshConfig) -> Vec<&str> {
    config.hosts.iter().map(|h| h.alias.as_str()).collect()
}

/// 获取默认 SSH 配置路径
pub fn default_config_path() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".ssh").join("config")
}

/// 确保 SSH 配置文件和目录存在
pub fn ensure_config(path: &PathBuf) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }
    if !path.exists() {
        std::fs::write(path, "# Quick-SSH 管理的 SSH 配置\n")?;
    }
    Ok(())
}
