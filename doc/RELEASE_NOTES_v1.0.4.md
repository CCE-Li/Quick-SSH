# Quick-SSH v1.0.4 — 模块化重构 + Bug 修复版

> PowerShell SSH 连接管理工具 — 像 Docker 一样管理你的 SSH 连接

## ✨ 更新内容

### ♻️ 文件结构重构
将原有的平铺结构重构为按职责划分的模块化布局，大幅提高可扩展性：

```
quick-ssh/
├── src/
│   ├── Quick-SSH.psm1          # PowerShell 模块
│   ├── lib/
│   │   └── index.js             # npm 生命周期钩子
│   └── tui/
│       ├── index.js             # TUI 主入口（UI + 键位绑定）
│       ├── modes.js             # 模式常量（新增模式只需加一条）
│       ├── data.js              # 数据层（纯文件操作）
│       └── network.js           # 网络层（SSH + 检测）
├── doc/images/
├── package.json
├── README.md
├── LICENSE
└── .gitignore
```

**可扩展性提升：**
- 添加新模式 → 仅需修改 `modes.js` + 键位绑定
- 添加新 UI 组件 → 在 `index.js` 中创建并挂载
- 添加网络功能 → 在 `network.js` 中扩展
- 数据层完全独立，不依赖 UI

### 🐛 Bug 修复

| 问题 | 根因 | 修复 |
|------|------|------|
| `$Script:TUIScript` 为空导致 TUI 无法启动 | `$MyInvocation.MyCommand.Path` 在某些上下文中返回 null | 改用 `$PSScriptRoot` 自动变量 |
| `Join-Path` 参数错误 | PS5.1 只接受 2 个路径参数 | 改用 `[System.IO.Path]::Combine()` |

### 🚀 安装

```powershell
# 首次安装
npm install -g quick-ssh
# 重启 PowerShell 终端

# 更新
npm update -g quick-ssh
# 重启 PowerShell 终端
```

### 📖 快速入门

```powershell
# 启动 TUI 界面（推荐）
qssh

# 添加连接
qssh add my-server root@192.168.1.100:22

# 列出连接
qssh ps

# 一键连接
qssh my-server
```

### ⚙️ 技术栈

- **PowerShell 模块** — CLI 命令分发与参数补全
- **Node.js + blessed** — TUI 终端界面（类似 yazi 操作体验）
- **npm lifecycle hooks** — 安装/卸载自动管理 $PROFILE

### 📦 下载

- [Source code (zip)](https://github.com/CCE-Li/Quick-SSH/archive/refs/tags/v1.0.4.zip)
- [Source code (tar.gz)](https://github.com/CCE-Li/Quick-SSH/archive/refs/tags/v1.0.4.tar.gz)

---

**Full Changelog**: https://github.com/CCE-Li/Quick-SSH/commits/v1.0.4
