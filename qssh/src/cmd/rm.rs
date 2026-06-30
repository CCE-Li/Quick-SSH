use anyhow::{Context, Result};

use crate::config::{self, SshConfig};

/// 删除 SSH 主机
pub fn run(alias: &str) -> Result<()> {
    let config_path = config::default_config_path();
    let mut config: SshConfig = config::parser::parse_config(&config_path)?;

    let initial_len = config.hosts.len();
    config.hosts.retain(|h| h.alias != alias);

    if config.hosts.len() == initial_len {
        anyhow::bail!("未找到主机 \"{}\"", alias);
    }

    // 写回 SSH 配置文件
    let content = config::render_config(&config);
    std::fs::write(&config_path, content)
        .with_context(|| format!("无法写入 SSH 配置文件: {}", config_path.display()))?;

    println!("🗑️ 主机 \"{}\" 已删除", alias);
    Ok(())
}
