use std::path::PathBuf;

use anyhow::{Context, Result};

use super::types::{HostBlock, SshConfig, SshDirective};

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
        let has_unknown = host
            .directives
            .iter()
            .any(|d| matches!(d, SshDirective::Unknown(k, _) if k == "ServerAliveInterval"));
        assert!(has_unknown);
    }

    #[test]
    fn test_render_roundtrip() {
        let content = "Host myhost\n    HostName example.com\n    User test\n";
        let config = parse_config_content(content);
        let rendered = crate::config::writer::render_config(&config);
        assert!(rendered.contains("Host myhost"));
        assert!(rendered.contains("HostName example.com"));
        assert!(rendered.contains("User test"));
    }
}
