// ── 交互式 SSH 会话（带拖拽上传） ─────────────────────────
//!
//! 使用 `Stdio::piped()` 代替 `inherit()`，拦截 stdin 数据流，
//! 检测文件拖拽粘贴事件，启动独立上传窗口。
//!
//! ## 设计要点
//!
//! - **零延迟输入转发**: 用户输入立即转发到 SSH，不等待防抖
//! - **并行拖拽检测**: 输入同时积累到检测缓存，250ms 寂静后检查
//! - **误触发处理**: 如果检测到拖拽，Ctrl+C 取消 SSH 中已输入的字符
//!
//! ## 流程
//!
//! 1. spawn ssh 进程 (stdin/stdout/stderr 全 piped)
//! 2. stdin 读取线程 → channel → 主循环
//! 3. stdout 转发线程 → 终端显示，同时检测 `__QSSH_PWD__` 标记
//! 4. stderr 转发线程 → 终端显示
//! 5. 检测到拖拽 → 发送 Ctrl+C → 探测远程 PWD → 启动上传窗口

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

use anyhow::{Context, Result};

use super::drag_detect::detect_drag_files;
use super::session::SshTarget;

// ── 常量 ─────────────────────────────────────────────────

/// 输入防抖延迟（ms）：拖拽粘贴后等待稳定状态再判断
const DEBOUNCE_MS: u64 = 250;
/// 等待远程 PWD 响应超时（秒）
const PWD_TIMEOUT_SECS: u64 = 5;

// ── 主入口 ───────────────────────────────────────────────

/// 启动交互式 SSH 会话，支持拖拽文件上传
///
/// 内部会启用终端 raw 模式，退出时自动恢复。
/// TUI 调用者需确保调用前已用 `ratatui::try_restore()` 退出 TUI 模式。
pub fn start_interactive_session(target: &SshTarget, extra_args: &[String]) -> Result<i32> {
    // 启用终端 raw 模式：禁用本地回显 + 行缓冲 + Ctrl+C 信号处理，
    // 同时在 Windows 上启用 ENABLE_VIRTUAL_TERMINAL_INPUT，
    // 使 std::io::stdin().read() 能正确读取每次按键的字节序列。
    let _ = enable_terminal_raw_mode();

    let result = start_interactive_session_inner(target, extra_args);

    // 恢复终端到 cooked 模式
    let _ = disable_terminal_raw_mode();
    result
}

// ── 跨平台终端 raw 模式 ──────────────────────────────────

/// Windows: 使用 WinAPI 设置控制台 raw 模式（含 VT 输入支持）
///
/// 同时保存并启用输出句柄的 ENABLE_VIRTUAL_TERMINAL_PROCESSING，
/// 确保 WriteConsoleW 能正确解析 ANSI 转义序列（如光标显示/隐藏）。
#[cfg(windows)]
fn enable_terminal_raw_mode() -> std::io::Result<()> {
    const STD_INPUT_HANDLE: u32 = 0xFFFFFFF6;
    const STD_OUTPUT_HANDLE: u32 = 0xFFFFFFF5;
    const ENABLE_PROCESSED_INPUT: u32 = 0x0001;
    const ENABLE_LINE_INPUT: u32 = 0x0002;
    const ENABLE_ECHO_INPUT: u32 = 0x0004;
    const ENABLE_VIRTUAL_TERMINAL_INPUT: u32 = 0x0200;
    const ENABLE_VIRTUAL_TERMINAL_PROCESSING: u32 = 0x0004;

    extern "system" {
        fn GetStdHandle(nStdHandle: u32) -> isize;
        fn GetConsoleMode(hConsoleHandle: isize, lpMode: *mut u32) -> i32;
        fn SetConsoleMode(hConsoleHandle: isize, dwMode: u32) -> i32;
    }

    unsafe {
        // ── 保存并设置输出句柄（用于 VT 序列处理） ──
        let out_handle = GetStdHandle(STD_OUTPUT_HANDLE);
        if out_handle != -1_isize && out_handle != 0_isize {
            let mut out_mode: u32 = 0;
            if GetConsoleMode(out_handle, &mut out_mode) != 0 {
                // 保存原始输出模式
                if let Ok(mut guard) = SAVED_OUTPUT_CONSOLE_MODE.lock() {
                    *guard = Some(out_mode);
                }
                // 确保 ENABLE_VIRTUAL_TERMINAL_PROCESSING 已启用
                if out_mode & ENABLE_VIRTUAL_TERMINAL_PROCESSING == 0 {
                    SetConsoleMode(out_handle, out_mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING);
                }
            }
        }

        // ── 设置输入句柄 raw 模式 ──
        let handle = GetStdHandle(STD_INPUT_HANDLE);
        if handle == -1_isize {
            return Err(std::io::Error::last_os_error());
        }
        let mut mode: u32 = 0;
        if GetConsoleMode(handle, &mut mode) == 0 {
            // 不是控制台（例如管道），忽略
            return Ok(());
        }
        // 保存原始模式用于恢复
        if let Ok(mut guard) = SAVED_CONSOLE_MODE.lock() {
            *guard = Some(mode);
        }

        // 禁用：ECHO, LINE_INPUT, PROCESSED_INPUT
        // 启用：VIRTUAL_TERMINAL_INPUT（使控制台输入转成 VT 字节序列，
        //       这样 std::io::stdin().read() 才能正确读取）
        let new_mode = (mode & !(ENABLE_ECHO_INPUT | ENABLE_LINE_INPUT | ENABLE_PROCESSED_INPUT))
            | ENABLE_VIRTUAL_TERMINAL_INPUT;
        SetConsoleMode(handle, new_mode);
        Ok(())
    }
}

/// 保存的控制台输入模式，用于退出时恢复
#[cfg(windows)]
static SAVED_CONSOLE_MODE: std::sync::Mutex<Option<u32>> = std::sync::Mutex::new(None);

/// 保存的控制台输出模式（含 VT 处理标志），用于退出时恢复
#[cfg(windows)]
static SAVED_OUTPUT_CONSOLE_MODE: std::sync::Mutex<Option<u32>> = std::sync::Mutex::new(None);

/// Windows: 恢复光标可见性（多种方法确保成功）
#[cfg(windows)]
fn restore_cursor() {
    const STD_OUTPUT_HANDLE: u32 = 0xFFFFFFF5;

    #[repr(C)]
    struct CONSOLE_CURSOR_INFO {
        dw_size: u32,
        b_visible: i32,
    }

    extern "system" {
        fn GetStdHandle(nStdHandle: u32) -> isize;
        fn SetConsoleCursorInfo(
            hConsoleOutput: isize,
            lpConsoleCursorInfo: *const CONSOLE_CURSOR_INFO,
        ) -> i32;
        fn WriteConsoleW(
            hConsoleOutput: isize,
            lpBuffer: *const u16,
            nNumberOfCharsToWrite: u32,
            lpNumberOfCharsWritten: *mut u32,
            lpReserved: *mut std::ffi::c_void,
        ) -> i32;
        fn WriteFile(
            hFile: isize,
            lpBuffer: *const std::ffi::c_void,
            nNumberOfBytesToWrite: u32,
            lpNumberOfBytesWritten: *mut u32,
            lpOverlapped: *mut std::ffi::c_void,
        ) -> i32;
    }

    unsafe {
        let handle = GetStdHandle(STD_OUTPUT_HANDLE);
        if handle == -1_isize || handle == 0_isize {
            return;
        }

        // 方法1: WriteConsoleW — 直接写入 ANSI 转义序列（需 VT 处理支持）
        let seq: Vec<u16> = "\x1b[?25h".encode_utf16().collect();
        let mut written = 0u32;
        let _ = WriteConsoleW(
            handle,
            seq.as_ptr(),
            seq.len() as u32,
            &mut written,
            std::ptr::null_mut(),
        );

        // 方法2: SetConsoleCursorInfo — 直接设置光标可见性（不依赖 VT 处理）
        let info = CONSOLE_CURSOR_INFO {
            dw_size: 25,
            b_visible: 1,
        };
        let _ = SetConsoleCursorInfo(handle, &info);

        // 方法3: WriteFile — 与 echo 相同的内核路径，作为备用
        let bytes = b"\x1b[?25h";
        let mut bytes_written = 0u32;
        let _ = WriteFile(
            handle,
            bytes.as_ptr() as *const std::ffi::c_void,
            bytes.len() as u32,
            &mut bytes_written,
            std::ptr::null_mut(),
        );
    }
}

#[cfg(windows)]
fn disable_terminal_raw_mode() -> std::io::Result<()> {
    const STD_INPUT_HANDLE: u32 = 0xFFFFFFF6;
    const STD_OUTPUT_HANDLE: u32 = 0xFFFFFFF5;
    extern "system" {
        fn GetStdHandle(nStdHandle: u32) -> isize;
        fn SetConsoleMode(hConsoleHandle: isize, dwMode: u32) -> i32;
    }

    // ── 1. 恢复光标可见性 ──
    // 必须在恢复控制台模式之前执行
    restore_cursor();

    // ── 2. 恢复原始控制台输出模式 ──
    if let Ok(mut guard) = SAVED_OUTPUT_CONSOLE_MODE.lock() {
        if let Some(mode) = guard.take() {
            unsafe {
                let handle = GetStdHandle(STD_OUTPUT_HANDLE);
                if handle != -1_isize && handle != 0_isize {
                    SetConsoleMode(handle, mode);
                }
            }
        }
    }

    // ── 3. 恢复原始控制台输入模式 ──
    if let Ok(mut guard) = SAVED_CONSOLE_MODE.lock() {
        if let Some(mode) = guard.take() {
            unsafe {
                let handle = GetStdHandle(STD_INPUT_HANDLE);
                SetConsoleMode(handle, mode);
            }
        }
    }

    // ── 4. 后备：通过 stdio 再写一次转义序列 ──
    use std::io::Write;
    let _ = std::io::stdout().write_all(b"\x1b[?25h");
    let _ = std::io::stdout().flush();

    Ok(())
}

#[cfg(not(windows))]
fn enable_terminal_raw_mode() -> std::io::Result<()> {
    // Unix 直接使用 crossterm（它使用 termios）
    crossterm::terminal::enable_raw_mode()
}

#[cfg(not(windows))]
fn disable_terminal_raw_mode() -> std::io::Result<()> {
    crossterm::terminal::disable_raw_mode()
}

fn start_interactive_session_inner(target: &SshTarget, extra_args: &[String]) -> Result<i32> {
    let mut args = build_ssh_args_with_pty(target);
    args.extend_from_slice(extra_args);

    let mut child = Command::new("ssh")
        .args(&args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("无法启动 ssh 进程，请确保已安装 OpenSSH Client")?;

    let mut ssh_stdin = child.stdin.take().context("无法获取 SSH 进程的 stdin")?;
    let ssh_stdout = child.stdout.take().context("无法获取 SSH 进程的 stdout")?;
    let ssh_stderr = child.stderr.take().context("无法获取 SSH 进程的 stderr")?;

    // ── 共享状态 ──────────────────────────────────────────
    let drag_pending = Arc::new(AtomicBool::new(false));
    let remote_pwd: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let pwd_condvar = Arc::new(Condvar::new());
    let target_clone = target.clone();

    // ── stdout 转发线程 ───────────────────────────────────
    let dp_out = Arc::clone(&drag_pending);
    let rp_out = Arc::clone(&remote_pwd);
    let cv_out = Arc::clone(&pwd_condvar);

    let stdout_handle = std::thread::Builder::new()
        .name("ssh-stdout".into())
        .spawn(move || {
            forward_stdout(ssh_stdout, dp_out, rp_out, cv_out);
        })
        .context("无法创建 stdout 转发线程")?;

    // ── stderr 转发线程 ───────────────────────────────────
    let stderr_handle = std::thread::Builder::new()
        .name("ssh-stderr".into())
        .spawn(move || {
            forward_stderr(ssh_stderr);
        })
        .context("无法创建 stderr 转发线程")?;

    // ── stdin 通道 ────────────────────────────────────────
    let (tx, rx) = std::sync::mpsc::channel::<Vec<u8>>();

    let _stdin_reader = std::thread::Builder::new()
        .name("ssh-stdin-reader".into())
        .spawn(move || {
            read_stdin(tx);
        })
        .context("无法创建 stdin 读取线程")?;

    // ── 主循环：零延迟转发 + 拖拽后台检测 ──────────────
    let mut detect_buf = Vec::<u8>::new();
    let debounce = Duration::from_millis(DEBOUNCE_MS);

    loop {
        match rx.recv_timeout(debounce) {
            Ok(chunk) => {
                // 积累到检测缓存（保留原始字节，不破坏拖拽检测）
                detect_buf.extend_from_slice(&chunk);

                // 转发到 SSH
                // 在 cooked 模式下，Windows 发送 \r\n；在 raw + VT 模式下，Enter 发送 \r。
                // 远程 PTY 的 icrnl 标志会自动将 \r → \n，所以直接透传即可。
                if let Err(e) = ssh_stdin.write_all(&chunk) {
                    log_write_error(&e);
                    break;
                }
                let _ = ssh_stdin.flush();
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                if detect_buf.is_empty() {
                    if let Ok(Some(_)) = child.try_wait() {
                        break;
                    }
                    continue;
                }

                // 250ms 寂静 → 检查积累的内容
                let text = String::from_utf8_lossy(&detect_buf);
                if let Some(files) = detect_drag_files(&text) {
                    // 内容已被转发到 SSH，用 Ctrl+C 取消
                    handle_drag(
                        &mut ssh_stdin,
                        &target_clone,
                        &files,
                        &remote_pwd,
                        &pwd_condvar,
                        &drag_pending,
                    );
                }
                detect_buf.clear();
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                break;
            }
        }
    }

    // ── 等待 ssh 退出 ─────────────────────────────────────
    drop(ssh_stdin);
    let _ = stdout_handle.join();
    let _ = stderr_handle.join();
    let status = child.wait().context("等待 ssh 进程结束失败")?;
    Ok(status.code().unwrap_or(-1))
}

// ── SSH 参数构建 ─────────────────────────────────────────

/// 构建 SSH 参数，强制分配 PTY（因为 stdin/stdout 是 piped 的）
///
/// 使用 `-tt`（双 t）而不是 `-t`，因为当 SSH 的 stdin/stdout 都是 pipe 时，
/// 单个 `-t` 不足以强制分配 PTY（SSH 会输出 "Pseudo-terminal will not be
/// allocated because stdin is not a terminal" 并拒绝分配）。
/// `-tt` 强制分配 PTY，使远程 shell 进入交互模式。
fn build_ssh_args_with_pty(target: &SshTarget) -> Vec<String> {
    let mut args = target.build_ssh_args();

    // ssh 的 stdin/stdout 都是 pipe，不会自动分配 PTY
    // 用 -tt 强制 SSH 在远程分配 PTY，使交互式程序正常运作
    let has_tt = args.iter().any(|a| a == "-tt");
    if !has_tt {
        if let Some(pos) = args.iter().rposition(|a| !a.starts_with('-')) {
            args.insert(pos, "-tt".into());
        } else {
            args.insert(0, "-tt".into());
        }
    }

    args
}

// ── stdin 读取线程 ───────────────────────────────────────

fn read_stdin(tx: std::sync::mpsc::Sender<Vec<u8>>) {
    let mut stdin = std::io::stdin();
    let mut buf = [0u8; 4096];
    while let Ok(n) = stdin.read(&mut buf) {
        if n == 0 {
            break;
        }
        if tx.send(buf[..n].to_vec()).is_err() {
            break;
        }
    }
}

// ── stdout 转发线程 ──────────────────────────────────────

fn forward_stdout(
    mut ssh_stdout: impl Read + Send + 'static,
    drag_pending: Arc<AtomicBool>,
    remote_pwd: Arc<Mutex<Option<String>>>,
    pwd_condvar: Arc<Condvar>,
) {
    let mut buf = [0u8; 8192];
    let mut stdout = std::io::stdout();
    let mut pwd_acc = String::new();

    while let Ok(n) = ssh_stdout.read(&mut buf) {
        if n == 0 {
            break;
        }

        let data = &buf[..n];

        if drag_pending.load(Ordering::SeqCst) {
            pwd_acc.push_str(&String::from_utf8_lossy(data));
            if let Some(pwd) = extract_remote_pwd(&pwd_acc) {
                let mut guard = remote_pwd.lock().unwrap();
                *guard = Some(pwd);
                drop(guard);
                pwd_condvar.notify_all();
                drag_pending.store(false, Ordering::SeqCst);
                pwd_acc.clear();
            }
        }

        let _ = stdout.write_all(data);
        let _ = stdout.flush();
    }
}

/// 从 SSH 输出中提取 `__QSSH_PWD__<path>__` 标记
///
/// 探测命令使用十六进制转义构造标记（`\x5f\x5f\x51\x53\x53\x48\x5f\x50\x57\x44\x5f\x5f`），
/// 因此 shell 回显命令时不会包含 `__QSSH_PWD__` 字面量，只有实际输出中才会出现。
/// 这避免了回显内容被误识别为命令输出。
fn extract_remote_pwd(acc: &str) -> Option<String> {
    // 标记 `__QSSH_PWD__` 长度为 12 字符
    const MARKER_LEN: usize = 12;
    if let Some(start) = acc.find("__QSSH_PWD__") {
        let after = &acc[start + MARKER_LEN..];
        if let Some(end) = after.find("__") {
            // 先拿到原始内容，然后去除所有控制字符
            let pwd = after[..end]
                .trim()
                .trim_matches(&['\r', '\n', ' '][..])
                .to_string();
            if !pwd.is_empty() {
                return Some(pwd);
            }
        }
    }
    None
}

// ── stderr 转发线程 ──────────────────────────────────────

/// 转发 SSH 的 stderr 到终端
fn forward_stderr(mut ssh_stderr: impl Read + Send + 'static) {
    let mut buf = [0u8; 4096];
    let mut stderr = std::io::stderr();

    while let Ok(n) = ssh_stderr.read(&mut buf) {
        if n == 0 {
            break;
        }

        let data = &buf[..n];
        let _ = stderr.write_all(data);
        let _ = stderr.flush();
    }
}

// ── 拖拽处理 ─────────────────────────────────────────────

fn handle_drag(
    ssh_stdin: &mut dyn Write,
    target: &SshTarget,
    files: &[PathBuf],
    remote_pwd: &Arc<Mutex<Option<String>>>,
    pwd_condvar: &Arc<Condvar>,
    drag_pending: &Arc<AtomicBool>,
) {
    writeln!(
        std::io::stdout(),
        "\r\n\x1b[33m\u{1f4e4} 检测到文件拖拽，正在准备上传...\x1b[0m"
    )
    .ok();

    // ── 步骤 1: 按 Enter 结束当前行 ─────────────────────
    // 已输入的拖拽路径已被零延迟转发到 SSH 远程 shell 的输入缓冲区。
    // Ctrl+C 通过 pipe 发送只是字节 0x03，不触发远程 SIGINT，
    // 所以改为发 Enter 让 shell 把已输入的内容作为一条命令执行
    //（会报 "command not found"，但不会造成危害）。
    let enter = [b'\r'];
    if let Err(e) = ssh_stdin.write_all(&enter) {
        log_write_error(&e);
        return;
    }
    if let Err(e) = ssh_stdin.flush() {
        log_write_error(&e);
        return;
    }
    std::thread::sleep(Duration::from_millis(200));

    // ── 步骤 2: 发送 PWD 探测命令 ───────────────────────
    // 先设置标志，让 stdout 线程开始捕获 PWD 输出
    drag_pending.store(true, Ordering::SeqCst);

    // 使用十六进制转义构造 __QSSH_PWD__ 标记，使得 shell 回显命令时不包含该字面量，
    // 只有实际输出中才出现，避免 extract_remote_pwd 误匹配到命令回显内容。
    // 注意：此处使用 \\x 双重转义，让远程 shell 收到字面 "\x5f" 字符串，
    //       而非 Rust 编译时直接转换成的 "_" 字节。
    // \x5f\x5f\x51\x53\x53\x48\x5f\x50\x57\x44\x5f\x5f = __QSSH_PWD__
    let probe_cmd = b"printf \"\\x5f\\x5f\\x51\\x53\\x53\\x48\\x5f\\x50\\x57\\x44\\x5f\\x5f%s\\x5f\\x5f\\n\" \"$PWD\"\n";
    if let Err(e) = ssh_stdin.write_all(probe_cmd) {
        log_write_error(&e);
        drag_pending.store(false, Ordering::SeqCst);
        return;
    }
    if let Err(e) = ssh_stdin.flush() {
        log_write_error(&e);
        drag_pending.store(false, Ordering::SeqCst);
        return;
    }

    // ── 步骤 3: 等待 PWD 结果（带超时） ─────────────────
    let pwd = {
        let guard = remote_pwd.lock().unwrap();
        let result = pwd_condvar.wait_timeout(guard, Duration::from_secs(PWD_TIMEOUT_SECS));
        match result {
            Ok((g, _timeout)) => g.clone(),
            Err(poisoned) => {
                let (g, _timeout) = poisoned.into_inner();
                writeln!(
                    std::io::stdout(),
                    "\r\n\x1b[31m\u{26a0}\u{fe0f} 获取远程路径超时\x1b[0m"
                )
                .ok();
                g.clone()
            }
        }
    };

    drag_pending.store(false, Ordering::SeqCst);

    // ── 步骤 4: 启动新窗口执行上传 ──────────────────────
    if let Some(ref remote_path) = pwd {
        writeln!(
            std::io::stdout(),
            "\r\n\x1b[32m\u{1f4c2} 远程路径: {}\x1b[0m",
            remote_path
        )
        .ok();
        spawn_upload_window(target, files, remote_path);
    } else {
        writeln!(
            std::io::stdout(),
            "\r\n\x1b[31m\u{26a0}\u{fe0f} 无法确定远程路径，使用当前目录\x1b[0m"
        )
        .ok();
        spawn_upload_window(target, files, ".");
    }

    writeln!(
        std::io::stdout(),
        "\r\n\x1b[32m\u{2705} 已在新窗口中启动上传任务\x1b[0m"
    )
    .ok();
}

// ── 上传窗口启动 ─────────────────────────────────────────

fn find_uploader_binary() -> PathBuf {
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let uploader_name = if cfg!(windows) {
                "qssh-uploader.exe"
            } else {
                "qssh-uploader"
            };
            let candidate = exe_dir.join(uploader_name);
            if candidate.exists() {
                return candidate;
            }
        }
    }
    PathBuf::from(if cfg!(windows) {
        "qssh-uploader.exe"
    } else {
        "qssh-uploader"
    })
}

fn spawn_upload_window(target: &SshTarget, files: &[PathBuf], remote_dir: &str) {
    let uploader = find_uploader_binary();

    let mut args: Vec<String> = Vec::new();
    args.push("--host".into());
    args.push(target.hostname.clone());

    if let Some(ref user) = target.user {
        args.push("--user".into());
        args.push(user.clone());
    }

    args.push("--port".into());
    args.push(target.port.to_string());

    if let Some(ref key) = target.identity_file {
        args.push("--key".into());
        args.push(key.display().to_string());
    }

    args.push("--remote-dir".into());
    args.push(remote_dir.to_string());

    for file in files {
        args.push(file.display().to_string());
    }

    #[cfg(windows)]
    {
        spawn_windows_upload_window(&uploader, &args);
    }

    #[cfg(not(windows))]
    {
        spawn_unix_upload_window(&uploader, &args);
    }
}

/// Windows: 在新控制台中启动上传器
///
/// 通过 `cmd.exe /c` 调用，末尾附加 `pause` 确保窗口保持打开。
/// 等价于在命令行中执行：
///   cmd.exe /c "qssh-uploader.exe --host ... & pause"
///
/// 注意：上传器路径不手动加引号，让 Rust 的 Command 参数序列化
/// 自动处理引号，避免 cmd.exe 解析错误。
#[cfg(windows)]
fn spawn_windows_upload_window(uploader: &Path, args: &[String]) {
    // 构建 cmd.exe 的完整命令字符串
    // cmd.exe /c <uploader> <args...> & pause
    let mut cmd_line = uploader.display().to_string();
    for arg in args {
        cmd_line.push(' ');
        // 只对含空格的参数加引号
        let needs_quote = arg.contains(' ');
        if needs_quote {
            cmd_line.push('"');
            cmd_line.push_str(arg);
            cmd_line.push('"');
        } else {
            cmd_line.push_str(arg);
        }
    }
    cmd_line.push_str(" & pause");

    // cmd.exe /c <cmd_line>
    // Rust 的 args() 会自动对 cmd_line 做必要转义
    let _ = Command::new("cmd.exe")
        .args(["/c", &cmd_line])
        .creation_flags(0x00000010) // CREATE_NEW_CONSOLE
        .spawn();
}

/// Unix: 尝试多种方式在新终端中启动
#[cfg(not(windows))]
fn spawn_unix_upload_window(uploader: &Path, args: &[String]) {
    let uploader_str = uploader.display().to_string();

    #[cfg(target_os = "macos")]
    {
        let mut cmd_parts = vec![uploader_str.clone()];
        cmd_parts.extend(args.iter().map(|a| {
            if a.contains(' ') {
                format!("\"{}\"", a)
            } else {
                a.clone()
            }
        }));
        let cmd_line = cmd_parts.join(" ");
        let _ = Command::new("open")
            .args(["-a", "Terminal", &cmd_line])
            .spawn();
    }

    #[cfg(not(target_os = "macos"))]
    {
        let terminal_cmds: [(&str, &[&str]); 4] = [
            ("x-terminal-emulator", &["-e"]),
            ("gnome-terminal", &["--"]),
            ("xterm", &["-e"]),
            ("konsole", &["-e"]),
        ];

        for (term, prefix) in &terminal_cmds {
            let mut cmd = Command::new(term);
            cmd.args(*prefix);
            cmd.arg(&uploader_str);
            cmd.args(args);
            cmd.stdout(Stdio::null());
            cmd.stderr(Stdio::null());
            if cmd.spawn().is_ok() {
                break;
            }
        }
    }
}

// ── 日志辅助 ─────────────────────────────────────────────

fn log_write_error(e: &std::io::Error) {
    if e.kind() == std::io::ErrorKind::BrokenPipe {
        return;
    }
    eprintln!("[qssh] 写入 SSH 进程失败: {}", e);
}
