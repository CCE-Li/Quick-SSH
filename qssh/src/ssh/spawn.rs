use std::process::{Command, Stdio};

use anyhow::{Context, Result};

use super::session::SshTarget;

/// 启动交互式 SSH 会话（使用系统 ssh）
///
/// 直接 spawn ssh 进程，继承 stdio 实现交互。
/// `extra_args` 是用户在 `--` 之后传递的额外 SSH 参数。
pub fn start_interactive_session(target: &SshTarget, extra_args: &[String]) -> Result<i32> {
    let mut args = target.build_ssh_args();
    args.extend_from_slice(extra_args);

    let mut child = Command::new("ssh")
        .args(&args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .context("无法启动 ssh 进程，请确保已安装 OpenSSH Client")?;

    let status = child.wait().context("等待 ssh 进程结束失败")?;
    Ok(status.code().unwrap_or(-1))
}
