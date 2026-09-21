//! 远程执行抽象：`RemoteExecutor` trait + SSH 子进程实现
//!
//! Phase 2 首版复用系统 `ssh` 命令执行远程命令（决策 D5：命令解析为主），
//! 未来 Phase 可替换为 ssh2 长连接通道（决策 D1-C 的监控通道）。

use std::io::Read;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use crate::ssh::session::SshTarget;

/// 远程命令执行结果
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecOutput {
    /// 命令标准输出（UTF-8 宽容解码）
    pub stdout: String,
    /// 命令标准错误
    pub stderr: String,
    /// 退出码（None = 被信号杀死/未启动）
    pub exit_code: Option<i32>,
}

impl ExecOutput {
    /// 退出码是否为 0（测试与后续 Phase 使用）
    #[allow(dead_code)]
    pub fn success(&self) -> bool {
        self.exit_code == Some(0)
    }
}

/// 远程执行抽象：屏蔽"系统 ssh 子进程" / "ssh2 长连接"等实现差异
pub trait RemoteExecutor: Send + Sync {
    /// 在远程目标上执行命令，返回合并输出
    fn exec(&self, command: &str) -> Result<ExecOutput, ExecError>;
}

/// 远程执行错误分类（决策 D4 / Phase 2.5）
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecError {
    /// 连接超时
    Timeout,
    /// SSH 断开 / 网络不可达
    Disconnect,
    /// 权限不足（认证失败 / Permission denied）
    Permission,
    /// 命令不存在 / 路径错误
    NotFound,
    /// 操作系统不支持
    UnsupportedOs,
    /// 其他错误
    Other(String),
}

impl std::fmt::Display for ExecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecError::Timeout => write!(f, "连接超时"),
            ExecError::Disconnect => write!(f, "SSH 连接断开"),
            ExecError::Permission => write!(f, "权限不足或认证失败"),
            ExecError::NotFound => write!(f, "命令不存在"),
            ExecError::UnsupportedOs => write!(f, "目标系统不受支持"),
            ExecError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

/// 通过系统 `ssh` 子进程执行远程命令
///
/// 关键设计：
/// - 复用 [`SshTarget::build_ssh_args`] 的认证参数
/// - `-o BatchMode=yes` 防止交互式密码输入挂起
/// - `-o ConnectTimeout` 控制连接超时
/// - 命令通过 `--` 后追加，避免参数注入
pub struct SshProcessExecutor {
    target: SshTarget,
    /// 连接超时（秒）
    connect_timeout: u64,
    /// 命令执行超时
    command_timeout: Duration,
}

impl SshProcessExecutor {
    pub fn new(target: SshTarget) -> Self {
        Self {
            target,
            connect_timeout: 10,
            command_timeout: Duration::from_secs(15),
        }
    }

    pub fn with_timeouts(mut self, connect_timeout: u64, command_timeout: Duration) -> Self {
        self.connect_timeout = connect_timeout;
        self.command_timeout = command_timeout;
        self
    }

    fn build_command(&self, command: &str) -> Command {
        let mut cmd = Command::new("ssh");
        cmd.args(self.target.build_ssh_args());
        cmd.arg("-o")
            .arg(format!("ConnectTimeout={}", self.connect_timeout));
        cmd.arg("-o").arg("BatchMode=yes");
        cmd.arg("--");
        cmd.arg(command);
        cmd
    }
}

impl RemoteExecutor for SshProcessExecutor {
    fn exec(&self, command: &str) -> Result<ExecOutput, ExecError> {
        let mut child = self
            .build_command(command)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| ExecError::Other(format!("无法启动 ssh 进程: {e}")))?;

        let mut stdout = child.stdout.take().expect("stdout 已配置为 piped");
        let mut stderr = child.stderr.take().expect("stderr 已配置为 piped");
        let stdout_thread = std::thread::spawn(move || read_to_string_lossy(&mut stdout));
        let stderr_thread = std::thread::spawn(move || read_to_string_lossy(&mut stderr));

        let status = wait_with_timeout(&mut child, self.command_timeout)?;

        let stdout = stdout_thread
            .join()
            .map_err(|_| ExecError::Other("读取 stdout 失败".into()))?;
        let stderr = stderr_thread
            .join()
            .map_err(|_| ExecError::Other("读取 stderr 失败".into()))?;

        classify(exit_code_of(&status), &stdout, &stderr)
    }
}

fn exit_code_of(status: &std::process::ExitStatus) -> Option<i32> {
    status.code()
}

/// 等待子进程结束，带超时；超时则 kill 并返回 [`ExecError::Timeout`]
fn wait_with_timeout(
    child: &mut Child,
    timeout: Duration,
) -> Result<std::process::ExitStatus, ExecError> {
    let deadline = Instant::now() + timeout;
    loop {
        if let Some(status) = child
            .try_wait()
            .map_err(|e| ExecError::Other(format!("wait 失败: {e}")))?
        {
            return Ok(status);
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Err(ExecError::Timeout);
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// 根据退出码 + 输出内容做错误分类
fn classify(exit_code: Option<i32>, stdout: &str, stderr: &str) -> Result<ExecOutput, ExecError> {
    if exit_code == Some(0) {
        return Ok(ExecOutput {
            stdout: stdout.to_string(),
            stderr: stderr.to_string(),
            exit_code,
        });
    }

    let combined = format!("{stdout} {stderr}").to_lowercase();
    if combined.contains("timed out") || combined.contains("connection timed out") {
        return Err(ExecError::Timeout);
    }
    if combined.contains("permission denied")
        || combined.contains("authentication failed")
        || combined.contains("publickey")
    {
        return Err(ExecError::Permission);
    }
    if combined.contains("no such file")
        || combined.contains("command not found")
        || combined.contains("no route to host")
        || combined.contains("unknown option")
    {
        return Err(ExecError::NotFound);
    }
    if combined.contains("connection refused")
        || combined.contains("connection reset")
        || combined.contains("closed by remote host")
        || combined.contains("network is unreachable")
    {
        return Err(ExecError::Disconnect);
    }
    // 连接成功但目标系统非 Linux（Windows PowerShell/cmd、其他平台）
    if combined.contains("microsoft windows")
        || combined.contains("the system cannot find")
        || combined.contains("is not recognized")
        || combined.contains("not a linux")
    {
        return Err(ExecError::UnsupportedOs);
    }

    Ok(ExecOutput {
        stdout: stdout.to_string(),
        stderr: stderr.to_string(),
        exit_code,
    })
}

/// 宽容 UTF-8 解码（非 UTF-8 字节替换为 U+FFFD）
fn read_to_string_lossy(reader: &mut impl Read) -> String {
    let mut buf = Vec::new();
    let _ = reader.read_to_end(&mut buf);
    String::from_utf8_lossy(&buf).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_classification_timeout() {
        assert_eq!(ExecError::Timeout.to_string(), "连接超时");
    }

    #[test]
    fn error_classification_permission() {
        assert!(ExecError::Permission.to_string().contains("权限"));
    }

    #[test]
    fn classify_zero_exit_ok() {
        let out = classify(Some(0), "ok", "").expect("zero exit should be ok");
        assert!(out.success());
    }

    #[test]
    fn classify_permission_denied() {
        let err = classify(Some(1), "", "Permission denied (publickey).").unwrap_err();
        assert_eq!(err, ExecError::Permission);
    }

    #[test]
    fn classify_command_not_found() {
        let err = classify(Some(127), "", "sh: df: command not found").unwrap_err();
        assert_eq!(err, ExecError::NotFound);
    }

    #[test]
    fn classify_connection_refused() {
        let err = classify(
            Some(255),
            "",
            "ssh: connect to host 1.2.3.4 port 22: Connection refused",
        )
        .unwrap_err();
        assert_eq!(err, ExecError::Disconnect);
    }

    #[test]
    fn unknown_error_keeps_output() {
        let out = classify(Some(3), "partial", "boom").expect("unknown exit should keep output");
        assert!(!out.success());
        assert_eq!(out.stdout, "partial");
    }

    #[test]
    fn classify_windows_target_as_unsupported() {
        let err = classify(
            Some(1),
            "Microsoft Windows [Version 10.0]",
            "'free' is not recognized",
        )
        .unwrap_err();
        assert_eq!(err, ExecError::UnsupportedOs);
    }
}
