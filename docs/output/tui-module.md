# TUI 模块

TUI 模块位于 [`qssh/src/tui/`](/qssh/src/tui/)，采用事件驱动架构，基于 ratatui 0.29 和 crossterm 0.28 构建。

## 模块结构

```
tui/
├── mod.rs        # 模块声明
├── action.rs     # Action / Mode / View 枚举
├── app.rs        # 应用状态与业务逻辑
├── event.rs      # 事件循环
├── keymap.rs     # 键盘映射 + Mode 标签/提示
├── term.rs       # 嵌入式终端（portable-pty + vt100）
├── mouse.rs      # 鼠标事件映射
├── ui.rs         # 渲染逻辑
├── widgets.rs    # 自定义组件（弹窗等）
├── editor.rs     # 主机 / Agent 编辑表单
└── dashboard/    # 工作台
    ├── config.rs # Profile 与布局持久化
    ├── layout.rs # 布局引擎与分隔线拖拽
    ├── widgets.rs# WidgetModule trait 与组件渲染
    ├── palette.rs# 命令面板
    └── ui.rs     # 工作台渲染、终端/Agent 面板、动画
```

## 架构模式：Event → Action → State → Render

TUI 采用单向数据流架构：

```
[Crossterm Event]
      ↓
[map_key_to_action() / mouse] → Action 枚举
      ↓
[App::apply(action)] → State 更新
      ↓
[ui::render(frame, app)] → ratatui 渲染
      ↓
[tick 等待 100ms] → 下一轮循环
```

| 层 | 文件 | 职责 |
|----|------|------|
| **事件** | `event.rs` | 事件循环，轮询键盘/鼠标，处理终端键位拦截 |
| **意图** | `keymap.rs` / `mouse.rs` | 将按键/鼠标映射为 Action，提供 Mode 标签和提示 |
| **逻辑** | `app.rs` | 处理 Action 更新状态，管理后台线程 |
| **渲染** | `ui.rs` / `widgets.rs` / `dashboard/ui.rs` | 根据状态渲染 UI |
| **表单** | `editor.rs` | 主机 / Agent 编辑弹窗表单 |

## action.rs — View / Action / Mode

### View

两个顶层视图：`HostList`（主机列表）与 `Dashboard`（工作台）。

### Action

`Action` 枚举覆盖导航、主机操作、视图切换、工作台、运维面板、Agent、远程命令和终端控制等几十种意图，例如 `MoveUp`、`MoveHost(isize)`、`Connect`、`ConnectNewWindow`、`ShowDashboard`、`OpenDockerOps`、`OpenAgentOps`、`TerminalKey`、`CloseTerminal` 等。Action 是纯数据，便于测试。

### Mode

`Mode` 枚举表示当前交互模式，每个模式实现 `label()`（状态栏徽标）与 `hint()`（状态栏提示）。可达的模式包括：

`Normal`、`Search`、`Add`、`Edit`、`Confirm`、`Help`、`Palette`、`DashboardConfig`、`DockerOps`、`DockerConfirm`、`ServiceOps`、`ServiceConfirm`、`FileOps`、`FileConfirm`、`LogOps`、`LogFilter`、`AgentOps`、`AgentConfig`、`AgentConfirm`、`Command`、`CommandResult`、`Terminal`。

<Note>
  `Rename` / `Export` / `Import` 仍有 label/hint 定义，但当前没有键位与 Action 驱动，属于旧版 UI 的遗留模式。
</Note>

## app.rs — 应用状态

`App` 是 TUI 的唯一数据源，除主机列表、搜索、标记、Ping 状态外，还持有工作台状态（监控快照、组件、布局）与终端会话。后台任务通过 mpsc channel 与 UI 通信：

| 后台任务 | 说明 |
|----------|------|
| Ping | 单机 / 全量 TCP 检测 |
| Monitor | 工作台周期性采集 |
| Agent | AI 会话与审批往返 |
| Remote command | `:` 远程命令执行 |
| Terminal | PTY 读取线程 |

`restore_mode_after_overlay()` 在关闭面板时决定焦点：若面板打开期间 SSH 终端仍存活则回到 `Mode::Terminal`，否则回到 `Mode::Normal`。

## event.rs — 事件循环

`start()` 是 TUI 入口，事件循环以 **100ms** 为 tick：

```
while app.running:
    poll_background_tasks()
    expire_flash_message()
    render()
    poll(100ms)
    if event:
        if Terminal 模式 → 转发给 PTY（Ctrl+B 前缀除外）
        elif Add/Edit/AgentConfig → handle_form_key()
        else → map_key_to_action() → apply()
```

在主机列表中按 `Enter` 连接时，会先关闭鼠标捕获，`ratatui::try_restore()` 退出备用屏幕，运行全屏 SSH 会话，返回后再 `try_init()`。Windows 下键盘轮询用 `catch_unwind` 包裹，吞掉 crossterm-winapi 在非法 `INPUT_RECORD` 上的 panic。

## term.rs — 嵌入式终端

- 用 `portable-pty` 分配 PTY（Windows 为 ConPTY），后台线程读取输出送入 `vt100::Parser`（回滚 1000 行）
- SSH 命令强制 `-tt`；有保存密码时注入 AskPass 环境变量
- `TermStatus`：`Connecting` / `Running` / `Exited`。`Running` 只在屏幕出现可见内容时触发，避免握手阶段的控制序列误判
- 断开检测同时依赖读取线程的 `Exited` 与主动 `poll_child_exit()`（Windows ConPTY 在 ssh 退出时不一定 EOF）
- `encode_key()` 负责把按键编码成远端字节（Enter `\r`、Backspace `0x7f`、方向键/功能键序列、Ctrl/Alt 组合等）

## keymap.rs — 键盘映射

- `map_key_to_action()`：按当前 Mode 映射按键
- `map_terminal_prefix_key()`：处理终端模式下的 `Ctrl+B` 前缀组合
- `module_key()`：工作台配置弹窗中数字键到组件的映射（`1`–`0`、`a`）

## dashboard/ — 工作台

| 文件 | 职责 |
|------|------|
| `config.rs` | `DashboardConfig` / `ProfileConfig` / `LayoutNode`，内置 Profile 与 `default_layout_for()` |
| `layout.rs` | 布局引擎与分隔线拖拽数学（`hit_test_divider`、`apply_split_resize`） |
| `widgets.rs` | `WidgetModule` trait 与组件渲染（真实快照或占位文本） |
| `palette.rs` | 命令面板动作表 `PALETTE_ACTIONS` 与筛选 |
| `ui.rs` | 工作台渲染、终端/Agent 面板、连接动画与待机动画 |

## editor.rs — 表单

主机表单与 Agent 设置表单共用编辑器框架。主机表单的 `build_submission()` 返回 `HostBlock` 与 `PasswordStorageAction`，用于区分新增留空、编辑保留、设置新密码和 `!clear` 清除四种语义。Agent 表单编辑 `agent.json` 的 provider / base_url / model / permission / timeout。

## widgets.rs — 自定义组件

| 组件 | 说明 |
|------|------|
| `centered_rect()` | 居中弹窗区域计算 |
| `render_help_popup()` | 帮助弹窗 |
| `render_host_form_popup()` | 主机编辑弹窗表单 |

## ui.rs — 渲染逻辑

界面为「1 行标题栏 / 内容区 / 1 行状态栏」结构，内容区按 `View` 渲染主机列表或工作台，并叠加各种弹窗（搜索、确认、Docker/服务/文件/日志、Agent、命令面板、Dashboard 配置、远程命令与结果）。
