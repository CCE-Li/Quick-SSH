use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

// ── 渐进式 SSH 配置解析 ──────────────────────────────────

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
        self.directives.iter().find_map(|d| {
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

// ── 解析器实现 ───────────────────────────────────────────

/// 解析 SSH 配置文件的文本内容
///
/// 渐进式策略：
/// - 只解析 Host/HostName/User/Port/IdentityFile
/// - 其他所有指令保留为 Unknown(key, value) 或 raw_text
/// - Match、Include 等复杂指令保持原样保留在 preamble
pub fn parse_config_content(content: &str) -> SshConfig {
    let mut hosts = Vec::new();
    let mut preamble_lines = Vec::new();
    let mut current_host: Option<HostBlock> = None;
    let mut current_raw_lines: Vec<String> = Vec::new();
    let mut in_host_block = false;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with('#') {
            // 空行和注释 — 追加到当前上下文的 raw 中
            if in_host_block {
                current_raw_lines.push(line.to_string());
            } else {
                preamble_lines.push(line.to_string());
            }
            continue;
        }

        // 检测 Host 块开始
        if let Some(rest) = trimmed
            .strip_prefix("Host ")
            .or_else(|| trimmed.strip_prefix("HOST "))
        {
            // 保存上一个 Host 块
            if let Some(mut host) = current_host.take() {
                host.raw_text = current_raw_lines.join("\n");
                hosts.push(host);
                current_raw_lines = Vec::new();
            }

            let alias = rest.trim().to_string();
            current_host = Some(HostBlock {
                alias,
                directives: Vec::new(),
                raw_text: String::new(),
            });
            in_host_block = true;
            current_raw_lines.push(line.to_string());
            continue;
        }

        if in_host_block {
            // 解析 Host 块内的指令
            if let Some(host) = &mut current_host {
                if let Some(val) = trimmed.strip_prefix("HostName ") {
                    host.directives
                        .push(SshDirective::HostName(val.trim().to_string()));
                } else if let Some(val) = trimmed.strip_prefix("User ") {
                    host.directives
                        .push(SshDirective::User(val.trim().to_string()));
                } else if let Some(val) = trimmed.strip_prefix("Port ") {
                    if let Ok(port) = val.trim().parse::<u16>() {
                        host.directives.push(SshDirective::Port(port));
                    } else {
                        host.directives.push(SshDirective::Unknown(
                            "Port".into(),
                            val.trim().to_string(),
                        ));
                    }
                } else if let Some(val) = trimmed.strip_prefix("IdentityFile ") {
                    let expanded = shellexpand::full(val.trim())
                        .unwrap_or(std::borrow::Cow::Borrowed(val.trim()));
                    let path = PathBuf::from(expanded.as_ref());
                    host.directives.push(SshDirective::IdentityFile(path));
                } else {
                    // 未识别的指令
                    if let Some((k, v)) = trimmed.split_once(char::is_whitespace) {
                        host.directives.push(SshDirective::Unknown(
                            k.to_string(),
                            v.trim().to_string(),
                        ));
                    } else {
                        host.directives.push(SshDirective::Unknown(
                            trimmed.to_string(),
                            String::new(),
                        ));
                    }
                }
            }
            current_raw_lines.push(line.to_string());
        } else {
            preamble_lines.push(line.to_string());
        }
    }

    // 保存最后一个 Host 块
    if let Some(mut host) = current_host.take() {
        host.raw_text = current_raw_lines.join("\n");
        hosts.push(host);
    }

    SshConfig {
        hosts,
        preamble: preamble_lines.join("\n"),
    }
}

/// 解析 ~/.ssh/config 文件
pub fn parse_config(path: &PathBuf) -> Result<SshConfig> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("无法读取 SSH 配置文件: {}", path.display()))?;
    Ok(parse_config_content(&content))
}

/// 将 Host 块列表渲染回完整的 SSH 配置文件内容
pub fn render_config(config: &SshConfig) -> String {
    let mut out = String::new();

    if !config.preamble.is_empty() {
        out.push_str(&config.preamble);
        out.push('\n');
    }

    for host in &config.hosts {
        if !out.is_empty() && !out.ends_with('\n') {
            out.push('\n');
        }
        out.push_str(&host.render());
    }

    out
}

/// 查找别名对应的 HostBlock
pub fn find_host<'a>(config: &'a SshConfig, alias: &str) -> Option<&'a HostBlock> {
    config.hosts.iter().find(|h| h.alias == alias)
}

/// 可变查找
pub fn find_host_mut<'a>(
    config: &'a mut SshConfig,
    alias: &str,
) -> Option<&'a mut HostBlock> {
    config.hosts.iter_mut().find(|h| h.alias == alias)
}

/// 获取所有别名
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

// ── 测试 ──────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_host() {
        let content = r#"Host myserver
    HostName example.com
    User root
    Port 2222
    IdentityFile ~/.ssh/id_rsa
"#;
        let config = parse_config_content(content);
        assert_eq!(config.hosts.len(), 1);
        let host = &config.hosts[0];
        assert_eq!(host.alias, "myserver");
        assert_eq!(host.hostname(), Some("example.com"));
        assert_eq!(host.user(), Some("root"));
        assert_eq!(host.port(), 2222);
        assert!(host.identity_file().is_some());
    }

    #[test]
    fn test_parse_with_unknown_directive() {
        let content = r#"Host test
    HostName 10.0.0.1
    ServerAliveInterval 60
    User admin
"#;
        let config = parse_config_content(content);
        assert_eq!(config.hosts.len(), 1);
        let host = &config.hosts[0];
        assert_eq!(host.hostname(), Some("10.0.0.1"));
        // ServerAliveInterval 应保留为 Unknown
        let has_unknown = host.directives.iter().any(|d| matches!(d, SshDirective::Unknown(k, _) if k == "ServerAliveInterval"));
        assert!(has_unknown);
    }

    #[test]
    fn test_render_roundtrip() {
        let content = "Host myhost\n    HostName example.com\n    User test\n";
        let config = parse_config_content(content);
        let rendered = render_config(&config);
        assert!(rendered.contains("Host myhost"));
        assert!(rendered.contains("HostName example.com"));
        assert!(rendered.contains("User test"));
    }
}
