//! Web UI：本地 HTTP 服务（零重型依赖）
//!
//! `qssh web` 子命令启动一个本地 HTTP 服务，提供 Agent 配置查看 / 修改，
//! 以及一键测试 Provider 连接（验证 API key 与网络连通性）。
//!
//! - `GET  /`            内嵌前端页面（暗色响应式 WebUI）
//! - `GET  /api/status`  运行状态（provider / 权限 / key 提示）
//! - `GET/POST /api/config`  读取 / 写入 Agent 配置
//! - `POST /api/test`    用当前配置向 Provider 发最小请求，验证连接
//!
//! 零新增重型依赖：使用 `std::net::TcpListener` + 极简 HTTP 解析。

pub mod server;

/// 版本常量（前端展示）
pub const WEBUI_VERSION: &str = env!("CARGO_PKG_VERSION");

/// 默认监听端口
pub const DEFAULT_PORT: u16 = 17890;
