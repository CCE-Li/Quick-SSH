import{n as e}from"./vendor-CQh2k-GV.js";import{n as t}from"./react-vendor-6GaKtW3l.js";var n=t();function r(t){let r={a:`a`,code:`code`,h1:`h1`,h2:`h2`,h3:`h3`,li:`li`,p:`p`,pre:`pre`,span:`span`,table:`table`,tbody:`tbody`,td:`td`,th:`th`,thead:`thead`,tr:`tr`,ul:`ul`,...e(),...t.components};return(0,n.jsxs)(n.Fragment,{children:[(0,n.jsx)(r.h1,{id:`发布流程`,children:`发布流程`}),`
`,(0,n.jsx)(r.p,{children:`Quick-SSH 使用 GitHub Actions 自动构建跨平台二进制并发布到 GitHub Release。`}),`
`,(0,n.jsx)(r.h2,{id:`发布流程图`,children:`发布流程图`}),`
`,(0,n.jsx)(r.pre,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`打 git tag v*
     ↓
GitHub Actions 自动构建
     ↓
生成各平台归档 + SHA256
     ↓
上传到 GitHub Release
     ↓
运行 update-packaging.ps1
     ↓
更新 packaging 目录下所有包管理器配置
     ↓
提交到各包管理器仓库
`})}),`
`,(0,n.jsx)(r.h2,{id:`完整发布步骤`,children:`完整发布步骤`}),`
`,(0,n.jsx)(r.h3,{id:`步骤-1更新版本号`,children:`步骤 1：更新版本号`}),`
`,(0,n.jsxs)(r.p,{children:[`修改工作空间根目录的 `,(0,n.jsx)(r.a,{href:`/Cargo.toml`,language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`Cargo.toml`})}),` 中的版本号：`]}),`
`,(0,n.jsx)(n.Fragment,{children:(0,n.jsx)(r.pre,{className:`shiki css-variables`,style:{backgroundColor:`var(--shiki-background)`,color:`var(--shiki-foreground)`},tabIndex:`0`,children:(0,n.jsxs)(r.code,{className:`language-toml`,children:[(0,n.jsx)(r.span,{className:`line`,children:(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`[workspace.package]`})}),`
`,(0,n.jsxs)(r.span,{className:`line`,children:[(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`version `}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-keyword)`},children:`=`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string-expression)`},children:` "2.1.0"`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-comment)`},children:`  # 更新此处`})]})]})})}),`
`,(0,n.jsx)(r.h3,{id:`步骤-2提交并打标签`,children:`步骤 2：提交并打标签`}),`
`,(0,n.jsx)(n.Fragment,{children:(0,n.jsx)(r.pre,{className:`shiki css-variables`,style:{backgroundColor:`var(--shiki-background)`,color:`var(--shiki-foreground)`},tabIndex:`0`,children:(0,n.jsxs)(r.code,{className:`language-bash`,children:[(0,n.jsxs)(r.span,{className:`line`,children:[(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-function)`},children:`git`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` add`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` .`})]}),`
`,(0,n.jsxs)(r.span,{className:`line`,children:[(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-function)`},children:`git`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` commit`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` -m`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string-expression)`},children:` "chore: bump version to v2.1.0"`})]}),`
`,(0,n.jsxs)(r.span,{className:`line`,children:[(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-function)`},children:`git`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` tag`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` v2.1.0`})]}),`
`,(0,n.jsxs)(r.span,{className:`line`,children:[(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-function)`},children:`git`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` push`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-punctuation)`},children:` &&`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-function)`},children:` git`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` push`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` --tags`})]})]})})}),`
`,(0,n.jsx)(r.h3,{id:`步骤-3github-actions-自动构建`,children:`步骤 3：GitHub Actions 自动构建`}),`
`,(0,n.jsxs)(r.p,{language:`txt`,children:[`推送 `,(0,n.jsx)(r.code,{language:`txt`,children:`v*`}),` 标签后，`,(0,n.jsx)(r.a,{href:`/.github/workflows/release.yml`,language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`release.yml`})}),` 自动触发构建：`]}),`
`,(0,n.jsxs)(r.table,{children:[(0,n.jsx)(r.thead,{children:(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.th,{children:`平台`}),(0,n.jsx)(r.th,{children:`归档格式`}),(0,n.jsx)(r.th,{children:`产物`})]})}),(0,n.jsxs)(r.tbody,{children:[(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{children:`Linux x86_64`}),(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`.tar.gz`})}),(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`qssh-x86_64-linux.tar.gz`})})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{children:`macOS x86_64`}),(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`.tar.gz`})}),(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`qssh-x86_64-macos.tar.gz`})})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{children:`macOS ARM64`}),(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`.tar.gz`})}),(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`qssh-aarch64-macos.tar.gz`})})]}),(0,n.jsxs)(r.tr,{children:[(0,n.jsx)(r.td,{children:`Windows x86_64`}),(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`.zip`})}),(0,n.jsx)(r.td,{language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`qssh-x86_64-windows.zip`})})]})]})]}),`
`,(0,n.jsx)(r.p,{children:`每个归档包含：`}),`
`,(0,n.jsxs)(r.ul,{children:[`
`,(0,n.jsxs)(r.li,{language:`txt`,children:[(0,n.jsx)(r.code,{language:`txt`,children:`qssh`}),` / `,(0,n.jsx)(r.code,{language:`txt`,children:`qssh.exe`}),` — 主程序`]}),`
`,(0,n.jsxs)(r.li,{language:`txt`,children:[(0,n.jsx)(r.code,{language:`txt`,children:`qssh-uploader`}),` / `,(0,n.jsx)(r.code,{language:`txt`,children:`qssh-uploader.exe`}),` — 文件上传工具`]}),`
`,(0,n.jsxs)(r.li,{language:`txt`,children:[(0,n.jsx)(r.code,{language:`txt`,children:`LICENSE`}),` — MIT 许可证`]}),`
`,(0,n.jsxs)(r.li,{language:`txt`,children:[(0,n.jsx)(r.code,{language:`txt`,children:`README.md`}),` — 说明文档`]}),`
`,(0,n.jsxs)(r.li,{language:`txt`,children:[`Shell 补全脚本（`,(0,n.jsx)(r.code,{language:`txt`,children:`qssh.bash`}),` / `,(0,n.jsx)(r.code,{language:`txt`,children:`qssh.zsh`}),` / `,(0,n.jsx)(r.code,{language:`txt`,children:`qssh.fish`}),` / `,(0,n.jsx)(r.code,{language:`txt`,children:`qssh.ps1`}),`）`]}),`
`]}),`
`,(0,n.jsx)(r.h3,{id:`步骤-4更新包管理器配置`,children:`步骤 4：更新包管理器配置`}),`
`,(0,n.jsx)(n.Fragment,{children:(0,n.jsx)(r.pre,{className:`shiki css-variables`,style:{backgroundColor:`var(--shiki-background)`,color:`var(--shiki-foreground)`},tabIndex:`0`,children:(0,n.jsx)(r.code,{className:`language-powershell`,children:(0,n.jsxs)(r.span,{className:`line`,children:[(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`.`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-function)`},children:`\\scripts\\update-packaging.ps1`})]})})})}),`
`,(0,n.jsx)(r.p,{children:`此脚本自动更新所有包管理器配置文件中的版本号和 SHA256 校验和。`}),`
`,(0,n.jsx)(r.h3,{id:`步骤-5提交到各包管理器仓库`,children:`步骤 5：提交到各包管理器仓库`}),`
`,(0,n.jsxs)(r.p,{children:[`各包管理器的具体提交流程详见 `,(0,n.jsx)(r.a,{href:`/package-managers`,children:`包管理器参考`}),`。`]}),`
`,(0,n.jsx)(r.h2,{id:`cicd-配置`,children:`CI/CD 配置`}),`
`,(0,n.jsx)(r.h3,{id:`ci持续集成`,children:`CI（持续集成）`}),`
`,(0,n.jsxs)(r.p,{children:[(0,n.jsx)(r.a,{href:`/.github/workflows/ci.yml`,language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`ci.yml`})}),` — 每次推送自动检查：`]}),`
`,(0,n.jsxs)(r.ul,{children:[`
`,(0,n.jsxs)(r.li,{language:`txt`,children:[`编译检查（`,(0,n.jsx)(r.code,{language:`txt`,children:`cargo check`}),`）`]}),`
`,(0,n.jsxs)(r.li,{language:`txt`,children:[`运行测试（`,(0,n.jsx)(r.code,{language:`txt`,children:`cargo test`}),`）`]}),`
`,(0,n.jsxs)(r.li,{language:`txt`,children:[`Lint 检查（`,(0,n.jsx)(r.code,{language:`txt`,children:`cargo clippy`}),`）`]}),`
`]}),`
`,(0,n.jsx)(r.h3,{id:`release自动发布`,children:`Release（自动发布）`}),`
`,(0,n.jsxs)(r.p,{children:[(0,n.jsx)(r.a,{href:`/.github/workflows/release.yml`,language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`release.yml`})}),` — 打标签触发：`]}),`
`,(0,n.jsxs)(r.ul,{children:[`
`,(0,n.jsx)(r.li,{children:`多平台交叉编译`}),`
`,(0,n.jsx)(r.li,{children:`生成归档并计算 SHA256`}),`
`,(0,n.jsx)(r.li,{children:`上传到 GitHub Release`}),`
`,(0,n.jsx)(r.li,{children:`生成 Shell 补全脚本`}),`
`]}),`
`,(0,n.jsx)(r.h2,{id:`手动发布备选`,children:`手动发布（备选）`}),`
`,(0,n.jsx)(r.p,{children:`如果 GitHub Actions 不可用，也可以手动构建：`}),`
`,(0,n.jsx)(n.Fragment,{children:(0,n.jsx)(r.pre,{className:`shiki css-variables`,style:{backgroundColor:`var(--shiki-background)`,color:`var(--shiki-foreground)`},tabIndex:`0`,children:(0,n.jsxs)(r.code,{className:`language-bash`,children:[(0,n.jsx)(r.span,{className:`line`,children:(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-comment)`},children:`# 1. 编译`})}),`
`,(0,n.jsxs)(r.span,{className:`line`,children:[(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-function)`},children:`cargo`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` build`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` --release`})]}),`
`,(0,n.jsx)(r.span,{className:`line`}),`
`,(0,n.jsx)(r.span,{className:`line`,children:(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-comment)`},children:`# 2. 创建归档`})}),`
`,(0,n.jsxs)(r.span,{className:`line`,children:[(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-function)`},children:`mkdir`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` -p`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` dist/Quick-SSH-v2.0.1-x86_64-pc-windows-msvc`})]}),`
`,(0,n.jsxs)(r.span,{className:`line`,children:[(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-function)`},children:`cp`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` target/release/qssh.exe`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` dist/`})]}),`
`,(0,n.jsxs)(r.span,{className:`line`,children:[(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-function)`},children:`cp`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` target/release/qssh-uploader.exe`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` dist/`})]}),`
`,(0,n.jsxs)(r.span,{className:`line`,children:[(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-function)`},children:`cp`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` LICENSE`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` README.md`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` dist/`})]}),`
`,(0,n.jsxs)(r.span,{className:`line`,children:[(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-function)`},children:`cd`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` dist`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-punctuation)`},children:` &&`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-function)`},children:` zip`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` -r`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` Quick-SSH-v2.0.1-x86_64-pc-windows-msvc.zip`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` ./`})]}),`
`,(0,n.jsx)(r.span,{className:`line`}),`
`,(0,n.jsx)(r.span,{className:`line`,children:(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-comment)`},children:`# 3. 创建 GitHub Release`})}),`
`,(0,n.jsxs)(r.span,{className:`line`,children:[(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-function)`},children:`gh`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` release`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` create`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` v2.0.1`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` dist/*.zip`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` dist/*.tar.gz`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:` --generate-notes`})]}),`
`,(0,n.jsx)(r.span,{className:`line`}),`
`,(0,n.jsx)(r.span,{className:`line`,children:(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-comment)`},children:`# 4. 更新 packaging 配置`})}),`
`,(0,n.jsxs)(r.span,{className:`line`,children:[(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-function)`},children:`.`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`\\`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:`scripts`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`\\`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string)`},children:`update-packaging.ps1`})]})]})})}),`
`,(0,n.jsx)(r.h2,{id:`构建脚本`,children:`构建脚本`}),`
`,(0,n.jsxs)(r.p,{children:[(0,n.jsx)(r.a,{href:`/qssh/build.rs`,language:`txt`,children:(0,n.jsx)(r.code,{language:`txt`,children:`build.rs`})}),` — 主程序的构建脚本：`]}),`
`,(0,n.jsx)(n.Fragment,{children:(0,n.jsx)(r.pre,{className:`shiki css-variables`,style:{backgroundColor:`var(--shiki-background)`,color:`var(--shiki-foreground)`},tabIndex:`0`,children:(0,n.jsxs)(r.code,{className:`language-rust`,children:[(0,n.jsxs)(r.span,{className:`line`,children:[(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-keyword)`},children:`fn`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-function)`},children:` main`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`() {`})]}),`
`,(0,n.jsxs)(r.span,{className:`line`,children:[(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-function)`},children:`    println!`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`(`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-token-string-expression)`},children:`"cargo::rerun-if-changed=src/cli.rs"`}),(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`);`})]}),`
`,(0,n.jsx)(r.span,{className:`line`,children:(0,n.jsx)(r.span,{style:{color:`var(--shiki-foreground)`},children:`}`})})]})})}),`
`,(0,n.jsxs)(r.p,{language:`txt`,children:[`补全脚本通过 `,(0,n.jsx)(r.code,{language:`txt`,children:`qssh completions <shell>`}),` 子命令在运行时生成，构建时无需额外操作。Release 流程在 CI 中生成并打包补全文件。`]})]})}function i(t={}){let{wrapper:i}={...e(),...t.components};return i?(0,n.jsx)(i,{...t,children:(0,n.jsx)(r,{...t})}):r(t)}export{i as default};