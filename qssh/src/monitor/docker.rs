//! Docker 容器数据模型 + `docker ps` 输出解析
//!
//! Phase 3：Docker 模块。
//! - 远程命令：`docker ps -a --format ...`（结构化输出，`|` 分隔）
//! - 解析为 [`DockerContainer`] 列表
//! - 危险操作（restart/stop/delete）由 UI 层确认后经 [`RemoteExecutor`] 执行

use super::executor::RemoteExecutor;

/// Docker 容器摘要（与 `docker ps` 一行的字段对应）
#[derive(Debug, Clone, PartialEq)]
pub struct DockerContainer {
    /// 容器 ID（短 ID）
    pub id: String,
    /// 容器名称
    pub name: String,
    /// 镜像名
    pub image: String,
    /// 状态描述（如 "Up 3 hours" / "Exited (0) 2 days ago"）
    pub status: String,
    /// 是否运行中（状态以 "Up" 开头）
    pub running: bool,
    /// 端口映射摘要
    pub ports: String,
}

/// 从 `docker ps -a` 的 `|` 分隔输出解析容器列表。
///
/// 期望命令：
/// `docker ps -a --format '{{.ID}}|{{.Names}}|{{.Image}}|{{.Status}}|{{.Ports}}'`
pub fn parse_docker_ps(text: &str) -> Vec<DockerContainer> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .filter_map(parse_docker_line)
        .collect()
}

fn parse_docker_line(line: &str) -> Option<DockerContainer> {
    let mut parts = line.split('|');
    let id = parts.next()?.trim().to_string();
    if id.is_empty() || id == "CONTAINER ID" {
        return None;
    }
    let name = parts.next().unwrap_or_default().trim().to_string();
    let image = parts.next().unwrap_or_default().trim().to_string();
    let status = parts.next().unwrap_or_default().trim().to_string();
    let ports = parts.next().unwrap_or_default().trim().to_string();
    let running = status.starts_with("Up");
    Some(DockerContainer {
        id,
        name,
        image,
        status,
        running,
        ports,
    })
}

/// 构建 `docker ps -a` 采集命令（结构化、无截断）
///
/// 目前由 [`LINUX_COLLECT_ALL`](crate::monitor::platform::LINUX_COLLECT_ALL)
/// 内嵌相同命令并直接走 `parse_docker_ps`；此函数为后续独立采集路径保留。
#[allow(dead_code)]
pub fn docker_ps_command() -> &'static str {
    "docker ps -a --format '{{.ID}}|{{.Names}}|{{.Image}}|{{.Status}}|{{.Ports}}'"
}

/// 通过远程执行器采集容器列表；失败返回空列表（由调用方决定如何降级）。
///
/// 预留的独立采集入口（当前由 Collector 全量快照统一采集）。
#[allow(dead_code)]
pub fn collect_containers(executor: &dyn RemoteExecutor) -> Vec<DockerContainer> {
    match executor.exec(docker_ps_command()) {
        Ok(out) => parse_docker_ps(&out.stdout),
        Err(_) => Vec::new(),
    }
}

/// 容器危险操作（UI 确认后执行）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockerAction {
    /// 重启容器
    Restart,
    /// 停止容器
    Stop,
    /// 删除容器（rm -f）
    Delete,
}

impl DockerAction {
    /// 对应 `docker` 子命令
    pub fn subcommand(self) -> &'static str {
        match self {
            DockerAction::Restart => "restart",
            DockerAction::Stop => "stop",
            DockerAction::Delete => "rm -f",
        }
    }

    /// 中文提示语
    pub fn label(self) -> &'static str {
        match self {
            DockerAction::Restart => "重启",
            DockerAction::Stop => "停止",
            DockerAction::Delete => "删除",
        }
    }
}

/// 生成危险操作命令（restart / stop / rm），由 UI 确认后执行
pub fn container_action_command(action: DockerAction, id: &str) -> String {
    format!("docker {} {}", action.subcommand(), id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_single_container_line() {
        let out = "abc123def456|web|nginx:latest|Up 3 hours|0.0.0.0:8080->80/tcp";
        let containers = parse_docker_ps(out);
        assert_eq!(containers.len(), 1);
        let c = &containers[0];
        assert_eq!(c.id, "abc123def456");
        assert_eq!(c.name, "web");
        assert_eq!(c.image, "nginx:latest");
        assert_eq!(c.status, "Up 3 hours");
        assert!(c.running);
        assert_eq!(c.ports, "0.0.0.0:8080->80/tcp");
    }

    #[test]
    fn parses_header_and_stopped_container() {
        let out = "CONTAINER ID|NAMES|IMAGE|STATUS|PORTS\n\
                   def456|old|ubuntu:22.04|Exited (0) 2 days ago|";
        let containers = parse_docker_ps(out);
        assert_eq!(containers.len(), 1);
        assert!(!containers[0].running);
    }

    #[test]
    fn empty_input_yields_empty_list() {
        assert!(parse_docker_ps("").is_empty());
        assert!(parse_docker_ps("   \n \n").is_empty());
    }

    #[test]
    fn action_command_format() {
        assert_eq!(
            container_action_command(DockerAction::Restart, "abc"),
            "docker restart abc"
        );
        assert_eq!(
            container_action_command(DockerAction::Delete, "abc"),
            "docker rm -f abc"
        );
    }

    #[test]
    fn collect_containers_gracefully_degrades_on_error() {
        use super::super::executor::{ExecError, ExecOutput};

        struct FailingExecutor;
        impl RemoteExecutor for FailingExecutor {
            fn exec(&self, _command: &str) -> Result<ExecOutput, ExecError> {
                Err(ExecError::NotFound)
            }
        }

        let containers = collect_containers(&FailingExecutor);
        assert!(containers.is_empty());
    }
}
