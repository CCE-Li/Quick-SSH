# Quick-SSH 3.0 架构迁移方案（Phase 0 分析报告）

> 版本：3.0 规划草案
> 分支：`feat/ai-assistant`
> 状态：**待确认** —— 确认后进入 Phase 1

---

## 一、Current Architecture（现有架构）

### 1.1 Workspace 结构

```
quick-ssh/                        # Cargo workspace (resolver = "2")
├── qssh/                         # 主二进制 crate（唯一业务代码）
│   └── src/
│       ├── main.rs               # 入口：AskPass 预处理 + clap 分发
│       ├── cli.rs                # clap derive 子命令定义
│       ├── config/               # SSH 配置系统
│       │   ├── types.rs          # SshDirective / HostBlock / SshConfig
│       │   ├── parser.rs         # 渐进式 SSH 配置解析器
│       │   ├── writer.rs         # 配置渲染回写
│       │   ├── credentials.rs    # keyring 密码库 + OpenSSH AskPass 协议
│       │   └── settings.rs       # ~/.qsshrc（目前 #[allow(dead_code)] 预留）
│       ├── ssh/
│       │   ├── session.rs        # SshTarget 解析与参数构建
│       │   ├── spawn.rs          # spawn 系统 OpenSSH + 跨平台 raw 模式
│       │   ├── upload.rs         # 内联 SCP 上传（#[allow(dead_code)] 预留）
│       │   └── drag_detect.rs    # 终端拖拽路径检测
│       ├── network/
│       │   └── ping.rs           # TCP 连通性检测
│       ├── tui/                  # 事件驱动 TUI
│       │   ├── action.rs         # Action / Mode 枚举
│       │   ├── app.rs            # App 状态 + 业务逻辑 + 后台任务
│       │   ├── event.rs          # 事件循环 (100ms tick)
│       │   ├── keymap.rs         # 按键 → Action 映射
│       │   ├── ui.rs             # 主界面渲染
│       │   ├── widgets.rs        # 弹窗组件
│       │   └── editor.rs         # 主机添加/编辑表单
│       └── cmd/                  # CLI 子命令实现
│           ├── ps / add / rm / connect / export / import / help / completions
├── qssh-uploader/                # 独立上传二进制（scp 并发上传 + 进度条）
├── docs/                         # VitePress 文档站
├── packaging/                    # Scoop/WinGet/Homebrew/AUR/APT
└── .github/workflows/            # ci / release / deploy / aur-publish
```

### 1.2 关键设计决策（现有）

| 决策 | 说明 | 文件 |
|------|------|------|
| 渐进式 SSH 解析 | 只解析 Host/HostName/User/Port/IdentityFile，其余存 `Unknown(key, value)` 或原样 `raw_text`，保证读写不丢信息 | [`parser.rs`](../qssh/src/config/parser.rs:15) |
| 事件驱动 TUI | `Event → Action → State → Render` 单向数据流；Action 是纯数据 | [`app.rs`](../qssh/src/tui/app.rs:175) |
| Spawn 系统 OpenSSH | 不实现 SSH 协议，复用系统 `ssh`，支持跳板/多因子/代理 | [`spawn.rs`](../qssh/src/ssh/spawn.rs:34) |
| 密码独立存储 | keyring 存系统凭据库，AskPass 协议自动填写 | [`credentials.rs`](../qssh/src/config/credentials.rs) |
| 后台任务模式 | `mpsc` channel + `thread::spawn`，事件循环 `poll_background_tasks()` 轮询 | [`app.rs`](../qssh/src/tui/app.rs:156) |
| 跨平台终端 | Unix 继承 stdio；Windows 用 WinAPI 控制台模式 + 转发线程 | [`spawn.rs`](../qssh/src/ssh/spawn.rs:46) |

### 1.3 数据流现状

```
CLI:  user → clap parse → cmd/* → config/* → 终端输出
TUI:  键盘 → keymap → Action → App::apply() → 状态 → ui::render()
      后台任务(ping) → mpsc → poll_background_tasks() → 状态更新
SSH:  目标 → resolve_target() → SshTarget → AskPass配置 → spawn ssh → 交互
```

### 1.4 依赖现状

- 已引入 `tokio(full)`，但**当前代码全部使用 `std::thread` + `mpsc`，未使用 async**
- 已引入 `ssh2`（libssh2），但**当前会话复用系统 OpenSSH，`ssh2` 实际未被使用**
- 已引入 `tui-textarea`、`regex`、`shellexpand`、`colored`、`dirs`、`strum` 等

---

## 二、Problems（现有架构问题）

### P1. 单 crate 巨石结构
`qssh` 一个二进制 crate 承载了 CLI、TUI、SSH、配置、网络全部逻辑。按 3.0 目标扩展到 Docker/Process/Files/Logs/Agent 后，单一 crate 会迅速膨胀，模块边界无法通过 crate 边界强制约束。

**影响**：后续 AI 模块（qssh-agent）与业务模块（qssh-core）无法独立编译、独立测试、独立演进。

### P2. `ssh2` 依赖存在但未使用
`qssh/Cargo.toml` 声明了 `ssh2`（libssh2），但 `session.rs`/`spawn.rs` 全部走系统 OpenSSH 子进程。而 3.0 的 ServerMonitor（CPU/Memory/Disk/Process）需要**非交互式批量远程命令执行**，必须引入新的远程执行通道。

**关键决策点**：
- 选项 A：继续用系统 OpenSSH（`ssh <host> <cmd>` 非交互模式），零新增依赖，但每次连接开销大、无法复用连接
- 选项 B：启用已声明的 `ssh2` crate 做持久连接 + 通道命令执行，适合监控高频采集
- 选项 C：两者并存 —— 交互式会话走系统 OpenSSH（保留现有最佳体验），监控采集走 `ssh2` 持久连接

**推荐 C**（兼容现有 + 满足监控需求）。需澄清：当前 `ssh2` 是遗留未用，还是为监控预留？

### P3. `App` 巨型状态结构
`App` 已经 750 行，字段涵盖列表状态、后台任务、弹窗表单、密码缓存、闪烁消息。3.0 加入多模块后，如果继续把每个模块状态塞进 `App`，将退化为"巨型 struct"，违背第 35 条代码质量要求。

### P4. UI 渲染写死布局
[`ui.rs`](../qssh/src/tui/ui.rs:54) 目前固定左列表右详情。3.0 要求用户可自定义 Dashboard 布局（模块选择 + 尺寸 + Profile），当前结构无法承载。

### P5. Mode 枚举 + keymap match 的硬编码
[`action.rs`](../qssh/src/tui/action.rs:50) 的 `Mode` 和 [`keymap.rs`](../qssh/src/tui/keymap.rs:7) 的单函数 match，新增一个模块就要改枚举 + 改 match + 改 hint，扩展性差。

### P6. 后台任务仅支持"一次性返回"
Ping 事件模式是"发起 → 单个结果事件"。[`app.rs`](../qssh/src/tui/app.rs:156) 的 `poll_background_tasks` 只处理 `PingEvent`。3.0 需要**周期性采集**（CPU 1s、Docker 3s）、**流式日志**（journalctl -f）、**Agent 多步执行**（timeline 逐步回报），当前模式需要泛化。

### P7. 远程操作无统一执行抽象
`spawn.rs` 只做交互式会话。3.0 的 Docker/Process/Services/Files/Logs 都需要"远程执行命令并拿回结构化结果"，当前没有 `RemoteExecutor` 抽象。

### P8. 连接失败无诊断能力
[`connect.rs`](../qssh/src/cmd/connect.rs:27) 只打印退出码。3.0 要求错误分类处理（Timeout/Denied/Not Found/Unsupported OS）与 AI 诊断。当前无法捕获 stderr 做结构化分析（Windows 下 stderr 被转发线程吞掉）。

### P9. 配置体系单一
目前只有 `~/.ssh/config`（数据）+ `~/.qsshrc`（预留设置）。3.0 需要 `dashboard.toml`、`keybindings.toml`、`agent.toml`、多 Profile、服务器级覆盖，需建立独立的程序配置目录。

### P10. 无会话历史/连接日志
第 1 节目标树包含 Connection History，当前无任何会话记录。

### P11. `settings.rs` 是死代码
`QsshSettings` 带 `#[allow(dead_code)]`，说明程序配置层从未启用。3.0 的 AI 配置、Dashboard 配置都必须先激活此层。

---

## 三、Recommended Architecture（推荐架构）

### 3.1 目标 Workspace 结构

```
quick-ssh/
├── crates/
│   ├── qssh-config/              # 配置层（新）
│   │   ├── server.rs             # ~/.ssh/config 模型 + 解析/渲染（从 config/ 迁移）
│   │   ├── dashboard.rs          # dashboard.toml 模型
│   │   ├── profile.rs            # Profile 管理
│   │   ├── keybindings.rs        # keybindings.toml
│   │   ├── agent.rs              # agent.toml（provider/base_url/model/permission）
│   │   └── settings.rs           # 通用设置（qsshrc）
│   │
│   ├── qssh-core/                # 业务核心（新）
│   │   ├── ssh/                  # SshTarget + 交互会话（从 ssh/ 迁移）
│   │   ├── exec.rs               # RemoteExecutor 抽象（非交互命令执行）
│   │   ├── monitor.rs            # ServerMonitor 统一采集 → ServerSnapshot
│   │   ├── docker.rs / process.rs / services.rs / network.rs
│   │   ├── files.rs              # 浏览/上传/下载
│   │   └── logs.rs               # 系统/服务/Docker/实时日志
│   │
│   ├── qssh-tui/                 # 界面层（新）
│   │   ├── app/                  # App + 状态机
│   │   ├── event/                # 事件循环 + 后台任务调度器
│   │   ├── input/                # 按键映射（可配置）
│   │   ├── layout/               # Layout Engine（Widget 布局计算）
│   │   ├── widgets/              # cpu/memory/disk/network/docker/process/
│   │   │                         #   services/files/logs/agent/timeline
│   │   ├── screens/              # 主机列表 / 服务器首页 / 设置
│   │   └── components/           # 弹窗 / 确认框 / Command Palette
│   │
│   ├── qssh-agent/               # AI Agent（新，Phase 8）
│   │   ├── agent.rs              # Agent 主循环
│   │   ├── tools/                # Tool Registry + 各 Tool
│   │   ├── context.rs            # Agent Context（服务器快照注入）
│   │   ├── permissions.rs        # 权限策略
│   │   ├── timeline.rs           # 执行时间线
│   │   └── provider.rs           # OpenAI 兼容 HTTP / 外部 CLI 适配
│   │
│   └── qssh/                     # 主二进制（瘦身）：clap 入口 + 组装 + CLI 命令
│
├── qssh-uploader/                # 保留（或并入 qssh-core/files 调用库）
└── ...
```

> ⚠️ 这**不是**一次性重构。Phase 1 不建新 crate 树，而是在现有 `qssh` crate 内部按模块目录演进，**当模块稳定后逐个抽成独立 crate**（见 §四 迁移计划）。

### 3.2 分阶段目标架构要点

#### 阶段性"内部模块化"（Phase 1-2，不拆 crate）
在 `qssh/src` 下先建立清晰的模块边界：

```
qssh/src/
├── core/          # 业务逻辑（从 app.rs / ssh/ 中抽离，无 UI 依赖）
│   ├── executor.rs
│   ├── monitor.rs
│   └── ...
├── ui/            # 渲染（当前 tui/ui.rs 演进）
├── state/         # 状态管理（从 app.rs 拆分）
├── input/         # 按键映射
└── app.rs         # 只做组装与事件分发（瘦身）
```

#### Widget / Panel / Module 抽象（Phase 1）
```rust
pub trait WidgetModule {
    fn id(&self) -> &str;
    fn title(&self) -> &str;
    fn render(&self, frame: &mut Frame, area: Rect);
    fn handle_event(&mut self, event: Event);
    fn update(&mut self, data: &ServerSnapshot);
}
```
- `Layout Engine`：读 `dashboard.toml` → 生成 `Rect` 集合 → 渲染活跃 Widget
- `ServerSnapshot`：CPU/Memory/Disk/Network/Uptime/Load 的统一数据载体
- `ServerMonitor`：后台调度器，按模块各自频率采集，写入快照

#### 后台任务调度器（Phase 2）
把现有"单 mpsc"泛化为：

```rust
enum BackgroundEvent {
    Ping(PingEvent),
    Monitor(ServerSnapshot),
    Agent(AgentEvent),
    Log(LogChunk),
    // ...
}
```
统一一个 channel 分发，事件循环 `poll_background_tasks()` 只做转发。

#### 远程执行抽象（Phase 2）
```rust
pub trait RemoteExecutor: Send + Sync {
    /// 执行命令并返回 (exit_code, stdout, stderr)
    fn exec(&self, cmd: &str, timeout: Duration) -> Result<ExecResult>;
}
```
- 实现 1：`SshProcessExecutor`（复用系统 ssh 非交互模式）
- 实现 2：`Ssh2Executor`（持久连接，监控采集用）
- 通过 `ServerSession` 管理连接复用

#### AI Agent（Phase 8）
```
User → Agent → Tool Registry → Quick-SSH Core → RemoteExecutor → Server
                 ├── server.status / processes / disk ...
                 ├── docker.list / stats / logs ...
                 ├── service.* / files.* / logs.* / network.*
                 └── 只允许调用 Tool，不直接持 SSH
```

---

## 四、Migration Plan（分阶段迁移计划）

> 原则：**每阶段保持可编译、可测试、CI 通过、功能不回退**。

### Phase 1：Dashboard Framework（新 UI 骨架，Mock 数据）

**范围**：Widget trait + Layout Engine + Dashboard 配置 + 持久化 + 键盘布局编辑
**目标**：把固定布局替换为配置驱动，其余功能不变

**具体步骤**：
1. `qssh/src` 下新增 `ui/`、`layout/`、`widgets/` 目录（内部模块化，不拆 crate）
2. 定义 `WidgetModule` trait + 一组 Mock Widget（CPU/Memory/...占位，显示模拟数据）
3. 定义 `DashboardConfig`（模块列表 + 布局）并持久化到 `~/.config/quick-ssh/dashboard.toml`
4. 实现 `Layout Engine`（解析配置 → 计算 Rect）
5. 保留现有主机列表/详情视图作为 `HostListScreen`，与 Dashboard 并存（可切换）
6. 实现 `Dashboard Configuration` 面板（勾选模块 + 调尺寸）+ `Ctrl+K` Command Palette 雏形
7. 保持 `Event → Action → State → Render` 架构不变，Action 新增 `OpenDashboard` / `ConfigureDashboard` 等

**验收**：`cargo check/test/clippy/fmt` 全绿；TUI 启动后能按配置渲染 Mock Dashboard；现有主机列表功能无回归。

### Phase 2：Server Monitor（真实数据 + 采集调度）

**范围**：RemoteExecutor 抽象 + ServerMonitor + CPU/Memory/Disk/Network/Uptime/Process 真实数据
**目标**：UI 永不因 SSH 阻塞；连接复用；不同频率刷新

**具体步骤**：
1. 新增 `core/executor.rs`：`RemoteExecutor` trait + `SshProcessExecutor`（首版复用系统 ssh）
2. 新增 `core/monitor.rs`：后台调度器，各指标独立频率采集 → `ServerSnapshot`
3. 实现 platform 命令层 `platform/linux.rs`（/proc、/sys、df、free、uptime、ps、ss、ip），未来扩展 windows/macos
4. Widget 改为消费 `ServerSnapshot`；`BackgroundEvent` 统一 channel
5. 错误处理：SSH Timeout / Disconnect / Permission / Not Found / Unsupported OS 分类显示
6. 连接管理：`ServerSession` 复用连接；断线自动重连策略

**验收**：连上服务器后 Dashboard 显示真实监控数据；刷新期间 UI 无卡顿；断网/权限错误有友好提示。

### Phase 3：Docker（Phase 4：Services / Phase 5：Network / Phase 6：Files / Phase 7：Logs）

按 §二-目标树逐模块实现，每个模块：
1. `core/<module>.rs`：远程命令封装（结构化输出解析）
2. `ui/widgets/<module>/`：列表 + 详情 + 操作
3. 危险操作（restart/stop/delete/kill）全部经过确认弹窗
4. 日志限流（max lines / max bytes）

### Phase 8：AI Agent

**前置条件**：Phase 1（Dashboard 稳定）与 Phase 2（RemoteExecutor）就绪后再开始。

**范围**：
1. `qssh-agent` crate（或 `qssh/src/agent/` 内部模块）：
   - `provider.rs`：OpenAI 兼容 HTTP（reqwest）/ 外部 CLI（ollama/claude）双后端
   - `tools/`：Tool Registry，每个 Tool 映射到 `qssh-core` 的 RemoteExecutor 调用
   - `context.rs`：注入当前服务器 `ServerSnapshot`
   - `permissions.rs`：READ_ONLY / ASK_BEFORE_EXECUTE（默认）/ AUTO_SAFE / FULL_ACCESS
   - `timeline.rs`：Agent 执行步骤逐步回报
2. `~/.config/quick-ssh/agent.toml`：provider/base_url/model/permission 配置
3. Agent 安全：命令预览（restart/stop/delete/kill/rm/systemctl/docker 前置确认）+ 权限提示
4. TUI：Agent 输入框 + Timeline 面板 + 状态栏 AI 状态（●Ready ◇Thinking ●Executing 3/7 !Approval ▲Error）

**MVP 先行（可提前验证）**：在 Phase 8 正式开发前，可先实现两个轻量 AI 能力验证链路：
- `qssh ai add "生产环境 nginx 服务器 root@192.168.1.10"` → 自然语言生成 HostBlock
- 连接失败时用 AI 分析 stderr 给出修复建议
（两者只依赖 provider + parser，不依赖 Dashboard/监控，可并行验证）

---

## 五、关键架构决策汇总（待确认）

| # | 决策 | 选项 | 建议 |
|---|------|------|------|
| D1 | 远程执行通道 | A.系统ssh / B.ssh2 / C.双通道 | **C**：交互走系统ssh（保留现状），监控走ssh2 |
| D2 | crate 拆分节奏 | A.先建多 crate / B.先内部模块后拆分 | **B**：内部模块化先行，稳定后逐个抽 crate |
| D3 | AI 接入方案 | A.OpenAI兼容HTTP / B.外部CLI / C.Coze | **A 或 B**（见 §六） |
| D4 | 配置目录 | 沿用 `~/.qsshrc` / 新建 `~/.config/quick-ssh/` | **新建目录**：servers 保留在 ~/.ssh/config，程序配置独立 |
| D5 | 数据采集实现 | 命令解析（df/free/ps...）/ libssh2 通道 | **命令解析为主**，符合轻量定位 |
| D6 | 交互会话与监控并存 | — | 保留 `spawn.rs` 交互会话，新增独立采集通道，互不干扰 |

---

## 六、AI 接入方案对比（补充）

| 维度 | A. OpenAI 兼容 HTTP | B. 外部 CLI 集成 | C. Coze/Agent 平台 |
|------|--------------------|------------------|-------------------|
| 依赖 | reqwest（新增 1 依赖） | 无新依赖 | SDK 或 HTTP |
| API Key | 需要，存 keyring | 不需要 | 平台 Token |
| 支持模型 | DeepSeek/Qwen/Ollama/OpenAI 统一协议 | 取决于用户已装 CLI | 平台限定 |
| 离线可用 | 否 | 可（ollama 本地） | 否 |
| 运维复杂度 | 低 | 中（需检测 CLI 存在） | 高（平台工作流） |
| 适合 | 默认推荐 | 本地优先用户 | 复杂编排（暂不建议） |

---

## 七、Next Steps（下一步）

本报告确认后，按以下顺序推进：

1. **[Phase 1]** 确认 D1-D6 决策 → 创建 `ui/layout/widgets` 内部模块 → Mock Dashboard + Layout Engine + dashboard.toml
2. **[Phase 1]** Command Palette 雏形 + Dashboard 配置面板
3. **每阶段完成**：`cargo check --workspace && cargo test --workspace && cargo clippy --workspace -- -D warnings && cargo fmt --all -- --check`
4. **每阶段可运行**：`cargo build` 必须通过，禁止积累编译错误

> 当前分支：`feat/ai-assistant`。所有 3.0 开发在此分支进行，稳定后合并回 `main`。
