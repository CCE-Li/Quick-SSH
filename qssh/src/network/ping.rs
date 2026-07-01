use std::net::{TcpStream, ToSocketAddrs};

use anyhow::{Context, Result};

/// TCP 连接测试 —— 检测远程主机端口是否可达
pub fn check_host(hostname: &str, port: u16, timeout_secs: u64) -> Result<bool> {
    let addr = format!("{}:{}", hostname, port);
    let mut addrs = addr
        .to_socket_addrs()
        .with_context(|| format!("无法解析主机名: {}", hostname))?;

    let addr = addrs.next().context("无法解析主机地址: 没有返回任何地址")?;

    match TcpStream::connect_timeout(&addr, std::time::Duration::from_secs(timeout_secs)) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}
