//! qssh-uploader - Quick-SSH 独立文件上传工具
//!
//! 由 `qssh` 在检测到拖拽操作时在新控制台窗口中启动。
//! 使用 OpenSSH `scp` 实现文件传输，逐个文件顺序上传。
//!
//! ## 设计要点
//!
//! - scp 直接继承控制台（不管道化），让 OpenSSH 显示原生传输进度
//!   （管道化时 scp 检测不到 TTY 会完全静默）。
//! - 配置 AskPass 复用已保存的密码（与主 qssh 的 AskPass 机制一致）。
//! - 出错时打印 scp 的具体错误并写入 `%TEMP%\qssh-uploader.log`。
//! - 窗口保持打开由 cmd.exe 的 `& pause` 负责。
//!
//! ## 防闪退设计
//!
//! - 启动时延迟 300ms 等待控制台初始化
//! - Panic hook 循环等待 Enter，确保窗口不闪退
//! - 错误日志写入 `%TEMP%\qssh-uploader.log`

use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;

use anyhow::{Context, Result};

// ── 参数解析 ────────────────────────────────────────────

struct UploadArgs {
    /// 主机别名（用于 AskPass 读取已保存密码）
    alias: Option<String>,
    hostname: String,
    user: Option<String>,
    port: u16,
    identity_file: Option<PathBuf>,
    remote_dir: String,
    files: Vec<PathBuf>,
}

fn parse_args() -> Result<UploadArgs> {
    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;
    let mut alias = None;
    let mut hostname = String::new();
    let mut user = None;
    let mut port = 22u16;
    let mut identity_file = None;
    let mut remote_dir = String::from(".");
    let mut files = Vec::new();

    while i < args.len() {
        match args[i].as_str() {
            "--alias" => {
                i += 1;
                if i >= args.len() {
                    anyhow::bail!("--alias 缺少参数值");
                }
                alias = Some(args[i].clone());
            }
            "--host" => {
                i += 1;
                if i >= args.len() {
                    anyhow::bail!("--host 缺少参数值");
                }
                hostname = args[i].clone();
            }
            "--user" => {
                i += 1;
                if i >= args.len() {
                    anyhow::bail!("--user 缺少参数值");
                }
                user = Some(args[i].clone());
            }
            "--port" => {
                i += 1;
                if i >= args.len() {
                    anyhow::bail!("--port 缺少参数值");
                }
                port = args[i].parse()?;
            }
            "--key" => {
                i += 1;
                if i >= args.len() {
                    anyhow::bail!("--key 缺少参数值");
                }
                identity_file = Some(PathBuf::from(&args[i]));
            }
            "--remote-dir" => {
                i += 1;
                if i >= args.len() {
                    anyhow::bail!("--remote-dir 缺少参数值");
                }
                remote_dir = args[i].clone();
            }
            _ => {
                files.push(PathBuf::from(&args[i]));
            }
        }
        i += 1;
    }

    if hostname.is_empty() {
        anyhow::bail!("缺少 --host 参数");
    }
    if files.is_empty() {
        anyhow::bail!("未指定上传文件");
    }

    Ok(UploadArgs {
        alias,
        hostname,
        user,
        port,
        identity_file,
        remote_dir,
        files,
    })
}

// ── SCP 上传 ────────────────────────────────────────────

/// 配置 AskPass，使 scp 能复用已保存的密码（与主 qssh 的 AskPass 机制一致）
fn configure_scp_askpass(cmd: &mut Command, alias: &str) {
    // 上传器与 qssh 同目录，AskPass 必须是 qssh.exe 本体
    let qssh = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|p| p.join("qssh.exe")))
        .unwrap_or_else(|| PathBuf::from("qssh"));
    cmd.env("SSH_ASKPASS", qssh)
        .env("SSH_ASKPASS_REQUIRE", "force")
        .env("QSSH_ASKPASS_ACTIVE", "1")
        .env("QSSH_ASKPASS_ALIAS", alias);
    if std::env::var_os("DISPLAY").is_none() {
        cmd.env("DISPLAY", "qssh-askpass");
    }
}

/// 构建 scp 命令并执行上传（继承控制台，显示原生进度）
fn upload_via_scp(args: &UploadArgs) -> Result<()> {
    let file_name = args
        .files
        .last()
        .and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    // 构建目标: [user@]host:remote_dir/filename
    let user_part = match args.user {
        Some(ref u) => format!("{}@", u),
        None => String::new(),
    };
    let remote_target = format!(
        "{}{}:{}/{}",
        user_part, args.hostname, args.remote_dir, file_name
    );

    let mut cmd = Command::new("scp");

    // scp -P <port> -i <key> -o ControlMaster=no <localfile> <target>
    if args.port != 22 {
        cmd.arg("-P");
        cmd.arg(args.port.to_string());
    }
    if let Some(ref key_path) = args.identity_file {
        cmd.arg("-i");
        cmd.arg(key_path.as_os_str());
    }
    // 禁用连接共享，避免与现有 SSH 会话冲突
    cmd.arg("-o").arg("ControlMaster=no");
    if let Some(ref alias) = args.alias {
        configure_scp_askpass(&mut cmd, alias);
    }
    cmd.arg(args.files[0].as_os_str());
    cmd.arg(&remote_target);

    // 继承控制台：scp 检测到 TTY 后显示原生传输进度；
    // 若需要密码/口令，也允许直接在窗口内输入（或经 AskPass 自动填写）
    let status = cmd.status().context("无法启动 scp 进程")?;

    if !status.success() {
        anyhow::bail!("scp 上传失败 (退出码: {:?})", status.code().unwrap_or(-1));
    }

    Ok(())
}

// ── 等待 Enter 辅助函数 ─────────────────────────────────

/// 在 panic hook 中使用的等待版本（需要 `Fn + 'static`）
fn wait_for_enter_panic() {
    loop {
        let mut line = String::new();
        if std::io::stdin().read_line(&mut line).is_ok() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
}

// ── 主入口 ───────────────────────────────────────────────

fn main() {
    // 延迟 300ms 等待控制台窗口初始化完成
    std::thread::sleep(std::time::Duration::from_millis(300));

    // 写日志到文件（便于排查问题）
    let log_path = std::env::temp_dir().join("qssh-uploader.log");
    let mut log_file = std::fs::File::create(&log_path).ok();
    let mut log = |msg: &str| {
        if let Some(ref mut f) = log_file {
            let _ = writeln!(f, "{}", msg);
        }
    };

    log(&format!("args: {:?}", std::env::args().collect::<Vec<_>>()));

    // 捕获 panic，显示错误信息并等待 Enter
    std::panic::set_hook(Box::new(|info| {
        let msg = format!("上传器内部错误: {}", info);
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .append(true)
            .open(std::env::temp_dir().join("qssh-uploader.log"))
        {
            let _ = writeln!(f, "PANIC: {}", msg);
        }
        print!("\x1b[2J\x1b[H");
        println!("{}", msg);
        println!();
        println!("按 Enter 键退出...");
        wait_for_enter_panic();
    }));

    // 执行实际逻辑
    if let Err(e) = run() {
        log("run() 返回错误");

        let err_msg = format!("上传失败: {}", e);
        log(&err_msg);
        for (i, cause) in e.chain().enumerate() {
            if i > 0 {
                let cause_msg = format!("  -> {}", cause);
                log(&cause_msg);
            }
        }

        print!("\x1b[2J\x1b[H");
        println!("上传失败");
        for (i, cause) in e.chain().enumerate() {
            if i == 0 {
                println!("  {}", cause);
            } else {
                println!("  -> {}", cause);
            }
        }
        // & pause 负责保持窗口打开
    } else {
        log("上传成功完成");
        // & pause 负责保持窗口打开
    }
}

/// 核心上传逻辑：逐个文件顺序执行 SCP 上传，scp 原生进度直接显示在窗口
fn run() -> Result<()> {
    let args = parse_args()?;
    let total = args.files.len();
    let start_time = Instant::now();

    let mut success_count = 0usize;
    let mut failures = Vec::new();

    println!("qssh-uploader - 文件上传工具");
    println!(
        "目标: {}@{}:{}",
        args.user.as_deref().unwrap_or("?"),
        args.hostname,
        args.port
    );
    println!();

    for (i, file) in args.files.iter().enumerate() {
        let name = file
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| format!("<file {}>", i + 1));

        println!("[{}/{}] 上传 {} ...", i + 1, total, name);
        println!();

        let file_args = UploadArgs {
            alias: args.alias.clone(),
            hostname: args.hostname.clone(),
            user: args.user.clone(),
            port: args.port,
            identity_file: args.identity_file.clone(),
            remote_dir: args.remote_dir.clone(),
            files: vec![file.clone()],
        };

        match upload_via_scp(&file_args) {
            Ok(()) => {
                println!();
                println!("  -> 上传成功: {}", name);
                success_count += 1;
            }
            Err(e) => {
                println!();
                println!("  -> 上传失败: {}", name);
                failures.push(format!("{}: {}", name, e));
            }
        }
        println!();
    }

    // 汇总
    let elapsed = start_time.elapsed().as_secs_f64();
    if failures.is_empty() {
        println!("全部完成! 共 {} 个文件。耗时: {:.0}s", total, elapsed);
    } else {
        println!(
            "完成: {}/{} 成功, {} 个失败。耗时: {:.0}s",
            success_count,
            total,
            failures.len(),
            elapsed
        );
        for f in &failures {
            println!("  - {f}");
        }
    }

    Ok(())
}
