# AGENTS.md

Quick-SSH — 跨平台 SSH 连接管理器（纯 Rust，无 Node.js）。一个 `qssh` 二进制同时提供 TUI 界面、CLI 子命令、AskPass 密码助手；另有独立上传工具 `qssh-uploader`。

## 必须遵守

- **每次完成任务后，在 `qssh/` 目录下执行 `cargo install --path .`**，把最新构建安装到 `~/.cargo/bin/qssh`。（package 名为 `quick-ssh`，产物二进制名为 `qssh`。）

## 开发命令（CI 全部强制执行，见 `.github/workflows/ci.yml`）

```bash
cargo check --workspace
cargo test --workspace
cargo fmt --all --check
cargo clippy --workspace -- -D warnings
```

- `cargo install --path .` 只在 `qssh/` 下有效；workspace 根目录跑会报 "package ID specification `qssh` did not match"。
- 注意 package 名是 `quick-ssh`，不是 `qssh`，`cargo build -p qssh` 是错的。

## 工作区结构

- 根 `Cargo.toml` 是 workspace：依赖集中在 `[workspace.dependencies]`，各成员用 `xxx.workspace = true` 引用。
- `qssh/` — 主二进制（入口 `src/main.rs`，CLI 定义 `src/cli.rs`）。
- `qssh-uploader/` — 独立 SCP 上传二进制。

## 架构要点

- **单二进制双模式**：`qssh` 无参数启动 TUI；子命令执行 CLI 操作；`cli.rs` 设了 `allow_external_subcommands` + `Bare(Vec<String>)`，所以任意未识别首参都会被视为连接目标（`qssh <alias>` 直接连接）。
- **不实现 SSH 协议**：一律 spawn 系统 `ssh`/`scp`。TUI 内嵌终端用 `portable-pty` 分配 PTY + `vt100` 解析 ANSI（`tui/term.rs`）。
- **AskPass 自调用**：保存过密码的主机 spawn ssh 时把 `current_exe` 设为 `SSH_ASKPASS`；`main.rs` 开头先调 `credentials::handle_askpass_request()`，若环境变量 `QSSH_ASKPASS_ACTIVE=1` 则直接打印密码并退出。因此运行中的 qssh 必须是最新构建，改动后 `cargo install` 才能让已安装的 qssh 正确自动填密码。
- **TUI 事件流**：`event.rs` 事件循环 → `keymap.rs` 按键映射 → `Action` → `App::apply()` → `ui.rs` 渲染，100ms tick；状态唯一来源是 `App`。Dashboard 监控走后台调度器 + mpsc 事件回传。

## 配置文件（改动前先确认改哪个）

| 文件 | 内容 |
|------|------|
| `~/.ssh/config` | 主机列表（标准 OpenSSH 格式） |
| `~/.qsshrc` | 程序设置（如 `UploadConcurrency=3`） |
| `~/.config/quick-ssh/dashboard.json`（Windows: `%APPDATA%\quick-ssh\dashboard.json`） | Dashboard 布局 / Profile |
| `~/.config/quick-ssh/agent.json` | AI Agent 配置（provider/model/权限/超时） |
| 系统凭据库（`keyring` crate） | 保存的密码，**永不写入** ssh config |

- `config/` 模块职责分离：`types.rs` 类型 → `parser.rs` 渐进式解析（保留未知指令与 preamble，不丢信息）→ `writer.rs` 忠实重建 → `settings.rs` 读 `~/.qsshrc`。
- Dashboard 布局加载时总是按 `enabled` 用 `default_layout_for` 重建，历史 JSON 残留的旧布局会被覆盖，别手动改 JSON 里的 layout。

## TUI 嵌入式终端（Mode::Terminal）

- 连接后键盘**默认全部转发给 PTY**（TUI 快捷键不生效，除非按前缀键）。
- `Ctrl+B` 是前缀键（tmux 风格），随后按键解释为 TUI 命令：`d` Docker、`s` 服务、`f` 文件、`l` 日志、`a` AI Agent、`k` 命令面板，`q`/`x` 断开，`Esc` 取消前缀；未识别按键放行给 PTY。
- 断开快捷键：`Esc` 或 `Ctrl+Shift+C`。`Ctrl+A` 在终端模式下会转发给远端 shell，不是 Agent 快捷键。
- 关闭操作面板（Docker/服务/文件/日志/Agent/命令面板/设置弹窗等）时若 SSH 仍连接，焦点回到终端（`Mode::Terminal`），键盘继续转发给 PTY；这是刻意设计（`App::restore_mode_after_overlay`）。

## 风格约定

- 注释与 UI 文案（状态栏提示、flash 消息、帮助、`Mode::label/hint`）用中文，保持一致。
- 单测内联在各模块 `#[cfg(test)]`；TUI 渲染测试用 `ratatui::TestBackend`（无需真实终端），终端渲染测试直接用 `vt100::Parser`。
- 预留/暂未实现项用 `#[allow(dead_code)]` 标注，不要删。

## 文档

- `docx/` — Markdown 设计文档（架构/路线图/发布）。
- `docs/` — Astro 文档站源码；`docs/output/` 是构建产物，**不要手动编辑**。
- 发布时用 `scripts/update-packaging.ps1` 刷新各包管理器清单。
