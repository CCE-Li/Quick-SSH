//! qssh-uploader - Quick-SSH 独立文件上传工具
//!
//! 由 `qssh connect` 在检测到拖拽操作时在新控制台窗口中启动。
//! 使用 OpenSSH `scp` 实现文件传输，支持最多 3 个文件并发上传。
//!
//! ## 防闪退设计
//!
//! - 启动时延迟 300ms 等待控制台初始化
//! - Panic hook 循环等待 Enter，确保窗口不闪退
//! - 正常退出由 cmd.exe 的 `& pause` 保持窗口打开
//! - 错误日志写入 `%TEMP%\qssh-uploader.log`

use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::time::Instant;

use anyhow::{Context, Result};

// ── 常量 ────────────────────────────────────────────────

/// 控制台显示宽度（字符列数）
const CONSOLE_WIDTH: usize = 100;
/// 最大并发上传数
const MAX_CONCURRENT: usize = 3;

// ── 参数解析 ────────────────────────────────────────────

struct UploadArgs {
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
    let mut hostname = String::new();
    let mut user = None;
    let mut port = 22u16;
    let mut identity_file = None;
    let mut remote_dir = String::from(".");
    let mut files = Vec::new();

    while i < args.len() {
        match args[i].as_str() {
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
        hostname,
        user,
        port,
        identity_file,
        remote_dir,
        files,
    })
}

// ── 进度跟踪 ────────────────────────────────────────────

/// 单个文件的上传进度
struct FileProgress {
    index: usize,
    name: String,
    size: u64,
    done: u64,
    finished: bool,
    success: bool,
    error: Option<String>,
}

// ── 格式化辅助函数 ───────────────────────────────────────

fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;
    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }
    format!("{:.1}{}", size, UNITS[unit_idx])
}

fn format_duration(secs: f64) -> String {
    if secs < 60.0 {
        format!("{:.0}s", secs)
    } else {
        format!("{:.0}m {:.0}s", secs / 60.0, secs % 60.0)
    }
}

/// 渲染进度条（使用 # 和 - 确保兼容性）
fn render_bar(ratio: f64, width: u16) -> String {
    let filled = (ratio * width as f64).round() as usize;
    let empty = width as usize - filled;
    let mut bar = String::with_capacity(width as usize + 2);
    bar.push('[');
    for _ in 0..filled {
        bar.push('#');
    }
    for _ in 0..empty {
        bar.push('-');
    }
    bar.push(']');
    bar
}

// ── 渲染函数 ────────────────────────────────────────────

/// 计算字符串在终端中的显示宽度（CJK 字符占 2 列，ASCII 占 1 列）
fn display_width(s: &str) -> usize {
    s.chars()
        .map(|c| {
            if c.is_ascii() {
                1
            } else if ('\u{1100}'..='\u{115f}').contains(&c) // Hangul Jamo
                || ('\u{2e80}'..='\u{303e}').contains(&c) // CJK Radicals Supplement ~ CJK Symbols
                || ('\u{3040}'..='\u{33bf}').contains(&c) // Hiragana ~ CJK Compatibility
                || ('\u{3400}'..='\u{4dbf}').contains(&c) // CJK Extension A
                || ('\u{4e00}'..='\u{a4cf}').contains(&c) // CJK Unified ~ Yi
                || ('\u{a960}'..='\u{a97c}').contains(&c) // Hangul Jamo Extended-A
                || ('\u{ac00}'..='\u{d7a3}').contains(&c) // Hangul Syllables
                || ('\u{f900}'..='\u{faff}').contains(&c) // CJK Compatibility Ideographs
                || ('\u{fe30}'..='\u{fe6f}').contains(&c) // CJK Compatibility Forms ~ Small Form Variants
                || ('\u{ff01}'..='\u{ff60}').contains(&c) // Fullwidth Forms
                || ('\u{fffe}'..='\u{ffff}').contains(&c) // Specials
            {
                2
            } else {
                1
            }
        })
        .sum()
}

/// 格式化一行：左侧文本左对齐，右侧进度右对齐
/// 正确处理 CJK 双宽字符，确保进度条在同一列对齐
fn format_left_right(left: &str, right: &str) -> String {
    let left_visual = display_width(left);
    let right_visual = display_width(right);
    let padding = if left_visual + right_visual + 2 >= CONSOLE_WIDTH {
        2
    } else {
        CONSOLE_WIDTH - left_visual - right_visual
    };
    let mut line = String::with_capacity(CONSOLE_WIDTH + 16);
    line.push_str(left);
    for _ in 0..padding {
        line.push(' ');
    }
    line.push_str(right);
    line
}

/// 渲染所有文件的上传进度
/// 已完成文件仍然显示 100% 进度条，不会消失
fn render_all(files: &[Arc<Mutex<FileProgress>>], start_time: Instant) {
    print!("\x1b[2J\x1b[H");

    println!("qssh-uploader - 文件上传工具");
    println!();

    let total = files.len();
    let mut done_count = 0;
    let mut fail_count = 0;
    let mut all_finished = true;

    for fp_arc in files {
        let fp = fp_arc.lock().unwrap();

        // 左侧：文件名（始终左对齐）
        let left = format!("文件 {}/{}: {}", fp.index + 1, total, fp.name);

        // 右侧：始终显示进度条，已完成文件显示 100%
        let (right, _file_finished) = if fp.finished {
            if fp.success {
                done_count += 1;
                let bar = render_bar(1.0, 24);
                (format!("{} 100%  {}/{}", bar, format_bytes(fp.size), format_bytes(fp.size)), true)
            } else {
                fail_count += 1;
                all_finished = false;
                let err = fp.error.as_deref().unwrap_or("失败");
                (format!("失败: {}", err), false)
            }
        } else {
            all_finished = false;
            let ratio = if fp.size > 0 {
                fp.done as f64 / fp.size as f64
            } else {
                0.0
            };
            let bar = render_bar(ratio.clamp(0.0, 1.0), 24);
            let pct = (ratio * 100.0) as u8;
            (format!("{} {:3}%  {}/{}", bar, pct, format_bytes(fp.done), format_bytes(fp.size)), false)
        };

        println!("{}", format_left_right(&left, &right));
    }

    println!();
    let elapsed = start_time.elapsed().as_secs_f64();

    if all_finished {
        if fail_count > 0 {
            println!(
                "全部完成! {}/{} 文件成功, {} 个失败。耗时: {}",
                done_count,
                total,
                fail_count,
                format_duration(elapsed)
            );
        } else {
            println!(
                "全部完成! 共 {} 个文件。耗时: {}",
                total,
                format_duration(elapsed)
            );
        }
    } else {
        println!(
            "进度: {}/{} 个文件完成  |  耗时: {}",
            done_count,
            total,
            format_duration(elapsed)
        );
    }

    std::io::stdout().flush().ok();
}

// ── SCP 上传 ────────────────────────────────────────────

/// 构建 scp 命令并执行上传
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

    let mut cmd = std::process::Command::new("scp");

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
    // 静默模式（无进度），我们自己显示进度
    cmd.arg("-q");
    cmd.arg(args.files[0].as_os_str());
    cmd.arg(&remote_target);

    let status = cmd.status().context("无法启动 scp 进程")?;

    if !status.success() {
        anyhow::bail!(
            "scp 上传失败 (退出码: {:?})",
            status.code().unwrap_or(-1)
        );
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

/// 核心上传逻辑：并发执行 SCP 上传，最多 MAX_CONCURRENT 个同时进行
fn run() -> Result<()> {
    let args = parse_args()?;
    let total_files = args.files.len();
    let start_time = Instant::now();

    // 初始化每个文件的进度跟踪
    let files: Vec<Arc<Mutex<FileProgress>>> = args
        .files
        .iter()
        .enumerate()
        .map(|(i, path)| {
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| format!("<file {}>", i));
            let size = std::fs::metadata(path)
                .map(|m| m.len())
                .unwrap_or(0);
            Arc::new(Mutex::new(FileProgress {
                index: i,
                name,
                size,
                done: 0,
                finished: false,
                success: false,
                error: None,
            }))
        })
        .collect();

    // 首次渲染
    render_all(&files, start_time);

    // ── 并发上传控制 ──
    // 使用 channel 接收上传完成通知
    let (tx, rx) = mpsc::channel::<usize>();
    // 活跃线程计数
    let active = Arc::new(AtomicUsize::new(0));
    // 参数用 Arc 包装，在线程间共享
    let args = Arc::new(args);

    let mut next_index: usize = 0;
    let mut completed_count: usize = 0;

    while completed_count < total_files {
        // 启动新的上传任务（不超过 MAX_CONCURRENT）
        while active.load(Ordering::Acquire) < MAX_CONCURRENT && next_index < total_files {
            let idx = next_index;
            next_index += 1;

            active.fetch_add(1, Ordering::Release);
            let tx = tx.clone();
            let active = active.clone();
            let args = Arc::clone(&args);
            let fp = Arc::clone(&files[idx]);

            std::thread::spawn(move || {
                // 提取该文件对应的参数
                let file_args = UploadArgs {
                    hostname: args.hostname.clone(),
                    user: args.user.clone(),
                    port: args.port,
                    identity_file: args.identity_file.clone(),
                    remote_dir: args.remote_dir.clone(),
                    files: vec![args.files[idx].clone()],
                };

                let result = upload_via_scp(&file_args);

                // 更新进度状态
                {
                    let mut f = fp.lock().unwrap();
                    if let Err(e) = result {
                        f.finished = true;
                        f.success = false;
                        f.error = Some(format!("{}", e));
                    } else {
                        f.done = f.size;
                        f.finished = true;
                        f.success = true;
                    }
                }

                active.fetch_sub(1, Ordering::Release);
                let _ = tx.send(idx);
            });
        }

        // 等待任意一个上传完成
        rx.recv().ok();
        completed_count += 1;

        // 重新渲染
        render_all(&files, start_time);
    }

    Ok(())
}
