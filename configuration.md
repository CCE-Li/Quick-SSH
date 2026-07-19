# 配置说明

Quick-SSH 使用两种配置：标准 OpenSSH 格式的 SSH 主机配置和 JSON 格式的程序设置。

## SSH 配置 (~/.ssh/config)

Quick-SSH 的所有连接数据保存在 `~/.ssh/config` 文件中，使用标准 OpenSSH 格式。

### 格式

```ini
Host mysrv
    HostName 192.168.1.100
    User root
    Port 2222
    IdentityFile ~/.ssh/id_rsa
    ServerAliveInterval 60
    # 自定义注释
```

### 托管字段

Quick-SSH 主动管理的字段（TUI 编辑器和 CLI 命令可读写）：

| 字段 | 说明 |
|------|------|
| `Host` | 主机别名（用于快速连接和标识） |
| `HostName` | 主机地址（IP 或域名） |
| `User` | 登录用户名 |
| `Port` | SSH 端口（默认 22） |
| `IdentityFile` | 身份验证密钥文件路径 |

### 非托管字段

以下字段不会被 qssh 主动解析，但会作为 `Unknown(key, value)` 忠实保留：

- `ServerAliveInterval`
- `ProxyJump`
- `LocalForward`
- `StrictHostKeyChecking`
- 以及其他所有未被托管的 SSH 指令

<Callout title="信息完整性">
  Quick-SSH 采用渐进式解析策略，保证读写过程中不会丢失任何配置信息。全局配置（如 `Host *`）、`Match` 块、`Include` 指令等复杂结构均完整保留在 preamble 中。
</Callout>

### 配置路径

默认路径为 `~/.ssh/config`。如果文件或目录不存在，Quick-SSH 会在首次操作时自动创建。

## 程序设置 (~/.qsshrc)

Quick-SSH 的程序行为可通过 `~/.qsshrc` 文件配置。该文件使用 JSON 格式。

### 配置项

<Properties>
  <Property name="default_port" type="u16" default="22">
    默认 SSH 端口
  </Property>
  <Property name="ping_timeout_secs" type="u64" default="3">
    TCP Ping 超时秒数
  </Property>
  <Property name="upload_concurrency" type="usize" default="3">
    文件上传并发数
  </Property>
  <Property name="ssh_config_path" type="Option<PathBuf>" default="null">
    自定义 SSH 配置路径，为 null 时使用默认路径
  </Property>
</Properties>

### 示例

```json title="~/.qsshrc"
{
  "default_port": 22,
  "ping_timeout_secs": 5,
  "upload_concurrency": 3,
  "ssh_config_path": null
}
```

<Note>
  当前版本中 `~/.qsshrc` 已预留但部分设置尚未启用，将在后续版本中逐步激活。
</Note>

## 配置工作原理

### 配置模块架构

配置系统采用类型/解析/渲染分离的设计：

```
SSH 配置文件 (文本)
       ↓
parser.rs → 文本解析 → types.rs (SshConfig / HostBlock / SshDirective)
       ↓
writer.rs → 数据结构 → 文本渲染 → 写回文件
```

### 渐进式解析

解析器只识别 `Host` / `HostName` / `User` / `Port` / `IdentityFile` 五个字段：

- 已知字段 → 解析为对应的 `SshDirective` 枚举变体
- 未知字段 → 解析为 `SshDirective::Unknown(key, value)` 保留原始内容
- 全局配置（非 Host 块内容）→ 保留在 `preamble` 字段中
- 每个 Host 块的原始文本 → 保留在 `raw_text` 字段中，用于忠实重建

### 渲染策略

渲染器优先使用 `raw_text` 重建 Host 块（保证注释和空白格式不变），仅在 `raw_text` 为空时使用结构化指令渲染。
