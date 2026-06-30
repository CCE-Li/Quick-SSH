use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

use anyhow::Result;

// ── TCP 在线检测 ─────────────────────────────────────────

/// 检测主机是否在线（TCP 连接测试）
///
/// 尝试连接 hostname:port，超时后返回 false
pub fn check_host(hostname: &str, port: u16, timeout_secs: u64) -> Result<bool> {
    let addr = format!("{}:{}", hostname, port);
    let timeout = Duration::from_secs(timeout_secs);

    match TcpStream::connect_timeout(
        &addr.to_socket_addrs()?.next().ok_or_else(|| {
            anyhow::anyhow!("无法解析地址: {}", addr)
        })?,
        timeout,
    ) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}
