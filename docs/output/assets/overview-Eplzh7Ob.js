import{n as e}from"./vendor-CQh2k-GV.js";import{n as t}from"./react-vendor-6GaKtW3l.js";var n=t();function r(t){let r={a:`a`,code:`code`,h1:`h1`,h2:`h2`,h3:`h3`,hr:`hr`,li:`li`,ol:`ol`,p:`p`,pre:`pre`,span:`span`,strong:`strong`,table:`table`,tbody:`tbody`,td:`td`,th:`th`,thead:`thead`,tr:`tr`,ul:`ul`,...e(),...t.components};return(0,n.jsxs)(n.Fragment,{children:[(0,n.jsx)(r.hr,{}),`
`,(0,n.jsx)(r.p,{children:`title: 打包总览\r
description: Quick-SSH 的包管理器配置总览，覆盖 Windows、macOS 和 Linux 平台。\r
keywords:`}),`
`,(0,n.jsxs)(r.ul,{children:[`
`,(0,n.jsx)(r.li,{children:`打包`}),`
`,(0,n.jsx)(r.li,{children:`包管理器`}),`
`,(0,n.jsx)(r.li,{children:`scoop`}),`
`,(0,n.jsx)(r.li,{children:`winget`}),`
`,(0,n.jsx)(r.li,{children:`homebrew`}),`
`,(0,n.jsx)(r.li,{children:`aur`}),`
`,(0,n.jsx)(r.li,{children:`apt`}),`
`,(0,n.jsx)(r.li,{children:`pacman`}),`
`]}),`
`,(0,n.jsx)(r.hr,{}),`
`,(0,n.jsx)(r.h1,{id:`打包总览`,children:`打包总览`}),`
`,(0,n.jsx)(r.p,{children:`Quick-SSH 为 5 类包管理器维护配置，覆盖三大操作系统。当前 Scoop 与 AUR 已可直接安装，其余渠道仍待发布。`}),`
`,(0,n.jsx)(r.h2,{id:`支持的包管理器`,children:`支持的包管理器`}),`
`,(0,n.jsxs)(r.table,{children:[(0,n.jsx)(r.thead,{children:(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.th,{children:`包管理器`}),(0,n.jsx)(r.th,{children:`平台`}),(0,n.jsx)(r.th,{children:`状态`}),(0,n.jsx)(r.th,{children:`配置位置`})]})}),(0,n.jsxs)(r.tbody,{children:[(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{children:(0,n.jsx)(r.strong,{children:`Scoop`})}),(0,n.jsx)(r.td,{children:`Windows`}),(0,n.jsx)(r.td,{children:`✅ 已发布`}),(0,n.jsx)(r.td,{children:(0,n.jsx)(r.a,{href:`/packaging/scoop/quick-ssh.json`,language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`packaging/scoop/quick-ssh.json`})})})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{children:(0,n.jsx)(r.strong,{children:`WinGet`})}),(0,n.jsx)(r.td,{children:`Windows`}),(0,n.jsx)(r.td,{children:`❌ 待发布`}),(0,n.jsx)(r.td,{children:(0,n.jsx)(r.a,{href:`/packaging/winget/`,language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`packaging/winget/`})})})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{children:(0,n.jsx)(r.strong,{children:`Homebrew`})}),(0,n.jsx)(r.td,{children:`macOS`}),(0,n.jsx)(r.td,{children:`⏳ 待发布`}),(0,n.jsx)(r.td,{children:(0,n.jsx)(r.a,{href:`/packaging/homebrew/quick-ssh.rb`,language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`packaging/homebrew/quick-ssh.rb`})})})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{children:(0,n.jsx)(r.strong,{children:`AUR`})}),(0,n.jsx)(r.td,{children:`Arch Linux`}),(0,n.jsx)(r.td,{children:`✅ 已发布`}),(0,n.jsx)(r.td,{children:(0,n.jsx)(r.a,{href:`/packaging/pacman/PKGBUILD`,language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`packaging/pacman/PKGBUILD`})})})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{children:(0,n.jsx)(r.strong,{children:`APT`})}),(0,n.jsx)(r.td,{children:`Debian/Ubuntu`}),(0,n.jsx)(r.td,{children:`❌ 仅打包配置`}),(0,n.jsx)(r.td,{children:(0,n.jsx)(r.a,{href:`/packaging/apt/`,language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`packaging/apt/`})})})]})]})]}),`
`,(0,n.jsx)(r.h2,{id:`配置目录结构`,children:`配置目录结构`}),`
`,(0,n.jsx)(r.pre,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`packaging/\r
├── scoop/\r
│   └── quick-ssh.json        # Scoop manifest\r
├── winget/\r
│   ├── CCE-Li.Quick-SSH.installer.yaml\r
│   ├── CCE-Li.Quick-SSH.locale.en-US.yaml\r
│   └── CCE-Li.Quick-SSH.yaml\r
├── homebrew/\r
│   └── quick-ssh.rb           # Homebrew Formula\r
├── pacman/\r
│   └── PKGBUILD               # AUR PKGBUILD\r
└── apt/\r
    ├── Makefile               # .deb 构建脚本\r
    └── DEBIAN/\r
        └── control            # APT 控制文件
`})}),`
`,(0,n.jsx)(r.h2,{id:`工作原理`,children:`工作原理`}),`
`,(0,n.jsxs)(r.p,{children:[`所有包管理器配置都引用了 `,(0,n.jsx)(r.strong,{children:`GitHub Release 上的预编译二进制文件`}),`。每次发布新版本后，运行 `,(0,n.jsx)(r.a,{href:`/scripts/update-packaging.ps1`,language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`update-packaging.ps1`})}),` 脚本即可自动更新所有配置。`]}),`
`,(0,n.jsx)(r.h3,{id:`工作流程`,children:`工作流程`}),`
`,(0,n.jsx)(r.pre,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`发布新版本 → 运行 update-packaging.ps1 → 自动更新所有配置\r
    ↓\r
各包管理器配置文件中的版本号和 SHA256 校验和同步更新\r
    ↓\r
提交到各包管理器仓库；APT 需另外构建并上传 .deb
`})}),`
`,(0,n.jsx)(r.h3,{id:`自动化脚本`,children:`自动化脚本`}),`
`,(0,n.jsxs)(r.p,{language:`txt`,children:[(0,n.jsx)(r.code,{language:`txt`,children:`scripts/update-packaging.ps1`}),` 的功能：`]}),`
`,(0,n.jsxs)(r.ol,{children:[`
`,(0,n.jsxs)(r.li,{language:`txt`,children:[(0,n.jsx)(r.strong,{children:`自动检测版本`}),` — 从 `,(0,n.jsx)(r.code,{language:`txt`,children:`Cargo.toml`}),` 读取当前版本号`]}),`
`,(0,n.jsxs)(r.li,{children:[(0,n.jsx)(r.strong,{children:`下载归档`}),` — 从 GitHub Release 拉取各平台归档文件`]}),`
`,(0,n.jsxs)(r.li,{children:[(0,n.jsx)(r.strong,{children:`计算 SHA256`}),` — 为每个文件生成校验和`]}),`
`,(0,n.jsxs)(r.li,{children:[(0,n.jsx)(r.strong,{children:`更新所有配置`}),` — 自动修改 version 和 hash/sha256 字段`]}),`
`,(0,n.jsxs)(r.li,{children:[(0,n.jsx)(r.strong,{children:`保存 SHA256SUMS`}),` — 生成校验和文件供验证`]}),`
`]}),`
`,(0,n.jsx)(n.Fragment,{children:(0,n.jsx)(r.pre,{className:`shiki css-variables`,style:{backgroundColor:`var(--shiki-background)`,color:`var(--shiki-foreground)`},tabIndex:`0`,children:(0,n.jsxs)(r.code,{className:`language-powershell`,children:[(0,n.jsx)(r.span,{className:`line`,children:(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-comment)`},children:`# 常用命令`})}),`
`,(0,n.jsxs)(r.span,{className:`line`,children:[(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`.`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-function)`},children:`\\scripts\\update-packaging.ps1`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-comment)`},children:`                    # 自动模式`})]}),`
`,(0,n.jsxs)(r.span,{className:`line`,children:[(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`.`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-function)`},children:`\\scripts\\update-packaging.ps1`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-keyword)`},children:` -`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`Version `}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string-expression)`},children:`"2.0.4"`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-comment)`},children:`    # 指定版本`})]}),`
`,(0,n.jsxs)(r.span,{className:`line`,children:[(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`.`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-function)`},children:`\\scripts\\update-packaging.ps1`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-keyword)`},children:` -`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`Help               `}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-comment)`},children:`# 查看帮助`})]}),`
`,(0,n.jsx)(r.span,{className:`line`}),`
`,(0,n.jsx)(r.span,{className:`line`,children:(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-comment)`},children:`# 离线使用（事先下载好归档文件）`})}),`
`,(0,n.jsxs)(r.span,{className:`line`,children:[(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`.`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-function)`},children:`\\scripts\\update-packaging.ps1`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-keyword)`},children:` -`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`Version `}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string-expression)`},children:`"2.1.0"`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-keyword)`},children:` -`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`LocalDir `}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string-expression)`},children:`".\\downloads"`})]})]})})})]})}function i(t={}){let{wrapper:i}={...e(),...t.components};return i?(0,n.jsx)(i,{...t,children:(0,n.jsx)(r,{...t})}):r(t)}export{i as default};