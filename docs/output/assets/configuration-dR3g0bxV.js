import{n as e}from"./vendor-CQh2k-GV.js";import{n as t}from"./react-vendor-6GaKtW3l.js";var n=t();function r(t){let r={code:`code`,h1:`h1`,h2:`h2`,h3:`h3`,hr:`hr`,li:`li`,p:`p`,pre:`pre`,span:`span`,table:`table`,tbody:`tbody`,td:`td`,th:`th`,thead:`thead`,tr:`tr`,ul:`ul`,...e(),...t.components},{Callout:i,Note:o,Properties:s,Property:c}=r;return i||a(`Callout`,!0),o||a(`Note`,!0),s||a(`Properties`,!0),c||a(`Property`,!0),(0,n.jsxs)(n.Fragment,{children:[(0,n.jsx)(r.hr,{}),`
`,(0,n.jsx)(r.p,{children:`title: 配置说明
description: Quick-SSH 的配置系统说明，包括 SSH 配置格式和程序设置。
keywords:`}),`
`,(0,n.jsxs)(r.ul,{children:[`
`,(0,n.jsx)(r.li,{children:`配置`}),`
`,(0,n.jsx)(r.li,{children:`ssh config`}),`
`,(0,n.jsx)(r.li,{children:`qsshrc`}),`
`,(0,n.jsx)(r.li,{children:`设置`}),`
`]}),`
`,(0,n.jsx)(r.hr,{}),`
`,(0,n.jsx)(r.h1,{id:`配置说明`,children:`配置说明`}),`
`,(0,n.jsx)(r.p,{children:`Quick-SSH 使用两种配置：标准 OpenSSH 格式的 SSH 主机配置和 JSON 格式的程序设置。`}),`
`,(0,n.jsx)(r.h2,{id:`ssh-配置-sshconfig`,children:`SSH 配置 (~/.ssh/config)`}),`
`,(0,n.jsxs)(r.p,{language:`txt`,children:[`Quick-SSH 的所有连接数据保存在 `,(0,n.jsx)(r.code,{language:`txt`,children:`~/.ssh/config`}),` 文件中，使用标准 OpenSSH 格式。`]}),`
`,(0,n.jsx)(r.h3,{id:`格式`,children:`格式`}),`
`,(0,n.jsx)(n.Fragment,{children:(0,n.jsx)(r.pre,{className:`shiki css-variables`,style:{backgroundColor:`var(--shiki-background)`,color:`var(--shiki-foreground)`},tabIndex:`0`,children:(0,n.jsxs)(r.code,{className:`language-ini`,children:[(0,n.jsx)(r.span,{className:`line`,children:(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`Host mysrv`})}),`
`,(0,n.jsx)(r.span,{className:`line`,children:(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`    HostName 192.168.1.100`})}),`
`,(0,n.jsx)(r.span,{className:`line`,children:(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`    User root`})}),`
`,(0,n.jsx)(r.span,{className:`line`,children:(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`    Port 2222`})}),`
`,(0,n.jsx)(r.span,{className:`line`,children:(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`    IdentityFile ~/.ssh/id_rsa`})}),`
`,(0,n.jsx)(r.span,{className:`line`,children:(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`    ServerAliveInterval 60`})}),`
`,(0,n.jsx)(r.span,{className:`line`,children:(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-comment)`},children:`    # 自定义注释`})})]})})}),`
`,(0,n.jsx)(r.h3,{id:`托管字段`,children:`托管字段`}),`
`,(0,n.jsx)(r.p,{children:`Quick-SSH 主动管理的字段（TUI 编辑器和 CLI 命令可读写）：`}),`
`,(0,n.jsxs)(r.table,{children:[(0,n.jsx)(r.thead,{children:(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.th,{children:`字段`}),(0,n.jsx)(r.th,{children:`说明`})]})}),(0,n.jsxs)(r.tbody,{children:[(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`Host`})}),(0,n.jsx)(r.td,{children:`主机别名（用于快速连接和标识）`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`HostName`})}),(0,n.jsx)(r.td,{children:`主机地址（IP 或域名）`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`User`})}),(0,n.jsx)(r.td,{children:`登录用户名`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`Port`})}),(0,n.jsx)(r.td,{children:`SSH 端口（默认 22）`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`IdentityFile`})}),(0,n.jsx)(r.td,{children:`身份验证密钥文件路径`})]})]})]}),`
`,(0,n.jsx)(r.h3,{id:`非托管字段`,children:`非托管字段`}),`
`,(0,n.jsxs)(r.p,{language:`txt`,children:[`以下字段不会被 qssh 主动解析，但会作为 `,(0,n.jsx)(r.code,{language:`txt`,children:`Unknown(key, value)`}),` 忠实保留：`]}),`
`,(0,n.jsxs)(r.ul,{children:[`
`,(0,n.jsx)(r.li,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`ServerAliveInterval`})}),`
`,(0,n.jsx)(r.li,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`ProxyJump`})}),`
`,(0,n.jsx)(r.li,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`LocalForward`})}),`
`,(0,n.jsx)(r.li,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`StrictHostKeyChecking`})}),`
`,(0,n.jsx)(r.li,{children:`以及其他所有未被托管的 SSH 指令`}),`
`]}),`
`,(0,n.jsx)(i,{title:`信息完整性`,children:(0,n.jsxs)(r.p,{language:`txt`,children:[`Quick-SSH 采用渐进式解析策略，保证读写过程中不会丢失任何配置信息。全局配置（如 `,(0,n.jsx)(r.code,{language:`txt`,children:`Host *`}),`）、`,(0,n.jsx)(r.code,{language:`txt`,children:`Match`}),` 块、`,(0,n.jsx)(r.code,{language:`txt`,children:`Include`}),` 指令等复杂结构均完整保留在 preamble 中。`]})}),`
`,(0,n.jsx)(r.h3,{id:`配置路径`,children:`配置路径`}),`
`,(0,n.jsxs)(r.p,{language:`txt`,children:[`默认路径为 `,(0,n.jsx)(r.code,{language:`txt`,children:`~/.ssh/config`}),`。如果文件或目录不存在，Quick-SSH 会在首次操作时自动创建。`]}),`
`,(0,n.jsx)(r.h2,{id:`程序设置-qsshrc`,children:`程序设置 (~/.qsshrc)`}),`
`,(0,n.jsxs)(r.p,{language:`txt`,children:[`Quick-SSH 的程序行为可通过 `,(0,n.jsx)(r.code,{language:`txt`,children:`~/.qsshrc`}),` 文件配置。该文件使用 JSON 格式。`]}),`
`,(0,n.jsx)(r.h3,{id:`配置项`,children:`配置项`}),`
`,(0,n.jsxs)(s,{children:[(0,n.jsx)(c,{name:`default_port`,type:`u16`,default:`22`,children:(0,n.jsx)(r.p,{children:`默认 SSH 端口`})}),(0,n.jsx)(c,{name:`ping_timeout_secs`,type:`u64`,default:`3`,children:(0,n.jsx)(r.p,{children:`TCP Ping 超时秒数`})}),(0,n.jsx)(c,{name:`upload_concurrency`,type:`usize`,default:`3`,children:(0,n.jsx)(r.p,{children:`文件上传并发数`})}),(0,n.jsx)(c,{name:`ssh_config_path`,type:`Option<PathBuf>`,default:`null`,children:(0,n.jsx)(r.p,{children:`自定义 SSH 配置路径，为 null 时使用默认路径`})})]}),`
`,(0,n.jsx)(r.h3,{id:`示例`,children:`示例`}),`
`,(0,n.jsx)(n.Fragment,{children:(0,n.jsx)(r.pre,{className:`shiki css-variables`,style:{backgroundColor:`var(--shiki-background)`,color:`var(--shiki-foreground)`},tabIndex:`0`,title:`~/.qsshrc`,children:(0,n.jsxs)(r.code,{className:`language-json`,children:[(0,n.jsx)(r.span,{className:`line`,children:(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`{`})}),`
`,(0,n.jsxs)(r.span,{className:`line`,children:[(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-keyword)`},children:`  "default_port"`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-punctuation)`},children:`:`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-constant)`},children:` 22`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-punctuation)`},children:`,`})]}),`
`,(0,n.jsxs)(r.span,{className:`line`,children:[(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-keyword)`},children:`  "ping_timeout_secs"`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-punctuation)`},children:`:`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-constant)`},children:` 5`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-punctuation)`},children:`,`})]}),`
`,(0,n.jsxs)(r.span,{className:`line`,children:[(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-keyword)`},children:`  "upload_concurrency"`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-punctuation)`},children:`:`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-constant)`},children:` 3`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-punctuation)`},children:`,`})]}),`
`,(0,n.jsxs)(r.span,{className:`line`,children:[(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-keyword)`},children:`  "ssh_config_path"`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-punctuation)`},children:`:`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-constant)`},children:` null`})]}),`
`,(0,n.jsx)(r.span,{className:`line`,children:(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`}`})})]})})}),`
`,(0,n.jsx)(o,{children:(0,n.jsxs)(r.p,{language:`txt`,children:[`当前版本中 `,(0,n.jsx)(r.code,{language:`txt`,children:`~/.qsshrc`}),` 已预留但部分设置尚未启用，将在后续版本中逐步激活。`]})}),`
`,(0,n.jsx)(r.h2,{id:`配置工作原理`,children:`配置工作原理`}),`
`,(0,n.jsx)(r.h3,{id:`配置模块架构`,children:`配置模块架构`}),`
`,(0,n.jsx)(r.p,{children:`配置系统采用类型/解析/渲染分离的设计：`}),`
`,(0,n.jsx)(r.pre,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`SSH 配置文件 (文本)
       ↓
parser.rs → 文本解析 → types.rs (SshConfig / HostBlock / SshDirective)
       ↓
writer.rs → 数据结构 → 文本渲染 → 写回文件
`})}),`
`,(0,n.jsx)(r.h3,{id:`渐进式解析`,children:`渐进式解析`}),`
`,(0,n.jsxs)(r.p,{language:`txt`,children:[`解析器只识别 `,(0,n.jsx)(r.code,{language:`txt`,children:`Host`}),` / `,(0,n.jsx)(r.code,{language:`txt`,children:`HostName`}),` / `,(0,n.jsx)(r.code,{language:`txt`,children:`User`}),` / `,(0,n.jsx)(r.code,{language:`txt`,children:`Port`}),` / `,(0,n.jsx)(r.code,{language:`txt`,children:`IdentityFile`}),` 五个字段：`]}),`
`,(0,n.jsxs)(r.ul,{children:[`
`,(0,n.jsxs)(r.li,{language:`txt`,children:[`已知字段 → 解析为对应的 `,(0,n.jsx)(r.code,{language:`txt`,children:`SshDirective`}),` 枚举变体`]}),`
`,(0,n.jsxs)(r.li,{language:`txt`,children:[`未知字段 → 解析为 `,(0,n.jsx)(r.code,{language:`txt`,children:`SshDirective::Unknown(key, value)`}),` 保留原始内容`]}),`
`,(0,n.jsxs)(r.li,{language:`txt`,children:[`全局配置（非 Host 块内容）→ 保留在 `,(0,n.jsx)(r.code,{language:`txt`,children:`preamble`}),` 字段中`]}),`
`,(0,n.jsxs)(r.li,{language:`txt`,children:[`每个 Host 块的原始文本 → 保留在 `,(0,n.jsx)(r.code,{language:`txt`,children:`raw_text`}),` 字段中，用于忠实重建`]}),`
`]}),`
`,(0,n.jsx)(r.h3,{id:`渲染策略`,children:`渲染策略`}),`
`,(0,n.jsxs)(r.p,{language:`txt`,children:[`渲染器优先使用 `,(0,n.jsx)(r.code,{language:`txt`,children:`raw_text`}),` 重建 Host 块（保证注释和空白格式不变），仅在 `,(0,n.jsx)(r.code,{language:`txt`,children:`raw_text`}),` 为空时使用结构化指令渲染。`]})]})}function i(t={}){let{wrapper:i}={...e(),...t.components};return i?(0,n.jsx)(i,{...t,children:(0,n.jsx)(r,{...t})}):r(t)}function a(e,t){throw Error(`Expected `+(t?`component`:`object`)+" `"+e+"` to be defined: you likely forgot to import, pass, or provide it.")}export{i as default};