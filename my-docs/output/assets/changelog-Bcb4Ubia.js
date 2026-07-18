import{n as e}from"./vendor-CQh2k-GV.js";import{n as t}from"./react-vendor-6GaKtW3l.js";var n=t();function r(t){let r={code:`code`,h1:`h1`,h2:`h2`,h3:`h3`,hr:`hr`,li:`li`,p:`p`,ul:`ul`,...e(),...t.components};return(0,n.jsxs)(n.Fragment,{children:[(0,n.jsx)(r.hr,{}),`
`,(0,n.jsx)(r.p,{children:`title: 变更日志
description: Quick-SSH 的版本发布历史记录。
keywords:`}),`
`,(0,n.jsxs)(r.ul,{children:[`
`,(0,n.jsx)(r.li,{children:`changelog`}),`
`,(0,n.jsx)(r.li,{children:`变更日志`}),`
`,(0,n.jsx)(r.li,{children:`版本`}),`
`,(0,n.jsx)(r.li,{children:`release`}),`
`]}),`
`,(0,n.jsx)(r.hr,{}),`
`,(0,n.jsx)(r.h1,{id:`变更日志`,children:`变更日志`}),`
`,(0,n.jsx)(r.h2,{id:`v201`,children:`v2.0.1`}),`
`,(0,n.jsx)(r.h3,{id:`修复`,children:`修复`}),`
`,(0,n.jsxs)(r.ul,{children:[`
`,(0,n.jsx)(r.li,{children:`改进 Windows 终端 raw 模式下的光标恢复机制`}),`
`,(0,n.jsx)(r.li,{children:`修复拖拽文件检测中 Windows 路径的引号处理`}),`
`,(0,n.jsx)(r.li,{children:`优化 TUI 列表滚动行为`}),`
`]}),`
`,(0,n.jsx)(r.h3,{id:`更改`,children:`更改`}),`
`,(0,n.jsxs)(r.ul,{children:[`
`,(0,n.jsx)(r.li,{children:`更新依赖版本`}),`
`]}),`
`,(0,n.jsx)(r.h2,{id:`v200`,children:`v2.0.0`}),`
`,(0,n.jsx)(r.h3,{id:`新增`,children:`新增`}),`
`,(0,n.jsxs)(r.ul,{children:[`
`,(0,n.jsx)(r.li,{children:`Rust 完整重写（移除 Node.js 运行时依赖）`}),`
`,(0,n.jsxs)(r.li,{children:[`事件驱动 TUI 界面（基于 ratatui + crossterm）`,`
`,(0,n.jsxs)(r.ul,{children:[`
`,(0,n.jsx)(r.li,{children:`主机列表浏览、搜索、标记`}),`
`,(0,n.jsx)(r.li,{children:`Ping 检测（单机和全量）`}),`
`,(0,n.jsx)(r.li,{children:`弹窗表单新增/编辑主机`}),`
`,(0,n.jsx)(r.li,{children:`隐私保护（地址默认隐藏）`}),`
`]}),`
`]}),`
`,(0,n.jsxs)(r.li,{children:[`完整 CLI 子命令`,`
`,(0,n.jsxs)(r.ul,{children:[`
`,(0,n.jsxs)(r.li,{language:`txt`,children:[(0,n.jsx)(r.code,{language:`txt`,children:`ps`}),` / `,(0,n.jsx)(r.code,{language:`txt`,children:`ls`}),` — 列出主机（支持关键词过滤）`]}),`
`,(0,n.jsxs)(r.li,{language:`txt`,children:[(0,n.jsx)(r.code,{language:`txt`,children:`add`}),` — 添加主机（支持密钥和端口选项）`]}),`
`,(0,n.jsxs)(r.li,{language:`txt`,children:[(0,n.jsx)(r.code,{language:`txt`,children:`rm`}),` / `,(0,n.jsx)(r.code,{language:`txt`,children:`remove`}),` — 删除主机`]}),`
`,(0,n.jsxs)(r.li,{language:`txt`,children:[(0,n.jsx)(r.code,{language:`txt`,children:`connect`}),` / `,(0,n.jsx)(r.code,{language:`txt`,children:`cn`}),` — 连接主机（别名或 user@host）`]}),`
`,(0,n.jsxs)(r.li,{language:`txt`,children:[(0,n.jsx)(r.code,{language:`txt`,children:`export`}),` — 导出为 JSON`]}),`
`,(0,n.jsxs)(r.li,{language:`txt`,children:[(0,n.jsx)(r.code,{language:`txt`,children:`import`}),` — 从 JSON 导入`]}),`
`,(0,n.jsxs)(r.li,{language:`txt`,children:[(0,n.jsx)(r.code,{language:`txt`,children:`help`}),` — 帮助信息`]}),`
`,(0,n.jsxs)(r.li,{language:`txt`,children:[(0,n.jsx)(r.code,{language:`txt`,children:`completions`}),` — Shell 补全生成`]}),`
`]}),`
`]}),`
`,(0,n.jsx)(r.li,{children:`Shell 补全生成（bash / zsh / fish / powershell / elvish）`}),`
`,(0,n.jsxs)(r.li,{language:`txt`,children:[`独立 SFTP 上传工具（`,(0,n.jsx)(r.code,{language:`txt`,children:`qssh-uploader`}),`）`,`
`,(0,n.jsxs)(r.ul,{children:[`
`,(0,n.jsx)(r.li,{children:`最多 3 文件并发上传`}),`
`,(0,n.jsx)(r.li,{children:`实时进度条显示`}),`
`,(0,n.jsx)(r.li,{children:`防闪退设计`}),`
`]}),`
`]}),`
`,(0,n.jsxs)(r.li,{children:[`渐进式 SSH 配置解析`,`
`,(0,n.jsxs)(r.ul,{children:[`
`,(0,n.jsx)(r.li,{children:`只管理 Host/HostName/User/Port/IdentityFile`}),`
`,(0,n.jsx)(r.li,{children:`其余指令完整保留`}),`
`,(0,n.jsx)(r.li,{children:`支持全局配置和 Match 块`}),`
`]}),`
`]}),`
`,(0,n.jsxs)(r.li,{language:`txt`,children:[(0,n.jsx)(r.code,{language:`txt`,children:`~/.qsshrc`}),` 程序设置`]}),`
`,(0,n.jsxs)(r.li,{children:[`6 种包管理器配置`,`
`,(0,n.jsxs)(r.ul,{children:[`
`,(0,n.jsx)(r.li,{children:`Scoop（Windows）`}),`
`,(0,n.jsx)(r.li,{children:`WinGet（Windows）`}),`
`,(0,n.jsx)(r.li,{children:`Homebrew（macOS）`}),`
`,(0,n.jsx)(r.li,{children:`AUR（Arch Linux）`}),`
`,(0,n.jsx)(r.li,{children:`APT（Debian/Ubuntu）`}),`
`]}),`
`]}),`
`]}),`
`,(0,n.jsx)(r.h3,{id:`基础设施`,children:`基础设施`}),`
`,(0,n.jsxs)(r.ul,{children:[`
`,(0,n.jsx)(r.li,{children:`GitHub Actions CI/CD（自动构建 + 发布）`}),`
`,(0,n.jsx)(r.li,{children:`跨平台交叉编译（Linux / macOS / Windows）`}),`
`,(0,n.jsx)(r.li,{children:`自动包管理器配置更新脚本`}),`
`]})]})}function i(t={}){let{wrapper:i}={...e(),...t.components};return i?(0,n.jsx)(i,{...t,children:(0,n.jsx)(r,{...t})}):r(t)}export{i as default};