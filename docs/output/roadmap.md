# 路线图

## v2.0.4（当前发布版本）

- [x] Rust 完整重写（移除 Node.js 依赖）
- [x] 事件驱动 TUI（ratatui + crossterm）
- [x] 完整 CLI 子命令（ps/add/rm/connect/export/import/help/web）
- [x] Shell 补全生成（bash/zsh/fish/powershell/elvish）
- [x] 独立 SCP 上传工具（qssh-uploader）
- [x] 渐进式 SSH 配置解析
- [x] 系统安全凭据库存储密码与 OpenSSH AskPass 自动填写
- [x] TUI 多行注释、认证方式详情与注释摘要
- [x] 标记主机批量删除并同步清理密码
- [x] Scoop/WinGet/Homebrew/AUR 配置与 APT 打包预留
- [x] CI/CD 自动构建与发布

## 开发中（main 分支，未发布）

- [x] Dashboard 监控工作台（12 个组件、可切换 Profile、鼠标拖动布局）
- [x] 运维面板：Docker / 系统服务 / 文件浏览 / 日志 / 远程命令
- [x] 嵌入式终端（portable-pty + vt100，Ctrl+B 前缀键、滚动回看、连接动画）
- [x] AI Agent（17 个工具、四级权限、审批流、时间线）
- [x] 本地 Web 界面（`qssh web`，schema 驱动设置页、模型连接测试）
- [x] 命令面板（Ctrl+K）
- [x] `Ctrl+Enter` 新窗口连接、`<` / `>` 主机调序
- [x] `~/.qsshrc` 设置接入 Web 界面

## v2.1（规划中）

- [ ] 批量命令执行（选中多台主机后执行命令）
- [ ] 自动检测更新（启动时检查 GitHub Release）
- [ ] 连接日志记录（会话时间、主机、退出码）
- [ ] TUI 主机分组与批量操作增强

## v2.2（远期规划）

- [ ] TUI 主题自定义
- [ ] 主机分组/标签
- [ ] 配置备份与恢复
- [ ] SSH 隧道管理
- [ ] 更多包管理器支持（Chocolatey、Nix）
