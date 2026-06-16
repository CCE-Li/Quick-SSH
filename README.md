# Quick-SSH 🚀

> 仿 Docker 命令行风格的 PowerShell SSH 连接管理工具

![License](https://img.shields.io/badge/license-MIT-green)
![Platform](https://img.shields.io/badge/platform-Windows-blue)
![PowerShell](https://img.shields.io/badge/PowerShell-5.1%2B-blue)

一键保存、管理、连接 SSH 服务器，告别记忆繁琐的 IP、端口和密钥路径。

---

## 安装

### 方式一：npm 全局安装（推荐）

```powershell
npm install -g quick-ssh
```

安装完成后，**重启 PowerShell 终端**即可使用 `qssh` 命令。

> 安装脚本会自动将 `Import-Module` 写入你的 PowerShell 配置文件 (`$PROFILE`)，重启终端后永久生效。

### 方式二：手动加载

```powershell
# 下载本仓库，在 Quick-SSH.psm1 所在目录执行：
Import-Module .\Quick-SSH.psm1 -DisableNameChecking
```

---

## 快速入门

```powershell
# 添加一个 SSH 连接
qssh add my-server root@192.168.1.100:22 --key C:\Users\You\.ssh\id_rsa

# 一键连接
qssh my-server

# 列出所有连接
qssh ps

# 删除连接
qssh rm my-server
```

---

## 命令参考

### `qssh ps [关键词]`

列出所有已保存的 SSH 连接（对应 `docker ps`）。

| 列 | 说明 |
|----|------|
| 别名 | 连接的自定义名称 |
| IP 地址 | 服务器主机名或 IP |
| 账号 | 登录用户名 |
| 端口 | SSH 端口（默认 22） |
| 私钥路径 | 认证私钥文件路径 |

**示例：**

```powershell
# 列出全部连接
qssh ps

# 筛选包含 "生产" 的连接
qssh ps 生产

# 无连接时的提示
# → 当前没有已保存的 SSH 连接。使用 'qssh add' 添加一个。
```

### `qssh add <别名> <用户名@IP:端口> [--key <私钥路径>]`

新增 SSH 连接记录（对应 `docker run` 的添加语义）。

**参数说明：**

| 参数 | 必填 | 默认值 | 说明 |
|------|------|--------|------|
| 别名 | ✅ | - | 连接的唯一标识名 |
| 用户名@IP:端口 | ✅ | - | `root@192.168.1.100` 或 `root@192.168.1.100:2222` |
| `--key` / `-k` | ❌ | `%USERPROFILE%\.ssh\id_rsa` | 私钥文件路径 |

**示例：**

```powershell
# 使用默认端口 22 和默认密钥
qssh add my-vm root@10.0.0.5

# 指定端口和密钥
qssh add prod-server deploy@192.168.1.100:2222 --key D:\keys\prod_id_rsa

# 支持 -k 简写
qssh add test-vm admin@172.16.0.10 -k C:\Users\Me\.ssh\test_rsa

# 别名重复会报错
# → 错误：别名 'my-vm' 已存在，请使用其他名称。
```

### `qssh rm <别名>`

删除指定别名的 SSH 主机配置（对应 `docker rm`）。

```powershell
qssh rm my-vm
# → ✔ 已删除 SSH 连接 'my-vm'。

qssh rm unknown
# → 错误：别名 'unknown' 不存在。使用 'qssh ps' 查看可用连接。
```

### `qssh <别名>`

一键连接 SSH 服务器。自动读取保存的 IP、账号、端口、私钥发起会话。

> 内置兼容老旧 `ssh-rsa` 密钥协商参数 `-o HostKeyAlgorithms=+ssh-rsa`，无需手动添加。

```powershell
qssh my-server
# → 正在连接到 'my-server' (root@192.168.1.100:22) ...
```

### `qssh export <文件路径>`

将全部主机配置导出到指定 JSON 文件。

```powershell
qssh export D:\backup\ssh-hosts.json
# → ✔ 已导出 3 个连接到 'D:\backup\ssh-hosts.json'。
```

### `qssh import <文件路径>`

从外部 JSON 文件批量导入连接，自动跳过别名重复的记录。

```powershell
qssh import D:\backup\ssh-hosts.json
# → ✔ 导入完成：新增 5 个，跳过 2 个（别名重复）。
```

### `qssh help`

显示帮助信息。

---

## Tab 自动补全

输入 `qssh` 后按 <kbd>Tab</kbd> 键，可自动补全：

- **子命令**：`ps`、`add`、`rm`、`export`、`import`、`help`
- **已保存的主机别名**：快速选择要连接的服务器

---

## 配置文件

所有连接数据存储在以下位置，**不会在卸载时删除**：

```
%USERPROFILE%\.quickssh\hosts.json
```

示例配置文件内容：

```json
[
    {
        "alias": "my-server",
        "host": "192.168.1.100",
        "user": "root",
        "port": 22,
        "key": "C:\\Users\\You\\.ssh\\id_rsa"
    }
]
```

---

## 卸载

```powershell
npm uninstall -g quick-ssh
```

卸载时：
- ✅ 自动从 `$PROFILE` 中移除 `Import-Module` 配置
- ✅ **保留** `%USERPROFILE%\.quickssh\hosts.json` 用户配置数据

---

## 颜色输出说明

| 颜色 | 含义 |
|------|------|
| 🟢 绿色 | 操作成功 |
| 🟡 黄色 | 提示 / 警告 |
| 🔴 红色 | 错误 |

---

## 项目文件结构

```
quick-ssh/
├── Quick-SSH.psm1    # 核心 PowerShell 模块（全部逻辑）
├── index.js           # npm 生命周期钩子（安装/卸载自动配置）
├── package.json       # npm 包配置
├── README.md          # 本文档
└── LICENSE            # MIT 许可证
```

---

## License

[MIT](LICENSE)
