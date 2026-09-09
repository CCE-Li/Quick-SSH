//! Agent 工具注册表（Tool Registry）
//!
//! Phase 8：每个 Tool 映射到 `qssh-core` 的 [`RemoteExecutor`] 调用，
//! 附带危险等级供 [`PermissionLevel`] 决策。
//!
//! 设计约束：Agent 只允许调用 Tool，不直接持有 SSH 连接。

use crate::monitor::docker::{container_action_command, DockerAction};
use crate::monitor::executor::RemoteExecutor;
use crate::monitor::files::ls_l_command;
use crate::monitor::logs::journalctl_command;
use crate::monitor::services::{service_action_command, ServiceAction};

use super::permissions::DangerLevel;

/// 工具标识
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ToolId {
    ServerStatus,
    Processes,
    Disks,
    Network,
    DockerList,
    DockerRestart,
    DockerStop,
    DockerDelete,
    ServiceStart,
    ServiceStop,
    ServiceRestart,
    FilesList,
    LogsTail,
    /// 任意 Shell 命令执行（指令控制服务器核心）
    ShellRun,
    /// 创建文件夹
    FilesMkdir,
    /// 写入文件内容
    FilesWrite,
    /// 删除文件或目录
    FilesRemove,
}

/// 全部工具标识（注册表顺序）
pub const ALL_TOOLS: [ToolId; 17] = [
    ToolId::ServerStatus,
    ToolId::Processes,
    ToolId::Disks,
    ToolId::Network,
    ToolId::DockerList,
    ToolId::DockerRestart,
    ToolId::DockerStop,
    ToolId::DockerDelete,
    ToolId::ServiceStart,
    ToolId::ServiceStop,
    ToolId::ServiceRestart,
    ToolId::FilesList,
    ToolId::LogsTail,
    ToolId::ShellRun,
    ToolId::FilesMkdir,
    ToolId::FilesWrite,
    ToolId::FilesRemove,
];

impl ToolId {
    /// 工具名称（LLM 调用名 + 时间线显示）
    pub fn name(self) -> &'static str {
        match self {
            ToolId::ServerStatus => "server.status",
            ToolId::Processes => "server.processes",
            ToolId::Disks => "server.disks",
            ToolId::Network => "server.network",
            ToolId::DockerList => "docker.list",
            ToolId::DockerRestart => "docker.restart",
            ToolId::DockerStop => "docker.stop",
            ToolId::DockerDelete => "docker.delete",
            ToolId::ServiceStart => "service.start",
            ToolId::ServiceStop => "service.stop",
            ToolId::ServiceRestart => "service.restart",
            ToolId::FilesList => "files.list",
            ToolId::LogsTail => "logs.tail",
            ToolId::ShellRun => "shell.run",
            ToolId::FilesMkdir => "files.mkdir",
            ToolId::FilesWrite => "files.write",
            ToolId::FilesRemove => "files.remove",
        }
    }

    /// 中文描述（时间线 / 面板展示）
    pub fn description(self) -> &'static str {
        match self {
            ToolId::ServerStatus => "服务器状态（CPU/内存/负载）",
            ToolId::Processes => "进程列表",
            ToolId::Disks => "磁盘使用情况",
            ToolId::Network => "网络与 TCP 连接统计",
            ToolId::DockerList => "Docker 容器列表",
            ToolId::DockerRestart => "重启 Docker 容器",
            ToolId::DockerStop => "停止 Docker 容器",
            ToolId::DockerDelete => "删除 Docker 容器（rm -f）",
            ToolId::ServiceStart => "启动系统服务",
            ToolId::ServiceStop => "停止系统服务",
            ToolId::ServiceRestart => "重启系统服务",
            ToolId::FilesList => "远程文件列表",
            ToolId::LogsTail => "查看系统日志（journalctl）",
            ToolId::ShellRun => "执行任意 Shell 命令（指令控制服务器）",
            ToolId::FilesMkdir => "创建文件夹（mkdir -p）",
            ToolId::FilesWrite => "写入文件内容",
            ToolId::FilesRemove => "删除文件或目录（rm -rf）",
        }
    }

    /// 危险等级（权限决策依据）
    pub fn danger(self) -> DangerLevel {
        match self {
            ToolId::ServerStatus
            | ToolId::Processes
            | ToolId::Disks
            | ToolId::Network
            | ToolId::DockerList
            | ToolId::FilesList
            | ToolId::LogsTail => DangerLevel::Read,
            ToolId::ServiceStart
            | ToolId::DockerRestart
            | ToolId::FilesMkdir
            | ToolId::FilesWrite => DangerLevel::SafeWrite,
            ToolId::DockerStop
            | ToolId::DockerDelete
            | ToolId::ServiceStop
            | ToolId::ServiceRestart
            | ToolId::ShellRun
            | ToolId::FilesRemove => DangerLevel::Dangerous,
        }
    }
}

/// 工具参数（解析后的调用参数）
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ToolArgs {
    /// 服务名 / 容器 ID / 路径等目标
    pub target: String,
    /// 日志行数（仅 logs.tail）
    pub lines: Option<usize>,
    /// 要执行的 Shell 命令（仅 shell.run）
    pub command: Option<String>,
    /// 要写入的文件内容（仅 files.write）
    pub content: Option<String>,
}

/// 工具调用请求（LLM 返回 / 用户手动构造）
#[derive(Debug, Clone, PartialEq)]
pub struct ToolCall {
    pub tool: ToolId,
    pub args: ToolArgs,
}

impl ToolCall {
    /// 是否只读
    pub fn is_read_only(&self) -> bool {
        self.tool.danger().is_read()
    }

    /// 时间线显示名称（如 `server.status`）
    pub fn name(&self) -> &'static str {
        self.tool.name()
    }
}

/// 工具执行结果
#[derive(Debug, Clone)]
pub struct ToolResult {
    /// 工具标识（元数据，供调用方回显）
    #[allow(dead_code)]
    pub tool: ToolId,
    /// 执行是否成功（退出码 0）
    pub ok: bool,
    /// 输出内容（截断到 [`MAX_TOOL_OUTPUT`]）
    pub output: String,
}

/// 工具输出最大长度（避免塞爆上下文 / UI）
pub const MAX_TOOL_OUTPUT: usize = 2000;

/// 从文本中解析工具调用
///
/// 支持两种格式：
/// - `tool:name arg1=value`
/// - `tool(name, arg=value, ...)`（LLM function-call 风格，宽松解析）
///
/// 未知工具名返回 `None`。
pub fn parse_tool_call(text: &str) -> Option<ToolCall> {
    let trimmed = text.trim();
    let (name_part, args_part) = if let Some(stripped) = trimmed.strip_prefix("tool:") {
        split_on_separator(stripped, |c| c == ' ')
    } else if let Some(open) = trimmed.find('(') {
        let close = trimmed.rfind(')')?;
        (trimmed[..open].trim(), trimmed[open + 1..close].trim())
    } else {
        return None;
    };

    let tool = match_tool_name(name_part)?;
    let mut args = ToolArgs::default();

    if !args_part.is_empty() {
        for pair in args_part.split(',') {
            let pair = pair.trim();
            if let Some((key, value)) = pair.split_once('=') {
                let key = key.trim().trim_start_matches("arg_").trim();
                let value = value.trim().trim_matches('"').trim_matches('\'');
                match key {
                    "target" | "id" | "name" | "service" | "path" | "container" | "目标"
                    | "目录" | "路径" | "文件夹" => {
                        args.target = value.to_string();
                    }
                    "lines" | "n" | "tail" => {
                        args.lines = value.parse::<usize>().ok();
                    }
                    "command" | "cmd" | "命令" | "指令" => {
                        args.command = Some(value.to_string());
                    }
                    "content" | "text" | "内容" | "文本" => {
                        args.content = Some(value.to_string());
                    }
                    _ => {}
                }
            } else {
                // 无 key 的第一个值视为 target；shell.run 视为命令
                let val = pair.trim_matches('"').trim_matches('\'').to_string();
                if !val.is_empty() {
                    if tool == ToolId::ShellRun {
                        args.command = Some(val);
                    } else if args.target.is_empty() {
                        args.target = val;
                    }
                }
            }
        }
    }

    Some(ToolCall { tool, args })
}

fn split_on_separator(text: &str, is_sep: impl Fn(char) -> bool) -> (&str, &str) {
    match text.find(is_sep) {
        Some(idx) => (&text[..idx], text[idx..].trim()),
        None => (text, ""),
    }
}

fn match_tool_name(name: &str) -> Option<ToolId> {
    let name = name.trim().to_ascii_lowercase();
    // 允许 "server.status" 或 "status" 等简写
    let normalized = name.replace('_', ".");
    let short = normalized.rsplit('.').next().unwrap_or("");
    let all = [
        ToolId::ServerStatus,
        ToolId::Processes,
        ToolId::Disks,
        ToolId::Network,
        ToolId::DockerList,
        ToolId::DockerRestart,
        ToolId::DockerStop,
        ToolId::DockerDelete,
        ToolId::ServiceStart,
        ToolId::ServiceStop,
        ToolId::ServiceRestart,
        ToolId::FilesList,
        ToolId::LogsTail,
        ToolId::ShellRun,
        ToolId::FilesMkdir,
        ToolId::FilesWrite,
        ToolId::FilesRemove,
    ];
    // 1) 精确匹配完整名称（消除 docker.restart / service.restart 等歧义）
    if let Some(id) = all.iter().copied().find(|id| id.name() == normalized) {
        return Some(id);
    }
    // 2) 短名后缀匹配（如 "restart"、"status"）
    for id in all {
        if id.name().ends_with(&format!(".{}", short)) {
            return Some(id);
        }
    }
    None
}

/// 执行一个工具调用，返回结果（错误归入 `output`，不 panic）
pub fn execute_tool(executor: &dyn RemoteExecutor, call: &ToolCall) -> ToolResult {
    let tool = call.tool;
    let result = match tool {
        ToolId::ServerStatus => executor.exec(crate::monitor::platform::server_status_command()),
        ToolId::Processes => executor.exec(crate::monitor::platform::processes_command()),
        ToolId::Disks => executor.exec(crate::monitor::platform::disks_command()),
        ToolId::Network => executor.exec(crate::monitor::platform::network_command()),
        ToolId::DockerList => executor.exec(crate::monitor::docker::docker_ps_command()),
        ToolId::DockerRestart => executor.exec(&container_action_command(
            DockerAction::Restart,
            &call.args.target,
        )),
        ToolId::DockerStop => executor.exec(&container_action_command(
            DockerAction::Stop,
            &call.args.target,
        )),
        ToolId::DockerDelete => executor.exec(&container_action_command(
            DockerAction::Delete,
            &call.args.target,
        )),
        ToolId::ServiceStart => executor.exec(&service_action_command(
            ServiceAction::Start,
            &call.args.target,
        )),
        ToolId::ServiceStop => executor.exec(&service_action_command(
            ServiceAction::Stop,
            &call.args.target,
        )),
        ToolId::ServiceRestart => executor.exec(&service_action_command(
            ServiceAction::Restart,
            &call.args.target,
        )),
        ToolId::FilesList => executor.exec(&ls_l_command(&call.args.target)),
        ToolId::LogsTail => {
            let lines = call.args.lines.unwrap_or(50).min(200);
            executor.exec(&journalctl_command(None, lines))
        }
        ToolId::ShellRun => {
            let command = call
                .args
                .command
                .clone()
                .or_else(|| (!call.args.target.is_empty()).then(|| call.args.target.clone()))
                .unwrap_or_default();
            executor.exec(&command)
        }
        ToolId::FilesMkdir => {
            let path = &call.args.target;
            executor.exec(&format!(
                "mkdir -p {}",
                crate::monitor::files::shell_quote(path)
            ))
        }
        ToolId::FilesWrite => {
            let path = &call.args.target;
            let content = call.args.content.clone().unwrap_or_default();
            // 用 printf 写入，避免 heredoc 特殊字符问题；内容经单引号转义
            executor.exec(&format!(
                "printf '%s' {} > {}",
                crate::monitor::files::shell_quote(&content),
                crate::monitor::files::shell_quote(path)
            ))
        }
        ToolId::FilesRemove => {
            let path = &call.args.target;
            executor.exec(&format!(
                "rm -rf -- {}",
                crate::monitor::files::shell_quote(path)
            ))
        }
    };

    match result {
        Ok(output) => {
            let trimmed = truncate(&output.stdout, MAX_TOOL_OUTPUT);
            ToolResult {
                tool,
                ok: true,
                output: trimmed,
            }
        }
        Err(error) => ToolResult {
            tool,
            ok: false,
            output: error.to_string(),
        },
    }
}

/// 截断字符串到最大长度（保留尾部，避免截断关键信息）
pub fn truncate(text: &str, max: usize) -> String {
    if text.len() <= max {
        return text.to_string();
    }
    let start = text.len() - max;
    let mut cut = text[start..].to_string();
    cut.insert_str(0, "\n… (已截断) …\n");
    cut
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_tool_call_plain_syntax() {
        let call = parse_tool_call("tool:server.status").expect("parse");
        assert_eq!(call.tool, ToolId::ServerStatus);
        assert_eq!(call.args.target, "");
    }

    #[test]
    fn parse_shell_run_command() {
        let call = parse_tool_call("tool:shell.run 命令=mkdir -p /tmp/demo").expect("parse");
        assert_eq!(call.tool, ToolId::ShellRun);
        assert_eq!(call.args.command.as_deref(), Some("mkdir -p /tmp/demo"));
        // 无 key 形式：裸命令视为 command
        let call2 = parse_tool_call("tool:shell.run ls -la /var").expect("parse");
        assert_eq!(call2.tool, ToolId::ShellRun);
        assert_eq!(call2.args.command.as_deref(), Some("ls -la /var"));
    }

    #[test]
    fn parse_files_mkdir_and_write() {
        let call = parse_tool_call("tool:files.mkdir 目录=/srv/app").expect("parse");
        assert_eq!(call.tool, ToolId::FilesMkdir);
        assert_eq!(call.args.target, "/srv/app");

        let call =
            parse_tool_call("tool:files.write 目标=/etc/hostname, 内容=myhost").expect("parse");
        assert_eq!(call.tool, ToolId::FilesWrite);
        assert_eq!(call.args.target, "/etc/hostname");
        assert_eq!(call.args.content.as_deref(), Some("myhost"));
    }

    #[test]
    fn execute_file_tools_build_commands() {
        use crate::monitor::executor::{ExecError, ExecOutput};
        struct Recording(std::sync::Mutex<Vec<String>>);
        impl RemoteExecutor for Recording {
            fn exec(&self, command: &str) -> Result<ExecOutput, ExecError> {
                self.0.lock().unwrap().push(command.to_string());
                Ok(ExecOutput {
                    exit_code: Some(0),
                    stdout: String::new(),
                    stderr: String::new(),
                })
            }
        }

        let cmds = std::sync::Arc::new(Recording(std::sync::Mutex::new(Vec::new())));

        let mkdir = ToolCall {
            tool: ToolId::FilesMkdir,
            args: ToolArgs {
                target: "/tmp/demo".into(),
                ..Default::default()
            },
        };
        execute_tool(&*cmds, &mkdir);
        let write = ToolCall {
            tool: ToolId::FilesWrite,
            args: ToolArgs {
                target: "/tmp/demo/app.conf".into(),
                content: Some("a=1\nb=2".into()),
                ..Default::default()
            },
        };
        execute_tool(&*cmds, &write);
        let remove = ToolCall {
            tool: ToolId::FilesRemove,
            args: ToolArgs {
                target: "/tmp/demo".into(),
                ..Default::default()
            },
        };
        execute_tool(&*cmds, &remove);

        let recorded = cmds.0.lock().unwrap().clone();
        assert!(recorded.iter().any(|c| c.starts_with("mkdir -p")));
        assert!(recorded.iter().any(|c| c.starts_with("printf")));
        assert!(recorded.iter().any(|c| c.starts_with("rm -rf")));
    }

    #[test]
    fn new_tools_danger_levels() {
        assert_eq!(ToolId::FilesMkdir.danger(), DangerLevel::SafeWrite);
        assert_eq!(ToolId::FilesWrite.danger(), DangerLevel::SafeWrite);
        assert_eq!(ToolId::FilesRemove.danger(), DangerLevel::Dangerous);
        assert_eq!(ToolId::ShellRun.danger(), DangerLevel::Dangerous);
        // 只读权限应拒绝 shell.run / files.remove
        use crate::agent::permissions::PermissionLevel;
        assert_eq!(
            PermissionLevel::ReadOnly.decide(ToolId::ShellRun.danger()),
            crate::agent::permissions::Approval::Denied
        );
    }

    #[test]
    fn parse_tool_call_with_target() {
        let call = parse_tool_call("tool:service.restart name=cron").expect("parse");
        assert_eq!(call.tool, ToolId::ServiceRestart);
        assert_eq!(call.args.target, "cron");
    }

    #[test]
    fn parse_tool_call_llm_style() {
        let call = parse_tool_call("docker.restart(container=web, )").expect("parse");
        assert_eq!(call.tool, ToolId::DockerRestart);
        assert_eq!(call.args.target, "web");
    }

    #[test]
    fn parse_tool_call_with_lines() {
        let call = parse_tool_call("tool:logs.tail n=100").expect("parse");
        assert_eq!(call.tool, ToolId::LogsTail);
        assert_eq!(call.args.lines, Some(100));
    }

    #[test]
    fn parse_unknown_tool_returns_none() {
        assert!(parse_tool_call("tool:unknown.tool").is_none());
        assert!(parse_tool_call("not a tool call").is_none());
    }

    #[test]
    fn tool_danger_levels() {
        assert!(ToolId::ServerStatus.danger().is_read());
        assert!(ToolId::LogsTail.danger().is_read());
        assert_eq!(ToolId::DockerDelete.danger(), DangerLevel::Dangerous);
        assert_eq!(ToolId::ServiceStart.danger(), DangerLevel::SafeWrite);
    }

    #[test]
    fn execute_read_tool_graceful_on_failure() {
        use crate::monitor::executor::{ExecError, ExecOutput};
        struct Failing;
        impl RemoteExecutor for Failing {
            fn exec(&self, _command: &str) -> Result<ExecOutput, ExecError> {
                Err(ExecError::Other("boom".to_string()))
            }
        }
        let call = ToolCall {
            tool: ToolId::ServerStatus,
            args: ToolArgs::default(),
        };
        let result = execute_tool(&Failing, &call);
        assert!(!result.ok);
        assert!(result.output.contains("boom"));
    }

    #[test]
    fn truncate_keeps_tail_and_marks() {
        let text = "x".repeat(3000);
        let truncated = truncate(&text, 100);
        assert!(truncated.contains("已截断"));
        assert!(truncated.ends_with("x"));
        assert!(truncated.len() < 300);
    }
}
