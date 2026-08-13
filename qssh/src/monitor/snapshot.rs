//! 监控数据模型：`ServerSnapshot` + 各指标结构 + 采集错误

use super::docker::DockerContainer;
use super::files::RemoteFile;
use super::network::ConnCounts;
use super::services::ServiceInfo;

/// 一次采集的全部监控数据快照
#[derive(Debug, Clone, Default)]
pub struct ServerSnapshot {
    /// 服务器别名（来自 SshTarget）
    pub alias: String,
    /// 采集时间戳（Unix 秒）
    #[allow(dead_code)]
    pub collected_at: u64,
    pub cpu: Option<CpuInfo>,
    pub memory: Option<MemoryInfo>,
    pub disks: Vec<DiskInfo>,
    pub network: Option<NetworkInfo>,
    pub system: Option<SystemInfo>,
    pub processes: Vec<ProcessInfo>,
    /// Docker 容器列表（Docker 不可用时为空）
    pub docker: Vec<DockerContainer>,
    /// 系统服务列表（systemd 不可用时为空）
    pub services: Vec<ServiceInfo>,
    /// TCP 连接状态统计（`ss` 不可用时为 None）
    pub conn_counts: Option<ConnCounts>,
    /// 远程文件列表（FileOps 浏览的当前目录内容）
    pub files: Vec<RemoteFile>,
    /// 文件浏览当前目录（默认 `~`）
    pub file_cwd: String,
    /// 本次采集的警告/错误信息
    pub warnings: Vec<String>,
}

impl ServerSnapshot {
    pub fn new(alias: impl Into<String>) -> Self {
        Self {
            alias: alias.into(),
            collected_at: now_unix(),
            file_cwd: "~".to_string(),
            ..Default::default()
        }
    }

    /// 是否有致命错误（如连接失败）
    pub fn has_fatal_error(&self) -> bool {
        self.cpu.is_none() && self.memory.is_none() && self.disks.is_empty()
    }
}

/// CPU 指标
#[derive(Debug, Clone)]
pub struct CpuInfo {
    /// 使用率百分比 0-100
    pub usage_percent: f32,
    /// 负载 1min / 5min / 15min
    pub load: (f32, f32, f32),
    /// 逻辑核心数
    pub cores: u32,
    /// 主频 GHz
    pub freq_ghz: f32,
}

/// 内存指标
#[derive(Debug, Clone)]
pub struct MemoryInfo {
    pub total_gb: f32,
    pub used_gb: f32,
    /// 使用率百分比 0-100
    pub usage_percent: f32,
}

/// 磁盘指标
#[derive(Debug, Clone)]
pub struct DiskInfo {
    pub mount: String,
    pub used_gb: f32,
    pub total_gb: f32,
    pub usage_percent: f32,
}

/// 网络指标
#[derive(Debug, Clone)]
pub struct NetworkInfo {
    /// 下行速率 MB/s
    pub rx_mbps: f32,
    /// 上行速率 MB/s
    pub tx_mbps: f32,
    /// 主网卡 IP
    pub ip: Option<String>,
}

/// 系统信息
#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub os: String,
    pub hostname: String,
    /// 运行时长（秒）
    pub uptime_secs: u64,
}

/// 进程指标
#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_percent: f32,
    pub mem_mb: f32,
}

fn now_unix() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_snapshot_has_alias_and_timestamp() {
        let snap = ServerSnapshot::new("web01");
        assert_eq!(snap.alias, "web01");
        assert!(snap.collected_at > 0);
        assert!(snap.disks.is_empty());
    }

    #[test]
    fn empty_snapshot_is_fatal() {
        let snap = ServerSnapshot::new("x");
        assert!(snap.has_fatal_error());
    }

    #[test]
    fn snapshot_with_cpu_is_not_fatal() {
        let mut snap = ServerSnapshot::new("x");
        snap.cpu = Some(CpuInfo {
            usage_percent: 10.0,
            load: (1.0, 1.0, 1.0),
            cores: 4,
            freq_ghz: 3.5,
        });
        assert!(!snap.has_fatal_error());
    }
}
