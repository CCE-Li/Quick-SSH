import{n as e}from"./vendor-CQh2k-GV.js";import{n as t}from"./react-vendor-6GaKtW3l.js";var n=t();function r(t){let r={a:`a`,code:`code`,h1:`h1`,h2:`h2`,h3:`h3`,hr:`hr`,li:`li`,ol:`ol`,p:`p`,pre:`pre`,strong:`strong`,table:`table`,tbody:`tbody`,td:`td`,th:`th`,thead:`thead`,tr:`tr`,ul:`ul`,...e(),...t.components},{Callout:i,Note:o,Properties:s,Property:c}=r;return i||a(`Callout`,!0),o||a(`Note`,!0),s||a(`Properties`,!0),c||a(`Property`,!0),(0,n.jsxs)(n.Fragment,{children:[(0,n.jsx)(r.hr,{}),`
`,(0,n.jsx)(r.p,{children:`title: TUI 界面指南\r
description: Quick-SSH 终端用户界面的详细使用说明，包含所有交互模式和快捷键。\r
keywords:`}),`
`,(0,n.jsxs)(r.ul,{children:[`
`,(0,n.jsx)(r.li,{children:`tui`}),`
`,(0,n.jsx)(r.li,{children:`终端界面`}),`
`,(0,n.jsx)(r.li,{children:`交互`}),`
`,(0,n.jsx)(r.li,{children:`快捷键`}),`
`,(0,n.jsx)(r.li,{children:`键盘映射`}),`
`]}),`
`,(0,n.jsx)(r.hr,{}),`
`,(0,n.jsx)(r.h1,{id:`tui-界面指南`,children:`TUI 界面指南`}),`
`,(0,n.jsx)(r.p,{children:`Quick-SSH 的 TUI 界面基于 ratatui 和 crossterm 构建，采用事件驱动架构，提供高效的键盘操作体验。`}),`
`,(0,n.jsx)(r.h2,{id:`界面布局`,children:`界面布局`}),`
`,(0,n.jsx)(r.p,{children:`TUI 界面由四个部分组成：`}),`
`,(0,n.jsx)(r.pre,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`┌─ 标题栏 ──────────────────────────────────────────┐\r
│ Quick-SSH v2.0.4  |  共 5 台主机  |  模式: NORMAL  │
├──────────────┬──────────────────────────────────────┤\r
│              │                                      │\r
│  主机列表    │           详情面板                    │\r
│              │                                      │\r
│  ○ mysrv     │   别名: mysrv                        │
│    生产环境  │   地址: ********                     │
│  ● devbox    │   认证: 密钥优先                     │
│  ○ web-prod  │   注释: 生产环境                     │
│              │                                      │\r
├──────────────┴──────────────────────────────────────┤\r
│ 状态栏: j↓ k↑ gg↕ G↕ /搜索 a添加 e编辑 ...         │\r
└─────────────────────────────────────────────────────┘
`})}),`
`,(0,n.jsxs)(r.ul,{children:[`
`,(0,n.jsxs)(r.li,{children:[(0,n.jsx)(r.strong,{children:`标题栏`}),`：显示版本号、主机数量、当前模式`]}),`
`,(0,n.jsxs)(r.li,{children:[(0,n.jsx)(r.strong,{children:`主机列表`}),`：显示所有 SSH 主机、在线状态和首行注释摘要（最多 10 个字符）`]}),`
`,(0,n.jsxs)(r.li,{children:[(0,n.jsx)(r.strong,{children:`详情面板`}),`：显示选中主机的地址、密钥、认证方式、状态和完整多行注释`]}),`
`,(0,n.jsxs)(r.li,{children:[(0,n.jsx)(r.strong,{children:`状态栏`}),`：显示当前模式的操作提示或闪烁消息`]}),`
`]}),`
`,(0,n.jsx)(r.h3,{id:`主机列表前缀含义`,children:`主机列表前缀含义`}),`
`,(0,n.jsxs)(r.table,{children:[(0,n.jsx)(r.thead,{children:(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.th,{children:`图标`}),(0,n.jsx)(r.th,{children:`含义`})]})}),(0,n.jsxs)(r.tbody,{children:[(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`○`})}),(0,n.jsx)(r.td,{children:`未检测`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsxs)(r.td,{language:`txt`,children:[(0,n.jsx)(r.code,{language:`txt`,children:`●`}),` (绿色)`]}),(0,n.jsx)(r.td,{children:`在线`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsxs)(r.td,{language:`txt`,children:[(0,n.jsx)(r.code,{language:`txt`,children:`●`}),` (红色)`]}),(0,n.jsx)(r.td,{children:`离线`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`◔`})}),(0,n.jsx)(r.td,{children:`检测中`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`> `})}),(0,n.jsx)(r.td,{children:`已标记`})]})]})]}),`
`,(0,n.jsx)(r.h2,{id:`交互模式`,children:`交互模式`}),`
`,(0,n.jsx)(r.p,{children:`TUI 支持多种交互模式，通过状态栏可识别当前所处模式：`}),`
`,(0,n.jsxs)(s,{children:[(0,n.jsx)(c,{name:`NORMAL`,type:`默认模式`,children:(0,n.jsx)(r.p,{children:`浏览主机列表，可使用所有导航和操作快捷键`})}),(0,n.jsx)(c,{name:`SEARCH`,type:`搜索模式`,children:(0,n.jsx)(r.p,{children:`输入关键词过滤主机列表，实时筛选`})}),(0,n.jsx)(c,{name:`ADD / EDIT`,type:`新增/编辑模式`,children:(0,n.jsx)(r.p,{children:`弹窗表单编辑主机字段，支持 Tab 切换字段，Ctrl+S 保存`})}),(0,n.jsx)(c,{name:`CONFIRM`,type:`确认模式`,children:(0,n.jsx)(r.p,{children:`删除确认对话框，按 y/Y 确认，n/N/Esc 取消`})}),(0,n.jsx)(c,{name:`HELP`,type:`帮助模式`,children:(0,n.jsx)(r.p,{children:`显示键盘快捷键帮助弹窗，按 q/Esc 关闭`})})]}),`
`,(0,n.jsx)(r.h2,{id:`完整快捷键`,children:`完整快捷键`}),`
`,(0,n.jsx)(r.h3,{id:`导航`,children:`导航`}),`
`,(0,n.jsxs)(r.table,{children:[(0,n.jsx)(r.thead,{children:(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.th,{children:`按键`}),(0,n.jsx)(r.th,{children:`功能`})]})}),(0,n.jsxs)(r.tbody,{children:[(0,n.jsxs)(r.tr,{children:[(0,n.jsxs)(r.td,{language:`txt`,children:[(0,n.jsx)(r.code,{language:`txt`,children:`j`}),` / `,(0,n.jsx)(r.code,{language:`txt`,children:`↓`})]}),(0,n.jsx)(r.td,{children:`向下移动`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsxs)(r.td,{language:`txt`,children:[(0,n.jsx)(r.code,{language:`txt`,children:`k`}),` / `,(0,n.jsx)(r.code,{language:`txt`,children:`↑`})]}),(0,n.jsx)(r.td,{children:`向上移动`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`Ctrl+N`})}),(0,n.jsx)(r.td,{children:`向下移动`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`Ctrl+P`})}),(0,n.jsx)(r.td,{children:`向上移动`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`gg`})}),(0,n.jsx)(r.td,{children:`跳到顶部`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`G`})}),(0,n.jsx)(r.td,{children:`跳到底部`})]})]})]}),`
`,(0,n.jsx)(r.h3,{id:`主机操作`,children:`主机操作`}),`
`,(0,n.jsxs)(r.table,{children:[(0,n.jsx)(r.thead,{children:(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.th,{children:`按键`}),(0,n.jsx)(r.th,{children:`功能`})]})}),(0,n.jsxs)(r.tbody,{children:[(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`Enter`})}),(0,n.jsx)(r.td,{children:`连接选中主机`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`Space`})}),(0,n.jsx)(r.td,{children:`标记/取消标记`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`a`})}),(0,n.jsx)(r.td,{children:`新增主机（弹窗表单）`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`e`})}),(0,n.jsx)(r.td,{children:`编辑当前主机（弹窗表单）`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`d`})}),(0,n.jsx)(r.td,{children:`无标记时删除选中主机；有标记时批量删除所有标记主机`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`.`})}),(0,n.jsx)(r.td,{children:`切换地址显示/隐藏（隐私保护）`})]})]})]}),`
`,(0,n.jsx)(r.h3,{id:`检测`,children:`检测`}),`
`,(0,n.jsxs)(r.table,{children:[(0,n.jsx)(r.thead,{children:(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.th,{children:`按键`}),(0,n.jsx)(r.th,{children:`功能`})]})}),(0,n.jsxs)(r.tbody,{children:[(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`p`})}),(0,n.jsx)(r.td,{children:`Ping 检测当前主机`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`P`})}),(0,n.jsx)(r.td,{children:`Ping 检测所有主机`})]})]})]}),`
`,(0,n.jsx)(r.h3,{id:`搜索`,children:`搜索`}),`
`,(0,n.jsxs)(r.table,{children:[(0,n.jsx)(r.thead,{children:(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.th,{children:`按键`}),(0,n.jsx)(r.th,{children:`功能`})]})}),(0,n.jsxs)(r.tbody,{children:[(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`/`})}),(0,n.jsx)(r.td,{children:`进入搜索模式`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`Enter`})}),(0,n.jsx)(r.td,{children:`确认搜索`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`Esc`})}),(0,n.jsx)(r.td,{children:`取消搜索`})]})]})]}),`
`,(0,n.jsx)(r.h3,{id:`系统`,children:`系统`}),`
`,(0,n.jsxs)(r.table,{children:[(0,n.jsx)(r.thead,{children:(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.th,{children:`按键`}),(0,n.jsx)(r.th,{children:`功能`})]})}),(0,n.jsxs)(r.tbody,{children:[(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`q`})}),(0,n.jsx)(r.td,{children:`退出 TUI`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`?`})}),(0,n.jsx)(r.td,{children:`显示帮助`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`Esc`})}),(0,n.jsx)(r.td,{children:`取消/返回`})]})]})]}),`
`,(0,n.jsx)(r.h2,{id:`新增编辑主机弹窗`,children:`新增/编辑主机弹窗`}),`
`,(0,n.jsxs)(r.p,{language:`txt`,children:[`按 `,(0,n.jsx)(r.code,{language:`txt`,children:`a`}),` 或 `,(0,n.jsx)(r.code,{language:`txt`,children:`e`}),` 会打开主机编辑弹窗，包含以下字段：`]}),`
`,(0,n.jsxs)(r.table,{children:[(0,n.jsx)(r.thead,{children:(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.th,{children:`字段`}),(0,n.jsx)(r.th,{children:`说明`})]})}),(0,n.jsxs)(r.tbody,{children:[(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{children:(0,n.jsx)(r.strong,{children:`Host`})}),(0,n.jsx)(r.td,{children:`主机别名（必填）`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{children:(0,n.jsx)(r.strong,{children:`HostName`})}),(0,n.jsx)(r.td,{children:`主机地址（IP 或域名）`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{children:(0,n.jsx)(r.strong,{children:`User`})}),(0,n.jsx)(r.td,{children:`登录用户名`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{children:(0,n.jsx)(r.strong,{children:`Port`})}),(0,n.jsx)(r.td,{children:`SSH 端口（默认 22）`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{children:(0,n.jsx)(r.strong,{children:`IdentityFile`})}),(0,n.jsx)(r.td,{children:`密钥文件路径（自动预填充默认路径）`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{children:(0,n.jsx)(r.strong,{children:`Password`})}),(0,n.jsxs)(r.td,{language:`txt`,children:[`登录密码，仅保存到系统安全凭据库，输入以 `,(0,n.jsx)(r.code,{language:`txt`,children:`*`}),` 遮罩`]})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{children:(0,n.jsx)(r.strong,{children:`注释（无需输入 #）`})}),(0,n.jsxs)(r.td,{language:`txt`,children:[`多行主机注释，保存时自动添加 `,(0,n.jsx)(r.code,{language:`txt`,children:`#`})]})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{children:(0,n.jsx)(r.strong,{children:`其他 SSH 指令`})}),(0,n.jsxs)(r.td,{language:`txt`,children:[`多行额外配置，如 `,(0,n.jsx)(r.code,{language:`txt`,children:`ServerAliveInterval 60`})]})]})]})]}),`
`,(0,n.jsx)(r.h3,{id:`弹窗操作`,children:`弹窗操作`}),`
`,(0,n.jsxs)(r.table,{children:[(0,n.jsx)(r.thead,{children:(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.th,{children:`按键`}),(0,n.jsx)(r.th,{children:`功能`})]})}),(0,n.jsxs)(r.tbody,{children:[(0,n.jsxs)(r.tr,{children:[(0,n.jsxs)(r.td,{language:`txt`,children:[(0,n.jsx)(r.code,{language:`txt`,children:`Tab`}),` / `,(0,n.jsx)(r.code,{language:`txt`,children:`↓`})]}),(0,n.jsx)(r.td,{children:`切换到下一字段`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsxs)(r.td,{language:`txt`,children:[(0,n.jsx)(r.code,{language:`txt`,children:`Shift+Tab`}),` / `,(0,n.jsx)(r.code,{language:`txt`,children:`↑`})]}),(0,n.jsx)(r.td,{children:`切换到上一字段`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`Enter`})}),(0,n.jsx)(r.td,{children:`跳转到下一字段（仅单行字段）`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`Ctrl+S`})}),(0,n.jsx)(r.td,{children:`保存并关闭弹窗`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`Esc`})}),(0,n.jsx)(r.td,{children:`取消编辑`})]})]})]}),`
`,(0,n.jsx)(o,{children:(0,n.jsxs)(r.p,{language:`txt`,children:[`“其他 SSH 指令”可填写 `,(0,n.jsx)(r.code,{language:`txt`,children:`ServerAliveInterval`}),`、`,(0,n.jsx)(r.code,{language:`txt`,children:`ProxyJump`}),` 等配置，但禁止重复填写 `,(0,n.jsx)(r.code,{language:`txt`,children:`Host`}),`、`,(0,n.jsx)(r.code,{language:`txt`,children:`HostName`}),`、`,(0,n.jsx)(r.code,{language:`txt`,children:`User`}),`、`,(0,n.jsx)(r.code,{language:`txt`,children:`Port`}),`、`,(0,n.jsx)(r.code,{language:`txt`,children:`IdentityFile`}),` 等托管字段。`]})}),`
`,(0,n.jsx)(r.h3,{id:`密码字段规则`,children:`密码字段规则`}),`
`,(0,n.jsxs)(r.ul,{children:[`
`,(0,n.jsx)(r.li,{children:`新增主机时留空：不保存密码`}),`
`,(0,n.jsx)(r.li,{children:`编辑已有主机时留空：保留已保存密码`}),`
`,(0,n.jsxs)(r.li,{language:`txt`,children:[`编辑时输入 `,(0,n.jsx)(r.code,{language:`txt`,children:`!clear`}),`：删除已保存密码`]}),`
`,(0,n.jsx)(r.li,{children:`修改主机别名：已保存密码会迁移到新别名`}),`
`]}),`
`,(0,n.jsx)(r.p,{children:`密码不会写入 SSH 配置。OpenSSH 会先尝试配置的密钥和 ssh-agent，只有请求登录密码时，Quick-SSH 才通过 AskPass 提供已保存密码。`}),`
`,(0,n.jsx)(r.h2,{id:`连接主机`,children:`连接主机`}),`
`,(0,n.jsxs)(r.p,{language:`txt`,children:[`在 NORMAL 模式下选中主机后按 `,(0,n.jsx)(r.code,{language:`txt`,children:`Enter`}),` 键即可连接。连接流程：`]}),`
`,(0,n.jsxs)(r.ol,{children:[`
`,(0,n.jsx)(r.li,{children:`TUI 退出备用屏幕，恢复终端`}),`
`,(0,n.jsx)(r.li,{children:`检查当前别名是否有已保存密码；读取失败时回退到普通系统 SSH`}),`
`,(0,n.jsx)(r.li,{children:`启动 SSH 交互式会话，OpenSSH 按自身认证顺序尝试密钥、agent 和密码`}),`
`,(0,n.jsx)(r.li,{children:`SSH 退出后自动重新进入 TUI 界面`}),`
`]}),`
`,(0,n.jsx)(i,{title:`首次连接`,children:(0,n.jsxs)(r.p,{language:`txt`,children:[`AskPass 只响应登录密码提示，不会自动确认首次连接的主机指纹，也不会填写私钥口令。首次连接请先使用 `,(0,n.jsx)(r.code,{language:`txt`,children:`ssh <主机别名>`}),` 手动确认指纹。`]})}),`
`,(0,n.jsx)(i,{title:`拖拽上传`,children:(0,n.jsxs)(r.p,{children:[`在 SSH 会话期间，将文件拖入终端窗口即可触发文件上传。详见 `,(0,n.jsx)(r.a,{href:`/file-upload`,children:`文件上传`}),`。`]})})]})}function i(t={}){let{wrapper:i}={...e(),...t.components};return i?(0,n.jsx)(i,{...t,children:(0,n.jsx)(r,{...t})}):r(t)}function a(e,t){throw Error(`Expected `+(t?`component`:`object`)+" `"+e+"` to be defined: you likely forgot to import, pass, or provide it.")}export{i as default};