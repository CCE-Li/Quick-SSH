// ── 交互式 SSH 会话 ──────────────────────────────────────
//!
//! SSH 进程直接继承父进程的 stdin（`Stdio::inherit()`），
//! 确保 SSH 可以正常从控制台读取密码输入。
//!
//! ## 平台差异
//!
//! - **Unix (Linux/WSL)**: stdout/stderr 直接继承终端，SSH
//!   通过 isatty() 检测到终端后正确处理 PTY 回显。
//! - **Windows**: stdout/stderr 使用 pipe 转发，配合 WinAPI
//!   控制台模式管理，后续可能用于拖拽上传功能。

#[cfg(windows)]
use std::io::{Read, Write};
use std::process::{Command, Stdio};

use anyhow::{Context, Result};

use super::session::SshTarget;

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
    let mut child = Command::new("ssh")
        .args(&args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .context("无法启动 ssh 进程，请确保已安装 OpenSSH Client")?;

    #[cfg(windows)]
    let mut child = Command::new("ssh")
        .args(&args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("无法启动 ssh 进程，请确保已安装 OpenSSH Client")?;

    // Unix 直接等待 SSH 结束（stdout/stderr 已继承，无需转发线程）
    #[cfg(unix)]
    {
        let status = child.wait().context("等待 ssh 进程结束失败")?;
        return Ok(status.code().unwrap_or(-1));
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
