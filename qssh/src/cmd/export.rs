use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::config::{self, SshConfig};

/// 导出所有主机配置为 JSON
pub fn run(file: Option<&str>) -> Result<()> {
    let config_path = config::default_config_path();
    let config: SshConfig = config::parser::parse_config(&config_path)?;

    let json = serde_json::to_string_pretty(&config.hosts)?;

    match file {
        Some(path) => {
            let out_path = PathBuf::from(path);
            std::fs::write(&out_path, &json)
                .with_context(|| format!("无法写入文件: {}", out_path.display()))?;
            println!(
                "📤 已导出 {} 台主机到 {}",
                config.hosts.len(),
                out_path.display()
            );
        }
        None => {
            println!("{}", json);
        }
    }

    Ok(())
}
