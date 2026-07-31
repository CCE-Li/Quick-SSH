import{n as e}from"./vendor-CQh2k-GV.js";import{n as t}from"./react-vendor-6GaKtW3l.js";var n=t();function r(t){let r={a:`a`,code:`code`,h1:`h1`,h2:`h2`,h3:`h3`,hr:`hr`,li:`li`,ol:`ol`,p:`p`,pre:`pre`,strong:`strong`,ul:`ul`,...e(),...t.components},{Properties:i,Property:o}=r;return i||a(`Properties`,!0),o||a(`Property`,!0),(0,n.jsxs)(n.Fragment,{children:[(0,n.jsx)(r.hr,{}),`
`,(0,n.jsx)(r.p,{children:`title: 架构设计\r
description: Quick-SSH 的 Rust 模块化架构设计详解，包括项目结构、数据流和设计决策。\r
keywords:`}),`
`,(0,n.jsxs)(r.ul,{children:[`
`,(0,n.jsx)(r.li,{children:`架构`}),`
`,(0,n.jsx)(r.li,{children:`设计`}),`
`,(0,n.jsx)(r.li,{children:`rust`}),`
`,(0,n.jsx)(r.li,{children:`模块`}),`
`,(0,n.jsx)(r.li,{children:`数据流`}),`
`]}),`
`,(0,n.jsx)(r.hr,{}),`
`,(0,n.jsx)(r.h1,{id:`架构设计`,children:`架构设计`}),`
`,(0,n.jsx)(r.h2,{id:`项目结构`,children:`项目结构`}),`
`,(0,n.jsx)(r.pre,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`quick-ssh/\r
├── Cargo.toml                   # 工作空间根\r
├── qssh/                        # 主二进制\r
│   ├── Cargo.toml\r
│   ├── build.rs                 # 构建脚本（监视 cli.rs 变更）\r
│   └── src/\r
│       ├── main.rs              # 入口：clap 参数解析 → 分发\r
│       ├── cli.rs               # clap derive CLI 定义\r
│       ├── config/              # SSH 配置 + 程序设置\r
│       │   ├── mod.rs           # 模块重导出\r
│       │   ├── types.rs         # SshDirective, HostBlock, SshConfig 类型定义\r
│       │   ├── parser.rs        # 渐进式 SSH 解析器
│       │   ├── writer.rs        # SSH 配置渲染器
│       │   ├── credentials.rs   # 系统凭据库与 OpenSSH AskPass
│       │   └── settings.rs      # ~/.qsshrc 加载/保存
│       ├── ssh/                 # SSH 会话 + SFTP 上传\r
│       │   ├── mod.rs\r
│       │   ├── session.rs       # SshTarget 解析与构建\r
│       │   ├── spawn.rs         # spawn ssh 进程与 AskPass 配置
│       │   ├── upload.rs        # 内联 SCP 上传（预留）\r
│       │   └── drag_detect.rs   # 拖拽文件路径检测\r
│       ├── network/             # TCP 在线检测\r
│       │   ├── mod.rs\r
│       │   └── ping.rs          # TcpStream 连接测试\r
│       ├── tui/                 # 终端 UI（事件驱动）\r
│       │   ├── mod.rs\r
│       │   ├── action.rs        # Action + Mode 枚举\r
│       │   ├── app.rs           # 应用状态与业务逻辑\r
│       │   ├── event.rs         # 事件循环\r
│       │   ├── keymap.rs        # 键盘映射 + Mode 标签/提示\r
│       │   ├── ui.rs            # 渲染逻辑\r
│       │   ├── widgets.rs       # 自定义组件（弹窗等）\r
│       │   └── editor.rs        # 主机编辑表单\r
│       └── cmd/                 # CLI 子命令实现\r
│           ├── mod.rs\r
│           ├── ps.rs            # 列出主机\r
│           ├── add.rs           # 添加主机\r
│           ├── rm.rs            # 删除主机\r
│           ├── connect.rs       # 连接主机\r
│           ├── export.rs        # 导出 JSON\r
│           ├── import.rs        # 导入 JSON\r
│           ├── help.rs          # 自定义帮助\r
│           └── completions.rs   # Shell 补全生成\r
├── qssh-uploader/               # 独立上传二进制\r
│   └── src/main.rs              # 并发 SCP 上传 + 进度显示\r
├── docs/                        # 文档\r
├── packaging/                   # 包管理器配置\r
└── .github/workflows/           # CI/CD\r
    ├── ci.yml                   # 每次推送自动检查\r
    └── release.yml              # 打标签触发发布
`})}),`
`,(0,n.jsx)(r.h2,{id:`工作空间架构`,children:`工作空间架构`}),`
`,(0,n.jsx)(r.p,{children:`项目使用 Cargo workspace 管理两个二进制 crate：`}),`
`,(0,n.jsxs)(i,{children:[(0,n.jsx)(o,{name:`qssh`,type:`主程序`,children:(0,n.jsx)(r.p,{children:`核心 SSH 管理工具，包含 CLI、TUI、配置管理、SSH 连接功能`})}),(0,n.jsx)(o,{name:`qssh-uploader`,type:`上传工具`,children:(0,n.jsx)(r.p,{children:`独立的文件上传程序，专注于 SCP 并发上传和进度显示`})})]}),`
`,(0,n.jsxs)(r.p,{language:`txt`,children:[`Workspace 共享的依赖包括：`,(0,n.jsx)(r.code,{language:`txt`,children:`tokio`}),`、`,(0,n.jsx)(r.code,{language:`txt`,children:`serde`}),`、`,(0,n.jsx)(r.code,{language:`txt`,children:`clap`}),`、`,(0,n.jsx)(r.code,{language:`txt`,children:`ratatui`}),`、`,(0,n.jsx)(r.code,{language:`txt`,children:`crossterm`}),`、`,(0,n.jsx)(r.code,{language:`txt`,children:`ssh2`}),` 等。`]}),`
`,(0,n.jsx)(r.h2,{id:`数据流`,children:`数据流`}),`
`,(0,n.jsx)(r.h3,{id:`cli-模式`,children:`CLI 模式`}),`
`,(0,n.jsx)(r.pre,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`用户输入 → clap parse → Command dispatch → cmd/ 实现 → config/ 读写 → 终端输出
`})}),`
`,(0,n.jsxs)(r.ol,{children:[`
`,(0,n.jsxs)(r.li,{children:[(0,n.jsx)(r.a,{href:`/qssh/src/main.rs`,language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`main.rs`})}),` 解析命令行参数`]}),`
`,(0,n.jsxs)(r.li,{children:[`匹配到对应 `,(0,n.jsx)(r.a,{href:`/qssh/src/cli.rs:32`,language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`Command`})}),` 枚举变体`]}),`
`,(0,n.jsxs)(r.li,{language:`txt`,children:[`调用 `,(0,n.jsx)(r.code,{language:`txt`,children:`cmd/`}),` 下对应的实现模块`]}),`
`,(0,n.jsxs)(r.li,{language:`txt`,children:[`各模块通过 `,(0,n.jsx)(r.code,{language:`txt`,children:`config/`}),` 模块读写 SSH 配置文件`]}),`
`,(0,n.jsx)(r.li,{children:`结果输出到终端`}),`
`]}),`
`,(0,n.jsx)(r.h3,{id:`tui-模式`,children:`TUI 模式`}),`
`,(0,n.jsx)(r.pre,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`键盘事件 → keymap::map_key_to_action() → Action → App::apply() → State 更新 → ui::render()\r
           ↑                                                              ↓\r
           └────────────────── 事件循环 (100ms tick) ──────────────────────┘
`})}),`
`,(0,n.jsxs)(r.ol,{children:[`
`,(0,n.jsx)(r.li,{children:`事件循环以 100ms 为周期轮询键盘输入`}),`
`,(0,n.jsxs)(r.li,{children:[`按键通过 `,(0,n.jsx)(r.a,{href:`/qssh/src/tui/keymap.rs`,language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`keymap.rs`})}),` 映射为 `,(0,n.jsx)(r.a,{href:`/qssh/src/tui/action.rs`,language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`Action`})}),` 枚举`]}),`
`,(0,n.jsxs)(r.li,{children:[`Action 通过 `,(0,n.jsx)(r.a,{href:`/qssh/src/tui/app.rs:168`,language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`App::apply()`})}),` 更新应用状态`]}),`
`,(0,n.jsxs)(r.li,{children:[`状态变更后触发 `,(0,n.jsx)(r.a,{href:`/qssh/src/tui/ui.rs`,language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`ui::render()`})}),` 重新绘制界面`]}),`
`,(0,n.jsx)(r.li,{children:`后台任务（如 Ping 检测）通过 channel 异步通信`}),`
`]}),`
`,(0,n.jsx)(r.h3,{id:`ssh-连接`,children:`SSH 连接`}),`
`,(0,n.jsx)(r.pre,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`目标输入 → resolve_target() → SshTarget → 查询系统凭据 → 配置 AskPass（可选）→ spawn ssh → SSH 交互
`})}),`
`,(0,n.jsxs)(r.ol,{children:[`
`,(0,n.jsxs)(r.li,{language:`txt`,children:[`用户输入目标（别名或 `,(0,n.jsx)(r.code,{language:`txt`,children:`user@host`}),`）`]}),`
`,(0,n.jsxs)(r.li,{children:[(0,n.jsx)(r.a,{href:`/qssh/src/ssh/session.rs:78`,language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`resolve_target()`})}),` 优先作为别名查找，否则视为直接连接`]}),`
`,(0,n.jsx)(r.li,{children:`按目标别名查询系统安全凭据库；失败时警告并回退到普通 SSH`}),`
`,(0,n.jsxs)(r.li,{language:`txt`,children:[`构建 SSH 命令行参数（包括 `,(0,n.jsx)(r.code,{language:`txt`,children:`-tt`}),` 强制 PTY），有密码时配置 OpenSSH AskPass`]}),`
`,(0,n.jsx)(r.li,{children:`启动 SSH 子进程：Unix 继承标准流，Windows 继承 stdin 并转发 stdout/stderr`}),`
`,(0,n.jsx)(r.li,{children:`SSH 退出后返回退出码`}),`
`]}),`
`,(0,n.jsx)(r.h3,{id:`文件上传`,children:`文件上传`}),`
`,(0,n.jsx)(r.pre,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`拖拽检测 → UploadPayload → spawn qssh-uploader → SFTP 连接 → 进度条 → 完成
`})}),`
`,(0,n.jsxs)(r.ol,{children:[`
`,(0,n.jsxs)(r.li,{children:[`SSH 会话期间，`,(0,n.jsx)(r.a,{href:`/qssh/src/ssh/drag_detect.rs`,language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`drag_detect.rs`})}),` 检测终端输入中的文件路径`]}),`
`,(0,n.jsxs)(r.li,{language:`txt`,children:[`检测到拖拽操作 → 在新控制台窗口中启动 `,(0,n.jsx)(r.code,{language:`txt`,children:`qssh-uploader`})]}),`
`,(0,n.jsxs)(r.li,{language:`txt`,children:[`上传器使用 `,(0,n.jsx)(r.code,{language:`txt`,children:`scp`}),` 实现文件传输，最多 3 文件并发`]}),`
`,(0,n.jsx)(r.li,{children:`实时渲染每个文件的进度条`}),`
`]}),`
`,(0,n.jsx)(r.h2,{id:`设计决策`,children:`设计决策`}),`
`,(0,n.jsx)(r.h3,{id:`1-渐进式-ssh-解析`,children:`1. 渐进式 SSH 解析`}),`
`,(0,n.jsxs)(r.p,{language:`txt`,children:[`只解析 `,(0,n.jsx)(r.code,{language:`txt`,children:`Host`}),` / `,(0,n.jsx)(r.code,{language:`txt`,children:`HostName`}),` / `,(0,n.jsx)(r.code,{language:`txt`,children:`User`}),` / `,(0,n.jsx)(r.code,{language:`txt`,children:`Port`}),` / `,(0,n.jsx)(r.code,{language:`txt`,children:`IdentityFile`}),` 五个字段，其余指令保留为 `,(0,n.jsx)(r.code,{language:`txt`,children:`Unknown(key, value)`}),`。`,(0,n.jsx)(r.code,{language:`txt`,children:`Match`}),`、`,(0,n.jsx)(r.code,{language:`txt`,children:`Include`}),` 等复杂指令保留在 `,(0,n.jsx)(r.code,{language:`txt`,children:`preamble`}),` 中。这种设计保证了信息的完整性——无论配置文件中有什么内容，读写一遍后不会丢失任何信息。`]}),`
`,(0,n.jsx)(r.h3,{id:`2-事件驱动-tui`,children:`2. 事件驱动 TUI`}),`
`,(0,n.jsxs)(r.p,{language:`txt`,children:[`采用 `,(0,n.jsx)(r.code,{language:`txt`,children:`Event → Action → State → Render`}),` 的单向数据流架构：`]}),`
`,(0,n.jsxs)(r.ul,{children:[`
`,(0,n.jsxs)(r.li,{children:[(0,n.jsx)(r.strong,{children:`Action 是纯数据`}),`：易于测试和序列化`]}),`
`,(0,n.jsxs)(r.li,{children:[(0,n.jsx)(r.strong,{children:`State 是唯一数据源`}),`：不会有多个地方维护状态副本`]}),`
`,(0,n.jsxs)(r.li,{children:[(0,n.jsx)(r.strong,{children:`不直接通过按键回调修改 UI`}),`：所有状态变更都通过 Action 分发`]}),`
`]}),`
`,(0,n.jsx)(r.h3,{id:`3-spawn-ssh复用系统-openssh`,children:`3. Spawn SSH（复用系统 OpenSSH）`}),`
`,(0,n.jsx)(r.p,{children:`不自行实现 SSH 协议，而是复用系统自带的 OpenSSH 客户端：`}),`
`,(0,n.jsxs)(r.ul,{children:[`
`,(0,n.jsx)(r.li,{children:`支持所有 SSH 特性（跳板机、多因子认证、SSH 代理等）`}),`
`,(0,n.jsx)(r.li,{children:`减少依赖和攻击面`}),`
`,(0,n.jsx)(r.li,{children:`无需处理 SSH 协议的复杂性`}),`
`,(0,n.jsx)(r.li,{children:`通过 stdin 继承保留完整交互能力`}),`
`,(0,n.jsx)(r.li,{children:`通过标准 AskPass 协议按需提供系统凭据库中的登录密码`}),`
`]}),`
`,(0,n.jsx)(r.h3,{id:`4-密码与-ssh-配置分离`,children:`4. 密码与 SSH 配置分离`}),`
`,(0,n.jsxs)(r.p,{language:`txt`,children:[`密码不写入 `,(0,n.jsx)(r.code,{language:`txt`,children:`~/.ssh/config`}),`，而是使用 `,(0,n.jsx)(r.code,{language:`txt`,children:`keyring`}),` crate 保存到 Windows Credential Manager、macOS Keychain 或 Linux Secret Service/keyutils。凭据以主机别名为键，并在主机重命名或删除时同步迁移或清理。AskPass 只响应登录密码提示，不自动处理私钥口令与主机指纹确认。`]}),`
`,(0,n.jsx)(r.h3,{id:`5-config-模块类型解析渲染分离`,children:`5. Config 模块类型/解析/渲染分离`}),`
`,(0,n.jsxs)(r.ul,{children:[`
`,(0,n.jsxs)(r.li,{children:[(0,n.jsx)(r.a,{href:`/qssh/src/config/types.rs`,language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`types.rs`})}),`：只定义数据结构和查询方法`]}),`
`,(0,n.jsxs)(r.li,{children:[(0,n.jsx)(r.a,{href:`/qssh/src/config/parser.rs`,language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`parser.rs`})}),`：只负责文本→数据结构`]}),`
`,(0,n.jsxs)(r.li,{children:[(0,n.jsx)(r.a,{href:`/qssh/src/config/writer.rs`,language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`writer.rs`})}),`：只负责数据结构→文本渲染`]}),`
`,(0,n.jsxs)(r.li,{children:[(0,n.jsx)(r.a,{href:`/qssh/src/config/credentials.rs`,language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`credentials.rs`})}),`：只负责系统凭据和 AskPass 请求`]}),`
`,(0,n.jsxs)(r.li,{children:[(0,n.jsx)(r.a,{href:`/qssh/src/config/settings.rs`,language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`settings.rs`})}),`：程序自身配置（独立于 SSH 配置）`]}),`
`]}),`
`,(0,n.jsx)(r.h3,{id:`6-跨平台终端策略`,children:`6. 跨平台终端策略`}),`
`,(0,n.jsxs)(r.p,{language:`txt`,children:[`SSH 会话在 Unix 上直接继承终端标准流；Windows 平台使用 WinAPI 操作控制台模式（`,(0,n.jsx)(r.code,{language:`txt`,children:`GetConsoleMode`}),` / `,(0,n.jsx)(r.code,{language:`txt`,children:`SetConsoleMode`}),`），并用线程转发 stdout/stderr。Windows 实现确保了：`]}),`
`,(0,n.jsxs)(r.ul,{children:[`
`,(0,n.jsx)(r.li,{children:`禁用 ECHO、LINE_INPUT、PROCESSED_INPUT`}),`
`,(0,n.jsxs)(r.li,{language:`txt`,children:[`启用 ENABLE_VIRTUAL_TERMINAL_INPUT（使 `,(0,n.jsx)(r.code,{language:`txt`,children:`std::io::stdin().read()`}),` 能正确读取按键字节序列）`]}),`
`,(0,n.jsx)(r.li,{children:`退出时三种方法确保恢复光标可见性`}),`
`]})]})}function i(t={}){let{wrapper:i}={...e(),...t.components};return i?(0,n.jsx)(i,{...t,children:(0,n.jsx)(r,{...t})}):r(t)}function a(e,t){throw Error(`Expected `+(t?`component`:`object`)+" `"+e+"` to be defined: you likely forgot to import, pass, or provide it.")}export{i as default};