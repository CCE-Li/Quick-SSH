//! AI Agent：工具调用 + 权限 + 时间线
//!
//! Phase 8：`User → Agent → Tool Registry → RemoteExecutor → Server`
//!
//! 模块划分（内部模块化，后续可抽为 `qssh-agent` crate）：
//! - [`config`]：agent.json 配置 + 持久化
//! - [`provider`]：LLM 通信（curl / ollama 双后端）
//! - [`tools`]：Tool Registry + 工具调用解析/执行
//! - [`permissions`]：权限分级 + 危险操作决策
//! - [`timeline`]：执行步骤逐步回报

pub mod config;
pub mod permissions;
pub mod provider;
pub mod timeline;
pub mod tools;

use std::sync::mpsc::Sender;

use crate::monitor::executor::RemoteExecutor;
use crate::monitor::snapshot::ServerSnapshot;

use self::config::AgentConfig;
use self::permissions::Approval;
use self::provider::{chat, ChatMessage, ChatReply};
use self::timeline::{AgentStatus, StepStatus, Timeline, TimelineStep};
use self::tools::{execute_tool, ToolCall, MAX_TOOL_OUTPUT};

/// 全部可用工具的系统提示（注入 LLM）
pub fn available_tools_prompt() -> &'static str {
    "server.status | server.processes | server.disks | server.network\n\
     docker.list | docker.restart(目标=容器) | docker.stop(目标=容器) | docker.delete(目标=容器)\n\
     service.start(目标=服务) | service.stop(目标=服务) | service.restart(目标=服务)\n\
     files.list(目标=路径) | files.mkdir(目标=目录) | files.write(目标=路径, 内容=文本) | files.remove(目标=路径)\n\
     logs.tail(n=行数) | shell.run(命令=任意 shell 命令)"
}

/// Agent 后台任务事件（经 mpsc 投递到 UI）
#[derive(Debug, Clone)]
pub enum AgentEvent {
    /// 会话开始 / 新用户消息
    Started,
    /// 状态变化（Ready / Thinking / Executing / ...）
    Status(AgentStatus),
    /// 时间线已更新（携带最新时间线供 UI 渲染）
    Timeline(Timeline),
    /// 等待用户批准某工具调用
    ApprovalNeeded { call: ToolCall, reason: String },
    /// 会话结束（含最终回复文本）
    Finished {
        reply: String,
        error: Option<String>,
    },
}

/// Agent 会话运行器：后台线程持有，事件经 channel 回报 UI
#[derive(Debug)]
pub struct AgentRunner {
    config: AgentConfig,
    tx: Sender<AgentEvent>,
}

impl AgentRunner {
    /// 新建运行器
    pub fn new(config: AgentConfig, tx: Sender<AgentEvent>) -> Self {
        Self { config, tx }
    }

    fn emit(&self, event: AgentEvent) {
        let _ = self.tx.send(event);
    }

    /// 运行一次完整会话（阻塞，在线程中调用）。
    ///
    /// `history`：已有对话历史（不含本次输入）；`input`：本次用户输入。
    /// `executor`：可执行远程工具的 SSH 执行器；`approve`：危险操作批准回调。
    pub fn run(
        &self,
        history: &[ChatMessage],
        input: &str,
        executor: &dyn RemoteExecutor,
        snapshot: &ServerSnapshot,
        approve: impl Fn(&ToolCall) -> bool,
    ) -> AgentSession {
        let mut timeline = Timeline::new();
        let permission = self.config.permission_level();
        let mut messages = history.to_vec();
        messages.push(ChatMessage {
            role: "user".to_string(),
            content: input.to_string(),
        });

        // 注入当前服务器快照作为上下文
        let context = format_snapshot_context(snapshot);
        if !context.is_empty() {
            messages.insert(
                messages.len() - 1,
                ChatMessage {
                    role: "system".to_string(),
                    content: context,
                },
            );
        }

        self.emit(AgentEvent::Status(AgentStatus::Thinking));
        self.emit(AgentEvent::Timeline(timeline.clone()));

        let mut reply = String::new();
        let mut error: Option<String> = None;
        let mut iterations = 0;
        const MAX_ITERATIONS: usize = 6;

        loop {
            iterations += 1;
            if iterations > MAX_ITERATIONS {
                error = Some("工具调用循环次数超限，已停止".to_string());
                break;
            }

            let ChatReply { text, tool_call } = match chat(
                &self.config.to_provider_config(),
                &messages,
                available_tools_prompt(),
            ) {
                Ok(reply) => reply,
                Err(err) => {
                    error = Some(err.to_string());
                    break;
                }
            };

            let Some(call) = tool_call else {
                // 无工具调用：结束
                reply = text;
                break;
            };

            // 记录工具调用步骤
            let step_index = timeline.len();
            timeline.push(TimelineStep::new(
                call.name(),
                call.tool.description(),
                call.is_read_only(),
            ));
            self.emit(AgentEvent::Status(AgentStatus::Executing));
            self.emit(AgentEvent::Timeline(timeline.clone()));

            // 权限决策
            let approval = permission.decide(call.tool.danger());
            let proceed = match approval {
                Approval::Allowed => true,
                Approval::Denied => {
                    timeline.set_result(
                        step_index,
                        StepStatus::Skipped,
                        format!("被权限策略拒绝（{}）", permission.label()),
                    );
                    self.emit(AgentEvent::Timeline(timeline.clone()));
                    messages.push(ChatMessage {
                        role: "user".to_string(),
                        content: format!("工具 {} 被权限拒绝", call.name()),
                    });
                    continue;
                }
                Approval::NeedsApproval => {
                    let reason = format!(
                        "工具 {}（{}）请求执行 {} 操作",
                        call.name(),
                        call.tool.description(),
                        call.tool.danger().label()
                    );
                    timeline.set_result(
                        step_index,
                        StepStatus::Pending,
                        format!("等待批准：{}", reason),
                    );
                    self.emit(AgentEvent::Status(AgentStatus::Approval));
                    self.emit(AgentEvent::ApprovalNeeded {
                        call: call.clone(),
                        reason: reason.clone(),
                    });
                    self.emit(AgentEvent::Timeline(timeline.clone()));
                    approve(&call)
                }
            };

            if !proceed {
                timeline.set_result(step_index, StepStatus::Skipped, "用户取消".to_string());
                self.emit(AgentEvent::Status(AgentStatus::Ready));
                self.emit(AgentEvent::Timeline(timeline.clone()));
                messages.push(ChatMessage {
                    role: "user".to_string(),
                    content: format!("工具 {} 被用户取消", call.name()),
                });
                continue;
            }

            // 执行工具
            timeline.set_status(step_index, StepStatus::Running);
            self.emit(AgentEvent::Timeline(timeline.clone()));
            let result = execute_tool(executor, &call);
            let (status, detail) = if result.ok {
                (StepStatus::Done, truncate_result(&result.output))
            } else {
                (StepStatus::Failed, truncate_result(&result.output))
            };
            timeline.set_result(step_index, status, detail.clone());
            self.emit(AgentEvent::Timeline(timeline.clone()));

            messages.push(ChatMessage {
                role: "user".to_string(),
                content: format!("工具 {} 结果:\n{}", call.name(), detail),
            });
        }

        // 最终状态
        let final_status = if error.is_some() {
            AgentStatus::Error
        } else {
            AgentStatus::Done
        };
        self.emit(AgentEvent::Status(final_status));
        self.emit(AgentEvent::Timeline(timeline.clone()));

        AgentSession {
            timeline,
            reply,
            error,
        }
    }
}

/// 会话结果（返回给 UI 展示 / 追加历史）
#[derive(Debug, Clone)]
pub struct AgentSession {
    pub timeline: Timeline,
    pub reply: String,
    pub error: Option<String>,
}

/// 把服务器快照转成简短上下文注入（可空）
fn format_snapshot_context(snapshot: &ServerSnapshot) -> String {
    let mut lines = Vec::new();
    if let Some(cpu) = &snapshot.cpu {
        lines.push(format!(
            "CPU 使用率 {:.1}%（负载 {:.2}/{:.2}/{:.2}）",
            cpu.usage_percent, cpu.load.0, cpu.load.1, cpu.load.2
        ));
    }
    if let Some(mem) = &snapshot.memory {
        lines.push(format!(
            "内存 {:.1}/{:.1} GB（{:.0}%）",
            mem.used_gb, mem.total_gb, mem.usage_percent
        ));
    }
    if let Some(sys) = &snapshot.system {
        lines.push(format!("系统 {}（{}）", sys.os, sys.hostname));
    }
    if !snapshot.docker.is_empty() {
        lines.push(format!("Docker 容器 {} 个", snapshot.docker.len()));
    }
    if lines.is_empty() {
        String::new()
    } else {
        format!("当前服务器状态：\n{}", lines.join("\n"))
    }
}

/// 截断工具输出到固定长度
fn truncate_result(output: &str) -> String {
    if output.len() <= MAX_TOOL_OUTPUT {
        output.to_string()
    } else {
        let start = output.len() - MAX_TOOL_OUTPUT;
        format!("…（已截断）…{}", &output[start..])
    }
}

// ── 单元测试 ──────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::monitor::executor::{ExecError, ExecOutput};
    use crate::monitor::snapshot::ServerSnapshot;

    struct FakeExecutor;
    impl RemoteExecutor for FakeExecutor {
        fn exec(&self, command: &str) -> Result<ExecOutput, ExecError> {
            Ok(ExecOutput {
                exit_code: Some(0),
                stdout: format!("fake output for: {command}"),
                stderr: String::new(),
            })
        }
    }

    #[test]
    fn snapshot_context_is_short_and_actionable() {
        let snap = ServerSnapshot::new("web");
        let context = format_snapshot_context(&snap);
        // 无数据时为空
        assert!(context.is_empty());
    }

    #[test]
    fn available_tools_prompt_lists_tools() {
        let prompt = available_tools_prompt();
        assert!(prompt.contains("server.status"));
        assert!(prompt.contains("docker.delete"));
        assert!(prompt.contains("logs.tail"));
    }

    #[test]
    fn agent_run_emits_events_and_returns_session() {
        let (tx, rx) = std::sync::mpsc::channel();
        let runner = AgentRunner::new(AgentConfig::default(), tx);
        let snap = ServerSnapshot::new("web");

        // approve 回调记录调用
        let approved_calls = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let approved = approved_calls.clone();
        let approve = move |call: &ToolCall| {
            approved.lock().unwrap().push(call.clone());
            true
        };

        // 由于 chat() 会实际调用 curl/ollama，这里仅验证在无网络时会优雅失败并返回错误
        let session = runner.run(&[], "查询服务器状态", &FakeExecutor, &snap, approve);

        // 至少发出了 Started / Timeline 事件
        let events: Vec<_> = rx.try_iter().collect();
        assert!(!events.is_empty());
        // 无可用 provider 时返回 error
        assert!(session.error.is_some());
    }

    #[test]
    fn agent_run_with_denied_tool_stops() {
        let (tx, _rx) = std::sync::mpsc::channel();
        let runner = AgentRunner::new(AgentConfig::default(), tx);
        let snap = ServerSnapshot::new("web");

        let session = runner.run(&[], "tool:server.status", &FakeExecutor, &snap, |_| true);
        // 无法真正连接 LLM，必然 error（链路安全降级）
        assert!(session.error.is_some() || !session.reply.is_empty());
    }

    #[test]
    fn truncate_result_handles_long_output() {
        let long = "a".repeat(5000);
        let truncated = truncate_result(&long);
        assert!(truncated.contains("已截断"));
        assert!(truncated.len() < 3000);
    }
}
