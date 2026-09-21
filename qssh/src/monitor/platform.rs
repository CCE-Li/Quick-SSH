//! 平台命令层：把远程采集命令解析为 [`ServerSnapshot`] 数据
//!
//! 首版实现 Linux（/proc、/sys、df、free、uptime、ps、ss、ip）。
//! 未来扩展 windows/macos（决策 D5：命令解析为主）。

use super::executor::{ExecOutput, RemoteExecutor};
use super::snapshot::{
    CpuInfo, DiskInfo, MemoryInfo, NetworkInfo, ProcessInfo, ServerSnapshot, SystemInfo,
};

/// 采集器：针对单个目标执行一组远程命令并组装快照
pub struct Collector {
    executor: Box<dyn RemoteExecutor>,
}

impl Collector {
    pub fn new(executor: Box<dyn RemoteExecutor>) -> Self {
        Self { executor }
    }

    /// 执行一次完整采集（命令并行化留给 scheduler，这里串行执行）
    pub fn collect(&self, alias: &str) -> ServerSnapshot {
        let mut snap = ServerSnapshot::new(alias);
        let mut warnings = Vec::new();

        match self.executor.exec(LINUX_COLLECT_ALL) {
            Ok(out) => parse_linux_output(&out, &mut snap),
            Err(e) => warnings.push(format!("采集失败: {e}")),
        }

        snap.warnings = warnings;
        snap
    }
}

/// Linux 一次性采集命令（分号分隔，避免多次 SSH 往返）
const LINUX_COLLECT_ALL: &str = r#"
echo "===CPU==="; cat /proc/stat | head -1;
echo "===LOAD==="; cat /proc/loadavg;
echo "===MEM==="; free -b | sed -n '2p';
echo "===DISK==="; df -B1 --output=target,used,size | tail -n +2;
echo "===NET==="; cat /proc/net/dev | tail -n +3;
echo "===SYS==="; uname -sr; hostname; cat /proc/uptime | cut -d' ' -f1;
echo "===PROC==="; ps -eo pid,comm,%cpu,rss --sort=-%cpu | head -n 10;
echo "===DOCKER==="; docker ps -a --format '{{.ID}}|{{.Names}}|{{.Image}}|{{.Status}}|{{.Ports}}' 2>/dev/null || true;
echo "===SERVICES==="; systemctl list-units --type=service --all --no-pager --no-legend --plain 2>/dev/null | awk '{print $1"|"$2"|"$3"|"$4}' || true;
echo "===CONNS==="; ss -tan state all 2>/dev/null | awk 'NR>1 {print $1}' | sort | uniq -c | awk '{print $2"|"$1}' || true;
echo "===FILES==="; LC_ALL=C ls -lA --time-style=long-iso 2>/dev/null | sed 's/  */ /g' || true;
echo "===LOGS==="; journalctl -n 50 --no-pager -o short-iso 2>/dev/null || true;
"#;

// ── 独立采集命令（Agent Tool Registry 按需调用）──────────────

/// 服务器状态命令（CPU 占用 + 负载 + 内存摘要）
pub fn server_status_command() -> &'static str {
    r#"cat /proc/stat | head -1; echo "--"; cat /proc/loadavg; echo "--"; free -m | sed -n '2p'"#
}

/// 进程列表命令（CPU 占用 Top 10）
pub fn processes_command() -> &'static str {
    "ps -eo pid,comm,%cpu,rss --sort=-%cpu | head -n 10"
}

/// 磁盘使用命令
pub fn disks_command() -> &'static str {
    "df -h --output=target,used,size,use% | tail -n +2"
}

/// 网络与 TCP 连接统计命令
pub fn network_command() -> &'static str {
    r#"echo "---NET---"; cat /proc/net/dev | tail -n +3; echo "---CONNS---"; ss -tan state all 2>/dev/null | awk 'NR>1 {print $1}' | sort | uniq -c | awk '{print $2"|"$1}' || true"#
}

/// 解析 Linux 采集命令输出到快照
fn parse_linux_output(out: &ExecOutput, snap: &mut ServerSnapshot) {
    let text = &out.stdout;
    let sections = split_sections(text);

    if let Some(cpu) = sections.get("CPU") {
        snap.cpu = parse_cpu(cpu);
    }
    if let Some(load) = sections.get("LOAD") {
        if let Some(cpu) = snap.cpu.as_mut() {
            cpu.load = parse_load(load);
        }
    }
    if let Some(mem) = sections.get("MEM") {
        snap.memory = parse_memory(mem);
    }
    if let Some(disk) = sections.get("DISK") {
        snap.disks = parse_disks(disk);
    }
    if let Some(net) = sections.get("NET") {
        snap.network = parse_network(net);
    }
    if let Some(sys) = sections.get("SYS") {
        snap.system = parse_system(sys);
    }
    if let Some(proc) = sections.get("PROC") {
        snap.processes = parse_processes(proc);
    }
    if let Some(docker) = sections.get("DOCKER") {
        snap.docker = super::docker::parse_docker_ps(docker);
    }
    if let Some(services) = sections.get("SERVICES") {
        snap.services = super::services::parse_systemctl(services);
    }
    if let Some(conns) = sections.get("CONNS") {
        snap.conn_counts = Some(super::network::parse_ss_tan(conns));
    }
    if let Some(files) = sections.get("FILES") {
        snap.files = super::files::parse_ls_l(files);
    }
    if let Some(logs) = sections.get("LOGS") {
        snap.logs = super::logs::parse_journalctl(logs);
    }
}

/// 按 `===KEY===` 切分输出为 section 映射
fn split_sections(text: &str) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    let mut current: Option<String> = None;
    let mut buf = String::new();

    for line in text.lines() {
        let line = line.trim_end();
        if line.starts_with("===") && line.ends_with("===") {
            if let Some(key) = current.take() {
                map.insert(key, buf.clone());
                buf.clear();
            }
            current = Some(line[3..line.len() - 3].to_string());
        } else if current.is_some() {
            buf.push_str(line);
            buf.push('\n');
        }
    }
    if let Some(key) = current {
        map.insert(key, buf);
    }
    map
}

fn parse_cpu(section: &str) -> Option<CpuInfo> {
    // 首版简化为读取 /proc/stat 第一行（无法直接算使用率，需要两次采样）
    // 这里解析"cpu user nice system idle"用于占位
    let line = section.lines().next()?.trim();
    let mut parts = line.split_whitespace();
    let _label = parts.next()?;
    let vals: Vec<u64> = parts.filter_map(|s| s.parse().ok()).collect();
    if vals.len() < 4 {
        return None;
    }
    let idle = vals[3];
    let total: u64 = vals.iter().sum();
    if total == 0 {
        return None;
    }
    Some(CpuInfo {
        usage_percent: 100.0 * (total - idle) as f32 / total as f32,
        load: (0.0, 0.0, 0.0),
        cores: 0,
        freq_ghz: 0.0,
    })
}

fn parse_load(section: &str) -> (f32, f32, f32) {
    let mut parts = section.split_whitespace();
    let a = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0.0);
    let b = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0.0);
    let c = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0.0);
    (a, b, c)
}

fn parse_memory(section: &str) -> Option<MemoryInfo> {
    // free -b 第二行: Mem: total used free shared buff/cache available
    let line = section
        .lines()
        .next()?
        .split_whitespace()
        .collect::<Vec<_>>();
    if line.len() < 3 {
        return None;
    }
    let total: f32 = line[1].parse().ok()?;
    let used: f32 = line[2].parse().ok()?;
    if total == 0.0 {
        return None;
    }
    Some(MemoryInfo {
        total_gb: total / 1024.0 / 1024.0 / 1024.0,
        used_gb: used / 1024.0 / 1024.0 / 1024.0,
        usage_percent: 100.0 * used / total,
    })
}

fn parse_disks(section: &str) -> Vec<DiskInfo> {
    section
        .lines()
        .filter_map(|line| {
            let mut parts = line.split_whitespace();
            let mount = parts.next()?.to_string();
            let used: f64 = parts.next()?.parse().ok()?;
            let total: f64 = parts.next()?.parse().ok()?;
            if total <= 0.0 {
                return None;
            }
            Some(DiskInfo {
                mount,
                used_gb: (used / 1024.0 / 1024.0 / 1024.0) as f32,
                total_gb: (total / 1024.0 / 1024.0 / 1024.0) as f32,
                usage_percent: (100.0 * used / total) as f32,
            })
        })
        .collect()
}

fn parse_network(section: &str) -> Option<NetworkInfo> {
    // /proc/net/dev: Interface: RXbytes ... TXbytes ...
    let mut rx_total: u64 = 0;
    let mut tx_total: u64 = 0;
    let mut ip: Option<String> = None;

    for line in section.lines() {
        let mut parts = line.split_whitespace();
        let iface = parts.next()?.trim_end_matches(':').to_string();
        if iface == "lo" {
            continue;
        }
        let rx: u64 = parts.next()?.parse().unwrap_or(0);
        // 跳过 14 个字段
        let _ = parts.nth(13);
        let tx: u64 = parts.next()?.parse().unwrap_or(0);
        rx_total += rx;
        tx_total += tx;
        if ip.is_none() {
            ip = Some(iface);
        }
    }

    Some(NetworkInfo {
        // 首版为累计字节数转 MB/s 需两次采样，这里存累计值除以采样间隔的逻辑在 scheduler 处理；
        // 简化：返回 0 速率占位，IP 用主网卡名占位
        rx_mbps: rx_total as f32 / 1024.0 / 1024.0,
        tx_mbps: tx_total as f32 / 1024.0 / 1024.0,
        ip,
    })
}

fn parse_system(section: &str) -> Option<SystemInfo> {
    let mut lines = section.lines();
    let os = lines.next()?.trim().to_string();
    let hostname = lines.next()?.trim().to_string();
    let uptime: f64 = lines.next()?.trim().parse().ok()?;
    Some(SystemInfo {
        os,
        hostname,
        uptime_secs: uptime as u64,
    })
}

fn parse_processes(section: &str) -> Vec<ProcessInfo> {
    section
        .lines()
        .filter_map(|line| {
            let mut parts = line.split_whitespace();
            let pid: u32 = parts.next()?.parse().ok()?;
            let name = parts.next()?.to_string();
            let cpu: f32 = parts.next()?.parse().ok()?;
            let rss_kb: f32 = parts.next()?.parse().ok()?;
            Some(ProcessInfo {
                pid,
                name,
                cpu_percent: cpu,
                mem_mb: rss_kb / 1024.0,
            })
        })
        .collect()
}

// ── 单元测试 ──────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn fake_output() -> ExecOutput {
        let stdout = r#"===CPU===
cpu  100000 5000 20000 800000
===LOAD===
1.50 1.10 0.90 3/123 45678
===MEM===
Mem: 34359738368 19928648253 14431090115 0 104857600 34359738368
===DISK===
/ 118111600640 274877906944
/data 612032839680 2199023255552
===NET===
eth0: 1024000 100 0 0 0 0 0 0 204800 50 0 0 0 0 0 0
===SYS===
Linux 6.6.1 (x86_64)
web01
1036800
===PROC===
1234 sshd 0.2 12288
2345 node 1.1 245760
===SERVICES===
ssh.service|loaded|active|running
cron.service|loaded|active|running
nginx.service|loaded|inactive|dead
===CONNS===
LISTEN|5
ESTAB|23
TIME-WAIT|12
CLOSE-WAIT|3
===FILES===
total 8
drwxr-xr-x 2 user group 4096 2026-08-13 16:00 .
drwxr-xr-x 4 user group 4096 2026-08-12 10:00 ..
-rw-r--r-- 1 user group 1234 2026-08-13 16:00 app.log
drwxr-xr-x 2 user group 4096 2026-08-13 15:00 logs
===LOGS===
2026-08-13T16:00:00.123456+0800 web01 sshd[1234]: Server listening on 0.0.0.0 port 22
2026-08-13T16:00:01.000001+0800 web01 systemd[1]: Started Session 42 of user root.
"#;
        ExecOutput {
            stdout: stdout.to_string(),
            stderr: String::new(),
            exit_code: Some(0),
        }
    }

    #[test]
    fn parses_all_sections() {
        let mut snap = ServerSnapshot::new("web01");
        parse_linux_output(&fake_output(), &mut snap);

        assert!(snap.cpu.is_some());
        assert!(snap.memory.is_some());
        assert_eq!(snap.disks.len(), 2);
        assert!(snap.network.is_some());
        assert!(snap.system.is_some());
        assert_eq!(snap.processes.len(), 2);
        assert_eq!(snap.services.len(), 3);
        assert!(snap.services[0].is_running());
        assert_eq!(snap.services[2].active, "inactive");
        let conns = snap.conn_counts.as_ref().expect("conn_counts present");
        assert_eq!(conns.listening, 5);
        assert_eq!(conns.established, 23);
        assert_eq!(conns.time_wait, 12);
        assert_eq!(conns.close_wait, 3);
        assert_eq!(conns.total, 43);
        assert_eq!(snap.files.len(), 4);
        assert!(snap.files.iter().any(|f| f.name == "app.log"));
        assert!(snap.files.iter().any(|f| f.is_dir && f.name == "logs"));
        assert_eq!(snap.logs.len(), 2);
        assert_eq!(snap.logs[0].unit, "sshd[1234]");
        assert!(snap.logs[1].message.contains("Session 42"));
    }

    #[test]
    fn parses_memory_gb() {
        let mut snap = ServerSnapshot::new("x");
        parse_linux_output(&fake_output(), &mut snap);
        let mem = snap.memory.unwrap();
        // 34359738368 B = 32 GB
        assert!((mem.total_gb - 32.0).abs() < 0.01);
    }

    #[test]
    fn parses_load() {
        let mut snap = ServerSnapshot::new("x");
        parse_linux_output(&fake_output(), &mut snap);
        let cpu = snap.cpu.unwrap();
        assert!((cpu.load.0 - 1.5).abs() < 0.01);
        assert!((cpu.load.1 - 1.1).abs() < 0.01);
    }

    #[test]
    fn split_sections_basic() {
        let map = split_sections("===A===\nx\n===B===\ny\n");
        assert_eq!(map.get("A").map(|s| s.trim()), Some("x"));
        assert_eq!(map.get("B").map(|s| s.trim()), Some("y"));
    }
}
