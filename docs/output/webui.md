# Web 界面

`qssh web` 会启动一个本地 Web 界面（WebUI），用于查看运行状态并用表单方式编辑配置。它只监听 `127.0.0.1`，不对外网开放。

```bash
# 使用默认端口 17890
qssh web

# 指定端口
qssh web --port 8080
# 或
qssh web -p 8080
```

启动后打开 `http://127.0.0.1:17890` 即可。服务端是一个零依赖的极简 HTTP 服务（基于标准库 `TcpListener`），无需任何额外运行时。

## 功能

| 页面 / 接口 | 说明 |
|------------|------|
| 设置表单 | 按分组编辑配置，保存即写回对应文件 |
| 状态 | 版本号、当前 Agent 服务商 / 模型 / 权限 / 超时 |
| 工具列表 | 列出 Agent 的全部工具及其危险等级与当前权限下的处理方式 |
| 测试连接 | 向当前 Agent 服务商发一次最小请求，返回耗时、回复与可用模型列表 |

## 设置分组

WebUI 的设置页由**配置 schema 驱动**，分三组：

### Agent

对应 `agent.json`。

| 字段 | 控件 | 说明 |
|------|------|------|
| `provider` | 下拉 | opencode-go（推荐）/ openai / ollama |
| `api_key` | 密码框 | API Key |
| `base_url` | 文本框 | API 地址 |
| `model` | 文本框 | 模型名 |
| `permission` | 下拉 | ask_before_execute / read_only / auto_safe / full_access |
| `timeout_secs` | 数字 | 请求超时（5–600 秒） |

这一组带有 **「测试连接」** 按钮，会调用 `/api/test` 验证配置是否可用。

### 程序设置

对应 `~/.qsshrc`。

| 字段 | 范围 |
|------|------|
| `default_port` | 1–65535 |
| `ping_timeout_secs` | 1–60 |
| `upload_concurrency` | 1–16 |

### Dashboard

对应 `dashboard.json`。

| 字段 | 说明 |
|------|------|
| `active_profile` | 当前启用的 Dashboard Profile |
| `enabled` | 启用的组件（多选，共 12 个组件） |

## 与 TUI 的配合

- Agent 配置在每次会话前热加载，WebUI 中改完立即对新的 Agent 会话生效。
- Dashboard 布局/Profile 在 WebUI 中修改后，下次进入工作台时按启用的组件重建。
- 详细的字段含义见 [配置说明](/configuration)。

## 下一步

- [AI Agent](/ai-agent) — Agent 的权限与工具说明
- [配置说明](/configuration) — 所有配置文件的位置与字段
