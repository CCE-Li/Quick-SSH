![Quick-SSH](doc/images/poster.png)

![License](https://img.shields.io/badge/license-MIT-green)
![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20Linux%20%7C%20macOS-blue)
![Binary](https://img.shields.io/badge/Binary-%E2%9C%93-orange)

一个跨平台的 SSH CLI客户端，支持 Windows、Linux 和 macOS。

> 💡 **遇到问题？** 请查阅 [常见问题 (FAQ)](doc/FAQ.md)，其中收录了密钥权限、终端恢复、编码错误等常见问题的解决方案。

---

## 功能特性 ✨

| 功能 | 说明 |
|------|------|
| 🔌 SSH 连接 | 一键连接已保存的 SSH 服务器 |
| 🖥️ TUI 终端界面 | 可视化键盘操作界面，类 `yazi` 操作体验 |
| ⌨️ Docker 风格 CLI | `ps`、`add`、`rm` 等子命令，上手即用 |
| 🏓 Ping 检测 | 列表中实时显示各主机连通状态（在线/离线） |
| 📤 拖拽上传 | 连接后可直接把本地文件或目录拖进终端，在新终端窗口中显示进度并上传到远端当前目录 |
| 📦 原生二进制 | **预编译可执行文件**，无需 Node.js 运行时，即下即用 |
| 🎯 全平台包管理 | **npm** / **yarn** / **Scoop** / **winget** / **apt** / **Pacman(AUR)** 全支持 |
| 🔄 导入/导出 | JSON 格式批量导入导出主机配置 |
| ⏹ Tab 自动补全 | 子命令 + 已保存主机别名自动补全 |
| 🌐 跨平台 | **Windows / Linux / macOS** 统一二进制，无需运行时依赖 |

---

## 工具截图


<img src="doc/images/sample01.png" width="45%" /> <img src="doc/images/sample02.png" width="45%" />

tui界面：

<div align="center">
<img src="doc/images/tui.gif" width="85%" />
</div>

拖拽上传：

<div align="center">
<img src="doc/images/upload.gif" width="85%" />
</div>


---

## 安装前准备 ⚠️

1. **确保已安装 SSH 客户端**（Windows 10+ / Linux / macOS 通常自带）
2. **确保 `~/.ssh/` 目录权限正确**（Linux/macOS: `chmod 700 ~/.ssh`）
3. **Windows 二进制版本无需 PowerShell 执行策略修改**

---

## 安装

Quick-SSH 提供 **原生二进制** 和 **npm 源码** 两种安装方式。

###  方式一：npm 全局安装

```bash
npm install -g quick-ssh
```

安装后会自动将二进制所在目录添加到 PATH，重启终端即可使用。

> npm 包中同时包含了源码和预编译二进制，`postinstall` 脚本自动执行 PATH 注册。


###  方式二：直接下载二进制

从 [GitHub Releases](https://github.com/CCE-Li/Quick-SSH/releases) 下载对应平台的二进制文件


下载完成并放入 PATH 后，打开新终端即可使用 `qssh` 命令。

```bash
qssh  # 启动 TUI 界面
```

> 无需安装 Node.js、Python 或任何运行时。单个文件，即下即用。



---

## 快速入门

```bash
# 直接输入 qssh 进入 TUI 界面
qssh

# 查看帮助
qssh help

# 添加一个 SSH 连接（Linux/macOS）
qssh add my-server root@192.168.1.100:22 --key ~/.ssh/id_rsa

# 添加一个 SSH 连接（Windows）
qssh add my-server root@192.168.1.100:22 --key C:\Users\You\.ssh\id_rsa

# 一键连接
qssh my-server

# 列出所有连接
qssh ps

# 删除连接
qssh rm my-server
```

TUI 常用键位：

- `Space` 选择/取消当前连接，可用于批量操作
- `P` 批量检测已选连接；未选择时检测当前筛选结果中的全部连接
- `d` 删除当前连接；若已多选则批量删除

---

## 连接配置

Quick-SSH 使用 **标准 OpenSSH 配置文件** (`~/.ssh/config`) 存储所有连接数据，**Win/Linux/macOS 完全统一**，不会在卸载时删除。

```
~/.ssh/config
```

你可以直接用文本编辑器编辑该文件，也可以使用 `qssh` 命令管理。以下是 Quick-SSH 管理的 Host 块示例：

```
Host my-server
    HostName 192.168.1.100
    User root
    Port 22
    IdentityFile ~/.ssh/id_rsa
```

> 除了 Quick-SSH 管理的 Host 块外，`~/.ssh/config` 中还可以包含全局选项、注释、以及其他 SSH 客户端（如原生 `ssh` 命令）使用的配置，Quick-SSH 会保留所有非自己管理的内容。

---

## Quick-SSH 配置

拖拽上传的运行参数暂时从 `~/.qsshrc` 读取。

示例：

```ini
UploadConcurrency=3
```

当前支持项：

- `UploadConcurrency`
  - 控制拖拽上传时新终端窗口内的并发上传数
  - 默认值为 `3`
  - 必须是大于 `0` 的整数

---

## 拖拽上传

当你通过 `qssh <alias>` 或 TUI 发起 SSH 会话后，Quick-SSH 会自动开启拖拽检测。

- 把本地文件或目录拖进当前终端
- Quick-SSH 会拦截终端粘贴进来的本地路径
- 然后自动打开一个新的本地终端窗口，用 SFTP 上传到远端当前目录

当前行为说明：

- 默认目标目录是你当前 shell 所在的远端目录
- 目录会自动递归上传
- 新终端窗口中会以固定列表方式显示每个文件的进度条、剩余时间和总进度条
- 新终端窗口内会按 `~/.qsshrc` 中的 `UploadConcurrency` 设置并发上传，默认 `3`
- 传输窗口在你按下 Enter 后会直接关闭
- 原 SSH 会话不会被上传任务占用，你可以继续操作
- 该功能依赖本机可用的 `ssh`，上传窗口内部使用 `ssh2`/SFTP

---

## 近期规划 🗓️

| 功能 | 说明 |
|------|------|
| 🔔 自动检测更新 | 启动时检查 npm 新版本并提示升级 |
| 🔄 批量执行 | 选中多台主机，批量发送同一命令 |
| 📋 一键保存 Log | 记录每次连接的时间戳与操作日志 |
| 🪟 新窗口打开 | 连接服务器时自动打开新终端窗口 |
| 🤖 AI 辅助指令 | 自然语言描述操作意图，AI 生成对应命令 |
| 📦 全平台包覆盖 | 实现 npm / yarn / Scoop / winget / apt / Pacman(AUR) |

---

## 卸载

### npm / yarn

```bash
npm uninstall -g quick-ssh
```

卸载时：
- ✅ **自动清理 PATH**: `preuninstall` 从 Shell 配置文件中移除二进制路径
- ✅ **保留** `~/.ssh/config` 用户配置数据（不会在卸载时删除）

---

### 架构概览

Quick-SSH 使用 **pkg** 将 Node.js 源码编译为 **原生二进制可执行文件**，直接运行，无需解释器。

```
                        快速开始:
                        ┌─────────────────┐
                        │  下载二进制文件   │
                        │  放入 PATH 目录   │
                        └────────┬────────┘
                                 │
                        ┌────────▼────────┐
                        │  qssh 命令       │
                        │  (原生二进制)     │
                        └────────┬────────┘
                                 │
                    ┌────────────┼────────────┐
                    ▼            ▼            ▼
               Windows       Linux        macOS
               qssh.exe    qssh          qssh
                    │            │            │
                    ▼            ▼            ▼
               ~/.ssh/config (跨平台统一数据存储)
                    │            │            │
                    ▼            ▼            ▼
                SSH / SFTP 连接管理
```


---

## 项目文件结构

```
quick-ssh/
├── src/                          # 源码目录
│   ├── unix/
│   │   └── cli.js              # 统一 CLI 入口（编译为二进制）
│   ├── lib/
│   │   ├── index.js            # npm 生命周期钩子（将二进制路径写入 PATH）
│   │   ├── session.js          # SSH 会话代理（拖拽检测 + scp 上传）
│   │   └── upload_runner.js    # 独立上传窗口脚本（scp 进度 + 结果汇总）
│   └── tui/
│       ├── index.js            # TUI 主入口（Blessed 界面 + 键位绑定）
│       ├── modes.js            # 模式常量/标签/提示
│       ├── data.js             # 数据层（配置读写，CLI 和 TUI 共用）
│       └── network.js          # 网络层（SSH 连接 + 在线检测）
├── scripts/
│   └── build.js                # 构建脚本（pkg 编译为原生二进制）
├── packaging/                   # 多包管理器配置文件
│   ├── scoop/
│   │   └── quick-ssh.json      # Scoop 清单（分发 .exe）
│   ├── winget/
│   │   └── CCE-Li.Quick-SSH.*.yaml  # winget 清单（分发 .exe）
│   ├── apt/
│   │   ├── DEBIAN/control      # Debian 包控制文件
│   │   └── Makefile            # .deb 构建脚本
│   └── pacman/
│       └── PKGBUILD            # Arch Linux 构建脚本
├── .github/
│   └── workflows/
│       └── release.yml         # CI/CD：构建二进制 → 发布 Release + npm
├── dist/                        # 编译产物（gitignored）
├── doc/                         # 文档
├── package.json                 # npm 包配置 + pkg 构建配置
├── README.md
├── LICENSE
└── .gitignore
```

---




### 核心模块

| 文件 | 作用 |
|------|------|
| [`scripts/build.js`](scripts/build.js) | **构建脚本** — 使用 `pkg` 将 Node.js 源码编译为原生二进制 |
| [`src/unix/cli.js`](src/unix/cli.js) | **统一 CLI 入口** — 所有平台共用，编译为二进制后直接执行 |
| [`src/lib/index.js`](src/lib/index.js) | **安装/卸载钩子** — 将二进制路径添加到 `PATH` 环境变量 |
| [`src/tui/data.js`](src/tui/data.js) | 共享数据层 — 读写 `~/.ssh/config`，路径归一化跨平台兼容 |
| [`src/tui/network.js`](src/tui/network.js) | `getSSHExe()` → `ssh.exe` (Windows) / `ssh` (Linux/macOS) |
| [`src/tui/index.js`](src/tui/index.js) | TUI 界面 — blessed 终端 UI，跨平台按键绑定 |
| [`src/tui/modes.js`](src/tui/modes.js) | 模式常量/标签/提示 |
| [`src/lib/session.js`](src/lib/session.js) | SSH 会话代理（拖拽检测 + scp 上传） |
| [`src/lib/upload_runner.js`](src/lib/upload_runner.js) | 独立上传窗口脚本（scp 进度 + 结果汇总） |


---


<details>
<summary>☕ Support Quick-SSH</summary>

<br>

如果 Quick-SSH 帮你节省了时间，欢迎请开发者喝杯咖啡 ☕

<table>
<tr>
<td align="center">
<b>微信支付</b><br>
<img src="doc\images\wechat-pay.jpg" width="250">
</td>

<td align="center">
<b>支付宝</b><br>
<img src="doc\images\alipay.jpg" width="250">
</td>
</tr>
</table>

感谢每一位支持 Quick-SSH 的朋友 ❤️

</details>



---

## License

[MIT](LICENSE)
