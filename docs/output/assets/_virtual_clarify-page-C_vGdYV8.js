import{n as e}from"./vendor-CQh2k-GV.js";import{n as t}from"./react-vendor-6GaKtW3l.js";var n=t();function r(t){let r={h1:`h1`,h2:`h2`,hr:`hr`,li:`li`,p:`p`,strong:`strong`,ul:`ul`,...e(),...t.components},{Callout:i,Card:o,CardGroup:s,Properties:c,Property:l}=r;return i||a(`Callout`,!0),o||a(`Card`,!0),s||a(`CardGroup`,!0),c||a(`Properties`,!0),l||a(`Property`,!0),(0,n.jsxs)(n.Fragment,{children:[(0,n.jsx)(r.hr,{}),`
`,(0,n.jsx)(r.p,{children:`title: 概述
description: Quick-SSH — 跨平台 SSH 连接管理工具，提供 TUI 界面与 Docker 风格 CLI，基于 Rust 实现。
keywords:`}),`
`,(0,n.jsxs)(r.ul,{children:[`
`,(0,n.jsx)(r.li,{children:`quick-ssh`}),`
`,(0,n.jsx)(r.li,{children:`ssh`}),`
`,(0,n.jsx)(r.li,{children:`终端工具`}),`
`,(0,n.jsx)(r.li,{children:`rust`}),`
`,(0,n.jsx)(r.li,{children:`连接管理`}),`
`]}),`
`,(0,n.jsx)(r.hr,{}),`
`,(0,n.jsx)(r.h1,{id:`quick-ssh-文档`,children:`Quick-SSH 文档`}),`
`,(0,n.jsxs)(r.p,{children:[(0,n.jsx)(r.strong,{children:`Quick-SSH`}),` 是一个跨平台 SSH 连接管理工具，提供 `,(0,n.jsx)(r.strong,{children:`TUI 界面`}),`与 `,(0,n.jsx)(r.strong,{children:`Docker 风格 CLI`}),` 双模式操作。基于 Rust 实现，无需 Node.js 运行时，即下即用。`]}),`
`,(0,n.jsx)(i,{title:`项目状态`,children:(0,n.jsx)(r.p,{children:`Quick-SSH v2.0 已完成 Rust 完整重写，支持 Windows / Linux / macOS 三大平台，提供 6 种包管理器安装方式。`})}),`
`,(0,n.jsx)(r.h2,{id:`核心特性`,children:`核心特性`}),`
`,(0,n.jsxs)(s,{cols:2,children:[(0,n.jsx)(o,{title:`TUI 界面`,icon:`LayoutPanelLeft`,children:(0,n.jsx)(r.p,{children:`事件驱动的终端 UI，支持主机列表浏览、搜索、标记、连接和 Ping 检测，支持新增/编辑主机弹窗表单。`})}),(0,n.jsx)(o,{title:`CLI 命令`,icon:`Terminal`,children:(0,n.jsx)(r.p,{children:`Docker 风格的命令行接口，支持 ps/add/rm/connect/export/import 等子命令，一键连接主机。`})}),(0,n.jsx)(o,{title:`文件拖拽上传`,icon:`Upload`,children:(0,n.jsx)(r.p,{children:`SSH 连接后，将文件或目录拖入终端窗口即可自动启动 SFTP 上传，显示每文件进度条和总进度。`})}),(0,n.jsx)(o,{title:`渐进式配置解析`,icon:`FileJson`,children:(0,n.jsx)(r.p,{children:`兼容标准 OpenSSH 格式，只管理 Host/HostName/User/Port/IdentityFile，其余指令完整保留。`})}),(0,n.jsx)(o,{title:`多包管理器支持`,icon:`Blocks`,children:(0,n.jsx)(r.p,{children:`支持 Scoop、WinGet、Homebrew、AUR、APT 等 6 种包管理器，一键安装。`})}),(0,n.jsx)(o,{title:`纯 Rust 实现`,icon:`Code`,children:(0,n.jsx)(r.p,{children:`单一二进制文件，无外部运行时依赖，高性能低内存，跨平台编译。`})})]}),`
`,(0,n.jsx)(r.h2,{id:`快速导航`,children:`快速导航`}),`
`,(0,n.jsxs)(s,{cols:3,children:[(0,n.jsx)(o,{title:`安装指南`,icon:`Download`,href:`/installation`,children:(0,n.jsx)(r.p,{children:`通过包管理器或直接下载安装 Quick-SSH`})}),(0,n.jsx)(o,{title:`快速入门`,icon:`Play`,href:`/getting-started`,children:(0,n.jsx)(r.p,{children:`5 分钟上手 TUI 和 CLI 基本操作`})}),(0,n.jsx)(o,{title:`CLI 命令参考`,icon:`Terminal`,href:`/cli-reference`,children:(0,n.jsx)(r.p,{children:`完整的子命令参考文档`})}),(0,n.jsx)(o,{title:`架构设计`,icon:`Layers`,href:`/architecture`,children:(0,n.jsx)(r.p,{children:`Rust 模块化架构设计详解`})}),(0,n.jsx)(o,{title:`发布流程`,icon:`Rocket`,href:`/release-process`,children:(0,n.jsx)(r.p,{children:`版本发布与包管理器维护指南`})}),(0,n.jsx)(o,{title:`路线图`,icon:`Map`,href:`/roadmap`,children:(0,n.jsx)(r.p,{children:`当前版本与未来规划`})})]}),`
`,(0,n.jsx)(r.h2,{id:`技术栈`,children:`技术栈`}),`
`,(0,n.jsxs)(c,{children:[(0,n.jsx)(l,{name:`语言`,type:`Rust`,children:(0,n.jsx)(r.p,{children:`基于 Rust 2021 Edition 编写，使用 workspace 管理多 crate`})}),(0,n.jsx)(l,{name:`TUI 框架`,type:`ratatui`,children:(0,n.jsx)(r.p,{children:`使用 ratatui 0.29 + crossterm 0.28 构建事件驱动终端界面`})}),(0,n.jsx)(l,{name:`SSH 协议`,type:`OpenSSH`,children:(0,n.jsx)(r.p,{children:`复用系统 OpenSSH 客户端（ssh/scp），不自行实现 SSH 协议`})}),(0,n.jsx)(l,{name:`CLI 框架`,type:`clap`,children:(0,n.jsx)(r.p,{children:`使用 clap 4 的 derive 模式定义命令行接口，支持自动补全`})}),(0,n.jsx)(l,{name:`配置格式`,type:`OpenSSH`,children:(0,n.jsx)(r.p,{children:`兼容 ~/.ssh/config 标准格式，渐进式解析`})})]})]})}function i(t={}){let{wrapper:i}={...e(),...t.components};return i?(0,n.jsx)(i,{...t,children:(0,n.jsx)(r,{...t})}):r(t)}function a(e,t){throw Error(`Expected `+(t?`component`:`object`)+" `"+e+"` to be defined: you likely forgot to import, pass, or provide it.")}export{i as default};