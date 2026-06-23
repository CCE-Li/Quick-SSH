![Quick-SSH](doc/images/poster.png)

![License](https://img.shields.io/badge/license-MIT-green)
![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20Linux%20%7C%20macOS-blue)
![PowerShell](https://img.shields.io/badge/PowerShell-5.1%2B%20%7C%20pwsh-blue)

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
| 📦 全平台包管理 | **npm** / **yarn** / **Scoop** / **winget** / **apt** / **Pacman(AUR)** 全支持 |
| 🔄 导入/导出 | JSON 格式批量导入导出主机配置 |
| ⏹ Tab 自动补全 | 子命令 + 已保存主机别名自动补全 |
| 🌐 跨平台 | **Windows**（PowerShell 模块）+ **Linux/macOS**（Node.js CLI）双后端 |

---

## 跨平台支持

Quick-SSH 支持 **Windows / Linux / macOS** 三大主流操作系统。

### 架构概览

Quick-SSH 根据操作系统选择不同的后端：

```
                        npm install -g quick-ssh
                                │
                                ▼
                         detectOS()
                        ┌────┴────┐
                        │         │
                     Windows    Linux / macOS
                        │         │
                        ▼         ▼
                  PowerShell    Node.js CLI
                  模块 (.psm1)   (src/unix/cli.js)
                        │         │
                        ▼         ▼
                  PowerShell    ~/.bashrc
                  $PROFILE      ~/.zshrc
                  (Import-      (qssh() 函数
                   Module)      调用 node)

  Windows 运行时:  PowerShell → Import-Module → qssh 命令
  Linux 运行时:    bash/zsh → qssh() → node src/unix/cli.js → ssh
```

### 系统检测与配置注入

| 操作系统 | 检测机制 | 配置文件 | 注入内容 | 运行时后端 |
|----------|----------|----------|----------|------------|
| **Windows** | `process.platform === "win32"` | PowerShell `$PROFILE` (PS7 / WinPS 5.1) | `Import-Module` 语句 | PowerShell 模块 ([`src/win/Quick-SSH.psm1`](src/win/Quick-SSH.psm1)) |
| **Linux** | `process.platform === "linux"` | `~/.bashrc` / `~/.zshrc` / `~/.profile` (自动检测 `$SHELL`) | `qssh()` → `node src/unix/cli.js "$@"` | Node.js CLI ([`src/unix/cli.js`](src/unix/cli.js)) |
| **macOS** | `process.platform === "darwin"` | `~/.zshrc` / `~/.bashrc` (自动检测 `$SHELL`) | `qssh()` → `node src/unix/cli.js "$@"` | Node.js CLI ([`src/unix/cli.js`](src/unix/cli.js)) |

> **Linux/macOS 无 PowerShell 依赖**：`qssh` 命令通过 `node src/unix/cli.js` 直接运行，不需要安装 PowerShell Core。所有功能（添加/删除/列出/连接/导入/导出/TUI）均由 Node.js 原生实现。
>
> **💡 WSL 用户注意**：在 WSL 中，Quick-SSH 使用 **Node.js CLI 后端**（与原生 Linux 相同），而不是 Windows PowerShell 后端。请确保 WSL 中已安装 `node` 和 `npm`。如果你的 `~/.ssh/config` 是从 Windows 侧共享的（如通过 `/mnt/c/` 挂载的符号链接），密钥路径中的反斜线会被 Quick-SSH 自动归一化为正斜线。

### 核心模块

| 文件 | 作用 |
|------|------|
| [`src/lib/index.js`](src/lib/index.js) | `detectOS()` → Windows(注入 `$PROFILE`) / Linux/macOS(注入 `~/.bashrc`/`~/.zshrc`) |
| [`src/win/Quick-SSH.psm1`](src/win/Quick-SSH.psm1) | **Windows 后端** — PowerShell 模块，实现所有 `qssh` 命令 |
| [`src/unix/cli.js`](src/unix/cli.js) | **Linux/macOS 后端** — Node.js CLI，实现所有 `qssh` 命令（复用 `data.js` 数据层） |
| [`src/tui/data.js`](src/tui/data.js) | 共享数据层 — 读写 `~/.ssh/config`，路径归一化跨平台兼容 |
| [`src/tui/network.js`](src/tui/network.js) | `getSSHExe()` → `ssh.exe` (Windows) / `ssh` (Linux/macOS) |
| [`src/tui/index.js`](src/tui/index.js) | TUI 界面 — blessed 终端 UI，跨平台按键绑定 |


---

## 安装前注意 ⚠️

1. Quick-SSH 依赖 PowerShell 执行策略运行脚本。安装前请先检查：

```powershell
Get-ExecutionPolicy
```

| 返回值 | 说明 | 操作 |
|--------|------|------|
| `RemoteSigned` 或 `Unrestricted` | ✅ 正常 | 直接安装即可 |
| `Restricted` | ❌ 无法运行脚本 | 需以管理员身份修改执行策略 |

**如果当前为 `Restricted`，请以管理员身份打开 PowerShell 并执行：**

```powershell
Set-ExecutionPolicy RemoteSigned -Scope CurrentUser
```

> 选择 `RemoteSigned` 表示仅信任来自互联网的脚本需要签名，本地脚本可直接运行，兼顾安全性与便利性。
>
> 如果希望更宽松（不推荐），可设为 `Unrestricted`。

修改完成后执行 `Get-ExecutionPolicy` 确认已生效，即可继续安装。

2. `.ssh/`文件夹的权限记得开够

---

## 安装

Quick-SSH 支持多种包管理器，选择适合你系统的安装方式。

### 📦 npm（推荐，跨平台）

```bash
npm install -g quick-ssh
```

安装脚本会自动检测你的操作系统并写入对应的配置文件：

| 操作系统 | 配置文件 | 注册方式 | 运行时后端 |
|----------|----------|----------|------------|
| **Windows** | PowerShell `$PROFILE` | `Import-Module` | PowerShell 模块 |
| **Linux** | `~/.bashrc` / `~/.zshrc` | `qssh()` 函数包装 | Node.js CLI |
| **macOS** | `~/.zshrc` / `~/.bashrc` | `qssh()` 函数包装 | Node.js CLI |

安装完成后：

- **Windows** → **重启 PowerShell 终端**，即可使用 `qssh` 命令
- **Linux/macOS** → 执行 `source ~/.bashrc`（或 `source ~/.zshrc`），即可使用 `qssh` 命令

> **对 Linux/macOS 用户**：Quick-SSH **不依赖 PowerShell**，所有功能通过 Node.js 原生实现。只需要系统已安装 `node`、`npm` 和 `ssh` 客户端即可。

如果希望在当前会话中立即加载（免重启）：

**Windows PowerShell：**
```powershell
& (Get-Content $PROFILE -Raw) | Invoke-Expression
```

**Linux/macOS bash/zsh：**
```bash
source ~/.bashrc
# 或
source ~/.zshrc
```

### 🧶 yarn（跨平台，npm 替代）

如果系统已安装 yarn，也可直接从 npm 注册表安装：

```bash
yarn global add quick-ssh
```

> yarn 与 npm 使用同一注册表，`postinstall`/`preuninstall` 生命周期钩子同样生效。

### 🪣 Scoop（Windows）

```powershell
# 添加 extras bucket（如果尚未添加）
scoop bucket add extras

# 安装 Quick-SSH
scoop install quick-ssh

# 注册到 PowerShell $PROFILE
qssh init
```

> Scoop 安装后需执行 `qssh init` 注册单次，或重启 PowerShell 终端后手动 `Import-Module`。

### 📟 winget（Windows）

```powershell
winget install CCE-Li.Quick-SSH
```

> winget 安装包会自动配置 PowerShell 模块路径，安装完成后重启终端即可使用 `qssh`。

### 🐧 apt（Debian / Ubuntu）

```bash
# 从 GitHub Releases 下载 .deb 包
wget https://github.com/CCE-Li/Quick-SSH/releases/latest/download/quick-ssh.deb

# 安装
sudo apt install ./quick-ssh.deb

# 注册到 Shell 配置文件
qssh init
```

> 或添加 APT 仓库（待支持）：
> ```bash
> echo "deb https://apt.fury.io/quick-ssh/ /" | sudo tee /etc/apt/sources.list.d/quick-ssh.list
> curl -fsSL https://apt.fury.io/quick-ssh/gpg.key | sudo gpg --dearmor -o /etc/apt/trusted.gpg.d/quick-ssh.gpg
> sudo apt update
> sudo apt install quick-ssh
> ```

### 🏔️ Pacman（Arch Linux / AUR）

```bash
# 方式一：通过 AUR helper（如 yay）
yay -S quick-ssh

# 方式二：手动构建
git clone https://aur.archlinux.org/quick-ssh.git
cd quick-ssh
makepkg -si
```

> AUR 包会在安装后自动注册 qssh 到 Shell 配置文件。

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

# 连接后可直接把文件拖进终端
# Quick-SSH 会自动识别拖拽路径，并上传到远端当前目录

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

## 配置文件

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
| 📦 全平台包覆盖 | 已实现 npm / yarn / Scoop / winget / apt / Pacman(AUR) |
| 🏪 提交至官方仓库 | Scoop extras bucket、winget-pkgs、AUR、Gemfury APT 仓库 |

---

## 卸载

### npm / yarn

```bash
npm uninstall -g quick-ssh
# 或
yarn global remove quick-ssh
```

卸载时：
- ✅ **Windows**: 自动从 PowerShell `$PROFILE` 中移除 `Import-Module` 配置
- ✅ **Linux/macOS**: 自动从 `~/.bashrc` / `~/.zshrc` 等文件中移除 `qssh()` 包装函数
- ✅ **保留** `~/.ssh/config` 用户配置数据（不会在卸载时删除）

### Scoop

```powershell
scoop uninstall quick-ssh
```

### winget

```powershell
winget uninstall CCE-Li.Quick-SSH
```

### apt

```bash
sudo apt remove quick-ssh
```

### Pacman

```bash
sudo pacman -R quick-ssh
```

---

## 项目文件结构

```
quick-ssh/
├── src/
│   ├── win/
│   │   └── Quick-SSH.psm1      # PowerShell 模块（Windows 后端）
│   ├── unix/
│   │   ├── cli.js              # Node.js CLI（Linux/macOS 后端）
│   │   └── install.sh          # Shell 配置文件注入脚本
│   ├── lib/
│   │   ├── index.js            # npm 生命周期钩子（安装/卸载自动配置）
│   │   ├── session.js          # SSH 会话代理（拖拽检测 + scp 上传）
│   │   └── upload_runner.js    # 独立上传窗口脚本（scp 进度 + 结果汇总）
│   └── tui/
│       ├── index.js            # TUI 主入口（Blessed 界面 + 键位绑定）
│       ├── modes.js            # 模式常量/标签/提示（易于扩展）
│       ├── data.js             # 数据层（配置读写，CLI 和 TUI 共用）
│       └── network.js          # 网络层（SSH 连接 + 在线检测）
├── packaging/                   # 多包管理器配置文件
│   ├── scoop/
│   │   └── quick-ssh.json      # Scoop 清单
│   ├── winget/
│   │   ├── CCE-Li.Quick-SSH.yaml
│   │   ├── CCE-Li.Quick-SSH.installer.yaml
│   │   └── CCE-Li.Quick-SSH.locale.en-US.yaml
│   ├── apt/
│   │   ├── DEBIAN/control      # Debian 包控制文件
│   │   └── Makefile            # .deb 构建脚本
│   └── pacman/
│       └── PKGBUILD            # Arch Linux 构建脚本
├── .github/
│   └── workflows/
│       └── release.yml         # CI/CD：自动发布到 npm + 构建 .deb + GitHub Release
├── doc/
│   ├── FAQ.md                   # 常见问题解答
│   └── images/                  # 截图展示
├── package.json                 # npm 包配置
├── README.md                    # 本文档
├── LICENSE                      # MIT 许可证
└── .gitignore
```
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
