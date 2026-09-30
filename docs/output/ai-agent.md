# AI Agent

AI Agent 是内嵌在 TUI 里的运维助手。你用自然语言描述目标，Agent 会选择合适的**工具**在选中的主机上执行，读取结果后再决定下一步，直到给出结论。

它是「Agent + 工具 + 远程执行」的循环，本身不直接持有 SSH 连接——每一步都通过工具调用完成，且受**权限分级**约束。

## 打开与关闭

| 入口 | 说明 |
|------|------|
| `Ctrl+A` / `Ctrl+Space` | 主机列表中直接打开 Agent |
| Dashboard 中按 `a` | 打开 Agent（Agent 组件） |
| `Ctrl+K` → 打开 AI Agent | 命令面板 |
| 终端中 `Ctrl+B` `a` | 嵌入式终端内打开 |

关闭：`Esc`；或当输入框为空时按 `q`。

## 对话面板

面板以「线程」形式展示：

- `You` — 你的输入
- `agent` — 模型的文字回复
- 工具步骤 — 每一步工具调用及状态（`○` 待执行 / `●` 执行中 / `✓` 完成 / `⊘` 跳过 / `✗` 失败）

| 按键 | 操作 |
|------|------|
| `Enter` | 发送输入 |
| 字符 / `Backspace` | 编辑输入 |
| `C` / `S` | 打开 Agent 设置 |
| `q`（输入为空时） | 关闭面板 |
| `Esc` | 关闭面板 |

右侧状态栏有 AI 指示器：`● Ready` / `◇ Thinking` / `● Executing` / `! Approval` / `▲ Error` / `✓ Done`。

## 工具与权限

Agent 可调用 17 个工具，按危险等级分类：

| 等级 | 工具 |
|------|------|
| **只读** | `server.status`、`server.processes`、`server.disks`、`server.network`、`docker.list`、`files.list`、`logs.tail` |
| **安全写** | `service.start`、`docker.restart`、`files.mkdir`、`files.write` |
| **危险** | `docker.stop`、`docker.delete`、`service.stop`、`service.restart`、`shell.run`、`files.remove` |

权限等级（`agent.json` 的 `permission` 字段）决定每个等级如何处置：

| 权限 | 只读 | 安全写 | 危险 |
|------|------|--------|------|
| `read_only` | 允许 | 拒绝 | 拒绝 |
| `ask_before_execute`（默认） | 允许 | 需审批 | 需审批 |
| `auto_safe` | 允许 | 允许 | 需审批 |
| `full_access` | 允许 | 允许 | 允许 |

需要审批时，TUI 会弹出确认框，显示工具名、说明、危险等级与目标主机，按 `y` 执行、`n` / `Esc` 拒绝（等待超时 300 秒后视为拒绝）。

<Callout title="危险等级配色">
  审批框按危险等级着色：只读为安全色、安全写为提示色、危险操作为告警色。执行 `shell.run` / `files.remove` 前请仔细确认目标。
</Callout>

## 配置 Agent

在 Agent 面板中按 `C` / `S` 打开设置弹窗，或通过 [Web 界面](/webui) 的「Agent」分组编辑。配置文件位于 `agent.json`（Windows 为 `%APPDATA%\quick-ssh\agent.json`，其余平台为 `~/.config/quick-ssh/agent.json`）。

```json title="~/.config/quick-ssh/agent.json"
{
  "provider": "openai",
  "base_url": "https://api.deepseek.com/v1",
  "model": "deepseek-chat",
  "permission": "ask_before_execute",
  "timeout_secs": 60,
  "api_key": ""
}
```

| 字段 | 默认值 | 说明 |
|------|--------|------|
| `provider` | `openai` | 服务商类型，见下表 |
| `base_url` | `https://api.deepseek.com/v1` | API 地址 |
| `model` | `deepseek-chat` | 模型名 |
| `permission` | `ask_before_execute` | 权限等级 |
| `timeout_secs` | `60` | 单次请求超时（秒） |
| `api_key` | 空 | API Key，留空则回退到环境变量 |

### 支持的服务商

| provider 取值 | 说明 |
|---------------|------|
| `openai` / `deepseek` / `qwen` / `openai-compatible` | 任意 OpenAI 兼容接口（走 `/chat/completions`） |
| `opencode` / `opencode-go` | OpenCode Go，自动补全 base_url 与模型 |
| `ollama` / `local` | 本地 Ollama，优先用 CLI，回退到 HTTP API |

### API Key 解析顺序

1. `agent.json` 中的 `api_key`
2. 环境变量 `QSSH_OPENAI_API_KEY`
3. 环境变量 `OPENCODE_API_KEY`
4. OpenCode 登录凭据（`~/.local/share/opencode/auth.json` 或 `~/.config/opencode/auth.json`）

<Callout title="配置热加载">
  每次开始 Agent 会话前都会从磁盘重新读取 `agent.json`，因此在 Web 界面里改完设置后立即生效，无需重启 qssh。
</Callout>

## 运行机制

- Agent 会拿到可用工具清单与当前主机的监控快照作为上下文。
- 每次会话最多循环 **6 轮**工具调用（`MAX_ITERATIONS`）。
- 单个工具输出超过 2000 字符会被截断（`MAX_TOOL_OUTPUT`）。
- 你也可以直接把一段工具调用文本粘贴进输入框，让 Agent 跳过模型直接执行。

## 下一步

- [Web 界面](/webui) — 在浏览器里编辑 Agent 与程序设置
- [工作台（Dashboard）](/dashboard) — 监控组件与运维面板
