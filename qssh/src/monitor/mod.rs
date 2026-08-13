//! 服务器监控：远程执行抽象 + 数据模型 + 采集调度
//!
//! Phase 2 目标：Dashboard 显示真实监控数据，UI 永不因 SSH 阻塞。
//!
//! 模块划分（内部模块化，后续可抽为 `qssh-core` crate）：
//! - [`executor`]：RemoteExecutor trait + SshProcessExecutor（系统 ssh）
//! - [`snapshot`]：ServerSnapshot 数据模型 + 错误分类
//! - [`platform`]：Linux 采集命令层 + 输出解析
//! - [`scheduler`]：后台调度器，事件经 mpsc 通道投递到 UI

pub mod docker;
pub mod executor;
pub mod files;
pub mod network;
pub mod platform;
pub mod scheduler;
pub mod services;
pub mod snapshot;

// 统一对外 re-export：当前为 bin crate，部分 API 尚未被 UI 消费，
// 保留公共 API 便于 Phase 2.4 集成与后续拆 crate。
#[allow(unused_imports)]
pub use docker::{collect_containers, container_action_command, DockerContainer};
#[allow(unused_imports)]
pub use executor::{ExecError, ExecOutput, RemoteExecutor, SshProcessExecutor};
#[allow(unused_imports)]
pub use files::{
    dir_entries, format_file_size, join_dir, ls_l_command, parent_dir, parse_ls_l, shell_quote,
    FileAction, RemoteFile,
};
#[allow(unused_imports)]
pub use network::{parse_ss_tan, ss_connections_command, ConnCounts};
#[allow(unused_imports)]
pub use platform::Collector;
#[allow(unused_imports)]
pub use scheduler::{BackgroundEvent, MonitorScheduler};
#[allow(unused_imports)]
pub use services::{collect_services, service_action_command, ServiceAction, ServiceInfo};
#[allow(unused_imports)]
pub use snapshot::{
    CpuInfo, DiskInfo, MemoryInfo, NetworkInfo, ProcessInfo, ServerSnapshot, SystemInfo,
};
