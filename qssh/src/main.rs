mod cli;
mod cmd;
mod config;
mod network;
mod ssh;
mod tui;

use clap::Parser;
use cli::{Cli, Command};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        None => {
            // 无子命令 → 启动 TUI
            tui::event::start()?;
        }
        Some(Command::Ps { keyword }) => {
            cmd::ps::run(keyword)?;
        }
        Some(Command::Add {
            alias,
            user_at_host,
            key,
            port,
        }) => {
            cmd::add::run(&alias, &user_at_host, key.as_deref(), port)?;
        }
        Some(Command::Rm { alias }) => {
            cmd::rm::run(&alias)?;
        }
        Some(Command::Connect { target, ssh_args }) => {
            cmd::connect::run(&target, &ssh_args)?;
        }
        Some(Command::Export { file }) => {
            cmd::export::run(file.as_deref())?;
        }
        Some(Command::Import { file }) => {
            cmd::import::run(&file)?;
        }
        Some(Command::Help) => {
            cmd::help::run()?;
        }
        Some(Command::Completions { shell }) => {
            cmd::completions::run(&shell)?;
        }
    }

    Ok(())
}
