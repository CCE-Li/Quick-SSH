//! 文件上传（SCP 方案，同进程内联执行）
//!
//! #![allow(dead_code)] — 当前使用独立 qssh-uploader 进程上传，
//!                        此模块保留以备将来可能的内联上传需求。
//!
//! 由 [`spawn.rs`](spawn.rs) 在检测到拖拽操作时直接调用。
//! 使用 OpenSSH `scp.exe` 实现文件传输，在当前终端内显示进度。
//!
//! ## 设计说明
//!
//! - 不启动独立进程/窗口，在同一终端内完成上传
//! - 每个文件调用一次 `scp`，上传完成后显示结果
//! - 上传期间 SSH 终端暂时冻结（stdin 转发暂停），但 stdout/stderr 仍正常显示

use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;

use anyhow::{Context, Result};

use crate::ssh::session::SshTarget;

// ── 格式化工具 ────────────────────────────────────────────

#[allow(dead_code)]
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

// ── 核心上传函数 ──────────────────────────────────────────

/// 上传文件到远程服务器（同进程内联执行）
///
/// 对每个文件调用 `scp.exe`，在终端内显示实时进度。
/// 上传期间当前线程被阻塞，但 stdout/stderr 转发线程不受影响。
#[allow(dead_code)]
pub fn upload_files(target: &SshTarget, files: &[PathBuf], remote_dir: &str) -> Result<()> {
    let total = files.len();
    let start_time = Instant::now();

    // ── 上传头 ────────────────────────────────────────────
    println!();
    println!("┌─ 📤 文件上传 ──────────────────────────────");
    println!("│  目标: {}:{}", target.hostname, remote_dir);
    println!("│  文件数: {}", total);
    println!("└────────────────────────────────────────────");
    println!();

    for (i, local_path) in files.iter().enumerate() {
        let file_name = local_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        let file_size = std::fs::metadata(local_path)
            .with_context(|| format!("无法读取本地文件: {}", local_path.display()))?
            .len();

        // 构建远程目标路径: [user@]host:remote_dir/filename
        let user_part = match target.user {
            Some(ref u) => format!("{}@", u),
            None => String::new(),
        };
        let remote_target = format!(
            "{}{}:{}/{}",
            user_part, target.hostname, remote_dir, file_name
        );

        // 显示当前文件信息
        println!(
            "  [{}/{}] {}  ({})",
            i + 1,
            total,
            file_name,
            format_bytes(file_size)
        );
        print!("   ");
        std::io::stdout().flush().ok();

        // 构建 scp 命令
        let mut cmd = std::process::Command::new("scp");

        if target.port != 22 {
            cmd.arg("-P");
            cmd.arg(target.port.to_string());
        }
        if let Some(ref key_path) = target.identity_file {
            cmd.arg("-i");
            cmd.arg(key_path.as_os_str());
        }
        // 禁用连接共享，避免与现有 SSH 会话冲突
        cmd.arg("-o").arg("ControlMaster=no");
        // 静默模式（无进度），我们自己显示状态
        cmd.arg("-q");
        cmd.arg(local_path.as_os_str());
        cmd.arg(&remote_target);

        let status = cmd.status().with_context(|| {
            format!(
                "无法启动 scp 进程 ({}: {})",
                file_name,
                local_path.display()
            )
        })?;

        if !status.success() {
            anyhow::bail!(
                "scp 上传失败: {} (退出码: {:?})",
                file_name,
                status.code().unwrap_or(-1)
            );
        }

        println!(" ✅");
    }

    // ── 上传完成总结 ─────────────────────────────────────
    let elapsed = start_time.elapsed().as_secs_f64();
    println!();
    println!("┌─ 🎉 上传完成 ──────────────────────────────");
    println!("│  文件数: {} | 耗时: {:.0}s", total, elapsed);
    println!("│  目标: {}:{}", target.hostname, remote_dir);
    println!("└────────────────────────────────────────────");

    Ok(())
}
