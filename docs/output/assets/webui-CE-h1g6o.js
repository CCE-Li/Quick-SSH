import{n as e}from"./vendor-CQh2k-GV.js";import{n as t}from"./react-vendor-6GaKtW3l.js";var n=t();function r(t){let r={a:`a`,code:`code`,h1:`h1`,h2:`h2`,h3:`h3`,hr:`hr`,li:`li`,p:`p`,pre:`pre`,span:`span`,strong:`strong`,table:`table`,tbody:`tbody`,td:`td`,th:`th`,thead:`thead`,tr:`tr`,ul:`ul`,...e(),...t.components};return(0,n.jsxs)(n.Fragment,{children:[(0,n.jsx)(r.hr,{}),`
`,(0,n.jsx)(r.p,{children:`title: Web 界面
description: 通过 qssh web 启动本地 WebUI，在浏览器中以表单方式查看状态并编辑 Agent、程序与 Dashboard 设置。
keywords:`}),`
`,(0,n.jsxs)(r.ul,{children:[`
`,(0,n.jsx)(r.li,{children:`web`}),`
`,(0,n.jsx)(r.li,{children:`webui`}),`
`,(0,n.jsx)(r.li,{children:`设置`}),`
`,(0,n.jsx)(r.li,{children:`配置`}),`
`,(0,n.jsx)(r.li,{children:`17890`}),`
`]}),`
`,(0,n.jsx)(r.hr,{}),`
`,(0,n.jsx)(r.h1,{id:`web-界面`,children:`Web 界面`}),`
`,(0,n.jsxs)(r.p,{language:`txt`,children:[(0,n.jsx)(r.code,{language:`txt`,children:`qssh web`}),` 会启动一个本地 Web 界面（WebUI），用于查看运行状态并用表单方式编辑配置。它只监听 `,(0,n.jsx)(r.code,{language:`txt`,children:`127.0.0.1`}),`，不对外网开放。`]}),`
`,(0,n.jsx)(n.Fragment,{children:(0,n.jsx)(r.pre,{className:`shiki css-variables`,style:{backgroundColor:`var(--shiki-background)`,color:`var(--shiki-foreground)`},tabIndex:`0`,children:(0,n.jsxs)(r.code,{className:`language-bash`,children:[(0,n.jsx)(r.span,{className:`line`,children:(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-comment)`},children:`# 使用默认端口 17890`})}),`
`,(0,n.jsxs)(r.span,{className:`line`,children:[(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-function)`},children:`qssh`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` web`})]}),`
`,(0,n.jsx)(r.span,{className:`line`}),`
`,(0,n.jsx)(r.span,{className:`line`,children:(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-comment)`},children:`# 指定端口`})}),`
`,(0,n.jsxs)(r.span,{className:`line`,children:[(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-function)`},children:`qssh`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` web`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` --port`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-constant)`},children:` 8080`})]}),`
`,(0,n.jsx)(r.span,{className:`line`,children:(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-comment)`},children:`# 或`})}),`
`,(0,n.jsxs)(r.span,{className:`line`,children:[(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-function)`},children:`qssh`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` web`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` -p`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-constant)`},children:` 8080`})]})]})})}),`
`,(0,n.jsxs)(r.p,{language:`txt`,children:[`启动后打开 `,(0,n.jsx)(r.code,{language:`txt`,children:`http://127.0.0.1:17890`}),` 即可。服务端是一个零依赖的极简 HTTP 服务（基于标准库 `,(0,n.jsx)(r.code,{language:`txt`,children:`TcpListener`}),`），无需任何额外运行时。`]}),`
`,(0,n.jsx)(r.h2,{id:`功能`,children:`功能`}),`
`,(0,n.jsxs)(r.table,{children:[(0,n.jsx)(r.thead,{children:(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.th,{children:`页面 / 接口`}),(0,n.jsx)(r.th,{children:`说明`})]})}),(0,n.jsxs)(r.tbody,{children:[(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{children:`设置表单`}),(0,n.jsx)(r.td,{children:`按分组编辑配置，保存即写回对应文件`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{children:`状态`}),(0,n.jsx)(r.td,{children:`版本号、当前 Agent 服务商 / 模型 / 权限 / 超时`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{children:`工具列表`}),(0,n.jsx)(r.td,{children:`列出 Agent 的全部工具及其危险等级与当前权限下的处理方式`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{children:`测试连接`}),(0,n.jsx)(r.td,{children:`向当前 Agent 服务商发一次最小请求，返回耗时、回复与可用模型列表`})]})]})]}),`
`,(0,n.jsx)(r.h2,{id:`设置分组`,children:`设置分组`}),`
`,(0,n.jsxs)(r.p,{children:[`WebUI 的设置页由`,(0,n.jsx)(r.strong,{children:`配置 schema 驱动`}),`，分三组：`]}),`
`,(0,n.jsx)(r.h3,{id:`agent`,children:`Agent`}),`
`,(0,n.jsxs)(r.p,{language:`txt`,children:[`对应 `,(0,n.jsx)(r.code,{language:`txt`,children:`agent.json`}),`。`]}),`
`,(0,n.jsxs)(r.table,{children:[(0,n.jsx)(r.thead,{children:(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.th,{children:`字段`}),(0,n.jsx)(r.th,{children:`控件`}),(0,n.jsx)(r.th,{children:`说明`})]})}),(0,n.jsxs)(r.tbody,{children:[(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`provider`})}),(0,n.jsx)(r.td,{children:`下拉`}),(0,n.jsx)(r.td,{children:`opencode-go（推荐）/ openai / ollama`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`api_key`})}),(0,n.jsx)(r.td,{children:`密码框`}),(0,n.jsx)(r.td,{children:`API Key`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`base_url`})}),(0,n.jsx)(r.td,{children:`文本框`}),(0,n.jsx)(r.td,{children:`API 地址`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`model`})}),(0,n.jsx)(r.td,{children:`文本框`}),(0,n.jsx)(r.td,{children:`模型名`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`permission`})}),(0,n.jsx)(r.td,{children:`下拉`}),(0,n.jsx)(r.td,{children:`ask_before_execute / read_only / auto_safe / full_access`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`timeout_secs`})}),(0,n.jsx)(r.td,{children:`数字`}),(0,n.jsx)(r.td,{children:`请求超时（5–600 秒）`})]})]})]}),`
`,(0,n.jsxs)(r.p,{language:`txt`,children:[`这一组带有 `,(0,n.jsx)(r.strong,{children:`「测试连接」`}),` 按钮，会调用 `,(0,n.jsx)(r.code,{language:`txt`,children:`/api/test`}),` 验证配置是否可用。`]}),`
`,(0,n.jsx)(r.h3,{id:`程序设置`,children:`程序设置`}),`
`,(0,n.jsxs)(r.p,{language:`txt`,children:[`对应 `,(0,n.jsx)(r.code,{language:`txt`,children:`~/.qsshrc`}),`。`]}),`
`,(0,n.jsxs)(r.table,{children:[(0,n.jsx)(r.thead,{children:(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.th,{children:`字段`}),(0,n.jsx)(r.th,{children:`范围`})]})}),(0,n.jsxs)(r.tbody,{children:[(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`default_port`})}),(0,n.jsx)(r.td,{children:`1–65535`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`ping_timeout_secs`})}),(0,n.jsx)(r.td,{children:`1–60`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`upload_concurrency`})}),(0,n.jsx)(r.td,{children:`1–16`})]})]})]}),`
`,(0,n.jsx)(r.h3,{id:`dashboard`,children:`Dashboard`}),`
`,(0,n.jsxs)(r.p,{language:`txt`,children:[`对应 `,(0,n.jsx)(r.code,{language:`txt`,children:`dashboard.json`}),`。`]}),`
`,(0,n.jsxs)(r.table,{children:[(0,n.jsx)(r.thead,{children:(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.th,{children:`字段`}),(0,n.jsx)(r.th,{children:`说明`})]})}),(0,n.jsxs)(r.tbody,{children:[(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`active_profile`})}),(0,n.jsx)(r.td,{children:`当前启用的 Dashboard Profile`})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`enabled`})}),(0,n.jsx)(r.td,{children:`启用的组件（多选，共 12 个组件）`})]})]})]}),`
`,(0,n.jsx)(r.h2,{id:`与-tui-的配合`,children:`与 TUI 的配合`}),`
`,(0,n.jsxs)(r.ul,{children:[`
`,(0,n.jsx)(r.li,{children:`Agent 配置在每次会话前热加载，WebUI 中改完立即对新的 Agent 会话生效。`}),`
`,(0,n.jsx)(r.li,{children:`Dashboard 布局/Profile 在 WebUI 中修改后，下次进入工作台时按启用的组件重建。`}),`
`,(0,n.jsxs)(r.li,{children:[`详细的字段含义见 `,(0,n.jsx)(r.a,{href:`/configuration`,children:`配置说明`}),`。`]}),`
`]}),`
`,(0,n.jsx)(r.h2,{id:`下一步`,children:`下一步`}),`
`,(0,n.jsxs)(r.ul,{children:[`
`,(0,n.jsxs)(r.li,{children:[(0,n.jsx)(r.a,{href:`/ai-agent`,children:`AI Agent`}),` — Agent 的权限与工具说明`]}),`
`,(0,n.jsxs)(r.li,{children:[(0,n.jsx)(r.a,{href:`/configuration`,children:`配置说明`}),` — 所有配置文件的位置与字段`]}),`
`]})]})}function i(t={}){let{wrapper:i}={...e(),...t.components};return i?(0,n.jsx)(i,{...t,children:(0,n.jsx)(r,{...t})}):r(t)}export{i as default};