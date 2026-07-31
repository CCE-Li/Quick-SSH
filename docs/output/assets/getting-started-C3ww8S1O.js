import{n as e}from"./vendor-CQh2k-GV.js";import{n as t}from"./react-vendor-6GaKtW3l.js";var n=t();function r(t){let r={a:`a`,code:`code`,h1:`h1`,h2:`h2`,h3:`h3`,hr:`hr`,li:`li`,p:`p`,pre:`pre`,span:`span`,strong:`strong`,table:`table`,tbody:`tbody`,td:`td`,th:`th`,thead:`thead`,tr:`tr`,ul:`ul`,...e(),...t.components},{Callout:i,Warning:o}=r;return i||a(`Callout`,!0),o||a(`Warning`,!0),(0,n.jsxs)(n.Fragment,{children:[(0,n.jsx)(r.hr,{}),`
`,(0,n.jsx)(r.p,{children:`title: 快速入门\r
description: 5 分钟上手 Quick-SSH 的基本操作，包括 TUI 界面使用和 CLI 命令操作。\r
keywords:`}),`
`,(0,n.jsxs)(r.ul,{children:[`
`,(0,n.jsx)(r.li,{children:`入门`}),`
`,(0,n.jsx)(r.li,{children:`教程`}),`
`,(0,n.jsx)(r.li,{children:`tui`}),`
`,(0,n.jsx)(r.li,{children:`cli`}),`
`,(0,n.jsx)(r.li,{children:`ssh`}),`
`]}),`
`,(0,n.jsx)(r.hr,{}),`
`,(0,n.jsx)(r.h1,{id:`快速入门`,children:`快速入门`}),`
`,(0,n.jsx)(r.p,{children:`5 分钟带你熟悉 Quick-SSH 的核心功能。`}),`
`,(0,n.jsx)(r.h2,{id:`启动-tui-界面`,children:`启动 TUI 界面`}),`
`,(0,n.jsxs)(r.p,{language:`txt`,children:[`直接运行 `,(0,n.jsx)(r.code,{language:`txt`,children:`qssh`}),` 即可启动 TUI 界面：`]}),`
`,(0,n.jsx)(n.Fragment,{children:(0,n.jsx)(r.pre,{className:`shiki css-variables`,style:{backgroundColor:`var(--shiki-background)`,color:`var(--shiki-foreground)`},tabIndex:`0`,children:(0,n.jsx)(r.code,{className:`language-bash`,children:(0,n.jsx)(r.span,{className:`line`,children:(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-function)`},children:`qssh`})})})})}),`
`,(0,n.jsxs)(r.p,{language:`txt`,children:[`TUI 界面默认读取 `,(0,n.jsx)(r.code,{language:`txt`,children:`~/.ssh/config`}),` 文件中的主机配置。`]}),`
`,(0,n.jsx)(r.h3,{id:`常用键位`,children:`常用键位`}),`
`,(0,n.jsxs)(r.table,{children:[(0,n.jsx)(r.thead,{children:(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.th,{children:`按键`}),(0,n.jsx)(r.th,{children:`功能`})]})}),(0,n.jsxs)(r.tbody,{children:[(0,n.jsxs)(r.tr,{children:[(0,n.jsxs)(r.td,{language:`txt`,children:[(0,n.jsx)(r.code,{language:`txt`,children:`↑`}),` / `,(0,n.jsx)(r.code,{language:`txt`,children:`k`}),` / `,(0,n.jsx)(r.code,{language:`txt`,children:`Ctrl+P`})]}),(0,n.jsx)(r.td,{children:`选择上一台主机`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsxs)(r.td,{language:`txt`,children:[(0,n.jsx)(r.code,{language:`txt`,children:`↓`}),` / `,(0,n.jsx)(r.code,{language:`txt`,children:`j`}),` / `,(0,n.jsx)(r.code,{language:`txt`,children:`Ctrl+N`})]}),(0,n.jsx)(r.td,{children:`选择下一台主机`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`Enter`})}),(0,n.jsx)(r.td,{children:`连接选中主机`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`/`})}),(0,n.jsx)(r.td,{children:`搜索过滤`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`Space`})}),(0,n.jsx)(r.td,{children:`标记/取消标记`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`a`})}),(0,n.jsx)(r.td,{children:`添加新主机`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`e`})}),(0,n.jsx)(r.td,{children:`编辑当前主机`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`d`})}),(0,n.jsx)(r.td,{children:`删除选中主机；存在标记时批量删除所有标记主机`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`p`})}),(0,n.jsx)(r.td,{children:`Ping 检测当前主机`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`P`})}),(0,n.jsx)(r.td,{children:`Ping 检测所有主机`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`.`})}),(0,n.jsx)(r.td,{children:`切换地址显示/隐藏`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsxs)(r.td,{language:`txt`,children:[(0,n.jsx)(r.code,{language:`txt`,children:`q`}),` / `,(0,n.jsx)(r.code,{language:`txt`,children:`Esc`})]}),(0,n.jsx)(r.td,{children:`退出 / 取消搜索`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`?`})}),(0,n.jsx)(r.td,{children:`显示帮助`})]})]})]}),`
`,(0,n.jsx)(i,{title:`地址隐私保护`,children:(0,n.jsxs)(r.p,{language:`txt`,children:[`默认情况下，主机地址以 `,(0,n.jsx)(r.code,{language:`txt`,children:`********`}),` 显示，防止旁人窥屏。按 `,(0,n.jsx)(r.code,{language:`txt`,children:`.`}),` 键可切换显示/隐藏。`]})}),`
`,(0,n.jsx)(r.h2,{id:`保存登录密码`,children:`保存登录密码`}),`
`,(0,n.jsxs)(r.p,{language:`txt`,children:[`在 TUI 中按 `,(0,n.jsx)(r.code,{language:`txt`,children:`a`}),` 添加或按 `,(0,n.jsx)(r.code,{language:`txt`,children:`e`}),` 编辑主机，在 `,(0,n.jsx)(r.strong,{children:`Password`}),` 字段输入密码并保存。输入内容会以 `,(0,n.jsx)(r.code,{language:`txt`,children:`*`}),` 遮罩，密码保存在系统安全凭据库中，不会写入 `,(0,n.jsx)(r.code,{language:`txt`,children:`~/.ssh/config`}),`：`]}),`
`,(0,n.jsxs)(r.ul,{children:[`
`,(0,n.jsx)(r.li,{children:`Windows：Credential Manager`}),`
`,(0,n.jsx)(r.li,{children:`macOS：Keychain`}),`
`,(0,n.jsx)(r.li,{children:`Linux：Secret Service / keyutils`}),`
`]}),`
`,(0,n.jsxs)(r.p,{language:`txt`,children:[`编辑已有主机时，密码留空表示保留原值，输入 `,(0,n.jsx)(r.code,{language:`txt`,children:`!clear`}),` 表示删除。删除主机时，对应密码也会同步清理。连接时仍由系统 OpenSSH 优先尝试密钥和 ssh-agent，只有出现登录密码提示时才通过 AskPass 自动填写。`]}),`
`,(0,n.jsx)(o,{children:(0,n.jsxs)(r.p,{language:`txt`,children:[`首次连接新主机时，请先运行 `,(0,n.jsx)(r.code,{language:`txt`,children:`ssh <主机别名>`}),` 并核对、确认主机指纹。Quick-SSH 不会自动回答主机指纹或私钥口令提示。`]})}),`
`,(0,n.jsx)(r.h2,{id:`cli-命令操作`,children:`CLI 命令操作`}),`
`,(0,n.jsx)(r.h3,{id:`添加主机`,children:`添加主机`}),`
`,(0,n.jsx)(n.Fragment,{children:(0,n.jsx)(r.pre,{className:`shiki css-variables`,style:{backgroundColor:`var(--shiki-background)`,color:`var(--shiki-foreground)`},tabIndex:`0`,children:(0,n.jsxs)(r.code,{className:`language-bash`,children:[(0,n.jsx)(r.span,{className:`line`,children:(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-comment)`},children:`# 基本用法`})}),`
`,(0,n.jsxs)(r.span,{className:`line`,children:[(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-function)`},children:`qssh`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` add`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` mysrv`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` root@192.168.1.100`})]}),`
`,(0,n.jsx)(r.span,{className:`line`}),`
`,(0,n.jsx)(r.span,{className:`line`,children:(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-comment)`},children:`# 指定密钥文件`})}),`
`,(0,n.jsxs)(r.span,{className:`line`,children:[(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-function)`},children:`qssh`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` add`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` mysrv`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` root@10.0.0.1`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` -k`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` ~/.ssh/id_rsa`})]}),`
`,(0,n.jsx)(r.span,{className:`line`}),`
`,(0,n.jsx)(r.span,{className:`line`,children:(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-comment)`},children:`# 指定端口`})}),`
`,(0,n.jsxs)(r.span,{className:`line`,children:[(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-function)`},children:`qssh`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` add`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` mysrv`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` root@example.com`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` -p`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-constant)`},children:` 2222`})]})]})})}),`
`,(0,n.jsx)(r.h3,{id:`列出主机`,children:`列出主机`}),`
`,(0,n.jsx)(n.Fragment,{children:(0,n.jsx)(r.pre,{className:`shiki css-variables`,style:{backgroundColor:`var(--shiki-background)`,color:`var(--shiki-foreground)`},tabIndex:`0`,children:(0,n.jsxs)(r.code,{className:`language-bash`,children:[(0,n.jsx)(r.span,{className:`line`,children:(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-comment)`},children:`# 列出所有主机`})}),`
`,(0,n.jsxs)(r.span,{className:`line`,children:[(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-function)`},children:`qssh`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` ps`})]}),`
`,(0,n.jsx)(r.span,{className:`line`}),`
`,(0,n.jsx)(r.span,{className:`line`,children:(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-comment)`},children:`# 搜索含 "dev" 的主机`})}),`
`,(0,n.jsxs)(r.span,{className:`line`,children:[(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-function)`},children:`qssh`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` ps`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` dev`})]})]})})}),`
`,(0,n.jsx)(r.h3,{id:`连接主机`,children:`连接主机`}),`
`,(0,n.jsx)(n.Fragment,{children:(0,n.jsx)(r.pre,{className:`shiki css-variables`,style:{backgroundColor:`var(--shiki-background)`,color:`var(--shiki-foreground)`},tabIndex:`0`,children:(0,n.jsxs)(r.code,{className:`language-bash`,children:[(0,n.jsx)(r.span,{className:`line`,children:(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-comment)`},children:`# 通过别名连接`})}),`
`,(0,n.jsxs)(r.span,{className:`line`,children:[(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-function)`},children:`qssh`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` mysrv`})]}),`
`,(0,n.jsx)(r.span,{className:`line`}),`
`,(0,n.jsx)(r.span,{className:`line`,children:(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-comment)`},children:`# 或使用 connect 子命令`})}),`
`,(0,n.jsxs)(r.span,{className:`line`,children:[(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-function)`},children:`qssh`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` connect`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` mysrv`})]}),`
`,(0,n.jsx)(r.span,{className:`line`}),`
`,(0,n.jsx)(r.span,{className:`line`,children:(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-comment)`},children:`# 直接连接 user@host`})}),`
`,(0,n.jsxs)(r.span,{className:`line`,children:[(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-function)`},children:`qssh`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` connect`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` root@192.168.1.100`})]}),`
`,(0,n.jsx)(r.span,{className:`line`}),`
`,(0,n.jsx)(r.span,{className:`line`,children:(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-comment)`},children:`# 传递额外 SSH 参数（放在 -- 之后）`})}),`
`,(0,n.jsxs)(r.span,{className:`line`,children:[(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-function)`},children:`qssh`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` mysrv`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` --`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` -o`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string-expression)`},children:` "ServerAliveInterval=60"`})]})]})})}),`
`,(0,n.jsx)(r.h3,{id:`删除主机`,children:`删除主机`}),`
`,(0,n.jsx)(n.Fragment,{children:(0,n.jsx)(r.pre,{className:`shiki css-variables`,style:{backgroundColor:`var(--shiki-background)`,color:`var(--shiki-foreground)`},tabIndex:`0`,children:(0,n.jsx)(r.code,{className:`language-bash`,children:(0,n.jsxs)(r.span,{className:`line`,children:[(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-function)`},children:`qssh`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` rm`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` mysrv`})]})})})}),`
`,(0,n.jsx)(r.h3,{id:`导出导入配置`,children:`导出/导入配置`}),`
`,(0,n.jsx)(n.Fragment,{children:(0,n.jsx)(r.pre,{className:`shiki css-variables`,style:{backgroundColor:`var(--shiki-background)`,color:`var(--shiki-foreground)`},tabIndex:`0`,children:(0,n.jsxs)(r.code,{className:`language-bash`,children:[(0,n.jsx)(r.span,{className:`line`,children:(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-comment)`},children:`# 导出为 JSON`})}),`
`,(0,n.jsxs)(r.span,{className:`line`,children:[(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-function)`},children:`qssh`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` export`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` backup.json`})]}),`
`,(0,n.jsx)(r.span,{className:`line`}),`
`,(0,n.jsx)(r.span,{className:`line`,children:(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-comment)`},children:`# 从 JSON 导入`})}),`
`,(0,n.jsxs)(r.span,{className:`line`,children:[(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-function)`},children:`qssh`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` import`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` backup.json`})]})]})})}),`
`,(0,n.jsx)(r.h2,{id:`文件上传`,children:`文件上传`}),`
`,(0,n.jsxs)(r.p,{children:[`SSH 连接后，将本地文件或目录`,(0,n.jsx)(r.strong,{children:`拖入终端窗口`}),`，Quick-SSH 会自动在新窗口中启动 SFTP 上传，显示每文件的进度条和总进度。`]}),`
`,(0,n.jsx)(r.p,{children:`也可使用独立上传工具：`}),`
`,(0,n.jsx)(n.Fragment,{children:(0,n.jsx)(r.pre,{className:`shiki css-variables`,style:{backgroundColor:`var(--shiki-background)`,color:`var(--shiki-foreground)`},tabIndex:`0`,children:(0,n.jsx)(r.code,{className:`language-bash`,children:(0,n.jsxs)(r.span,{className:`line`,children:[(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-function)`},children:`qssh-uploader`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` mysrv`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` ./myfile.zip`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` /remote/path/`})]})})})}),`
`,(0,n.jsx)(r.h2,{id:`配置查看`,children:`配置查看`}),`
`,(0,n.jsxs)(r.p,{language:`txt`,children:[`主机配置保存在 `,(0,n.jsx)(r.code,{language:`txt`,children:`~/.ssh/config`}),`（标准 OpenSSH 格式），Quick-SSH 会保留文件中所有非自己管理的内容；保存的密码独立存放在系统安全凭据库中。程序行为设置预留在 `,(0,n.jsx)(r.code,{language:`txt`,children:`~/.qsshrc`}),`（详见 `,(0,n.jsx)(r.a,{href:`/configuration`,children:`配置说明`}),`）。`]}),`
`,(0,n.jsx)(r.h2,{id:`下一步`,children:`下一步`}),`
`,(0,n.jsxs)(r.ul,{children:[`
`,(0,n.jsxs)(r.li,{children:[(0,n.jsx)(r.a,{href:`/cli-reference`,children:`CLI 命令参考`}),` — 完整的子命令文档`]}),`
`,(0,n.jsxs)(r.li,{children:[(0,n.jsx)(r.a,{href:`/tui-guide`,children:`TUI 界面指南`}),` — TUI 使用详解`]}),`
`,(0,n.jsxs)(r.li,{children:[(0,n.jsx)(r.a,{href:`/configuration`,children:`配置说明`}),` — SSH 配置与程序设置`]}),`
`,(0,n.jsxs)(r.li,{children:[(0,n.jsx)(r.a,{href:`/file-upload`,children:`文件上传`}),` — 文件上传详细用法`]}),`
`]})]})}function i(t={}){let{wrapper:i}={...e(),...t.components};return i?(0,n.jsx)(i,{...t,children:(0,n.jsx)(r,{...t})}):r(t)}function a(e,t){throw Error(`Expected `+(t?`component`:`object`)+" `"+e+"` to be defined: you likely forgot to import, pass, or provide it.")}export{i as default};