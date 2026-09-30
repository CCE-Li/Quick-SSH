import{n as e}from"./vendor-CQh2k-GV.js";import{n as t}from"./react-vendor-6GaKtW3l.js";var n=t();function r(t){let r={a:`a`,code:`code`,h1:`h1`,h2:`h2`,h3:`h3`,hr:`hr`,li:`li`,p:`p`,pre:`pre`,span:`span`,strong:`strong`,table:`table`,tbody:`tbody`,td:`td`,th:`th`,thead:`thead`,tr:`tr`,ul:`ul`,...e(),...t.components};return(0,n.jsxs)(n.Fragment,{children:[(0,n.jsx)(r.hr,{}),`
`,(0,n.jsx)(r.p,{children:`title: 上传器
description: qssh-uploader 独立文件上传工具的详细文档，包含参数、顺序 SCP 上传、AskPass 复用与防闪退设计。
keywords:`}),`
`,(0,n.jsxs)(r.ul,{children:[`
`,(0,n.jsx)(r.li,{children:`uploader`}),`
`,(0,n.jsx)(r.li,{children:`scp`}),`
`,(0,n.jsx)(r.li,{children:`上传`}),`
`,(0,n.jsx)(r.li,{children:`askpass`}),`
`]}),`
`,(0,n.jsx)(r.hr,{}),`
`,(0,n.jsx)(r.h1,{id:`上传器`,children:`上传器`}),`
`,(0,n.jsxs)(r.p,{language:`txt`,children:[(0,n.jsx)(r.code,{language:`txt`,children:`qssh-uploader`}),` 是 Quick-SSH 的独立文件上传工具，位于 `,(0,n.jsx)(r.a,{href:`/qssh-uploader/`,language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`qssh-uploader/`})}),`，是一个独立的二进制 crate。`]}),`
`,(0,n.jsx)(r.h2,{id:`设计目标`,children:`设计目标`}),`
`,(0,n.jsxs)(r.ul,{children:[`
`,(0,n.jsxs)(r.li,{children:[(0,n.jsx)(r.strong,{children:`独立可执行`}),`：与主程序分开构建，可独立使用`]}),`
`,(0,n.jsxs)(r.li,{language:`txt`,children:[(0,n.jsx)(r.strong,{children:`顺序上传`}),`：逐个文件通过 `,(0,n.jsx)(r.code,{language:`txt`,children:`scp`}),` 传输`]}),`
`,(0,n.jsxs)(r.li,{language:`txt`,children:[(0,n.jsx)(r.strong,{children:`原生进度`}),`：`,(0,n.jsx)(r.code,{language:`txt`,children:`scp`}),` 继承控制台，检测到 TTY 后显示原生传输进度`]}),`
`,(0,n.jsxs)(r.li,{children:[(0,n.jsx)(r.strong,{children:`复用密码`}),`：通过 AskPass 复用系统凭据库中保存的密码`]}),`
`,(0,n.jsxs)(r.li,{children:[(0,n.jsx)(r.strong,{children:`防闪退`}),`：多种机制确保窗口在出错时不会立即关闭`]}),`
`]}),`
`,(0,n.jsx)(r.h2,{id:`架构概览`,children:`架构概览`}),`
`,(0,n.jsx)(r.pre,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`main()
  → 延迟 300ms（等待控制台初始化）
  → 安装 panic hook（防闪退）
  → run()
    → parse_args()            // 解析命令行参数
    → 打印目标信息
    → for 每个文件:
        → upload_via_scp()    // scp 继承控制台，显示原生进度
        → 打印成功 / 失败
    → 打印汇总（成功数 / 耗时）
`})}),`
`,(0,n.jsx)(r.h2,{id:`命令行接口`,children:`命令行接口`}),`
`,(0,n.jsx)(r.pre,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`qssh-uploader --host <host> [选项] <文件路径>...
`})}),`
`,(0,n.jsxs)(r.table,{children:[(0,n.jsx)(r.thead,{children:(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.th,{children:`参数`}),(0,n.jsx)(r.th,{children:`说明`})]})}),(0,n.jsxs)(r.tbody,{children:[(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`--alias <alias>`})}),(0,n.jsx)(r.td,{children:`主机别名，用于 AskPass 读取已保存密码`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`--host <host>`})}),(0,n.jsxs)(r.td,{children:[`服务器地址（`,(0,n.jsx)(r.strong,{children:`必填`}),`）`]})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`--user <user>`})}),(0,n.jsx)(r.td,{children:`SSH 用户名`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`--port <port>`})}),(0,n.jsx)(r.td,{children:`SSH 端口（默认 22）`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`--key <path>`})}),(0,n.jsx)(r.td,{children:`密钥文件路径`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`--remote-dir <dir>`})}),(0,n.jsxs)(r.td,{language:`txt`,children:[`远程目录（默认 `,(0,n.jsx)(r.code,{language:`txt`,children:`.`}),`）`]})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{children:`位置参数`}),(0,n.jsxs)(r.td,{children:[`要上传的本地文件（`,(0,n.jsx)(r.strong,{children:`至少一个`}),`）`]})]})]})]}),`
`,(0,n.jsxs)(r.p,{language:`txt`,children:[`未指定 `,(0,n.jsx)(r.code,{language:`txt`,children:`--host`}),` 或没有文件时会直接报错退出。`]}),`
`,(0,n.jsx)(r.h2,{id:`顺序上传`,children:`顺序上传`}),`
`,(0,n.jsxs)(r.p,{language:`txt`,children:[(0,n.jsx)(r.code,{language:`txt`,children:`run()`}),` 逐个文件调用一次 `,(0,n.jsx)(r.code,{language:`txt`,children:`upload_via_scp()`}),`：`]}),`
`,(0,n.jsx)(n.Fragment,{children:(0,n.jsx)(r.pre,{className:`shiki css-variables`,style:{backgroundColor:`var(--shiki-background)`,color:`var(--shiki-foreground)`},tabIndex:`0`,children:(0,n.jsxs)(r.code,{className:`language-rust`,children:[(0,n.jsxs)(r.span,{className:`line`,children:[(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-keyword)`},children:`for`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:` (i, file) `}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-keyword)`},children:`in`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:` args`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-keyword)`},children:`.`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`files`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-keyword)`},children:`.`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-function)`},children:`iter`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`()`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-keyword)`},children:`.`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-function)`},children:`enumerate`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`() {`})]}),`
`,(0,n.jsxs)(r.span,{className:`line`,children:[(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-function)`},children:`    println!`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`(`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string-expression)`},children:`"[{}/{}] 上传 {} ..."`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`, i `}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-keyword)`},children:`+`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-constant)`},children:` 1`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`, total, name);`})]}),`
`,(0,n.jsxs)(r.span,{className:`line`,children:[(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-keyword)`},children:`    match`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-function)`},children:` upload_via_scp`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`(`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-keyword)`},children:`&`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`file_args) { `}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-comment)`},children:`/* 成功 / 失败计数 */`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:` }`})]}),`
`,(0,n.jsx)(r.span,{className:`line`,children:(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`}`})})]})})}),`
`,(0,n.jsx)(r.h3,{id:`上传实现`,children:`上传实现`}),`
`,(0,n.jsxs)(r.p,{language:`txt`,children:[(0,n.jsx)(r.code,{language:`txt`,children:`upload_via_scp()`}),` 使用系统 `,(0,n.jsx)(r.code,{language:`txt`,children:`scp`}),`，目标为 `,(0,n.jsx)(r.code,{language:`txt`,children:`[user@]host:remote_dir/filename`}),`：`]}),`
`,(0,n.jsx)(n.Fragment,{children:(0,n.jsx)(r.pre,{className:`shiki css-variables`,style:{backgroundColor:`var(--shiki-background)`,color:`var(--shiki-foreground)`},tabIndex:`0`,children:(0,n.jsx)(r.code,{className:`language-bash`,children:(0,n.jsxs)(r.span,{className:`line`,children:[(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-function)`},children:`scp`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` -P`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-keyword)`},children:` <`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:`por`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`t`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-keyword)`},children:`>`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` -i`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-keyword)`},children:` <`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:`ke`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`y`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-keyword)`},children:`>`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` -o`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` ControlMaster=no`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-keyword)`},children:` <`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:`local_fil`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`e`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-keyword)`},children:`>`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:` [user@]host:remote_dir/filename`})]})})})}),`
`,(0,n.jsxs)(r.ul,{children:[`
`,(0,n.jsxs)(r.li,{language:`txt`,children:[`端口为 22 时省略 `,(0,n.jsx)(r.code,{language:`txt`,children:`-P`})]}),`
`,(0,n.jsxs)(r.li,{language:`txt`,children:[(0,n.jsx)(r.code,{language:`txt`,children:`-o ControlMaster=no`}),` 避免与现有 SSH 会话冲突`]}),`
`,(0,n.jsxs)(r.li,{language:`txt`,children:[`不管道化 stdout/stderr，让 `,(0,n.jsx)(r.code,{language:`txt`,children:`scp`}),` 检测到 TTY 并显示原生进度；需要密码时也能直接在窗口内输入`]}),`
`]}),`
`,(0,n.jsx)(r.h3,{id:`askpass-复用`,children:`AskPass 复用`}),`
`,(0,n.jsxs)(r.p,{language:`txt`,children:[`指定了 `,(0,n.jsx)(r.code,{language:`txt`,children:`--alias`}),` 时，`,(0,n.jsx)(r.code,{language:`txt`,children:`configure_scp_askpass()`}),` 把同目录下的 `,(0,n.jsx)(r.code,{language:`txt`,children:`qssh(.exe)`}),` 设为 AskPass 程序：`]}),`
`,(0,n.jsxs)(r.table,{children:[(0,n.jsx)(r.thead,{children:(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.th,{children:`环境变量`}),(0,n.jsx)(r.th,{children:`值`})]})}),(0,n.jsxs)(r.tbody,{children:[(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`SSH_ASKPASS`})}),(0,n.jsxs)(r.td,{language:`txt`,children:[`同目录的 `,(0,n.jsx)(r.code,{language:`txt`,children:`qssh.exe`})]})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`SSH_ASKPASS_REQUIRE`})}),(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`force`})})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`QSSH_ASKPASS_ACTIVE`})}),(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`1`})})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`QSSH_ASKPASS_ALIAS`})}),(0,n.jsx)(r.td,{children:`主机别名`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`DISPLAY`})}),(0,n.jsxs)(r.td,{language:`txt`,children:[`缺失时设置为 `,(0,n.jsx)(r.code,{language:`txt`,children:`qssh-askpass`})]})]})]})]}),`
`,(0,n.jsx)(r.h2,{id:`防闪退机制`,children:`防闪退机制`}),`
`,(0,n.jsxs)(r.ul,{children:[`
`,(0,n.jsxs)(r.li,{language:`txt`,children:[(0,n.jsx)(r.strong,{children:`启动延迟`}),`：`,(0,n.jsx)(r.code,{language:`txt`,children:`sleep(300ms)`}),` 等待控制台窗口初始化完成`]}),`
`,(0,n.jsxs)(r.li,{children:[(0,n.jsx)(r.strong,{children:`Panic hook`}),`：捕获 panic 后写入日志并循环等待 Enter`]}),`
`,(0,n.jsxs)(r.li,{language:`txt`,children:[(0,n.jsx)(r.strong,{children:`错误处理`}),`：`,(0,n.jsx)(r.code,{language:`txt`,children:`run()`}),` 出错时打印错误链，窗口由调用方的 `,(0,n.jsx)(r.code,{language:`txt`,children:`cmd /c ... & pause`}),` 保持打开`]}),`
`,(0,n.jsxs)(r.li,{language:`txt`,children:[(0,n.jsx)(r.strong,{children:`日志记录`}),`：所有错误信息同时写入 `,(0,n.jsx)(r.code,{language:`txt`,children:`%TEMP%\\qssh-uploader.log`})]}),`
`]}),`
`,(0,n.jsx)(r.h2,{id:`与主程序的集成`,children:`与主程序的集成`}),`
`,(0,n.jsxs)(r.p,{language:`txt`,children:[(0,n.jsx)(r.code,{language:`txt`,children:`qssh`}),` 主程序在检测到拖拽文件操作时，在新控制台窗口中启动上传器，并传入别名与目标参数：`]}),`
`,(0,n.jsx)(r.pre,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`qssh-uploader --alias <alias> --host <host> [--user <u>] [--port <p>] [--key <k>] --remote-dir <dir> <files...>
`})}),`
`,(0,n.jsxs)(r.p,{language:`txt`,children:[`上传结束后由 `,(0,n.jsx)(r.code,{language:`txt`,children:`cmd.exe ... & pause`}),` 保持窗口打开，方便查看结果。`]})]})}function i(t={}){let{wrapper:i}={...e(),...t.components};return i?(0,n.jsx)(i,{...t,children:(0,n.jsx)(r,{...t})}):r(t)}export{i as default};