//! Web UI：本地 HTTP 服务（零重型依赖）
//!
//! `qssh web` 子命令启动一个本地 HTTP 服务，提供 Schema 驱动的设置页
//! （查看 / 修改各分组配置），以及一键测试 Provider 连接（验证 API key 与网络连通性）。
//!
//! - `GET  /`                   内嵌前端页面（暗色响应式 WebUI）
//! - `GET  /api/status`         运行状态（provider / 权限 / 版本）
//! - `GET/POST /api/settings`   读取 / 保存设置分组（Schema 驱动）
//! - `GET  /api/tools`          Agent 可用工具清单与审批方式
//! - `POST /api/test`           用当前配置向 Provider 发最小请求，验证连接
//!
//! 零新增重型依赖：使用 `std::net::TcpListener` + 极简 HTTP 解析。

pub mod schema;
pub mod server;

/// 版本常量（前端展示）
pub const WEBUI_VERSION: &str = env!("CARGO_PKG_VERSION");

/// 默认监听端口
pub const DEFAULT_PORT: u16 = 17890;
