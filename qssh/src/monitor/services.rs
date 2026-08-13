//! 系统服务数据模型 + `systemctl` 输出解析
//!
//! Phase 4：Services 模块。
//! - 远程命令：`systemctl list-units --type=service --all ...`（`awk` 提取 4 列，`|` 分隔）
//! - 解析为 [`ServiceInfo`] 列表
//! - 危险操作（start/stop/restart）由 UI 层确认后经 [`RemoteExecutor`] 执行

use super::executor::RemoteExecutor;

/// 系统服务摘要（与 `systemctl` 一行的 UNIT/LOAD/ACTIVE/SUB 对应）
#[derive(Debug, Clone, PartialEq)]
pub struct ServiceInfo {
    /// 服务单元名（如 "cron.service"）
    pub name: String,
    /// load 状态（通常为 "loaded"）
    pub load: String,
    /// active 状态（active / inactive / failed）
    pub active: String,
    /// sub 状态（running / exited / dead / failed）
    pub sub: String,
}

impl ServiceInfo {
    /// 是否处于运行状态（active + running）
    pub fn is_running(&self) -> bool {
        self.active == "active" && self.sub == "running"
    }
}

/// 从 `systemctl list-units` 的 `|` 分隔输出解析服务列表。
///
/// 期望命令：
/// `systemctl list-units --type=service --all --no-pager --no-legend --plain | awk '{print $1"|"$2"|"$3"|"$4}'`
pub fn parse_systemctl(text: &str) -> Vec<ServiceInfo> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .filter_map(parse_service_line)
        .collect()
}

fn parse_service_line(line: &str) -> Option<ServiceInfo> {
    let mut parts = line.split('|');
    let name = parts.next()?.trim().to_string();
    if name.is_empty() || !name.ends_with(".service") {
        return None;
    }
    let load = parts.next().unwrap_or_default().trim().to_string();
    let active = parts.next().unwrap_or_default().trim().to_string();
    let sub = parts.next().unwrap_or_default().trim().to_string();
    Some(ServiceInfo {
        name,
        load,
        active,
        sub,
    })
}

/// 构建 `systemctl list-units` 采集命令（结构化、`|` 分隔、无分页）
pub fn systemctl_units_command() -> &'static str {
    "systemctl list-units --type=service --all --no-pager --no-legend --plain | awk '{print $1\"|\"$2\"|\"$3\"|\"$4}'"
}

/// 通过远程执行器采集服务列表；失败返回空列表（由调用方决定如何降级）
///
/// 预留的独立采集入口（当前由 Collector 全量快照统一采集）。
#[allow(dead_code)]
pub fn collect_services(executor: &dyn RemoteExecutor) -> Vec<ServiceInfo> {
    match executor.exec(systemctl_units_command()) {
        Ok(out) => parse_systemctl(&out.stdout),
        Err(_) => Vec::new(),
    }
}

/// 服务危险操作（UI 确认后执行）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceAction {
    /// 启动服务
    Start,
    /// 停止服务
    Stop,
    /// 重启服务
    Restart,
}

impl ServiceAction {
    /// 对应 `systemctl` 子命令
    pub fn subcommand(self) -> &'static str {
        match self {
            ServiceAction::Start => "start",
            ServiceAction::Stop => "stop",
            ServiceAction::Restart => "restart",
        }
    }

    /// 中文提示语
    pub fn label(self) -> &'static str {
        match self {
            ServiceAction::Start => "启动",
            ServiceAction::Stop => "停止",
            ServiceAction::Restart => "重启",
        }
    }
}

/// 生成危险操作命令（start / stop / restart），由 UI 确认后执行
pub fn service_action_command(action: ServiceAction, name: &str) -> String {
    format!("systemctl {} {}", action.subcommand(), name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_single_service_line() {
        let out = "cron.service|loaded|active|running";
        let services = parse_systemctl(out);
        assert_eq!(services.len(), 1);
        let s = &services[0];
        assert_eq!(s.name, "cron.service");
        assert_eq!(s.load, "loaded");
        assert_eq!(s.active, "active");
        assert_eq!(s.sub, "running");
        assert!(s.is_running());
    }

    #[test]
    fn parses_multiple_states() {
        let out = "ssh.service|loaded|active|running\n\
                   docker.service|loaded|active|exited\n\
                   nginx.service|loaded|inactive|dead\n\
                   mysvc.service|loaded|failed|failed";
        let services = parse_systemctl(out);
        assert_eq!(services.len(), 4);
        assert!(services[0].is_running());
        assert!(!services[1].is_running());
        assert!(!services[2].is_running());
        assert!(!services[3].is_running());
        assert_eq!(services[3].active, "failed");
    }

    #[test]
    fn skips_non_service_units() {
        let out = "dev-sda1.device|loaded|active|plugged\n\
                   home.mount|loaded|active|mounted\n\
                   cron.service|loaded|active|running";
        let services = parse_systemctl(out);
        assert_eq!(services.len(), 1);
        assert_eq!(services[0].name, "cron.service");
    }

    #[test]
    fn empty_input_yields_empty_list() {
        assert!(parse_systemctl("").is_empty());
        assert!(parse_systemctl("   \n\n  ").is_empty());
    }

    #[test]
    fn action_command_format() {
        assert_eq!(
            service_action_command(ServiceAction::Start, "nginx.service"),
            "systemctl start nginx.service"
        );
        assert_eq!(
            service_action_command(ServiceAction::Stop, "nginx.service"),
            "systemctl stop nginx.service"
        );
        assert_eq!(
            service_action_command(ServiceAction::Restart, "nginx.service"),
            "systemctl restart nginx.service"
        );
    }
}
