import{n as e}from"./vendor-CQh2k-GV.js";import{n as t}from"./react-vendor-6GaKtW3l.js";var n=t();function r(t){let r={code:`code`,h1:`h1`,h2:`h2`,hr:`hr`,input:`input`,li:`li`,p:`p`,ul:`ul`,...e(),...t.components};return(0,n.jsxs)(n.Fragment,{children:[(0,n.jsx)(r.hr,{}),`
`,(0,n.jsx)(r.p,{children:`title: 路线图
description: Quick-SSH 的开发路线图，包含当前版本、开发中的功能以及远期规划。
keywords:`}),`
`,(0,n.jsxs)(r.ul,{children:[`
`,(0,n.jsx)(r.li,{children:`路线图`}),`
`,(0,n.jsx)(r.li,{children:`roadmap`}),`
`,(0,n.jsx)(r.li,{children:`规划`}),`
`,(0,n.jsx)(r.li,{children:`版本`}),`
`]}),`
`,(0,n.jsx)(r.hr,{}),`
`,(0,n.jsx)(r.h1,{id:`路线图`,children:`路线图`}),`
`,(0,n.jsx)(r.h2,{id:`v204当前发布版本`,children:`v2.0.4（当前发布版本）`}),`
`,(0,n.jsxs)(r.ul,{className:`contains-task-list`,children:[`
`,(0,n.jsxs)(r.li,{className:`task-list-item`,children:[(0,n.jsx)(r.input,{type:`checkbox`,checked:!0,disabled:!0}),` `,`Rust 完整重写（移除 Node.js 依赖）`]}),`
`,(0,n.jsxs)(r.li,{className:`task-list-item`,children:[(0,n.jsx)(r.input,{type:`checkbox`,checked:!0,disabled:!0}),` `,`事件驱动 TUI（ratatui + crossterm）`]}),`
`,(0,n.jsxs)(r.li,{className:`task-list-item`,children:[(0,n.jsx)(r.input,{type:`checkbox`,checked:!0,disabled:!0}),` `,`完整 CLI 子命令（ps/add/rm/connect/export/import/help/web）`]}),`
`,(0,n.jsxs)(r.li,{className:`task-list-item`,children:[(0,n.jsx)(r.input,{type:`checkbox`,checked:!0,disabled:!0}),` `,`Shell 补全生成（bash/zsh/fish/powershell/elvish）`]}),`
`,(0,n.jsxs)(r.li,{className:`task-list-item`,children:[(0,n.jsx)(r.input,{type:`checkbox`,checked:!0,disabled:!0}),` `,`独立 SCP 上传工具（qssh-uploader）`]}),`
`,(0,n.jsxs)(r.li,{className:`task-list-item`,children:[(0,n.jsx)(r.input,{type:`checkbox`,checked:!0,disabled:!0}),` `,`渐进式 SSH 配置解析`]}),`
`,(0,n.jsxs)(r.li,{className:`task-list-item`,children:[(0,n.jsx)(r.input,{type:`checkbox`,checked:!0,disabled:!0}),` `,`系统安全凭据库存储密码与 OpenSSH AskPass 自动填写`]}),`
`,(0,n.jsxs)(r.li,{className:`task-list-item`,children:[(0,n.jsx)(r.input,{type:`checkbox`,checked:!0,disabled:!0}),` `,`TUI 多行注释、认证方式详情与注释摘要`]}),`
`,(0,n.jsxs)(r.li,{className:`task-list-item`,children:[(0,n.jsx)(r.input,{type:`checkbox`,checked:!0,disabled:!0}),` `,`标记主机批量删除并同步清理密码`]}),`
`,(0,n.jsxs)(r.li,{className:`task-list-item`,children:[(0,n.jsx)(r.input,{type:`checkbox`,checked:!0,disabled:!0}),` `,`Scoop/WinGet/Homebrew/AUR 配置与 APT 打包预留`]}),`
`,(0,n.jsxs)(r.li,{className:`task-list-item`,children:[(0,n.jsx)(r.input,{type:`checkbox`,checked:!0,disabled:!0}),` `,`CI/CD 自动构建与发布`]}),`
`]}),`
`,(0,n.jsx)(r.h2,{id:`开发中main-分支未发布`,children:`开发中（main 分支，未发布）`}),`
`,(0,n.jsxs)(r.ul,{className:`contains-task-list`,children:[`
`,(0,n.jsxs)(r.li,{className:`task-list-item`,children:[(0,n.jsx)(r.input,{type:`checkbox`,checked:!0,disabled:!0}),` `,`Dashboard 监控工作台（12 个组件、可切换 Profile、鼠标拖动布局）`]}),`
`,(0,n.jsxs)(r.li,{className:`task-list-item`,children:[(0,n.jsx)(r.input,{type:`checkbox`,checked:!0,disabled:!0}),` `,`运维面板：Docker / 系统服务 / 文件浏览 / 日志 / 远程命令`]}),`
`,(0,n.jsxs)(r.li,{className:`task-list-item`,children:[(0,n.jsx)(r.input,{type:`checkbox`,checked:!0,disabled:!0}),` `,`嵌入式终端（portable-pty + vt100，Ctrl+B 前缀键、滚动回看、连接动画）`]}),`
`,(0,n.jsxs)(r.li,{className:`task-list-item`,children:[(0,n.jsx)(r.input,{type:`checkbox`,checked:!0,disabled:!0}),` `,`AI Agent（17 个工具、四级权限、审批流、时间线）`]}),`
`,(0,n.jsxs)(r.li,{className:`task-list-item`,language:`txt`,children:[(0,n.jsx)(r.input,{type:`checkbox`,checked:!0,disabled:!0}),` `,`本地 Web 界面（`,(0,n.jsx)(r.code,{language:`txt`,children:`qssh web`}),`，schema 驱动设置页、模型连接测试）`]}),`
`,(0,n.jsxs)(r.li,{className:`task-list-item`,children:[(0,n.jsx)(r.input,{type:`checkbox`,checked:!0,disabled:!0}),` `,`命令面板（Ctrl+K）`]}),`
`,(0,n.jsxs)(r.li,{className:`task-list-item`,language:`txt`,children:[(0,n.jsx)(r.input,{type:`checkbox`,checked:!0,disabled:!0}),` `,(0,n.jsx)(r.code,{language:`txt`,children:`Ctrl+Enter`}),` 新窗口连接、`,(0,n.jsx)(r.code,{language:`txt`,children:`<`}),` / `,(0,n.jsx)(r.code,{language:`txt`,children:`>`}),` 主机调序`]}),`
`,(0,n.jsxs)(r.li,{className:`task-list-item`,language:`txt`,children:[(0,n.jsx)(r.input,{type:`checkbox`,checked:!0,disabled:!0}),` `,(0,n.jsx)(r.code,{language:`txt`,children:`~/.qsshrc`}),` 设置接入 Web 界面`]}),`
`]}),`
`,(0,n.jsx)(r.h2,{id:`v21规划中`,children:`v2.1（规划中）`}),`
`,(0,n.jsxs)(r.ul,{className:`contains-task-list`,children:[`
`,(0,n.jsxs)(r.li,{className:`task-list-item`,children:[(0,n.jsx)(r.input,{type:`checkbox`,disabled:!0}),` `,`批量命令执行（选中多台主机后执行命令）`]}),`
`,(0,n.jsxs)(r.li,{className:`task-list-item`,children:[(0,n.jsx)(r.input,{type:`checkbox`,disabled:!0}),` `,`自动检测更新（启动时检查 GitHub Release）`]}),`
`,(0,n.jsxs)(r.li,{className:`task-list-item`,children:[(0,n.jsx)(r.input,{type:`checkbox`,disabled:!0}),` `,`连接日志记录（会话时间、主机、退出码）`]}),`
`,(0,n.jsxs)(r.li,{className:`task-list-item`,children:[(0,n.jsx)(r.input,{type:`checkbox`,disabled:!0}),` `,`TUI 主机分组与批量操作增强`]}),`
`]}),`
`,(0,n.jsx)(r.h2,{id:`v22远期规划`,children:`v2.2（远期规划）`}),`
`,(0,n.jsxs)(r.ul,{className:`contains-task-list`,children:[`
`,(0,n.jsxs)(r.li,{className:`task-list-item`,children:[(0,n.jsx)(r.input,{type:`checkbox`,disabled:!0}),` `,`TUI 主题自定义`]}),`
`,(0,n.jsxs)(r.li,{className:`task-list-item`,children:[(0,n.jsx)(r.input,{type:`checkbox`,disabled:!0}),` `,`主机分组/标签`]}),`
`,(0,n.jsxs)(r.li,{className:`task-list-item`,children:[(0,n.jsx)(r.input,{type:`checkbox`,disabled:!0}),` `,`配置备份与恢复`]}),`
`,(0,n.jsxs)(r.li,{className:`task-list-item`,children:[(0,n.jsx)(r.input,{type:`checkbox`,disabled:!0}),` `,`SSH 隧道管理`]}),`
`,(0,n.jsxs)(r.li,{className:`task-list-item`,children:[(0,n.jsx)(r.input,{type:`checkbox`,disabled:!0}),` `,`更多包管理器支持（Chocolatey、Nix）`]}),`
`]})]})}function i(t={}){let{wrapper:i}={...e(),...t.components};return i?(0,n.jsx)(i,{...t,children:(0,n.jsx)(r,{...t})}):r(t)}export{i as default};