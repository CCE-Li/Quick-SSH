// ── 交互式 SSH 会话 ──────────────────────────────────────
//!
//! SSH 进程直接继承父进程的 stdin（`Stdio::inherit()`），
//! 确保 SSH 可以正常从控制台读取密码输入。
//!
//! ## 平台差异
//!
//! - **Unix (Linux/WSL)**: stdin/stdout/stderr 直接继承终端，SSH
//!   通过 isatty() 检测到终端后正确处理 PTY 回显。
//! - **Windows**: stdout/stderr 使用 pipe 转发，配合 WinAPI
//!   控制台模式管理，支持拖拽文件上传。

#[cfg(windows)]
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
#[cfg(windows)]
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(windows)]
use std::sync::{Arc, Condvar, Mutex};
#[cfg(windows)]
use std::time::Duration;

use anyhow::{Context, Result};

use super::drag_detect::detect_drag_files;
use super::session::SshTarget;
use crate::config::credentials;

// ── 主入口 ───────────────────────────────────────────────

/// 启动交互式 SSH 会话，支持拖拽文件上传
///
/// `drag_enabled` 为 true 时接管 stdin 检测拖拽粘贴路径（Windows 下需要配合
/// AskPass，避免与 ssh 密码提示抢控制台输入）；为 false 时继承 stdin，
/// 保证密码/密钥 passphrase 可以正常手动输入。
///
/// ## 平台差异
///
/// - **Unix (Linux/WSL)**: 不预先设置 raw 模式，SSH 自己管理终端。
///   `crossterm::enable_raw_mode()` 会禁用本地 ECHO，SSH 会将这些
///   终端设置传播到远程 PTY，导致远程也不回显键盘输入。
/// - **Windows**: 手动设置 raw 模式以启用 VT 输入支持。
///
/// TUI 调用者需确保调用前已用 `ratatui::try_restore()` 退出 TUI 模式。
pub fn start_interactive_session(
    target: &SshTarget,
    extra_args: &[String],
    drag_enabled: bool,
) -> Result<i32> {
    let use_saved_password = match credentials::has_password(&target.alias) {
        Ok(has_password) => has_password,
        Err(err) => {
            eprintln!("⚠️  无法读取已保存密码，将回退到系统 ssh: {err}");
            false
        }
    };

    start_openssh_session(target, extra_args, use_saved_password, drag_enabled)
}

#[cfg_attr(windows, allow(unused_variables))]
fn start_openssh_session(
    target: &SshTarget,
    extra_args: &[String],
    use_saved_password: bool,
    drag_enabled: bool,
) -> Result<i32> {
    // Unix: SSH 自己管理终端，不预先设置 raw 模式
    #[cfg(unix)]
    let result = start_interactive_session_inner(target, extra_args, use_saved_password);

    // Windows: 手动启用 raw 模式 + VT 输入支持
    //
    // drag_enabled 时使用拖拽会话（接管 stdin 检测拖拽粘贴路径）：
    // 拖拽检测需要读取控制台输入，而 SSH 密码提示也读同一控制台输入缓冲，
    // 二者会互相抢键。因此统一用 AskPass（SSH_ASKPASS_REQUIRE=force）处理一切
    // 密码/口令提示：有保存密码时自动填；没有保存密码时 AskPass 明确报错退出，
    // 避免 ssh 直接读控制台与拖拽会话抢输入导致卡死。
    // 未启用拖拽时继承 stdin，保证密码/密钥 passphrase 可正常手动输入。
    #[cfg(windows)]
    let result = {
        let _ = enable_terminal_raw_mode();
        let r = if drag_enabled {
            start_drag_session(target, extra_args)
        } else {
            start_interactive_session_inner(target, extra_args, use_saved_password)
        };
        let _ = disable_terminal_raw_mode();
        r
    };

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
#[allow(dead_code)]
fn enable_terminal_raw_mode() -> std::io::Result<()> {
    // Unix 直接使用 crossterm（它使用 termios）
    crossterm::terminal::enable_raw_mode()
}

#[cfg(not(windows))]
#[allow(dead_code)]
fn disable_terminal_raw_mode() -> std::io::Result<()> {
    crossterm::terminal::disable_raw_mode()
}

#[cfg_attr(windows, allow(dead_code))]
fn start_interactive_session_inner(
    target: &SshTarget,
    extra_args: &[String],
    use_saved_password: bool,
) -> Result<i32> {
    let mut args = build_ssh_args_with_pty(target);
    args.extend_from_slice(extra_args);

    let mut command = Command::new("ssh");
    command.args(&args).stdin(Stdio::inherit());

    if use_saved_password {
        configure_askpass(&mut command, &target.alias)?;
    }

    // stdin 使用 inherit()，让 SSH 直接从控制台读取输入，
    // 确保 SSH 的密码提示（ReadConsole API）能正常获取键盘输入。
    //
    // ── stdout/stderr 策略 ─────────────────────────────────
    // Unix (Linux/WSL): 使用 inherit() 直接继承终端，SSH 通过
    //   isatty(stdout) 检测到终端后会自动处理 PTY 回显。
    //   管道方式会导致 SSH 认为没有本地 TTY，即使有 -tt 也无法
    //   正确处理键盘输入的回显。
    // Windows: 使用 pipe 转发，因为 Windows 控制台 API 不同，
    //   且后续可能用于拖拽上传功能。
    #[cfg(unix)]
    let mut child = command
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .context("无法启动 ssh 进程，请确保已安装 OpenSSH Client")?;

    #[cfg(windows)]
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("无法启动 ssh 进程，请确保已安装 OpenSSH Client")?;

    // Unix 直接等待 SSH 结束（stdout/stderr 已继承，无需转发线程）
    #[cfg(unix)]
    {
        let status = child.wait().context("等待 ssh 进程结束失败")?;
        Ok(status.code().unwrap_or(-1))
    }

    // Windows 使用 pipe 转发 stdout/stderr
    #[cfg(windows)]
    {
        let ssh_stdout = child.stdout.take().context("无法获取 SSH 进程的 stdout")?;
        let ssh_stderr = child.stderr.take().context("无法获取 SSH 进程的 stderr")?;

        let stdout_handle = std::thread::Builder::new()
            .name("ssh-stdout".into())
            .spawn(move || {
                forward_stdout(ssh_stdout);
            })
            .context("无法创建 stdout 转发线程")?;

        let stderr_handle = std::thread::Builder::new()
            .name("ssh-stderr".into())
            .spawn(move || {
                forward_stderr(ssh_stderr);
            })
            .context("无法创建 stderr 转发线程")?;

        let status = child.wait().context("等待 ssh 进程结束失败")?;
        let _ = stdout_handle.join();
        let _ = stderr_handle.join();
        Ok(status.code().unwrap_or(-1))
    }
}

// ── 拖拽上传会话 (仅 Windows) ─────────────────────────────
//
// 接管 stdin 数据流读取键盘输入，同时累积到检测缓冲，250ms 寂静后
// 判断是否为拖拽粘贴的文件路径。检测到拖拽时向远程 shell 发送 PWD
// 探测命令获取当前目录，再启动独立 qssh-uploader 窗口执行 SCP 上传。
//
// 仅当主机已保存密码（AskPass 自动填密码、无交互输入）时启用，
// 避免与 SSH 密码提示共用控制台输入缓冲导致抢键。

/// 输入防抖延迟（ms）：拖拽粘贴后等待稳定状态再判断
#[cfg(windows)]
const DEBOUNCE_MS: u64 = 250;
/// 等待远程 PWD 响应超时（秒）
#[cfg(windows)]
const PWD_TIMEOUT_SECS: u64 = 5;

/// 拖拽会话：stdin/stdout/stderr 全部 piped，由本进程转发并检测拖拽
#[cfg(windows)]
fn start_drag_session(target: &SshTarget, extra_args: &[String]) -> Result<i32> {
    let mut args = build_ssh_args_with_pty(target);
    args.extend_from_slice(extra_args);

    let mut command = Command::new("ssh");
    command.args(&args);

    // 总是配置 AskPass：有保存密码时自动填；无保存密码时若意外出现密码/口令
    // 提示（如密钥带 passphrase 且未载入 agent），由 AskPass 明确报错退出，
    // 避免与拖拽会话抢控制台输入导致卡死。
    configure_askpass(&mut command, &target.alias)?;

    let mut child = command
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

    // ── stdout 转发线程（同时捕获 PWD 探测结果） ───────────
    let dp_out = Arc::clone(&drag_pending);
    let rp_out = Arc::clone(&remote_pwd);
    let cv_out = Arc::clone(&pwd_condvar);

    let stdout_handle = std::thread::Builder::new()
        .name("ssh-stdout".into())
        .spawn(move || {
            forward_stdout_with_pwd(ssh_stdout, dp_out, rp_out, cv_out);
        })
        .context("无法创建 stdout 转发线程")?;

    // ── stderr 转发线程 ───────────────────────────────────
    let stderr_handle = std::thread::Builder::new()
        .name("ssh-stderr".into())
        .spawn(move || {
            forward_stderr(ssh_stderr);
        })
        .context("无法创建 stderr 转发线程")?;

    // ── stdin 读取线程（控制台 → channel） ─────────────────
    let (tx, rx) = std::sync::mpsc::channel::<Vec<u8>>();

    let _stdin_reader = std::thread::Builder::new()
        .name("ssh-stdin-reader".into())
        .spawn(move || {
            read_stdin(tx);
        })
        .context("无法创建 stdin 读取线程")?;

    // ── 主循环：零延迟转发 + 拖拽后台检测 ──────────────────
    let mut detect_buf = Vec::<u8>::new();
    let debounce = Duration::from_millis(DEBOUNCE_MS);

    loop {
        match rx.recv_timeout(debounce) {
            Ok(chunk) => {
                // 累积到检测缓冲（保留原始字节，不破坏拖拽检测）
                detect_buf.extend_from_slice(&chunk);

                // 转发到 SSH（raw + VT 模式下 Enter 发送 \r，远程 PTY 的
                // icrnl 标志会自动将 \r → \n，直接透传即可）
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
                    // 内容已被转发到 SSH，用 Enter 让 shell 将其作为命令结束
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

/// stdin 读取线程：从控制台读取输入并通过 channel 发送
#[cfg(windows)]
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

/// 转发 SSH 的 stdout 到终端，并在拖拽处理期间捕获 `__QSSH_PWD__` 探测标记
#[cfg(windows)]
fn forward_stdout_with_pwd(
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
#[cfg(windows)]
fn extract_remote_pwd(acc: &str) -> Option<String> {
    // 标记 `__QSSH_PWD__` 长度 12
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

/// 处理一次拖拽：确认已粘贴路径、探测远程 PWD、启动上传窗口
#[cfg(windows)]
fn handle_drag(
    ssh_stdin: &mut dyn Write,
    target: &SshTarget,
    files: &[PathBuf],
    remote_pwd: &Arc<Mutex<Option<String>>>,
    pwd_condvar: &Arc<Condvar>,
    drag_pending: &Arc<AtomicBool>,
) {
    // ── 步骤 1: 按 Enter 结束当前行 ───────────────────────
    // 已输入的拖拽路径已被零延迟转发到 SSH 远程 shell 的输入缓冲区。
    // 发送 Enter 让 shell 把已输入的内容作为一条命令执行
    //（会报 "command not found"，但不会造成危害）。
    let enter = *b"\r";
    if let Err(e) = ssh_stdin.write_all(&enter) {
        log_write_error(&e);
        return;
    }
    if let Err(e) = ssh_stdin.flush() {
        log_write_error(&e);
        return;
    }
    std::thread::sleep(Duration::from_millis(200));

    // ── 步骤 2: 发送 PWD 探测命令 ─────────────────────────
    // 先设置标记，让 stdout 线程开始捕获 PWD 输出。
    // 使用十六进制转义构造 `__QSSH_PWD__` 标记，使 shell 回显命令时
    // 不包含该字面量，只有实际输出中才出现，避免误匹配到命令回显内容。
    // \x5f\x5f\x51\x53\x53\x48\x5f\x50\x57\x44\x5f\x5f = __QSSH_PWD__
    drag_pending.store(true, Ordering::SeqCst);

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

    // ── 步骤 3: 等待 PWD 结果（带超时） ───────────────────
    let pwd = {
        let guard = remote_pwd.lock().unwrap();
        let result = pwd_condvar.wait_timeout(guard, Duration::from_secs(PWD_TIMEOUT_SECS));
        match result {
            Ok((g, _timeout)) => g.clone(),
            Err(poisoned) => poisoned.into_inner().0.clone(),
        }
    };

    drag_pending.store(false, Ordering::SeqCst);

    // ── 步骤 4: 启动新窗口执行上传 ────────────────────────
    if let Some(ref remote_path) = pwd {
        spawn_upload_window(target, files, remote_path);
    } else {
        spawn_upload_window(target, files, ".");
    }
}

// ── 上传窗口启动 ─────────────────────────────────────────

/// 查找 qssh-uploader 可执行文件（优先与 qssh 同目录）
#[cfg_attr(not(windows), allow(dead_code))]
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

/// 在新窗口启动上传器（SCP 上传当前目录）
#[cfg_attr(not(windows), allow(dead_code))]
fn spawn_upload_window(target: &SshTarget, files: &[PathBuf], remote_dir: &str) {
    let uploader = find_uploader_binary();

    let mut args = vec![
        "--alias".to_string(),
        target.alias.clone(),
        "--host".to_string(),
        target.hostname.clone(),
    ];

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

/// Windows: 在新控制台窗口中启动上传器
///
/// 通过 `cmd.exe /c` 调用，末尾附加 `pause` 确保窗口保持打开。
#[cfg(windows)]
fn spawn_windows_upload_window(uploader: &Path, args: &[String]) {
    use std::os::windows::process::CommandExt;

    // 构建 cmd.exe 的完整命令字符串
    // cmd.exe /c <uploader> <args...> & pause
    let mut cmd_line = uploader.display().to_string();
    for arg in args {
        cmd_line.push(' ');
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

    let _ = Command::new("cmd.exe")
        .args(["/c", &cmd_line])
        .creation_flags(0x0000_0010) // CREATE_NEW_CONSOLE
        .spawn();
}

/// Unix: 尝试多种方式在新终端中启动上传器
#[cfg(not(windows))]
#[allow(dead_code)]
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

/// 写入 SSH 进程失败日志（忽略管道关闭）
#[cfg_attr(not(windows), allow(dead_code))]
fn log_write_error(e: &std::io::Error) {
    if e.kind() == std::io::ErrorKind::BrokenPipe {
        return;
    }
    eprintln!("[QSSH] 写入 SSH 进程失败: {}", e);
}

fn configure_askpass(command: &mut Command, alias: &str) -> Result<()> {
    let executable = std::env::current_exe().context("无法定位 qssh 可执行文件")?;
    command
        .env("SSH_ASKPASS", executable)
        .env("SSH_ASKPASS_REQUIRE", "force")
        .env(credentials::ASKPASS_ACTIVE_ENV, "1")
        .env(credentials::ASKPASS_ALIAS_ENV, alias);

    if std::env::var_os("DISPLAY").is_none() {
        command.env("DISPLAY", "qssh-askpass");
    }

    Ok(())
}

// ── SSH 参数构建 ─────────────────────────────────────────

/// 构建 SSH 参数，强制分配 PTY
///
/// 使用 `-tt`（双 t）确保 SSH 在远程分配 PTY，使交互式程序正常运作。
fn build_ssh_args_with_pty(target: &SshTarget) -> Vec<String> {
    let mut args = target.build_ssh_args();

    // 用 -tt 确保 SSH 在远程分配 PTY，使交互式程序正常运作
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

// ── stdout 转发线程 (仅 Windows) ──────────────────────────

/// 转发 SSH 的 stdout 到终端 (Windows: pipe → terminal)
#[cfg(windows)]
#[allow(dead_code)]
fn forward_stdout(mut ssh_stdout: impl Read + Send + 'static) {
    let mut buf = [0u8; 8192];
    let mut stdout = std::io::stdout();

    while let Ok(n) = ssh_stdout.read(&mut buf) {
        if n == 0 {
            break;
        }

        let data = &buf[..n];
        let _ = stdout.write_all(data);
        let _ = stdout.flush();
    }
}

// ── stderr 转发线程 (仅 Windows) ──────────────────────────

/// 转发 SSH 的 stderr 到终端 (Windows: pipe → terminal)
#[cfg(windows)]
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
