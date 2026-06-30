use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::config::{self, types::SshDirective, HostBlock, SshConfig};

/// 添加新的 SSH 主机
///
/// 将新主机条目追加到 ~/.ssh/config 文件末尾
pub fn run(alias: &str, user_at_host: &str, key: Option<&str>, port: u16) -> Result<()> {
    let config_path = config::default_config_path();
    config::ensure_config(&config_path)?;
    let mut config: SshConfig = config::parser::parse_config(&config_path)?;

    // 检查别名是否已存在
    if config::find_host(&config, alias).is_some() {
        anyhow::bail!(
            "主机 \"{}\" 已存在。使用 `qssh rm {}` 删除后重试",
            alias,
            alias
        );
    }

    // 解析 user@host
    let (user, hostname) = user_at_host
        .split_once('@')
        .map(|(u, h)| (u.to_string(), h.to_string()))
        .unwrap_or_else(|| ("".to_string(), user_at_host.to_string()));

    let mut directives = Vec::new();
    directives.push(SshDirective::HostName(hostname.clone()));

    if !user.is_empty() {
        directives.push(SshDirective::User(user.clone()));
    }

    if port != 22 {
        directives.push(SshDirective::Port(port));
    }

    if let Some(key_path) = key {
        let expanded = shellexpand::full(key_path)?;
        directives.push(SshDirective::IdentityFile(PathBuf::from(expanded.as_ref())));
    }

    let block = HostBlock {
        alias: alias.to_string(),
        directives,
        raw_text: String::new(),
    };

    config.hosts.push(block);

    // 写回 SSH 配置文件
    let content = config::render_config(&config);
    std::fs::write(&config_path, content)
        .with_context(|| format!("无法写入 SSH 配置文件: {}", config_path.display()))?;

    println!(
        "✅ 主机 \"{}\" 已添加: {}@{}:{}",
        alias, user, hostname, port
    );
    if let Some(k) = key {
        println!("   🔑 密钥: {}", k);
    }

    Ok(())
}
