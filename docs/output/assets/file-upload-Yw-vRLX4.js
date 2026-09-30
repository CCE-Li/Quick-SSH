import{n as e}from"./vendor-CQh2k-GV.js";import{n as t}from"./react-vendor-6GaKtW3l.js";var n=t();function r(t){let r={code:`code`,h1:`h1`,h2:`h2`,h3:`h3`,hr:`hr`,li:`li`,ol:`ol`,p:`p`,pre:`pre`,span:`span`,strong:`strong`,table:`table`,tbody:`tbody`,td:`td`,th:`th`,thead:`thead`,tr:`tr`,ul:`ul`,...e(),...t.components},{Properties:i,Property:o}=r;return i||a(`Properties`,!0),o||a(`Property`,!0),(0,n.jsxs)(n.Fragment,{children:[(0,n.jsx)(r.hr,{}),`
`,(0,n.jsx)(r.p,{children:`title: 文件上传\r
description: Quick-SSH 的文件上传功能说明，支持拖拽上传和独立上传工具。\r
keywords:`}),`
`,(0,n.jsxs)(r.ul,{children:[`
`,(0,n.jsx)(r.li,{children:`上传`}),`
`,(0,n.jsx)(r.li,{children:`sftp`}),`
`,(0,n.jsx)(r.li,{children:`scp`}),`
`,(0,n.jsx)(r.li,{children:`拖拽`}),`
`,(0,n.jsx)(r.li,{children:`qssh-uploader`}),`
`]}),`
`,(0,n.jsx)(r.hr,{}),`
`,(0,n.jsx)(r.h1,{id:`文件上传`,children:`文件上传`}),`
`,(0,n.jsxs)(r.p,{language:`txt`,children:[`Quick-SSH 提供两种文件上传方式：SSH 会话中的拖拽上传和独立的 `,(0,n.jsx)(r.code,{language:`txt`,children:`qssh-uploader`}),` 工具。两者最终都调用系统 `,(0,n.jsx)(r.code,{language:`txt`,children:`scp`}),`。`]}),`
`,(0,n.jsx)(r.h2,{id:`拖拽上传`,children:`拖拽上传`}),`
`,(0,n.jsxs)(r.p,{children:[`SSH 连接后，将本地文件或目录`,(0,n.jsx)(r.strong,{children:`拖入终端窗口`}),`，Quick-SSH 会自动在新窗口中启动 SCP 上传。`]}),`
`,(0,n.jsx)(r.h3,{id:`工作原理`,children:`工作原理`}),`
`,(0,n.jsxs)(r.p,{language:`txt`,children:[`当用户在终端中拖拽文件时，终端模拟器会将文件路径"粘贴"到输入流中。`,(0,n.jsx)(r.code,{language:`txt`,children:`drag_detect.rs`}),` 模块负责检测这种拖拽行为：`]}),`
`,(0,n.jsxs)(r.ol,{children:[`
`,(0,n.jsxs)(r.li,{children:[(0,n.jsx)(r.strong,{children:`路径检测`}),`：从输入流中提取文件路径`]}),`
`,(0,n.jsxs)(r.li,{children:[(0,n.jsx)(r.strong,{children:`路径验证`}),`：检查所有 token 是否都是合法的绝对路径（防止误判普通命令）`]}),`
`,(0,n.jsxs)(r.li,{children:[(0,n.jsx)(r.strong,{children:`文件存在性验证`}),`：确认路径在本地文件系统中确实存在`]}),`
`,(0,n.jsxs)(r.li,{language:`txt`,children:[(0,n.jsx)(r.strong,{children:`启动上传器`}),`：在新控制台窗口中启动 `,(0,n.jsx)(r.code,{language:`txt`,children:`qssh-uploader`}),` 进行上传`]}),`
`]}),`
`,(0,n.jsx)(r.h3,{id:`检测策略`,children:`检测策略`}),`
`,(0,n.jsxs)(i,{children:[(0,n.jsx)(o,{name:`Windows 路径`,type:`X:\\...`,children:(0,n.jsxs)(r.p,{language:`txt`,children:[`检测 `,(0,n.jsx)(r.code,{language:`txt`,children:`C:\\Users\\...`}),` 格式的 Windows 绝对路径`]})}),(0,n.jsx)(o,{name:`Unix 路径`,type:`/home/...`,children:(0,n.jsxs)(r.p,{language:`txt`,children:[`检测 `,(0,n.jsx)(r.code,{language:`txt`,children:`/home/user/...`}),` 格式的 Unix 绝对路径`]})}),(0,n.jsx)(o,{name:`引用路径`,type:`带空格的路径`,children:(0,n.jsxs)(r.p,{language:`txt`,children:[`支持双引号/单引号包裹的路径，如 `,(0,n.jsx)(r.code,{language:`txt`,children:`"C:\\My Documents\\file.txt"`})]})}),(0,n.jsx)(o,{name:`多文件`,type:`空格分隔`,children:(0,n.jsx)(r.p,{children:`支持同时拖拽多个文件，所有路径必须都是合法的绝对路径`})})]}),`
`,(0,n.jsx)(r.h3,{id:`防误判机制`,children:`防误判机制`}),`
`,(0,n.jsxs)(r.p,{language:`txt`,children:[`为防止将普通命令（如 `,(0,n.jsx)(r.code,{language:`txt`,children:`ls -la /home/user`}),`）误判为拖拽操作，检测器要求`,(0,n.jsx)(r.strong,{children:`所有 token 都必须是绝对路径`}),`才会触发上传。只要有一个 token 不是合法路径，就视为普通键盘输入。`]}),`
`,(0,n.jsx)(r.h2,{id:`独立上传工具`,children:`独立上传工具`}),`
`,(0,n.jsxs)(r.p,{language:`txt`,children:[(0,n.jsx)(r.code,{language:`txt`,children:`qssh-uploader`}),` 是一个独立的可执行文件，与主程序一起分发。`]}),`
`,(0,n.jsx)(r.h3,{id:`用法`,children:`用法`}),`
`,(0,n.jsx)(n.Fragment,{children:(0,n.jsx)(r.pre,{className:`shiki css-variables`,style:{backgroundColor:`var(--shiki-background)`,color:`var(--shiki-foreground)`},tabIndex:`0`,children:(0,n.jsx)(r.code,{className:`language-bash`,children:(0,n.jsxs)(r.span,{className:`line`,children:[(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-function)`},children:`qssh-uploader`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` --host`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-keyword)`},children:` <`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:`hos`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`t`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-keyword)`},children:`>`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:` [选项] `}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-keyword)`},children:`<`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`本地文件`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-keyword)`},children:`>`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`...`})]})})})}),`
`,(0,n.jsx)(r.p,{children:(0,n.jsx)(r.strong,{children:`参数：`})}),`
`,(0,n.jsxs)(r.table,{children:[(0,n.jsx)(r.thead,{children:(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.th,{children:`参数`}),(0,n.jsx)(r.th,{children:`说明`})]})}),(0,n.jsxs)(r.tbody,{children:[(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`--host`})}),(0,n.jsx)(r.td,{children:`服务器地址（必填）`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`--alias`})}),(0,n.jsx)(r.td,{children:`主机别名（用于读取已保存密码并配置 AskPass）`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`--user`})}),(0,n.jsx)(r.td,{children:`SSH 用户名（可选）`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`--port`})}),(0,n.jsx)(r.td,{children:`SSH 端口（默认 22）`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`--key`})}),(0,n.jsx)(r.td,{children:`密钥文件路径（可选）`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`--remote-dir`})}),(0,n.jsxs)(r.td,{language:`txt`,children:[`远程目录（默认当前目录 `,(0,n.jsx)(r.code,{language:`txt`,children:`.`}),`）`]})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{children:`位置参数`}),(0,n.jsx)(r.td,{children:`要上传的本地文件路径（至少一个，必填）`})]})]})]}),`
`,(0,n.jsx)(r.p,{children:(0,n.jsx)(r.strong,{children:`示例：`})}),`
`,(0,n.jsx)(n.Fragment,{children:(0,n.jsx)(r.pre,{className:`shiki css-variables`,style:{backgroundColor:`var(--shiki-background)`,color:`var(--shiki-foreground)`},tabIndex:`0`,children:(0,n.jsxs)(r.code,{className:`language-bash`,children:[(0,n.jsx)(r.span,{className:`line`,children:(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-comment)`},children:`# 基本用法`})}),`
`,(0,n.jsxs)(r.span,{className:`line`,children:[(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-function)`},children:`qssh-uploader`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` --host`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-constant)`},children:` 192.168.1.100`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` --user`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` root`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` ./myfile.zip`})]}),`
`,(0,n.jsx)(r.span,{className:`line`}),`
`,(0,n.jsx)(r.span,{className:`line`,children:(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-comment)`},children:`# 指定别名、端口、密钥和远程目录`})}),`
`,(0,n.jsxs)(r.span,{className:`line`,children:[(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-function)`},children:`qssh-uploader`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` --alias`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` mysrv`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` --host`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-constant)`},children:` 192.168.1.100`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` --user`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` root`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` --port`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-constant)`},children:` 22`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:` \\`})]}),`
`,(0,n.jsxs)(r.span,{className:`line`,children:[(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:`  --key`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` ~/.ssh/id_rsa`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` --remote-dir`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` /tmp`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` ./file.zip`})]})]})})}),`
`,(0,n.jsx)(r.h3,{id:`功能介绍`,children:`功能介绍`}),`
`,(0,n.jsxs)(r.ul,{children:[`
`,(0,n.jsxs)(r.li,{language:`txt`,children:[(0,n.jsx)(r.strong,{children:`顺序上传`}),`：逐个文件通过 `,(0,n.jsx)(r.code,{language:`txt`,children:`scp`}),` 传输，进度由 `,(0,n.jsx)(r.code,{language:`txt`,children:`scp`}),` 原生显示（继承控制台，因此能检测到 TTY）`]}),`
`,(0,n.jsxs)(r.li,{children:[(0,n.jsx)(r.strong,{children:`逐文件结果`}),`：每个文件完成后打印成功 / 失败，最后汇总成功数与总耗时`]}),`
`,(0,n.jsxs)(r.li,{children:[(0,n.jsx)(r.strong,{children:`防闪退设计`}),`：启动时延迟 300ms 等待控制台初始化；panic hook 循环等待 Enter，确保窗口不闪退`]}),`
`,(0,n.jsxs)(r.li,{language:`txt`,children:[(0,n.jsx)(r.strong,{children:`错误日志`}),`：错误日志写入 `,(0,n.jsx)(r.code,{language:`txt`,children:`%TEMP%\\qssh-uploader.log`})]}),`
`]}),`
`,(0,n.jsx)(r.h3,{id:`上传输出示例`,children:`上传输出示例`}),`
`,(0,n.jsx)(r.pre,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`qssh-uploader - 文件上传工具\r
目标: root@192.168.1.100:22\r
\r
[1/2] 上传 project.zip ...\r
\r
project.zip    45%   45.2MB  10.1MB/s   00:02 ETA\r
project.zip   100%   45.2MB  11.3MB/s   00:04\r
\r
  -> 上传成功: project.zip\r
\r
[2/2] 上传 README.md ...\r
\r
  -> 上传成功: README.md\r
\r
全部完成! 共 2 个文件。耗时: 5s
`})}),`
`,(0,n.jsx)(r.h2,{id:`内联上传预留`,children:`内联上传（预留）`}),`
`,(0,n.jsxs)(r.p,{language:`txt`,children:[(0,n.jsx)(r.code,{language:`txt`,children:`qssh/src/ssh/upload.rs`}),` 中预留了内联上传实现（当前被 `,(0,n.jsx)(r.code,{language:`txt`,children:`#[allow(dead_code)]`}),` 标记）。该方案使用 `,(0,n.jsx)(r.code,{language:`txt`,children:`scp`}),` 命令在同一终端内完成上传，不启动独立窗口。`]}),`
`,(0,n.jsx)(r.p,{children:`设计要点：`}),`
`,(0,n.jsxs)(r.ul,{children:[`
`,(0,n.jsxs)(r.li,{language:`txt`,children:[`每个文件调用一次 `,(0,n.jsx)(r.code,{language:`txt`,children:`scp`}),`，上传完成后显示结果`]}),`
`,(0,n.jsx)(r.li,{children:`上传期间 SSH 终端暂时冻结（stdin 转发暂停），但 stdout/stderr 仍正常显示`}),`
`,(0,n.jsxs)(r.li,{language:`txt`,children:[`使用 `,(0,n.jsx)(r.code,{language:`txt`,children:`-o ControlMaster=no`}),` 避免与现有 SSH 会话冲突`]}),`
`]}),`
`,(0,n.jsxs)(r.p,{language:`txt`,children:[`当前版本默认使用独立的 `,(0,n.jsx)(r.code,{language:`txt`,children:`qssh-uploader`}),` 进程方案。内联上传模块保留以备将来可能的上传策略切换。`]})]})}function i(t={}){let{wrapper:i}={...e(),...t.components};return i?(0,n.jsx)(i,{...t,children:(0,n.jsx)(r,{...t})}):r(t)}function a(e,t){throw Error(`Expected `+(t?`component`:`object`)+" `"+e+"` to be defined: you likely forgot to import, pass, or provide it.")}export{i as default};