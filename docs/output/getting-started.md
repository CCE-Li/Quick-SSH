# 快速入门

5 分钟带你熟悉 Quick-SSH 的核心功能。

## 启动 TUI 界面

直接运行 `qssh` 即可启动 TUI 界面：

```bash
qssh
```

TUI 界面默认读取 `~/.ssh/config` 文件中的主机配置。

### 常用键位

| 按键 | 功能 |
|------|------|
| `↑` / `↓` 或 `j` / `k` | 选择主机 |
| `Enter` | 连接选中主机 |
| `/` | 搜索过滤 |
| `Space` | 标记/取消标记 |
| `a` | 添加新主机 |
| `e` | 编辑当前主机 |
| `d` | 删除选中主机 |
| `p` | Ping 检测当前主机 |
| `P` | Ping 检测所有主机 |
| `.` | 切换地址显示/隐藏 |
| `q` / `Esc` | 退出 / 取消搜索 |
| `?` | 显示帮助 |

<Callout title="地址隐私保护">
  默认情况下，主机地址以 `********` 显示，防止旁人窥屏。按 `.` 键可切换显示/隐藏。
</Callout>

## CLI 命令操作

### 添加主机

```bash
# 基本用法
qssh add mysrv root@192.168.1.100

# 指定密钥文件
qssh add mysrv root@10.0.0.1 -k ~/.ssh/id_rsa

# 指定端口
qssh add mysrv root@example.com -p 2222
```

### 列出主机

```bash
# 列出所有主机
qssh ps

# 搜索含 "dev" 的主机
qssh ps dev
```

### 连接主机

```bash
# 通过别名连接
qssh mysrv

# 或使用 connect 子命令
qssh connect mysrv

# 直接连接 user@host
qssh connect root@192.168.1.100

# 传递额外 SSH 参数（放在 -- 之后）
qssh mysrv -- -o "ServerAliveInterval=60"
```

### 删除主机

```bash
qssh rm mysrv
```

### 导出/导入配置

```bash
# 导出为 JSON
qssh export backup.json

# 从 JSON 导入
qssh import backup.json
```

## 文件上传

SSH 连接后，将本地文件或目录**拖入终端窗口**，Quick-SSH 会自动在新窗口中启动 SFTP 上传，显示每文件的进度条和总进度。

也可使用独立上传工具：

```bash
qssh-uploader mysrv ./myfile.zip /remote/path/
```

## 配置查看

所有连接数据保存在 `~/.ssh/config`（标准 OpenSSH 格式），Quick-SSH 会保留文件中所有非自己管理的内容。程序行为可通过 `~/.qsshrc` 配置（详见 [配置说明](/configuration)）。

## 下一步

- [CLI 命令参考](/cli-reference) — 完整的子命令文档
- [TUI 界面指南](/tui-guide) — TUI 使用详解
- [配置说明](/configuration) — SSH 配置与程序设置
- [文件上传](/file-upload) — 文件上传详细用法
