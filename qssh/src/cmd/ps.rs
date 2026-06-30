use anyhow::Result;

use crate::config;

/// 列出所有 SSH 主机
pub fn run(keyword: Option<String>) -> Result<()> {
    let config_path = config::default_config_path();
    let config = config::parser::parse_config(&config_path)?;

    if config.hosts.is_empty() {
        println!("📭 没有找到 SSH 主机");
        println!("   使用 `qssh add <alias> <user@host>` 添加主机");
        return Ok(());
    }

    let hosts = match &keyword {
        Some(kw) => {
            let kw = kw.to_lowercase();
            config
                .hosts
                .iter()
                .filter(|h| h.alias.to_lowercase().contains(&kw))
                .collect::<Vec<_>>()
        }
        None => config.hosts.iter().collect::<Vec<_>>(),
    };

    if hosts.is_empty() {
        println!("🔍 没有找到匹配 \"{}\" 的主机", keyword.unwrap());
        return Ok(());
    }

    println!("📋 SSH 主机列表 (共 {} 台):", hosts.len());
    println!();

    for host in &hosts {
        let hostname = host.hostname().unwrap_or("-");
        let user = host.user().map(|u| format!("{}@", u)).unwrap_or_default();
        let port = host.port();
        let key = host
            .identity_file()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "(agent)".to_string());

        println!(
            "  {:<20} {}{}:{}   🔑 {}",
            host.alias, user, hostname, port, key
        );
    }

    Ok(())
}
