use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::config::ssh_config::{self, render_config, HostBlock, SshConfig};

/// 从 JSON 文件导入主机配置
pub fn run(file: &str) -> Result<()> {
    let import_path = PathBuf::from(file);
    let content = std::fs::read_to_string(&import_path)
        .with_context(|| format!("无法读取文件: {}", import_path.display()))?;

    let imported: Vec<HostBlock> = serde_json::from_str(&content)?;

    if imported.is_empty() {
        anyhow::bail!("文件中没有找到主机配置");
    }

    let config_path = ssh_config::default_config_path();
    ssh_config::ensure_config(&config_path)?;
    let mut config: SshConfig = ssh_config::parse_config(&config_path)?;

    let mut added = 0;
    let mut skipped = 0;

    for host in imported {
        if ssh_config::find_host(&config, &host.alias).is_some() {
            skipped += 1;
            eprintln!("⚠️  跳过已存在的主机: {}", host.alias);
            continue;
        }
        config.hosts.push(host);
        added += 1;
    }

    // 写回 SSH 配置文件
    let out = render_config(&config);
    std::fs::write(&config_path, out)
        .with_context(|| format!("无法写入 SSH 配置文件: {}", config_path.display()))?;

    println!("📥 导入完成: 新增 {} 台, 跳过 {} 台", added, skipped);

    Ok(())
}
