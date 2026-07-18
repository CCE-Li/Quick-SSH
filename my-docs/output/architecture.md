# 架构设计

## 项目结构

```
quick-ssh/
├── Cargo.toml                   # 工作空间根
├── qssh/                        # 主二进制
│   ├── Cargo.toml
│   ├── build.rs                 # 构建脚本（监视 cli.rs 变更）
│   └── src/
│       ├── main.rs              # 入口：clap 参数解析 → 分发
│       ├── cli.rs               # clap derive CLI 定义
│       ├── config/              # SSH 配置 + 程序设置
│       │   ├── mod.rs           # 模块重导出
│       │   ├── types.rs         # SshDirective, HostBlock, SshConfig 类型定义
│       │   ├── parser.rs        # 渐进式 SSH 解析器
│       │   ├── writer.rs        # SSH 配置渲染器
│       │   └── settings.rs      # ~/.qsshrc 加载/保存
│       ├── ssh/                 # SSH 会话 + SFTP 上传
│       │   ├── mod.rs
│       │   ├── session.rs       # SshTarget 解析与构建
│       │   ├── spawn.rs         # spawn ssh 进程（跨平台 raw 模式）
│       │   ├── upload.rs        # 内联 SCP 上传（预留）
│       │   └── drag_detect.rs   # 拖拽文件路径检测
│       ├── network/             # TCP 在线检测
│       │   ├── mod.rs
│       │   └── ping.rs          # TcpStream 连接测试
│       ├── tui/                 # 终端 UI（事件驱动）
│       │   ├── mod.rs
│       │   ├── action.rs        # Action + Mode 枚举
│       │   ├── app.rs           # 应用状态与业务逻辑
│       │   ├── event.rs         # 事件循环
│       │   ├── keymap.rs        # 键盘映射 + Mode 标签/提示
│       │   ├── ui.rs            # 渲染逻辑
│       │   ├── widgets.rs       # 自定义组件（弹窗等）
│       │   └── editor.rs        # 主机编辑表单
│       └── cmd/                 # CLI 子命令实现
│           ├── mod.rs
│           ├── ps.rs            # 列出主机
│           ├── add.rs           # 添加主机
│           ├── rm.rs            # 删除主机
│           ├── connect.rs       # 连接主机
│           ├── export.rs        # 导出 JSON
│           ├── import.rs        # 导入 JSON
│           ├── help.rs          # 自定义帮助
│           └── completions.rs   # Shell 补全生成
├── qssh-uploader/               # 独立上传二进制
│   └── src/main.rs              # 并发 SCP 上传 + 进度显示
├── docs/                        # 文档
├── packaging/                   # 包管理器配置
└── .github/workflows/           # CI/CD
    ├── ci.yml                   # 每次推送自动检查
    └── release.yml              # 打标签触发发布
```

## 工作空间架构

项目使用 Cargo workspace 管理两个二进制 crate：

<Properties>
  <Property name="qssh" type="主程序">
    核心 SSH 管理工具，包含 CLI、TUI、配置管理、SSH 连接功能
  </Property>
  <Property name="qssh-uploader" type="上传工具">
    独立的文件上传程序，专注于 SCP 并发上传和进度显示
  </Property>
</Properties>

Workspace 共享的依赖包括：`tokio`、`serde`、`clap`、`ratatui`、`crossterm`、`ssh2` 等。

## 数据流

### CLI 模式

```
用户输入 → clap parse → Command dispatch → cmd/ 实现 → config/ 读写 → 终端输出
```

1. [`main.rs`](/qssh/src/main.rs) 解析命令行参数
2. 匹配到对应 [`Command`](/qssh/src/cli.rs:32) 枚举变体
3. 调用 `cmd/` 下对应的实现模块
4. 各模块通过 `config/` 模块读写 SSH 配置文件
5. 结果输出到终端

### TUI 模式

```
键盘事件 → keymap::map_key_to_action() → Action → App::apply() → State 更新 → ui::render()
           ↑                                                              ↓
           └────────────────── 事件循环 (100ms tick) ──────────────────────┘
```

1. 事件循环以 100ms 为周期轮询键盘输入
2. 按键通过 [`keymap.rs`](/qssh/src/tui/keymap.rs) 映射为 [`Action`](/qssh/src/tui/action.rs) 枚举
3. Action 通过 [`App::apply()`](/qssh/src/tui/app.rs:168) 更新应用状态
4. 状态变更后触发 [`ui::render()`](/qssh/src/tui/ui.rs) 重新绘制界面
5. 后台任务（如 Ping 检测）通过 channel 异步通信

### SSH 连接

```
目标输入 → session::resolve_target() → SshTarget → build_ssh_args() → spawn::start_interactive_session() → SSH 交互
```

1. 用户输入目标（别名或 `user@host`）
2. [`resolve_target()`](/qssh/src/ssh/session.rs:78) 优先作为别名查找，否则视为直接连接
3. 构建 SSH 命令行参数（包括 `-tt` 强制 PTY）
4. 启动 SSH 子进程（stdin 继承，stdout/stderr 转发）
5. SSH 退出后返回退出码

### 文件上传

```
拖拽检测 → UploadPayload → spawn qssh-uploader → SFTP 连接 → 进度条 → 完成
```

1. SSH 会话期间，[`drag_detect.rs`](/qssh/src/ssh/drag_detect.rs) 检测终端输入中的文件路径
2. 检测到拖拽操作 → 在新控制台窗口中启动 `qssh-uploader`
3. 上传器使用 `scp` 实现文件传输，最多 3 文件并发
4. 实时渲染每个文件的进度条

## 设计决策

### 1. 渐进式 SSH 解析

只解析 `Host` / `HostName` / `User` / `Port` / `IdentityFile` 五个字段，其余指令保留为 `Unknown(key, value)`。`Match`、`Include` 等复杂指令保留在 `preamble` 中。这种设计保证了信息的完整性——无论配置文件中有什么内容，读写一遍后不会丢失任何信息。

### 2. 事件驱动 TUI

采用 `Event → Action → State → Render` 的单向数据流架构：

- **Action 是纯数据**：易于测试和序列化
- **State 是唯一数据源**：不会有多个地方维护状态副本
- **不直接通过按键回调修改 UI**：所有状态变更都通过 Action 分发

### 3. Spawn SSH（复用系统 OpenSSH）

不自行实现 SSH 协议，而是复用系统自带的 OpenSSH 客户端：

- 支持所有 SSH 特性（跳板机、多因子认证、SSH 代理等）
- 减少依赖和攻击面
- 无需处理 SSH 协议的复杂性
- stdin 继承使得密码认证正常工作

### 4. Config 模块类型/解析/渲染分离

- [`types.rs`](/qssh/src/config/types.rs)：只定义数据结构和查询方法
- [`parser.rs`](/qssh/src/config/parser.rs)：只负责文本→数据结构
- [`writer.rs`](/qssh/src/config/writer.rs)：只负责数据结构→文本渲染
- [`settings.rs`](/qssh/src/config/settings.rs)：程序自身配置（独立于 SSH 配置）

### 5. 跨平台终端 Raw 模式

Windows 平台使用 WinAPI 直接操作控制台模式（`GetConsoleMode` / `SetConsoleMode`），Unix 平台使用 crossterm 的 termios 封装。Windows 实现确保了：

- 禁用 ECHO、LINE_INPUT、PROCESSED_INPUT
- 启用 ENABLE_VIRTUAL_TERMINAL_INPUT（使 `std::io::stdin().read()` 能正确读取按键字节序列）
- 退出时三种方法确保恢复光标可见性
