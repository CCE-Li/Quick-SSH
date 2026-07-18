import{n as e}from"./vendor-CQh2k-GV.js";import{n as t}from"./react-vendor-6GaKtW3l.js";var n=t();function r(t){let r={code:`code`,h1:`h1`,h2:`h2`,hr:`hr`,input:`input`,li:`li`,p:`p`,ul:`ul`,...e(),...t.components};return(0,n.jsxs)(n.Fragment,{children:[(0,n.jsx)(r.hr,{}),`
`,(0,n.jsx)(r.p,{children:`title: 路线图
description: Quick-SSH 的开发路线图，包含当前版本、规划中的功能以及远期规划。
keywords:`}),`
`,(0,n.jsxs)(r.ul,{children:[`
`,(0,n.jsx)(r.li,{children:`路线图`}),`
`,(0,n.jsx)(r.li,{children:`roadmap`}),`
`,(0,n.jsx)(r.li,{children:`规划`}),`
`,(0,n.jsx)(r.li,{children:`版本`}),`
`]}),`
`,(0,n.jsx)(r.hr,{}),`
`,(0,n.jsx)(r.h1,{id:`路线图`,children:`路线图`}),`
`,(0,n.jsx)(r.h2,{id:`v20当前版本`,children:`v2.0（当前版本）`}),`
`,(0,n.jsx)(r.p,{children:`已完成的功能：`}),`
`,(0,n.jsxs)(r.ul,{className:`contains-task-list`,children:[`
`,(0,n.jsxs)(r.li,{className:`task-list-item`,children:[(0,n.jsx)(r.input,{type:`checkbox`,checked:!0,disabled:!0}),` `,`Rust 完整重写（移除 Node.js 依赖）`]}),`
`,(0,n.jsxs)(r.li,{className:`task-list-item`,children:[(0,n.jsx)(r.input,{type:`checkbox`,checked:!0,disabled:!0}),` `,`事件驱动 TUI（ratatui + crossterm）`]}),`
`,(0,n.jsxs)(r.li,{className:`task-list-item`,children:[(0,n.jsx)(r.input,{type:`checkbox`,checked:!0,disabled:!0}),` `,`完整 CLI 子命令（ps/add/rm/connect/export/import/help）`]}),`
`,(0,n.jsxs)(r.li,{className:`task-list-item`,children:[(0,n.jsx)(r.input,{type:`checkbox`,checked:!0,disabled:!0}),` `,`Shell 补全生成（bash/zsh/fish/powershell/elvish）`]}),`
`,(0,n.jsxs)(r.li,{className:`task-list-item`,children:[(0,n.jsx)(r.input,{type:`checkbox`,checked:!0,disabled:!0}),` `,`独立 SFTP 上传工具（qssh-uploader）`]}),`
`,(0,n.jsxs)(r.li,{className:`task-list-item`,children:[(0,n.jsx)(r.input,{type:`checkbox`,checked:!0,disabled:!0}),` `,`渐进式 SSH 配置解析`]}),`
`,(0,n.jsxs)(r.li,{className:`task-list-item`,language:`txt`,children:[(0,n.jsx)(r.input,{type:`checkbox`,checked:!0,disabled:!0}),` `,(0,n.jsx)(r.code,{language:`txt`,children:`~/.qsshrc`}),` 程序设置`]}),`
`,(0,n.jsxs)(r.li,{className:`task-list-item`,children:[(0,n.jsx)(r.input,{type:`checkbox`,checked:!0,disabled:!0}),` `,`6 种包管理器配置（Scoop/WinGet/Homebrew/AUR/APT）`]}),`
`,(0,n.jsxs)(r.li,{className:`task-list-item`,children:[(0,n.jsx)(r.input,{type:`checkbox`,checked:!0,disabled:!0}),` `,`CI/CD 自动构建与发布`]}),`
`]}),`
`,(0,n.jsx)(r.h2,{id:`v21规划中`,children:`v2.1（规划中）`}),`
`,(0,n.jsx)(r.p,{children:`计划中的功能：`}),`
`,(0,n.jsxs)(r.ul,{className:`contains-task-list`,children:[`
`,(0,n.jsxs)(r.li,{className:`task-list-item`,children:[(0,n.jsx)(r.input,{type:`checkbox`,disabled:!0}),` `,`批量命令执行（选中多台主机后执行命令）`]}),`
`,(0,n.jsxs)(r.li,{className:`task-list-item`,children:[(0,n.jsx)(r.input,{type:`checkbox`,disabled:!0}),` `,`自动检测更新（启动时检查 GitHub Release）`]}),`
`,(0,n.jsxs)(r.li,{className:`task-list-item`,children:[(0,n.jsx)(r.input,{type:`checkbox`,disabled:!0}),` `,`连接日志记录（会话时间、主机、退出码）`]}),`
`,(0,n.jsxs)(r.li,{className:`task-list-item`,children:[(0,n.jsx)(r.input,{type:`checkbox`,disabled:!0}),` `,`TUI 主机详情增强（显示配置片段）`]}),`
`]}),`
`,(0,n.jsx)(r.h2,{id:`v22远期规划`,children:`v2.2（远期规划）`}),`
`,(0,n.jsx)(r.p,{children:`远期考虑的功能：`}),`
`,(0,n.jsxs)(r.ul,{className:`contains-task-list`,children:[`
`,(0,n.jsxs)(r.li,{className:`task-list-item`,children:[(0,n.jsx)(r.input,{type:`checkbox`,disabled:!0}),` `,`TUI 主题自定义`]}),`
`,(0,n.jsxs)(r.li,{className:`task-list-item`,children:[(0,n.jsx)(r.input,{type:`checkbox`,disabled:!0}),` `,`主机分组/标签`]}),`
`,(0,n.jsxs)(r.li,{className:`task-list-item`,children:[(0,n.jsx)(r.input,{type:`checkbox`,disabled:!0}),` `,`配置备份与恢复`]}),`
`,(0,n.jsxs)(r.li,{className:`task-list-item`,children:[(0,n.jsx)(r.input,{type:`checkbox`,disabled:!0}),` `,`SSH 隧道管理`]}),`
`,(0,n.jsxs)(r.li,{className:`task-list-item`,children:[(0,n.jsx)(r.input,{type:`checkbox`,disabled:!0}),` `,`更多包管理器支持（Chocolatey、Nix）`]}),`
`]})]})}function i(t={}){let{wrapper:i}={...e(),...t.components};return i?(0,n.jsx)(i,{...t,children:(0,n.jsx)(r,{...t})}):r(t)}export{i as default};