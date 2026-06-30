# 架构文档

## 项目结构

```
quick-ssh/
├── Cargo.toml               # 工作空间根
├── crates/
│   ├── qssh/                # 主二进制
│   │   ├── src/
│   │   │   ├── main.rs      # 入口：解析 clap 参数 → 分发
│   │   │   ├── cli.rs       # clap derive CLI 定义
│   │   │   ├── config/      # SSH 配置 + 程序设置
│   │   │   │   ├── ssh_config.rs  # 渐进式 SSH 解析器
│   │   │   │   └── settings.rs    # ~/.qsshrc 加载/保存
│   │   │   ├── ssh/         # SSH 会话 + SFTP 上传
│   │   │   │   ├── session.rs     # spawn ssh, 目标解析
│   │   │   │   └── upload.rs      # SFTP 上传逻辑
│   │   │   ├── network/     # TCP 在线检测
│   │   │   │   └── ping.rs        # TcpStream 连接测试
│   │   │   ├── tui/         # 终端 UI（事件驱动）
│   │   │   │   ├── app.rs         # 应用状态
│   │   │   │   ├── event.rs       # Event/Action 定义
│   │   │   │   ├── modes.rs       # 模式定义
│   │   │   │   ├── ui.rs          # 渲染逻辑
│   │   │   │   └── widgets.rs     # 自定义组件
│   │   │   └── cmd/         # CLI 子命令实现
│   │   │       ├── ps.rs, add.rs, rm.rs, connect.rs
│   │   │       ├── export.rs, import.rs, help.rs
│   │   └── build.rs         # Shell 补全生成
│   └── qssh-uploader/       # 独立上传二进制
│       └── src/main.rs
├── docs/                    # 文档
├── packaging/               # 包管理器配置
└── .github/workflows/       # CI/CD
```

## 数据流

### CLI 模式
```
用户输入 → clap parse → Command dispatch → cmd/ 实现 → config/ 读写 → 输出
```

### TUI 模式
```
键盘事件 → event::map_key_to_action → Action → App::apply → State 更新 → ui::render
           ↑                                     ↓
           └────────── 事件循环 (100ms tick) ─────┘
```

### SSH 连接
```
目标输入 → resolve_target() → SshTarget → build_ssh_args() → Command::new("ssh") → 交互
```

### 文件上传
```
拖拽检测 → UploadPayload → spawn qssh-uploader → SFTP 连接 → 进度条 → 完成
```

## 设计决策

1. **渐进式 SSH 解析**：只解析 Host/HostName/User/Port/IdentityFile
   - 其余指令保留为 Unknown(key, value)
   - Match/Include 等复杂指令保留在 preamble
   - 忠实重建，不丢失信息

2. **事件驱动 TUI**：Event → Action → State → Render
   - 不直接通过按键回调修改 UI
   - Action 是纯数据，易于测试
   - State 是唯一数据源

3. **spawn ssh**：不实现 SSH 协议
   - 复用系统 OpenSSH 客户端
   - 支持所有 SSH 特性（跳板机、多因子认证等）
   - 减少依赖和攻击面
