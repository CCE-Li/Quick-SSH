import{n as e}from"./vendor-CQh2k-GV.js";import{n as t}from"./react-vendor-6GaKtW3l.js";var n=t();function r(t){let r={a:`a`,code:`code`,h1:`h1`,h2:`h2`,h3:`h3`,hr:`hr`,li:`li`,ol:`ol`,p:`p`,pre:`pre`,span:`span`,strong:`strong`,table:`table`,tbody:`tbody`,td:`td`,th:`th`,thead:`thead`,tr:`tr`,ul:`ul`,...e(),...t.components};return(0,n.jsxs)(n.Fragment,{children:[(0,n.jsx)(r.hr,{}),`
`,(0,n.jsx)(r.p,{children:`title: SSH 连接模块
description: Quick-SSH SSH 模块的详细文档，包含会话管理、进程生成、拖拽检测和内联上传。
keywords:`}),`
`,(0,n.jsxs)(r.ul,{children:[`
`,(0,n.jsx)(r.li,{children:`ssh`}),`
`,(0,n.jsx)(r.li,{children:`session`}),`
`,(0,n.jsx)(r.li,{children:`spawn`}),`
`,(0,n.jsx)(r.li,{children:`拖拽检测`}),`
`,(0,n.jsx)(r.li,{children:`上传`}),`
`,(0,n.jsx)(r.li,{children:`scp`}),`
`]}),`
`,(0,n.jsx)(r.hr,{}),`
`,(0,n.jsx)(r.h1,{id:`ssh-连接模块`,children:`SSH 连接模块`}),`
`,(0,n.jsxs)(r.p,{children:[`SSH 模块位于 `,(0,n.jsx)(r.a,{href:`/qssh/src/ssh/`,language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`qssh/src/ssh/`})}),`，负责 SSH 连接的全流程管理，包括目标解析、进程生成、文件拖拽检测和内联上传。`]}),`
`,(0,n.jsx)(r.h2,{id:`模块结构`,children:`模块结构`}),`
`,(0,n.jsx)(r.pre,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`ssh/
├── mod.rs           # 模块声明
├── session.rs       # SshTarget 解析与 SSH 参数构建
├── spawn.rs         # 交互式 SSH 会话（跨平台 raw 模式）
├── drag_detect.rs   # 拖拽文件路径检测
└── upload.rs        # 内联 SCP 上传（预留）
`})}),`
`,(0,n.jsx)(r.h2,{id:`sessionrs--连接目标解析`,children:`session.rs — 连接目标解析`}),`
`,(0,n.jsx)(r.h3,{id:`sshtarget`,children:`SshTarget`}),`
`,(0,n.jsxs)(r.p,{language:`txt`,children:[(0,n.jsx)(r.code,{language:`txt`,children:`SshTarget`}),` 结构体表示已解析的 SSH 连接目标：`]}),`
`,(0,n.jsx)(n.Fragment,{children:(0,n.jsx)(r.pre,{className:`shiki css-variables`,style:{backgroundColor:`var(--shiki-background)`,color:`var(--shiki-foreground)`},tabIndex:`0`,children:(0,n.jsxs)(r.code,{className:`language-rust`,children:[(0,n.jsxs)(r.span,{className:`line`,children:[(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-keyword)`},children:`pub`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-keyword)`},children:` struct`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-function)`},children:` SshTarget`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:` {`})]}),`
`,(0,n.jsxs)(r.span,{className:`line`,children:[(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-keyword)`},children:`    pub`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:` alias`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-keyword)`},children:`:`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-function)`},children:` String`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`,`})]}),`
`,(0,n.jsxs)(r.span,{className:`line`,children:[(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-keyword)`},children:`    pub`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:` hostname`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-keyword)`},children:`:`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-function)`},children:` String`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`,`})]}),`
`,(0,n.jsxs)(r.span,{className:`line`,children:[(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-keyword)`},children:`    pub`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:` user`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-keyword)`},children:`:`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-function)`},children:` Option`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`<`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-function)`},children:`String`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`>,`})]}),`
`,(0,n.jsxs)(r.span,{className:`line`,children:[(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-keyword)`},children:`    pub`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:` port`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-keyword)`},children:`:`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-function)`},children:` u16`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`,`})]}),`
`,(0,n.jsxs)(r.span,{className:`line`,children:[(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-keyword)`},children:`    pub`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:` identity_file`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-keyword)`},children:`:`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-function)`},children:` Option`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`<`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-function)`},children:`PathBuf`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`>,`})]}),`
`,(0,n.jsx)(r.span,{className:`line`,children:(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`}`})})]})})}),`
`,(0,n.jsx)(r.h3,{id:`构建方式`,children:`构建方式`}),`
`,(0,n.jsxs)(r.table,{children:[(0,n.jsx)(r.thead,{children:(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.th,{children:`方法`}),(0,n.jsx)(r.th,{children:`来源`}),(0,n.jsx)(r.th,{children:`说明`})]})}),(0,n.jsxs)(r.tbody,{children:[(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`from_host(host)`})}),(0,n.jsx)(r.td,{children:`HostBlock`}),(0,n.jsx)(r.td,{children:`从配置文件中的 Host 块构建`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`from_user_at_host(input, port)`})}),(0,n.jsx)(r.td,{children:`用户输入`}),(0,n.jsxs)(r.td,{language:`txt`,children:[`从 `,(0,n.jsx)(r.code,{language:`txt`,children:`user@hostname`}),` 字符串解析`]})]})]})]}),`
`,(0,n.jsx)(r.h3,{id:`ssh-参数构建`,children:`SSH 参数构建`}),`
`,(0,n.jsxs)(r.p,{language:`txt`,children:[(0,n.jsx)(r.code,{language:`txt`,children:`build_ssh_args()`}),` 方法根据 SshTarget 构建 SSH 命令行参数：`]}),`
`,(0,n.jsx)(n.Fragment,{children:(0,n.jsx)(r.pre,{className:`shiki css-variables`,style:{backgroundColor:`var(--shiki-background)`,color:`var(--shiki-foreground)`},tabIndex:`0`,children:(0,n.jsxs)(r.code,{className:`language-rust`,children:[(0,n.jsx)(r.span,{className:`line`,children:(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-comment)`},children:`// 示例输出`})}),`
`,(0,n.jsxs)(r.span,{className:`line`,children:[(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`[`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string-expression)`},children:`"-l"`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`, `}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string-expression)`},children:`"root"`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`, `}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string-expression)`},children:`"-p"`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`, `}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string-expression)`},children:`"2222"`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`, `}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string-expression)`},children:`"-i"`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`, `}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string-expression)`},children:`"~/.ssh/id_rsa"`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`, `}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string-expression)`},children:`"-o"`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`, `}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string-expression)`},children:`"ControlMaster=no"`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`, `}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string-expression)`},children:`"example.com"`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`]`})]})]})})}),`
`,(0,n.jsx)(r.p,{children:`参数规则：`}),`
`,(0,n.jsxs)(r.table,{children:[(0,n.jsx)(r.thead,{children:(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.th,{children:`条件`}),(0,n.jsx)(r.th,{children:`参数`}),(0,n.jsx)(r.th,{children:`说明`})]})}),(0,n.jsxs)(r.tbody,{children:[(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{children:`有 user`}),(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`-l <user>`})}),(0,n.jsx)(r.td,{children:`指定登录用户名`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{children:`port != 22`}),(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`-p <port>`})}),(0,n.jsx)(r.td,{children:`指定端口`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{children:`有 identity_file`}),(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`-i <path>`})}),(0,n.jsxs)(r.td,{language:`txt`,children:[`指定密钥文件（自动展开 `,(0,n.jsx)(r.code,{language:`txt`,children:`~`}),`）`]})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{children:`始终`}),(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`-o ControlMaster=no`})}),(0,n.jsx)(r.td,{children:`禁用连接复用，确保新连接`})]})]})]}),`
`,(0,n.jsx)(r.h3,{id:`目标解析`,children:`目标解析`}),`
`,(0,n.jsxs)(r.p,{language:`txt`,children:[(0,n.jsx)(r.code,{language:`txt`,children:`resolve_target()`}),` 函数实现双模式解析：`]}),`
`,(0,n.jsxs)(r.ol,{children:[`
`,(0,n.jsxs)(r.li,{language:`txt`,children:[(0,n.jsx)(r.strong,{children:`别名模式`}),`：优先在 `,(0,n.jsx)(r.code,{language:`txt`,children:`~/.ssh/config`}),` 中查找匹配的 HostBlock`]}),`
`,(0,n.jsxs)(r.li,{language:`txt`,children:[(0,n.jsx)(r.strong,{children:`直接模式`}),`：未找到别名时，将输入视为 `,(0,n.jsx)(r.code,{language:`txt`,children:`user@hostname`}),` 直接连接`]}),`
`]}),`
`,(0,n.jsx)(r.h2,{id:`spawnrs--ssh-进程生成`,children:`spawn.rs — SSH 进程生成`}),`
`,(0,n.jsx)(r.h3,{id:`设计要点`,children:`设计要点`}),`
`,(0,n.jsxs)(r.ul,{children:[`
`,(0,n.jsxs)(r.li,{children:[(0,n.jsx)(r.strong,{children:`stdin 继承`}),`：SSH 直接读取控制台输入，密码认证正常工作`]}),`
`,(0,n.jsxs)(r.li,{children:[(0,n.jsx)(r.strong,{children:`stdout/stderr 转发`}),`：通过独立线程转发到终端显示`]}),`
`,(0,n.jsxs)(r.li,{language:`txt`,children:[(0,n.jsx)(r.strong,{children:`强制 PTY`}),`：使用 `,(0,n.jsx)(r.code,{language:`txt`,children:`-tt`}),` 确保 SSH 在远程分配 PTY`]}),`
`]}),`
`,(0,n.jsx)(r.h3,{id:`启动流程`,children:`启动流程`}),`
`,(0,n.jsx)(r.pre,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`start_interactive_session()
  → enable_terminal_raw_mode()    // 启用终端 raw 模式
  → start_interactive_session_inner()
    → build_ssh_args_with_pty()   // 构建带 -tt 的参数
    → Command::new("ssh").spawn() // 启动 SSH 进程
    → forward_stdout() (线程)     // stdout 转发
    → forward_stderr() (线程)     // stderr 转发
    → child.wait()                // 等待 SSH 退出
  → disable_terminal_raw_mode()   // 恢复终端模式
  → 返回退出码
`})}),`
`,(0,n.jsx)(r.h3,{id:`跨平台终端-raw-模式`,children:`跨平台终端 Raw 模式`}),`
`,(0,n.jsxs)(r.table,{children:[(0,n.jsx)(r.thead,{children:(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.th,{children:`平台`}),(0,n.jsx)(r.th,{children:`实现方式`})]})}),(0,n.jsxs)(r.tbody,{children:[(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{children:(0,n.jsx)(r.strong,{children:`Windows`})}),(0,n.jsxs)(r.td,{language:`txt`,children:[`使用 WinAPI `,(0,n.jsx)(r.code,{language:`txt`,children:`GetConsoleMode`}),` / `,(0,n.jsx)(r.code,{language:`txt`,children:`SetConsoleMode`}),` 直接操作控制台`]})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{children:(0,n.jsx)(r.strong,{children:`Unix`})}),(0,n.jsx)(r.td,{children:`使用 crossterm 的 termios 封装`})]})]})]}),`
`,(0,n.jsx)(r.p,{children:(0,n.jsx)(r.strong,{children:`Windows 实现细节：`})}),`
`,(0,n.jsxs)(r.ol,{children:[`
`,(0,n.jsx)(r.li,{children:`获取并保存原始控制台输入/输出模式`}),`
`,(0,n.jsxs)(r.li,{language:`txt`,children:[`禁用 `,(0,n.jsx)(r.code,{language:`txt`,children:`ENABLE_ECHO_INPUT`}),`、`,(0,n.jsx)(r.code,{language:`txt`,children:`ENABLE_LINE_INPUT`}),`、`,(0,n.jsx)(r.code,{language:`txt`,children:`ENABLE_PROCESSED_INPUT`})]}),`
`,(0,n.jsxs)(r.li,{language:`txt`,children:[`启用 `,(0,n.jsx)(r.code,{language:`txt`,children:`ENABLE_VIRTUAL_TERMINAL_INPUT`}),`（使 `,(0,n.jsx)(r.code,{language:`txt`,children:`std::io::stdin().read()`}),` 能正确读取按键）`]}),`
`,(0,n.jsxs)(r.li,{language:`txt`,children:[`启用输出句柄的 `,(0,n.jsx)(r.code,{language:`txt`,children:`ENABLE_VIRTUAL_TERMINAL_PROCESSING`})]}),`
`,(0,n.jsxs)(r.li,{children:[`退出时使用三种方法确保恢复光标可见性：`,`
`,(0,n.jsxs)(r.ul,{children:[`
`,(0,n.jsxs)(r.li,{language:`txt`,children:[(0,n.jsx)(r.code,{language:`txt`,children:`WriteConsoleW`}),` — ANSI 转义序列（`,(0,n.jsx)(r.code,{language:`txt`,children:`\\x1b[?25h`}),`）`]}),`
`,(0,n.jsxs)(r.li,{language:`txt`,children:[(0,n.jsx)(r.code,{language:`txt`,children:`SetConsoleCursorInfo`}),` — 直接 WinAPI 设置`]}),`
`,(0,n.jsxs)(r.li,{language:`txt`,children:[(0,n.jsx)(r.code,{language:`txt`,children:`WriteFile`}),` — 内核路径写入`]}),`
`]}),`
`]}),`
`]}),`
`,(0,n.jsx)(r.h2,{id:`drag_detectrs--拖拽文件检测`,children:`drag_detect.rs — 拖拽文件检测`}),`
`,(0,n.jsx)(r.h3,{id:`检测原理`,children:`检测原理`}),`
`,(0,n.jsx)(r.p,{children:`当用户在终端中拖拽文件时，终端模拟器会将文件路径"粘贴"到输入流中。检测模块识别这种拖拽行为。`}),`
`,(0,n.jsx)(r.h3,{id:`检测流程`,children:`检测流程`}),`
`,(0,n.jsx)(r.pre,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`输入文本 → strip_paste_markers() → tokenize() → 路径验证 → 存在性验证 → 返回文件列表
`})}),`
`,(0,n.jsx)(r.h3,{id:`关键函数`,children:`关键函数`}),`
`,(0,n.jsxs)(r.table,{children:[(0,n.jsx)(r.thead,{children:(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.th,{children:`函数`}),(0,n.jsx)(r.th,{children:`说明`})]})}),(0,n.jsxs)(r.tbody,{children:[(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`strip_paste_markers()`})}),(0,n.jsx)(r.td,{children:`去除终端粘贴模式标记（bracketed paste mode）`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`tokenize()`})}),(0,n.jsx)(r.td,{children:`分割文本为 token，支持引号包裹`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`looks_like_windows_path()`})}),(0,n.jsxs)(r.td,{language:`txt`,children:[`检测 `,(0,n.jsx)(r.code,{language:`txt`,children:`X:\\...`}),` 格式（仅 Windows）`]})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`looks_like_unix_path()`})}),(0,n.jsxs)(r.td,{language:`txt`,children:[`检测 `,(0,n.jsx)(r.code,{language:`txt`,children:`/...`}),` 格式（仅 Unix）`]})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`parse_windows_drag()`})}),(0,n.jsx)(r.td,{children:`解析 Windows 拖拽路径`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`parse_unix_drag()`})}),(0,n.jsx)(r.td,{children:`解析 Unix 拖拽路径`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`detect_drag_files()`})}),(0,n.jsx)(r.td,{children:`平台自适应入口函数`})]})]})]}),`
`,(0,n.jsx)(r.h3,{id:`防误判机制`,children:`防误判机制`}),`
`,(0,n.jsxs)(r.p,{language:`txt`,children:[`所有 token 都必须是合法的绝对路径才会判定为拖拽操作，防止将 `,(0,n.jsx)(r.code,{language:`txt`,children:`ls -la /home/user`}),` 等命令误判。`]}),`
`,(0,n.jsx)(r.h2,{id:`uploadrs--内联上传预留`,children:`upload.rs — 内联上传（预留）`}),`
`,(0,n.jsxs)(r.p,{language:`txt`,children:[`当前被 `,(0,n.jsx)(r.code,{language:`txt`,children:`#[allow(dead_code)]`}),` 标记，作为预留实现。使用 `,(0,n.jsx)(r.code,{language:`txt`,children:`scp`}),` 命令在同一终端内完成上传：`]}),`
`,(0,n.jsxs)(r.ul,{children:[`
`,(0,n.jsxs)(r.li,{language:`txt`,children:[`每个文件调用一次 `,(0,n.jsx)(r.code,{language:`txt`,children:`scp`})]}),`
`,(0,n.jsx)(r.li,{children:`上传完成后显示结果`}),`
`,(0,n.jsxs)(r.li,{language:`txt`,children:[`使用 `,(0,n.jsx)(r.code,{language:`txt`,children:`-q`}),` 静默模式（不自带进度），自行计算和显示进度`]}),`
`,(0,n.jsxs)(r.li,{language:`txt`,children:[`使用 `,(0,n.jsx)(r.code,{language:`txt`,children:`-o ControlMaster=no`}),` 避免与现有 SSH 会话冲突`]}),`
`]})]})}function i(t={}){let{wrapper:i}={...e(),...t.components};return i?(0,n.jsx)(i,{...t,children:(0,n.jsx)(r,{...t})}):r(t)}export{i as default};