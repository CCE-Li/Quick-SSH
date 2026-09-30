# AI Agent 模块

AI Agent 模块位于 [`qssh/src/agent/`](/qssh/src/agent/)，实现「用户 → Agent → 工具注册表 → 远程执行器 → 服务器」的循环。Agent 自身不持有 SSH 连接，只通过工具调用与远端交互。

## 模块结构

```
agent/
├── mod.rs          # AgentRunner 运行循环、AgentEvent
├── config.rs       # agent.json 加载/保存
├── provider.rs     # 服务商适配（OpenAI / OpenCode Go / Ollama）
├── tools.rs        # 工具注册表与解析执行
├── permissions.rs  # 权限等级与危险等级
└── timeline.rs     # 时间线与状态枚举
```

## mod.rs — 运行循环

```rust
pub enum AgentEvent { Started, Status, Timeline, ApprovalNeeded, Finished }

pub struct AgentRunner { /* config, tx */ }

impl AgentRunner {
    pub fn new(config: AgentConfig, tx: Sender<AgentEvent>) -> Self;
    pub fn run(&self, history: &[ChatMessage], input: &str,
               executor: &dyn RemoteExecutor, snapshot: Option<&ServerSnapshot>,
               approve: impl Fn(&ToolCall, DangerLevel) -> bool) -> AgentSession;
}
```

运行流程：

1. 注入系统提示（工具清单）与当前监控快照上下文
2. 调用服务商 `chat()`，得到文字回复或一个工具调用
3. 按权限模型决定是否需审批（`approve` 回调阻塞等待 UI 确认，超时 300 秒视为拒绝）
4. 通过 `execute_tool()` 在远端执行，结果回填为下一条消息
5. 最多循环 `MAX_ITERATIONS = 6` 轮

单次工具输出截断到 `MAX_TOOL_OUTPUT = 2000` 字符。

## provider.rs — 服务商适配

```rust
pub enum ProviderKind { OpenAI, OpenCodeGo, Ollama }
pub struct ChatReply { pub text: Option<String>, pub tool_call: Option<String> }
```

| 服务商 | 别名字符串 | 调用方式 |
|--------|-----------|---------|
| OpenAI 兼容 | `openai` / `openai-compatible` / `deepseek` / `qwen` | `curl <base_url>/chat/completions` |
| OpenCode Go | `opencode` / `opencode-go` | 同上，缺省 base_url/model 自动补全 |
| Ollama | `ollama` / `local` | 优先 `ollama run`，回退到 `/api/chat` |

请求参数：`temperature 0.2`、`max_tokens 1024`。`list_models()` 读取 OpenAI `/models` 或 Ollama `/api/tags`。

API Key 解析顺序（`resolve_api_key`）：`agent.json` → `QSSH_OPENAI_API_KEY` → `OPENCODE_API_KEY` → OpenCode `auth.json`。

## tools.rs — 工具注册表

`ToolId` 枚举共 17 个工具，分三个危险等级：

| 等级 | 工具 |
|------|------|
| `Read` | `server.status`、`server.processes`、`server.disks`、`server.network`、`docker.list`、`files.list`、`logs.tail` |
| `SafeWrite` | `service.start`、`docker.restart`、`files.mkdir`、`files.write` |
| `Dangerous` | `docker.stop`、`docker.delete`、`service.stop`、`service.restart`、`shell.run`、`files.remove` |

`parse_tool_call()` 同时接受 `tool:name arg=value` 与模型风格的 `name(target=..., ...)` 两种写法，并支持中文参数键（`目标`、`目录`、`路径`、`命令`、`内容`、`文本`）。未知工具名返回 `None`。

## permissions.rs — 权限模型

```rust
pub enum PermissionLevel { ReadOnly, AskBeforeExecute, AutoSafe, FullAccess }
pub enum DangerLevel { Read, SafeWrite, Dangerous }
pub enum Approval { Allowed, NeedsApproval, Denied }

impl PermissionLevel {
    pub fn decide(&self, danger: DangerLevel) -> Approval;
}
```

默认 `ask_before_execute`。决策矩阵见 [AI Agent](/ai-agent) 页面。

## timeline.rs — 时间线

`TimelineStep` 的状态：`○` 待执行 / `●` 执行中 / `✓` 完成 / `⊘` 跳过 / `✗` 失败。

`AgentStatus`：`● Ready` / `◇ Thinking` / `● Executing` / `! Approval` / `▲ Error` / `✓ Done`，用于状态栏指示。
