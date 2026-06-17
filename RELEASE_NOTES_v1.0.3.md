# Quick-SSH v1.0.3 — 模块化重构版

> PowerShell SSH 连接管理工具 — 像 Docker 一样管理你的 SSH 连接

## 🎯 主要更新

### ♻️ 文件结构重构（核心变更）
将原有的平铺结构重构为按职责划分的模块化布局，便于后续扩展：

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
- 添加新模式 → 仅需修改 `modes.js` + `index.js` 键位绑定
- 添加新 UI 组件 → 在 `index.js` 中创建并挂载
- 添加网络功能 → 在 `network.js` 中扩展
- 数据层完全独立，不依赖 UI

### ✨ 历史功能回顾

| 版本 | 功能 |
|------|------|
| v1.0.0 | 基础 CLI：qssh ps/add/rm/export/import |
| v1.0.1 | TUI 终端界面（blessed + Vim 键位） |
| v1.0.2 | 在线检测（p/P）、透明背景、Windows PowerShell 兼容 |
| **v1.0.3** | **模块化重构、文件结构调整** |

## 🚀 安装

### 首次安装
```powershell
npm install -g quick-ssh
# 重启 PowerShell 终端
```

### 更新
```powershell
npm update -g quick-ssh
# 重启 PowerShell 终端
```

### 手动加载（开发）
```powershell
# 在仓库根目录执行：
Import-Module .\src\Quick-SSH.psm1 -DisableNameChecking
```

## 📖 快速使用

```powershell
# 添加连接
qssh add my-server root@192.168.1.100:22

# 启动 TUI 界面
qssh

# 列出连接
qssh ps

# 一键连接
qssh my-server
```

## 🧩 技术栈

- **PowerShell 模块** — CLI 命令分发与参数补全
- **Node.js + blessed** — TUI 终端界面（类似 yazi 操作体验）
- **npm lifecycle hooks** — 安装/卸载自动管理 $PROFILE

## 📦 下载

- Source code (zip)
- Source code (tar.gz)

---

**Full Changelog**: https://github.com/CCE-Li/Quick-SSH/commits/v1.0.3
