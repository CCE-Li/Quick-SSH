use clap::CommandFactory;
use clap_complete::{generate, Shell};
use std::io;

use crate::cli::Cli;

/// 生成并输出 Shell 补全脚本到 stdout
pub fn run(shell: &str) -> anyhow::Result<()> {
    let shell = match shell.to_lowercase().as_str() {
        "bash" => Shell::Bash,
        "zsh" => Shell::Zsh,
        "fish" => Shell::Fish,
        "powershell" | "ps1" => Shell::PowerShell,
        "elvish" => Shell::Elvish,
        other => {
            anyhow::bail!(
                "不支持的 shell 类型: {other}\n支持的选项: bash, zsh, fish, powershell, elvish"
            );
        }
    };

    let mut cmd = Cli::command();
    generate(shell, &mut cmd, "qssh", &mut io::stdout());
    Ok(())
}
