import{n as e}from"./vendor-CQh2k-GV.js";import{n as t}from"./react-vendor-6GaKtW3l.js";var n=t();function r(t){let r={a:`a`,code:`code`,h1:`h1`,h2:`h2`,h3:`h3`,hr:`hr`,li:`li`,p:`p`,pre:`pre`,strong:`strong`,table:`table`,tbody:`tbody`,td:`td`,th:`th`,thead:`thead`,tr:`tr`,ul:`ul`,...e(),...t.components},{Note:i}=r;return i||a(`Note`,!0),(0,n.jsxs)(n.Fragment,{children:[(0,n.jsx)(r.hr,{}),`
`,(0,n.jsx)(r.p,{children:`title: TUI 模块
description: Quick-SSH TUI 模块的详细文档，包含事件驱动架构、视图与模式、工作台、嵌入式终端、鼠标与键盘映射。
keywords:`}),`
`,(0,n.jsxs)(r.ul,{children:[`
`,(0,n.jsx)(r.li,{children:`tui`}),`
`,(0,n.jsx)(r.li,{children:`ratatui`}),`
`,(0,n.jsx)(r.li,{children:`crossterm`}),`
`,(0,n.jsx)(r.li,{children:`事件驱动`}),`
`,(0,n.jsx)(r.li,{children:`状态管理`}),`
`,(0,n.jsx)(r.li,{children:`键盘映射`}),`
`,(0,n.jsx)(r.li,{children:`dashboard`}),`
`,(0,n.jsx)(r.li,{children:`terminal`}),`
`]}),`
`,(0,n.jsx)(r.hr,{}),`
`,(0,n.jsx)(r.h1,{id:`tui-模块`,children:`TUI 模块`}),`
`,(0,n.jsxs)(r.p,{children:[`TUI 模块位于 `,(0,n.jsx)(r.a,{href:`/qssh/src/tui/`,language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`qssh/src/tui/`})}),`，采用事件驱动架构，基于 ratatui 0.29 和 crossterm 0.28 构建。`]}),`
`,(0,n.jsx)(r.h2,{id:`模块结构`,children:`模块结构`}),`
`,(0,n.jsx)(r.pre,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`tui/
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
`})}),`
`,(0,n.jsx)(r.h2,{id:`架构模式event--action--state--render`,children:`架构模式：Event → Action → State → Render`}),`
`,(0,n.jsx)(r.p,{children:`TUI 采用单向数据流架构：`}),`
`,(0,n.jsx)(r.pre,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`[Crossterm Event]
      ↓
[map_key_to_action() / mouse] → Action 枚举
      ↓
[App::apply(action)] → State 更新
      ↓
[ui::render(frame, app)] → ratatui 渲染
      ↓
[tick 等待 100ms] → 下一轮循环
`})}),`
`,(0,n.jsxs)(r.table,{children:[(0,n.jsx)(r.thead,{children:(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.th,{children:`层`}),(0,n.jsx)(r.th,{children:`文件`}),(0,n.jsx)(r.th,{children:`职责`})]})}),(0,n.jsxs)(r.tbody,{children:[(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{children:(0,n.jsx)(r.strong,{children:`事件`})}),(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`event.rs`})}),(0,n.jsx)(r.td,{children:`事件循环，轮询键盘/鼠标，处理终端键位拦截`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{children:(0,n.jsx)(r.strong,{children:`意图`})}),(0,n.jsxs)(r.td,{language:`txt`,children:[(0,n.jsx)(r.code,{language:`txt`,children:`keymap.rs`}),` / `,(0,n.jsx)(r.code,{language:`txt`,children:`mouse.rs`})]}),(0,n.jsx)(r.td,{children:`将按键/鼠标映射为 Action，提供 Mode 标签和提示`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{children:(0,n.jsx)(r.strong,{children:`逻辑`})}),(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`app.rs`})}),(0,n.jsx)(r.td,{children:`处理 Action 更新状态，管理后台线程`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{children:(0,n.jsx)(r.strong,{children:`渲染`})}),(0,n.jsxs)(r.td,{language:`txt`,children:[(0,n.jsx)(r.code,{language:`txt`,children:`ui.rs`}),` / `,(0,n.jsx)(r.code,{language:`txt`,children:`widgets.rs`}),` / `,(0,n.jsx)(r.code,{language:`txt`,children:`dashboard/ui.rs`})]}),(0,n.jsx)(r.td,{children:`根据状态渲染 UI`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{children:(0,n.jsx)(r.strong,{children:`表单`})}),(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`editor.rs`})}),(0,n.jsx)(r.td,{children:`主机 / Agent 编辑弹窗表单`})]})]})]}),`
`,(0,n.jsx)(r.h2,{id:`actionrs--view--action--mode`,children:`action.rs — View / Action / Mode`}),`
`,(0,n.jsx)(r.h3,{id:`view`,children:`View`}),`
`,(0,n.jsxs)(r.p,{language:`txt`,children:[`两个顶层视图：`,(0,n.jsx)(r.code,{language:`txt`,children:`HostList`}),`（主机列表）与 `,(0,n.jsx)(r.code,{language:`txt`,children:`Dashboard`}),`（工作台）。`]}),`
`,(0,n.jsx)(r.h3,{id:`action`,children:`Action`}),`
`,(0,n.jsxs)(r.p,{language:`txt`,children:[(0,n.jsx)(r.code,{language:`txt`,children:`Action`}),` 枚举覆盖导航、主机操作、视图切换、工作台、运维面板、Agent、远程命令和终端控制等几十种意图，例如 `,(0,n.jsx)(r.code,{language:`txt`,children:`MoveUp`}),`、`,(0,n.jsx)(r.code,{language:`txt`,children:`MoveHost(isize)`}),`、`,(0,n.jsx)(r.code,{language:`txt`,children:`Connect`}),`、`,(0,n.jsx)(r.code,{language:`txt`,children:`ConnectNewWindow`}),`、`,(0,n.jsx)(r.code,{language:`txt`,children:`ShowDashboard`}),`、`,(0,n.jsx)(r.code,{language:`txt`,children:`OpenDockerOps`}),`、`,(0,n.jsx)(r.code,{language:`txt`,children:`OpenAgentOps`}),`、`,(0,n.jsx)(r.code,{language:`txt`,children:`TerminalKey`}),`、`,(0,n.jsx)(r.code,{language:`txt`,children:`CloseTerminal`}),` 等。Action 是纯数据，便于测试。`]}),`
`,(0,n.jsx)(r.h3,{id:`mode`,children:`Mode`}),`
`,(0,n.jsxs)(r.p,{language:`txt`,children:[(0,n.jsx)(r.code,{language:`txt`,children:`Mode`}),` 枚举表示当前交互模式，每个模式实现 `,(0,n.jsx)(r.code,{language:`txt`,children:`label()`}),`（状态栏徽标）与 `,(0,n.jsx)(r.code,{language:`txt`,children:`hint()`}),`（状态栏提示）。可达的模式包括：`]}),`
`,(0,n.jsxs)(r.p,{language:`txt`,children:[(0,n.jsx)(r.code,{language:`txt`,children:`Normal`}),`、`,(0,n.jsx)(r.code,{language:`txt`,children:`Search`}),`、`,(0,n.jsx)(r.code,{language:`txt`,children:`Add`}),`、`,(0,n.jsx)(r.code,{language:`txt`,children:`Edit`}),`、`,(0,n.jsx)(r.code,{language:`txt`,children:`Confirm`}),`、`,(0,n.jsx)(r.code,{language:`txt`,children:`Help`}),`、`,(0,n.jsx)(r.code,{language:`txt`,children:`Palette`}),`、`,(0,n.jsx)(r.code,{language:`txt`,children:`DashboardConfig`}),`、`,(0,n.jsx)(r.code,{language:`txt`,children:`DockerOps`}),`、`,(0,n.jsx)(r.code,{language:`txt`,children:`DockerConfirm`}),`、`,(0,n.jsx)(r.code,{language:`txt`,children:`ServiceOps`}),`、`,(0,n.jsx)(r.code,{language:`txt`,children:`ServiceConfirm`}),`、`,(0,n.jsx)(r.code,{language:`txt`,children:`FileOps`}),`、`,(0,n.jsx)(r.code,{language:`txt`,children:`FileConfirm`}),`、`,(0,n.jsx)(r.code,{language:`txt`,children:`LogOps`}),`、`,(0,n.jsx)(r.code,{language:`txt`,children:`LogFilter`}),`、`,(0,n.jsx)(r.code,{language:`txt`,children:`AgentOps`}),`、`,(0,n.jsx)(r.code,{language:`txt`,children:`AgentConfig`}),`、`,(0,n.jsx)(r.code,{language:`txt`,children:`AgentConfirm`}),`、`,(0,n.jsx)(r.code,{language:`txt`,children:`Command`}),`、`,(0,n.jsx)(r.code,{language:`txt`,children:`CommandResult`}),`、`,(0,n.jsx)(r.code,{language:`txt`,children:`Terminal`}),`。`]}),`
`,(0,n.jsx)(i,{children:(0,n.jsxs)(r.p,{language:`txt`,children:[(0,n.jsx)(r.code,{language:`txt`,children:`Rename`}),` / `,(0,n.jsx)(r.code,{language:`txt`,children:`Export`}),` / `,(0,n.jsx)(r.code,{language:`txt`,children:`Import`}),` 仍有 label/hint 定义，但当前没有键位与 Action 驱动，属于旧版 UI 的遗留模式。`]})}),`
`,(0,n.jsx)(r.h2,{id:`apprs--应用状态`,children:`app.rs — 应用状态`}),`
`,(0,n.jsxs)(r.p,{language:`txt`,children:[(0,n.jsx)(r.code,{language:`txt`,children:`App`}),` 是 TUI 的唯一数据源，除主机列表、搜索、标记、Ping 状态外，还持有工作台状态（监控快照、组件、布局）与终端会话。后台任务通过 mpsc channel 与 UI 通信：`]}),`
`,(0,n.jsxs)(r.table,{children:[(0,n.jsx)(r.thead,{children:(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.th,{children:`后台任务`}),(0,n.jsx)(r.th,{children:`说明`})]})}),(0,n.jsxs)(r.tbody,{children:[(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{children:`Ping`}),(0,n.jsx)(r.td,{children:`单机 / 全量 TCP 检测`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{children:`Monitor`}),(0,n.jsx)(r.td,{children:`工作台周期性采集`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{children:`Agent`}),(0,n.jsx)(r.td,{children:`AI 会话与审批往返`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{children:`Remote command`}),(0,n.jsxs)(r.td,{language:`txt`,children:[(0,n.jsx)(r.code,{language:`txt`,children:`:`}),` 远程命令执行`]})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{children:`Terminal`}),(0,n.jsx)(r.td,{children:`PTY 读取线程`})]})]})]}),`
`,(0,n.jsxs)(r.p,{language:`txt`,children:[(0,n.jsx)(r.code,{language:`txt`,children:`restore_mode_after_overlay()`}),` 在关闭面板时决定焦点：若面板打开期间 SSH 终端仍存活则回到 `,(0,n.jsx)(r.code,{language:`txt`,children:`Mode::Terminal`}),`，否则回到 `,(0,n.jsx)(r.code,{language:`txt`,children:`Mode::Normal`}),`。`]}),`
`,(0,n.jsx)(r.h2,{id:`eventrs--事件循环`,children:`event.rs — 事件循环`}),`
`,(0,n.jsxs)(r.p,{language:`txt`,children:[(0,n.jsx)(r.code,{language:`txt`,children:`start()`}),` 是 TUI 入口，事件循环以 `,(0,n.jsx)(r.strong,{children:`100ms`}),` 为 tick：`]}),`
`,(0,n.jsx)(r.pre,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`while app.running:
    poll_background_tasks()
    expire_flash_message()
    render()
    poll(100ms)
    if event:
        if Terminal 模式 → 转发给 PTY（Ctrl+B 前缀除外）
        elif Add/Edit/AgentConfig → handle_form_key()
        else → map_key_to_action() → apply()
`})}),`
`,(0,n.jsxs)(r.p,{language:`txt`,children:[`在主机列表中按 `,(0,n.jsx)(r.code,{language:`txt`,children:`Enter`}),` 连接时，会先关闭鼠标捕获，`,(0,n.jsx)(r.code,{language:`txt`,children:`ratatui::try_restore()`}),` 退出备用屏幕，运行全屏 SSH 会话，返回后再 `,(0,n.jsx)(r.code,{language:`txt`,children:`try_init()`}),`。Windows 下键盘轮询用 `,(0,n.jsx)(r.code,{language:`txt`,children:`catch_unwind`}),` 包裹，吞掉 crossterm-winapi 在非法 `,(0,n.jsx)(r.code,{language:`txt`,children:`INPUT_RECORD`}),` 上的 panic。`]}),`
`,(0,n.jsx)(r.h2,{id:`termrs--嵌入式终端`,children:`term.rs — 嵌入式终端`}),`
`,(0,n.jsxs)(r.ul,{children:[`
`,(0,n.jsxs)(r.li,{language:`txt`,children:[`用 `,(0,n.jsx)(r.code,{language:`txt`,children:`portable-pty`}),` 分配 PTY（Windows 为 ConPTY），后台线程读取输出送入 `,(0,n.jsx)(r.code,{language:`txt`,children:`vt100::Parser`}),`（回滚 1000 行）`]}),`
`,(0,n.jsxs)(r.li,{language:`txt`,children:[`SSH 命令强制 `,(0,n.jsx)(r.code,{language:`txt`,children:`-tt`}),`；有保存密码时注入 AskPass 环境变量`]}),`
`,(0,n.jsxs)(r.li,{language:`txt`,children:[(0,n.jsx)(r.code,{language:`txt`,children:`TermStatus`}),`：`,(0,n.jsx)(r.code,{language:`txt`,children:`Connecting`}),` / `,(0,n.jsx)(r.code,{language:`txt`,children:`Running`}),` / `,(0,n.jsx)(r.code,{language:`txt`,children:`Exited`}),`。`,(0,n.jsx)(r.code,{language:`txt`,children:`Running`}),` 只在屏幕出现可见内容时触发，避免握手阶段的控制序列误判`]}),`
`,(0,n.jsxs)(r.li,{language:`txt`,children:[`断开检测同时依赖读取线程的 `,(0,n.jsx)(r.code,{language:`txt`,children:`Exited`}),` 与主动 `,(0,n.jsx)(r.code,{language:`txt`,children:`poll_child_exit()`}),`（Windows ConPTY 在 ssh 退出时不一定 EOF）`]}),`
`,(0,n.jsxs)(r.li,{language:`txt`,children:[(0,n.jsx)(r.code,{language:`txt`,children:`encode_key()`}),` 负责把按键编码成远端字节（Enter `,(0,n.jsx)(r.code,{language:`txt`,children:`\\r`}),`、Backspace `,(0,n.jsx)(r.code,{language:`txt`,children:`0x7f`}),`、方向键/功能键序列、Ctrl/Alt 组合等）`]}),`
`]}),`
`,(0,n.jsx)(r.h2,{id:`keymaprs--键盘映射`,children:`keymap.rs — 键盘映射`}),`
`,(0,n.jsxs)(r.ul,{children:[`
`,(0,n.jsxs)(r.li,{language:`txt`,children:[(0,n.jsx)(r.code,{language:`txt`,children:`map_key_to_action()`}),`：按当前 Mode 映射按键`]}),`
`,(0,n.jsxs)(r.li,{language:`txt`,children:[(0,n.jsx)(r.code,{language:`txt`,children:`map_terminal_prefix_key()`}),`：处理终端模式下的 `,(0,n.jsx)(r.code,{language:`txt`,children:`Ctrl+B`}),` 前缀组合`]}),`
`,(0,n.jsxs)(r.li,{language:`txt`,children:[(0,n.jsx)(r.code,{language:`txt`,children:`module_key()`}),`：工作台配置弹窗中数字键到组件的映射（`,(0,n.jsx)(r.code,{language:`txt`,children:`1`}),`–`,(0,n.jsx)(r.code,{language:`txt`,children:`0`}),`、`,(0,n.jsx)(r.code,{language:`txt`,children:`a`}),`）`]}),`
`]}),`
`,(0,n.jsx)(r.h2,{id:`dashboard--工作台`,children:`dashboard/ — 工作台`}),`
`,(0,n.jsxs)(r.table,{children:[(0,n.jsx)(r.thead,{children:(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.th,{children:`文件`}),(0,n.jsx)(r.th,{children:`职责`})]})}),(0,n.jsxs)(r.tbody,{children:[(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`config.rs`})}),(0,n.jsxs)(r.td,{language:`txt`,children:[(0,n.jsx)(r.code,{language:`txt`,children:`DashboardConfig`}),` / `,(0,n.jsx)(r.code,{language:`txt`,children:`ProfileConfig`}),` / `,(0,n.jsx)(r.code,{language:`txt`,children:`LayoutNode`}),`，内置 Profile 与 `,(0,n.jsx)(r.code,{language:`txt`,children:`default_layout_for()`})]})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`layout.rs`})}),(0,n.jsxs)(r.td,{language:`txt`,children:[`布局引擎与分隔线拖拽数学（`,(0,n.jsx)(r.code,{language:`txt`,children:`hit_test_divider`}),`、`,(0,n.jsx)(r.code,{language:`txt`,children:`apply_split_resize`}),`）`]})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`widgets.rs`})}),(0,n.jsxs)(r.td,{language:`txt`,children:[(0,n.jsx)(r.code,{language:`txt`,children:`WidgetModule`}),` trait 与组件渲染（真实快照或占位文本）`]})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`palette.rs`})}),(0,n.jsxs)(r.td,{language:`txt`,children:[`命令面板动作表 `,(0,n.jsx)(r.code,{language:`txt`,children:`PALETTE_ACTIONS`}),` 与筛选`]})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`ui.rs`})}),(0,n.jsx)(r.td,{children:`工作台渲染、终端/Agent 面板、连接动画与待机动画`})]})]})]}),`
`,(0,n.jsx)(r.h2,{id:`editorrs--表单`,children:`editor.rs — 表单`}),`
`,(0,n.jsxs)(r.p,{language:`txt`,children:[`主机表单与 Agent 设置表单共用编辑器框架。主机表单的 `,(0,n.jsx)(r.code,{language:`txt`,children:`build_submission()`}),` 返回 `,(0,n.jsx)(r.code,{language:`txt`,children:`HostBlock`}),` 与 `,(0,n.jsx)(r.code,{language:`txt`,children:`PasswordStorageAction`}),`，用于区分新增留空、编辑保留、设置新密码和 `,(0,n.jsx)(r.code,{language:`txt`,children:`!clear`}),` 清除四种语义。Agent 表单编辑 `,(0,n.jsx)(r.code,{language:`txt`,children:`agent.json`}),` 的 provider / base_url / model / permission / timeout。`]}),`
`,(0,n.jsx)(r.h2,{id:`widgetsrs--自定义组件`,children:`widgets.rs — 自定义组件`}),`
`,(0,n.jsxs)(r.table,{children:[(0,n.jsx)(r.thead,{children:(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.th,{children:`组件`}),(0,n.jsx)(r.th,{children:`说明`})]})}),(0,n.jsxs)(r.tbody,{children:[(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`centered_rect()`})}),(0,n.jsx)(r.td,{children:`居中弹窗区域计算`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`render_help_popup()`})}),(0,n.jsx)(r.td,{children:`帮助弹窗`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`render_host_form_popup()`})}),(0,n.jsx)(r.td,{children:`主机编辑弹窗表单`})]})]})]}),`
`,(0,n.jsx)(r.h2,{id:`uirs--渲染逻辑`,children:`ui.rs — 渲染逻辑`}),`
`,(0,n.jsxs)(r.p,{language:`txt`,children:[`界面为「1 行标题栏 / 内容区 / 1 行状态栏」结构，内容区按 `,(0,n.jsx)(r.code,{language:`txt`,children:`View`}),` 渲染主机列表或工作台，并叠加各种弹窗（搜索、确认、Docker/服务/文件/日志、Agent、命令面板、Dashboard 配置、远程命令与结果）。`]})]})}function i(t={}){let{wrapper:i}={...e(),...t.components};return i?(0,n.jsx)(i,{...t,children:(0,n.jsx)(r,{...t})}):r(t)}function a(e,t){throw Error(`Expected `+(t?`component`:`object`)+" `"+e+"` to be defined: you likely forgot to import, pass, or provide it.")}export{i as default};