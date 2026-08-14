//! Web UI：本地 HTTP 服务（零重型依赖）
//!
//! `qssh web` 子命令启动一个本地 HTTP 服务，提供：
//! - `GET  /`            内嵌前端页面（暗色响应式 WebUI）
//! - `GET  /api/status`  运行状态（provider / 权限 / key / 主机数）
//! - `GET  /api/hosts`   SSH 主机列表
//! - `POST /api/chat`    发送消息，运行 Agent 会话（流式回报事件）
//! - `POST /api/approve` 危险操作批准回调
//! - `GET/POST /api/config`  读取 / 写入 Agent 配置
//!
//! 零新增重型依赖：使用 `std::net::TcpListener` + 极简 HTTP 解析。

pub mod server;

/// 版本常量（前端展示）
pub const WEBUI_VERSION: &str = env!("CARGO_PKG_VERSION");

/// 默认监听端口
pub const DEFAULT_PORT: u16 = 17890;
