![License](https://img.shields.io/badge/license-MIT-green)
![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20Linux%20%7C%20macOS-blue)

**Quick-SSH** — 跨平台 SSH 连接管理器，提供 TUI 界面与 Docker 风格 CLI。

基于 Rust 实现，**无需 Node.js 运行时**，即下即用。

---

## 快速安装

### 方式一：包管理器

| 平台 | 命令 |
|------|------|
| **Scoop** (Windows) | `scoop bucket add extras && scoop install quick-ssh` |
| **winget** (Windows) | `winget install CCE-Li.Quick-SSH` |
| **Homebrew** (macOS) | `brew install quick-ssh` |
| **AUR** (Arch Linux) | `yay -S quick-ssh` |
| **APT** (Debian/Ubuntu) | 从 Release 下载 `.deb` 后用 `sudo dpkg -i` 安装 |

### 方式二：直接下载

从 [GitHub Releases](https://github.com/CCE-Li/Quick-SSH/releases) 下载对应平台的 `.zip` / `.tar.gz` 归档，解压后将二进制放入 `PATH` 即可。

```bash
# Linux / macOS
tar xzf qssh-x86_64-linux.tar.gz
sudo cp qssh/qssh /usr/local/bin/

# Windows — 解压 zip，将 qssh.exe 所在目录加入 PATH
```

---

## 5 分钟上手

### 启动 TUI

```bash
qssh
```

TUI 默认读取 `~/.ssh/config`。常用键位：

| 按键 | 功能 |
|------|------|
| `↑` / `↓` | 选择主机 |
| `Enter` | 连接选中主机 |
| `/` | 搜索 |
| `Space` | 标记/取消标记 |
| `d` | 删除选中 |
| `p` | Ping 检测 |
| `q` / `Esc` | 退出 / 取消搜索 |
| `?` | 帮助 |

### CLI 命令

```bash
qssh add mysrv root@192.168.1.100       # 添加主机
qssh add mysrv root@10.0.0.1 -k ~/.ssh/id_rsa  # 指定密钥
qssh ps                                  # 列出所有主机
qssh ps dev                              # 搜索含"dev"的主机
qssh mysrv                               # 一键连接
qssh rm mysrv                            # 删除主机
qssh export                              # 导出为 JSON
qssh import backup.json                  # 从 JSON 导入
```

### 文件上传

SSH 连接后，将本地文件或目录**拖入终端窗口**，Quick-SSH 会自动在新窗口中启动 SFTP 上传，显示每文件的进度条和总进度。

也可使用独立上传工具：

```bash
qssh-uploader mysrv ./myfile.zip /remote/path/
```

---

## 配置

所有连接数据保存在 **`~/.ssh/config`**（标准 OpenSSH 格式），Quick-SSH 会保留文件中所有非自己管理的内容。

程序行为可通过 **`~/.qsshrc** 配置：

```ini
UploadConcurrency=3
```

---

## 文档

| 文档 | 说明 |
|------|------|
| [`docs/architecture.md`](docs/architecture.md) | Rust 架构设计 |
| [`docs/release.md`](docs/release.md) | 发布流程与包管理器维护 |
| [`docs/roadmap.md`](docs/roadmap.md) | 开发路线图 |

---

## License

[MIT](LICENSE)
