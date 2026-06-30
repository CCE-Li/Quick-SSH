use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use ssh2::Session;

use crate::config::ssh_config::HostBlock;

// ── SFTP 上传 ────────────────────────────────────────────

/// 单个文件上传条目
#[derive(Debug, Clone)]
pub struct UploadEntry {
    pub local_path: PathBuf,
    pub relative_path: String,
    pub size: u64,
}

/// SFTP 上传参数
#[derive(Debug, Clone)]
pub struct UploadPayload {
    pub hostname: String,
    pub user: Option<String>,
    pub port: u16,
    pub identity_file: Option<PathBuf>,
    pub remote_dir: PathBuf,
    pub files: Vec<UploadEntry>,
}

/// 从 HostBlock 构建上传参数
impl UploadPayload {
    pub fn from_host(host: &HostBlock, files: Vec<PathBuf>, remote_dir: PathBuf) -> Result<Self> {
        let hostname = host
            .hostname()
            .context("Host 缺少 HostName 配置")?
            .to_string();

        let entries = files
            .iter()
            .map(|p| -> Result<UploadEntry> {
                let metadata = std::fs::metadata(p)?;
                let relative = p
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                Ok(UploadEntry {
                    local_path: p.clone(),
                    relative_path: relative,
                    size: metadata.len(),
                })
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(Self {
            hostname,
            user: host.user().map(|s| s.to_string()),
            port: host.port(),
            identity_file: host.identity_file().cloned(),
            remote_dir,
            files: entries,
        })
    }
}

/// 建立 SSH 连接并打开 SFTP 会话
fn connect_sftp(payload: &UploadPayload) -> Result<(Session, ssh2::Sftp)> {
    let addr = format!("{}:{}", payload.hostname, payload.port);
    let tcp = std::net::TcpStream::connect(&addr)
        .with_context(|| format!("无法连接到 {}", addr))?;

    let mut session = Session::new().context("无法创建 SSH 会话")?;
    session.set_tcp_stream(tcp);
    session.handshake().context("SSH 握手失败")?;

    // 认证
    if let Some(ref key_path) = payload.identity_file {
        let key_str = key_path.to_string_lossy().to_string();
        let expanded = shellexpand::full(&key_str)?;
        session
            .userauth_pubkey_file(
                payload.user.as_deref().unwrap_or("root"),
                None,
                Path::new(expanded.as_ref()),
                None, // passphrase
            )
            .context("公钥认证失败")?;
    } else {
        session
            .userauth_agent(payload.user.as_deref().unwrap_or("root"))
            .context("agent 认证失败")?;
    }

    if !session.authenticated() {
        anyhow::bail!("SSH 认证失败");
    }

    let sftp = session.sftp().context("无法打开 SFTP 会话")?;
    Ok((session, sftp))
}

/// 确保远程目录存在
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

/// 上传单个文件，带进度回调
fn upload_one_file(
    sftp: &ssh2::Sftp,
    entry: &UploadEntry,
    remote_dir: &Path,
    on_progress: &dyn Fn(u64, u64),
) -> Result<()> {
    let remote_path = remote_dir.join(&entry.relative_path);
    let mut remote_file = sftp
        .create(&remote_path)
        .with_context(|| format!("无法创建远程文件: {}", remote_path.display()))?;

    let local_data = std::fs::read(&entry.local_path)
        .with_context(|| format!("无法读取本地文件: {}", entry.local_path.display()))?;

    // 分块写入以触发进度回调
    let chunk_size = 65536u64; // 64KB
    let total = local_data.len() as u64;
    let mut written = 0u64;

    for chunk in local_data.chunks(chunk_size as usize) {
        remote_file.write_all(chunk)?;
        written += chunk.len() as u64;
        on_progress(written, total);
    }

    remote_file.close()?;
    Ok(())
}

/// 执行文件上传任务
///
/// 建立 SFTP 连接 → 创建远程目录 → 逐个上传文件（带进度回调）
pub fn run_upload(
    payload: &UploadPayload,
    on_progress: &dyn Fn(usize, u64, u64),
) -> Result<()> {
    let (_session, sftp) = connect_sftp(payload)?;

    // 确保远程根目录存在
    ensure_remote_dir(&sftp, &payload.remote_dir)?;

    for (i, entry) in payload.files.iter().enumerate() {
        // 每个文件上传前确保其子目录存在
        if let Some(parent) = Path::new(&entry.relative_path).parent() {
            if !parent.as_os_str().is_empty() {
                ensure_remote_dir(&sftp, &payload.remote_dir.join(parent))?;
            }
        }

        upload_one_file(&sftp, entry, &payload.remote_dir, &|written, total| {
            on_progress(i, written, total);
        })?;
    }

    Ok(())
}

/// 生成 qssh-uploader 的 CLI 参数（用于独立进程启动）
pub fn build_uploader_args(payload: &UploadPayload) -> Vec<String> {
    let mut args = Vec::new();

    args.push("--host".into());
    args.push(payload.hostname.clone());

    if let Some(ref user) = payload.user {
        args.push("--user".into());
        args.push(user.clone());
    }

    args.push("--port".into());
    args.push(payload.port.to_string());

    if let Some(ref key) = payload.identity_file {
        args.push("--key".into());
        args.push(key.display().to_string());
    }

    args.push("--remote-dir".into());
    args.push(payload.remote_dir.display().to_string());

    for file in &payload.files {
        args.push(file.local_path.display().to_string());
    }

    args
}
