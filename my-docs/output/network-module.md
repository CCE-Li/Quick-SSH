# 网络工具模块

网络工具模块位于 [`qssh/src/network/`](/qssh/src/network/)，提供 TCP 连接检测功能。

## 模块结构

```
network/
├── mod.rs      # 模块声明
└── ping.rs     # TCP Ping 检测
```

## ping.rs — TCP 连接测试

`check_host()` 函数提供基于 TCP 连接的超时检测：

```rust
pub fn check_host(hostname: &str, port: u16, timeout_secs: u64) -> Result<bool> {
    let addr = format!("{}:{}", hostname, port);
    let mut addrs = addr.to_socket_addrs()?;
    let addr = addrs.next()?;
    match TcpStream::connect_timeout(&addr, Duration::from_secs(timeout_secs)) {
        Ok(_) => Ok(true),     // 连接成功 → 在线
        Err(_) => Ok(false),   // 连接失败 → 离线
    }
}
```

### 工作流程

1. **地址解析**：将 `hostname:port` 通过 `ToSocketAddrs` 解析为 SocketAddr
2. **超时连接**：使用 `TcpStream::connect_timeout()` 尝试建立 TCP 连接
3. **结果判定**：
   - 连接成功 → 返回 `true`（在线）
   - 连接失败（超时/拒绝/无路由）→ 返回 `false`（离线）

### 调用场景

Ping 检测在 TUI 中通过后台线程调用：

- 单机检测：按 `p` 键检测当前选中主机
- 全量检测：按 `P` 键检测所有主机

默认超时时间为 3 秒，可通过 `~/.qsshrc` 的 `ping_timeout_secs` 配置。

### 设计说明

- **纯 TCP 检测**：不依赖 ICMP 协议（Windows 上不需要管理员权限）
- **非阻塞**：通过后台线程异步执行，不阻塞 TUI 事件循环
- **结果缓存**：检测结果存储在 `App.host_status` 中，实时更新 UI 图标
- **防重复检测**：`App.pending_pings` 集合防止同一主机重复检测
