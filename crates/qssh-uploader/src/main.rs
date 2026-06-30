/// qssh-uploader - Quick-SSH 独立文件上传工具
///
/// 由 qssh connect 在检测到拖拽操作时自动启动。
/// 使用 libssh2 (ssh2 crate) 实现 SFTP 上传，显示独立进度窗口。
///
/// 使用方式:
///   qssh-uploader --host <host> --user <user> --port <port>
///                --remote-dir <dir> [--key <keyfile>] <file1> [file2 ...]

use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::{Context, Result};
use ssh2::Session;

// ── 参数解析 ────────────────────────────────────────────

struct UploadArgs {
    hostname: String,
    user: Option<String>,
    port: u16,
    identity_file: Option<PathBuf>,
    remote_dir: PathBuf,
    files: Vec<PathBuf>,
}

fn parse_args() -> Result<UploadArgs> {
    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;
    let mut hostname = String::new();
    let mut user = None;
    let mut port = 22u16;
    let mut identity_file = None;
    let mut remote_dir = PathBuf::from(".");
    let mut files = Vec::new();

    while i < args.len() {
        match args[i].as_str() {
            "--host" => {
                i += 1;
                hostname = args[i].clone();
            }
            "--user" => {
                i += 1;
                user = Some(args[i].clone());
            }
            "--port" => {
                i += 1;
                port = args[i].parse()?;
            }
            "--key" => {
                i += 1;
                identity_file = Some(PathBuf::from(&args[i]));
            }
            "--remote-dir" => {
                i += 1;
                remote_dir = PathBuf::from(&args[i]);
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

// ── 进度条渲染 ──────────────────────────────────────────

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

fn render_bar(ratio: f64, width: u16) -> String {
    let filled = (ratio * width as f64).round() as u16;
    let empty = width.saturating_sub(filled);
    let bar: String = std::iter::repeat('█')
        .take(filled as usize)
        .chain(std::iter::repeat('░').take(empty as usize))
        .collect();
    bar
}

struct UploadState {
    file_index: usize,
    file_name: String,
    file_size: u64,
    file_done: u64,
    start_time: Instant,
    total_files: usize,
    done_files: usize,
}

impl UploadState {
    fn start_file(&mut self, index: usize, name: String, size: u64) {
        self.file_index = index;
        self.file_name = name;
        self.file_size = size;
        self.file_done = 0;
    }

    fn update(&mut self, done: u64) {
        self.file_done = done;
    }

    fn finish_file(&mut self) {
        self.done_files += 1;
    }

    fn render(&self) {
        // 清除之前的内容
        print!("\x1b[2J\x1b[H");

        println!("📤 qssh-uploader - 文件上传工具\n");

        // 总体进度
        let overall_ratio = if self.total_files > 0 {
            (self.done_files as f64 + if self.file_size > 0 {
                self.file_done as f64 / self.file_size as f64
            } else {
                0.0
            }) / self.total_files as f64
        } else {
            0.0
        };

        println!(
            "总体进度: [{}/{}] {}",
            self.done_files, self.total_files,
            render_bar(overall_ratio.clamp(0.0, 1.0), 40)
        );
        println!(
            "         {:.0}%  ({})",
            overall_ratio * 100.0,
            format_bytes(self.files_done_bytes())
        );
        println!();

        // 当前文件进度
        if self.file_size > 0 {
            let file_ratio = self.file_done as f64 / self.file_size as f64;
            println!(
                "当前文件: {}  {}",
                self.file_name,
                render_bar(file_ratio.clamp(0.0, 1.0), 30)
            );
            println!(
                "         {:.0}%  {}/{}",
                file_ratio * 100.0,
                format_bytes(self.file_done),
                format_bytes(self.file_size),
            );
        }

        // 耗时
        let elapsed = self.start_time.elapsed().as_secs_f64();
        println!("\n耗时: {}", format_duration(elapsed));

        std::io::stdout().flush().ok();
    }

    fn files_done_bytes(&self) -> u64 {
        // 简化：不追踪每个已完成文件的具体字节数
        self.file_done
    }
}

// ── SFTP 上传逻辑 ───────────────────────────────────────

fn connect_sftp(args: &UploadArgs) -> Result<(Session, ssh2::Sftp)> {
    let addr = format!("{}:{}", args.hostname, args.port);
    let tcp = std::net::TcpStream::connect(&addr)
        .with_context(|| format!("无法连接到 {}", addr))?;

    let mut session = Session::new()?;
    session.set_tcp_stream(tcp);
    session.handshake().context("SSH 握手失败")?;

    if let Some(ref key_path) = args.identity_file {
        let key_str = key_path.to_string_lossy().to_string();
        let expanded = shellexpand::full(&key_str)?;
        session
            .userauth_pubkey_file(
                args.user.as_deref().unwrap_or("root"),
                None,
                Path::new(expanded.as_ref()),
                None, // passphrase
            )
            .context("公钥认证失败")?;
    } else {
        session
            .userauth_agent(args.user.as_deref().unwrap_or("root"))
            .context("agent 认证失败")?;
    }

    if !session.authenticated() {
        anyhow::bail!("SSH 认证失败");
    }

    let sftp = session.sftp()?;
    Ok((session, sftp))
}

fn ensure_remote_dir(sftp: &ssh2::Sftp, dir: &Path) -> Result<()> {
    let mut current = PathBuf::new();
    for component in dir.components() {
        current.push(component);
        if sftp.stat(&current).is_err() {
            sftp.mkdir(&current, 0o755)
                .with_context(|| format!("无法创建远程目录: {}", current.display()))?;
        }
    }
    Ok(())
}

fn upload_file(
    sftp: &ssh2::Sftp,
    local: &Path,
    remote: &Path,
    on_progress: &mut dyn FnMut(u64, u64),
) -> Result<()> {
    let mut remote_file = sftp
        .create(remote)
        .with_context(|| format!("无法创建远程文件: {}", remote.display()))?;

    let data = std::fs::read(local)
        .with_context(|| format!("无法读取本地文件: {}", local.display()))?;

    let total = data.len() as u64;
    let chunk_size = 65536u64;
    let mut written = 0u64;

    for chunk in data.chunks(chunk_size as usize) {
        remote_file.write_all(chunk)?;
        written += chunk.len() as u64;
        on_progress(written, total);
    }

    remote_file.close()?;
    Ok(())
}

// ── 主入口 ───────────────────────────────────────────────

fn main() -> Result<()> {
    let args = parse_args()?;

    // 启用终端原始模式（但保留回显用于进度条）
    let _raw = enable_raw_mode();

    let mut state = UploadState {
        file_index: 0,
        file_name: String::new(),
        file_size: 0,
        file_done: 0,
        start_time: Instant::now(),
        total_files: args.files.len(),
        done_files: 0,
    };

    state.render();
    println!("\n🔗 正在连接 {}:{} ...", args.hostname, args.port);

    let (_session, sftp) = connect_sftp(&args)?;

    // 确保远程根目录存在
    ensure_remote_dir(&sftp, &args.remote_dir)?;

    // 逐个上传文件
    for (i, local_path) in args.files.iter().enumerate() {
        let file_name = local_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        let metadata = std::fs::metadata(local_path)?;
        let file_size = metadata.len();

        state.start_file(i, file_name.clone(), file_size);
        state.render();
        println!("\n📄 正在上传: {} ({})", file_name, format_bytes(file_size));

        let remote_path = args.remote_dir.join(&file_name);

        upload_file(&sftp, local_path, &remote_path, &mut |written, _total| {
            state.update(written);
            state.render();
        })?;

        state.finish_file();
        state.render();
        println!("✅ 上传完成: {}", file_name);
    }

    // 恢复终端
    disable_raw_mode();
    println!("\n🎉 所有文件上传完成！耗时: {}", format_duration(state.start_time.elapsed().as_secs_f64()));
    println!("按 Enter 键退出...");
    let _ = std::io::stdin().read_line(&mut String::new());

    Ok(())
}

// ── 终端控制 ─────────────────────────────────────────────

/// Windows 10+ 和控制台默认启用了 ANSI 转义序列支持，
/// 因此使用 print!("\x1b[...") 即可工作。
/// 此函数仅用于预留扩展点。
fn enable_raw_mode() -> Result<()> {
    Ok(())
}

fn disable_raw_mode() {}
