import{n as e}from"./vendor-CQh2k-GV.js";import{n as t}from"./react-vendor-6GaKtW3l.js";var n=t();function r(t){let r={a:`a`,code:`code`,h1:`h1`,h2:`h2`,h3:`h3`,hr:`hr`,li:`li`,ol:`ol`,p:`p`,pre:`pre`,span:`span`,strong:`strong`,ul:`ul`,...e(),...t.components};return(0,n.jsxs)(n.Fragment,{children:[(0,n.jsx)(r.hr,{}),`
`,(0,n.jsx)(r.p,{children:`title: 网络工具模块
description: Quick-SSH 网络工具模块的文档，包含 TCP Ping 检测功能。
keywords:`}),`
`,(0,n.jsxs)(r.ul,{children:[`
`,(0,n.jsx)(r.li,{children:`network`}),`
`,(0,n.jsx)(r.li,{children:`ping`}),`
`,(0,n.jsx)(r.li,{children:`tcp`}),`
`,(0,n.jsx)(r.li,{children:`检测`}),`
`]}),`
`,(0,n.jsx)(r.hr,{}),`
`,(0,n.jsx)(r.h1,{id:`网络工具模块`,children:`网络工具模块`}),`
`,(0,n.jsxs)(r.p,{children:[`网络工具模块位于 `,(0,n.jsx)(r.a,{href:`/qssh/src/network/`,language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`qssh/src/network/`})}),`，提供 TCP 连接检测功能。`]}),`
`,(0,n.jsx)(r.h2,{id:`模块结构`,children:`模块结构`}),`
`,(0,n.jsx)(r.pre,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`network/
├── mod.rs      # 模块声明
└── ping.rs     # TCP Ping 检测
`})}),`
`,(0,n.jsx)(r.h2,{id:`pingrs--tcp-连接测试`,children:`ping.rs — TCP 连接测试`}),`
`,(0,n.jsxs)(r.p,{language:`txt`,children:[(0,n.jsx)(r.code,{language:`txt`,children:`check_host()`}),` 函数提供基于 TCP 连接的超时检测：`]}),`
`,(0,n.jsx)(n.Fragment,{children:(0,n.jsx)(r.pre,{className:`shiki css-variables`,style:{backgroundColor:`var(--shiki-background)`,color:`var(--shiki-foreground)`},tabIndex:`0`,children:(0,n.jsxs)(r.code,{className:`language-rust`,children:[(0,n.jsxs)(r.span,{className:`line`,children:[(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-keyword)`},children:`pub`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-keyword)`},children:` fn`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-function)`},children:` check_host`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`(hostname`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-keyword)`},children:`:`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-keyword)`},children:` &`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-function)`},children:`str`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`, port`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-keyword)`},children:`:`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-function)`},children:` u16`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`, timeout_secs`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-keyword)`},children:`:`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-function)`},children:` u64`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`) `}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-keyword)`},children:`->`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-function)`},children:` Result`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`<`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-function)`},children:`bool`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`> {`})]}),`
`,(0,n.jsxs)(r.span,{className:`line`,children:[(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-keyword)`},children:`    let`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:` addr `}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-keyword)`},children:`=`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-function)`},children:` format!`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`(`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string-expression)`},children:`"{}:{}"`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`, hostname, port);`})]}),`
`,(0,n.jsxs)(r.span,{className:`line`,children:[(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-keyword)`},children:`    let`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-keyword)`},children:` mut`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:` addrs `}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-keyword)`},children:`=`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:` addr`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-keyword)`},children:`.`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-function)`},children:`to_socket_addrs`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`()`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-keyword)`},children:`?`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`;`})]}),`
`,(0,n.jsxs)(r.span,{className:`line`,children:[(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-keyword)`},children:`    let`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:` addr `}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-keyword)`},children:`=`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:` addrs`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-keyword)`},children:`.`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-function)`},children:`next`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`()`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-keyword)`},children:`?`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`;`})]}),`
`,(0,n.jsxs)(r.span,{className:`line`,children:[(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-keyword)`},children:`    match`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-function)`},children:` TcpStream`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-keyword)`},children:`::`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-function)`},children:`connect_timeout`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`(`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-keyword)`},children:`&`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`addr, Duration`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-keyword)`},children:`::`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-function)`},children:`from_secs`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`(timeout_secs)) {`})]}),`
`,(0,n.jsxs)(r.span,{className:`line`,children:[(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-function)`},children:`        Ok`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`(_) `}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-keyword)`},children:`=>`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-function)`},children:` Ok`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`(`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-constant)`},children:`true`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`),     `}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-comment)`},children:`// 连接成功 → 在线`})]}),`
`,(0,n.jsxs)(r.span,{className:`line`,children:[(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-function)`},children:`        Err`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`(_) `}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-keyword)`},children:`=>`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-function)`},children:` Ok`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`(`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-constant)`},children:`false`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`),   `}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-comment)`},children:`// 连接失败 → 离线`})]}),`
`,(0,n.jsx)(r.span,{className:`line`,children:(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`    }`})}),`
`,(0,n.jsx)(r.span,{className:`line`,children:(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`}`})})]})})}),`
`,(0,n.jsx)(r.h3,{id:`工作流程`,children:`工作流程`}),`
`,(0,n.jsxs)(r.ol,{children:[`
`,(0,n.jsxs)(r.li,{language:`txt`,children:[(0,n.jsx)(r.strong,{children:`地址解析`}),`：将 `,(0,n.jsx)(r.code,{language:`txt`,children:`hostname:port`}),` 通过 `,(0,n.jsx)(r.code,{language:`txt`,children:`ToSocketAddrs`}),` 解析为 SocketAddr`]}),`
`,(0,n.jsxs)(r.li,{language:`txt`,children:[(0,n.jsx)(r.strong,{children:`超时连接`}),`：使用 `,(0,n.jsx)(r.code,{language:`txt`,children:`TcpStream::connect_timeout()`}),` 尝试建立 TCP 连接`]}),`
`,(0,n.jsxs)(r.li,{children:[(0,n.jsx)(r.strong,{children:`结果判定`}),`：`,`
`,(0,n.jsxs)(r.ul,{children:[`
`,(0,n.jsxs)(r.li,{language:`txt`,children:[`连接成功 → 返回 `,(0,n.jsx)(r.code,{language:`txt`,children:`true`}),`（在线）`]}),`
`,(0,n.jsxs)(r.li,{language:`txt`,children:[`连接失败（超时/拒绝/无路由）→ 返回 `,(0,n.jsx)(r.code,{language:`txt`,children:`false`}),`（离线）`]}),`
`]}),`
`]}),`
`]}),`
`,(0,n.jsx)(r.h3,{id:`调用场景`,children:`调用场景`}),`
`,(0,n.jsx)(r.p,{children:`Ping 检测在 TUI 中通过后台线程调用：`}),`
`,(0,n.jsxs)(r.ul,{children:[`
`,(0,n.jsxs)(r.li,{language:`txt`,children:[`单机检测：按 `,(0,n.jsx)(r.code,{language:`txt`,children:`p`}),` 键检测当前选中主机`]}),`
`,(0,n.jsxs)(r.li,{language:`txt`,children:[`全量检测：按 `,(0,n.jsx)(r.code,{language:`txt`,children:`P`}),` 键检测所有主机`]}),`
`]}),`
`,(0,n.jsxs)(r.p,{language:`txt`,children:[`默认超时时间为 3 秒，可通过 `,(0,n.jsx)(r.code,{language:`txt`,children:`~/.qsshrc`}),` 的 `,(0,n.jsx)(r.code,{language:`txt`,children:`ping_timeout_secs`}),` 配置。`]}),`
`,(0,n.jsx)(r.h3,{id:`设计说明`,children:`设计说明`}),`
`,(0,n.jsxs)(r.ul,{children:[`
`,(0,n.jsxs)(r.li,{children:[(0,n.jsx)(r.strong,{children:`纯 TCP 检测`}),`：不依赖 ICMP 协议（Windows 上不需要管理员权限）`]}),`
`,(0,n.jsxs)(r.li,{children:[(0,n.jsx)(r.strong,{children:`非阻塞`}),`：通过后台线程异步执行，不阻塞 TUI 事件循环`]}),`
`,(0,n.jsxs)(r.li,{language:`txt`,children:[(0,n.jsx)(r.strong,{children:`结果缓存`}),`：检测结果存储在 `,(0,n.jsx)(r.code,{language:`txt`,children:`App.host_status`}),` 中，实时更新 UI 图标`]}),`
`,(0,n.jsxs)(r.li,{language:`txt`,children:[(0,n.jsx)(r.strong,{children:`防重复检测`}),`：`,(0,n.jsx)(r.code,{language:`txt`,children:`App.pending_pings`}),` 集合防止同一主机重复检测`]}),`
`]})]})}function i(t={}){let{wrapper:i}={...e(),...t.components};return i?(0,n.jsx)(i,{...t,children:(0,n.jsx)(r,{...t})}):r(t)}export{i as default};