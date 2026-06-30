use anyhow::{Context, Result};
use colored::*;

use crate::config::{self, SshConfig};
use crate::ssh::session::resolve_target;
use crate::ssh::spawn::start_interactive_session;

/// 连接 SSH 主机
///
/// 支持两种输入：
/// 1. 别名（从 ~/.ssh/config 查找）
/// 2. user@hostname（直接连接）
pub fn run(target: &str, ssh_args: &[String]) -> Result<()> {
    let config_path = config::default_config_path();
    let config: SshConfig = config::parser::parse_config(&config_path)?;

    let resolved = resolve_target(&config, target);

    println!(
        "🔗 正在连接到 {} ({}@{}:{}) ...",
        resolved.alias.yellow(),
        resolved.user.as_deref().unwrap_or("?").green(),
        resolved.hostname.cyan(),
        resolved.port.to_string().blue(),
    );

    let exit_code = start_interactive_session(&resolved, ssh_args)
        .context("SSH 会话失败")?;

    if exit_code != 0 {
        println!(
            "⚠️  SSH 会话已退出 (代码: {})",
            exit_code.to_string().yellow()
        );
    }

    Ok(())
}
