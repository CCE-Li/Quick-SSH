# API 参考

本文档列出 Quick-SSH 各模块的主要公开接口。

## config 模块

位于 [`qssh/src/config/`](/qssh/src/config/)。

### types

```rust
// 指令枚举
pub enum SshDirective {
    HostName(String),
    User(String),
    Port(u16),
    IdentityFile(PathBuf),
    Unknown(String, String),
}

// Host 块
pub struct HostBlock {
    pub alias: String,
    pub directives: Vec<SshDirective>,
    pub raw_text: String,
}

// SSH 配置
pub struct SshConfig {
    pub hosts: Vec<HostBlock>,
    pub preamble: String,
}

// 查询函数
pub fn find_host(config: &SshConfig, alias: &str) -> Option<&HostBlock>;
pub fn find_host_mut(config: &mut SshConfig, alias: &str) -> Option<&mut HostBlock>;
pub fn list_aliases(config: &SshConfig) -> Vec<&str>;
pub fn default_config_path() -> PathBuf;
pub fn ensure_config(path: &PathBuf) -> Result<()>;
```

### parser

```rust
pub fn parse_config_content(content: &str) -> SshConfig;
pub fn parse_config(path: &PathBuf) -> Result<SshConfig>;
```

### writer

```rust
pub fn render_config(config: &SshConfig) -> String;
```

### settings

```rust
pub struct QsshSettings {
    pub default_port: u16,
    pub ping_timeout_secs: u64,
    pub upload_concurrency: usize,
    pub ssh_config_path: Option<PathBuf>,
}

pub fn load_settings() -> Result<QsshSettings>;
pub fn save_settings(settings: &QsshSettings) -> Result<()>;
```

## ssh 模块

位于 [`qssh/src/ssh/`](/qssh/src/ssh/)。

### session

```rust
pub struct SshTarget {
    pub alias: String,
    pub hostname: String,
    pub user: Option<String>,
    pub port: u16,
    pub identity_file: Option<PathBuf>,
}

impl SshTarget {
    pub fn from_host(host: &HostBlock) -> Self;
    pub fn from_user_at_host(input: &str, port: u16) -> Self;
    pub fn build_ssh_args(&self) -> Vec<String>;
}

pub fn resolve_target(config: &SshConfig, input: &str) -> SshTarget;
```

### spawn

```rust
pub fn start_interactive_session(target: &SshTarget, extra_args: &[String]) -> Result<i32>;
```

### drag_detect

```rust
pub fn strip_paste_markers(text: &str) -> String;
pub fn detect_drag_files(text: &str) -> Option<Vec<PathBuf>>;
```

### upload

```rust
pub fn upload_files(target: &SshTarget, files: &[PathBuf], remote_dir: &str) -> Result<()>;
```

## network 模块

位于 [`qssh/src/network/`](/qssh/src/network/)。

### ping

```rust
pub fn check_host(hostname: &str, port: u16, timeout_secs: u64) -> Result<bool>;
```

## tui 模块

位于 [`qssh/src/tui/`](/qssh/src/tui/)。

### action

```rust
pub enum Action { /* 20+ 变体 */ }
pub enum Mode { Normal, Search, Add, Edit, Confirm, Help, /* ... */ }
```

### app

```rust
pub struct App { /* 应用状态 */ }
impl App {
    pub fn new(config: SshConfig, config_path: PathBuf) -> Self;
    pub fn apply(&mut self, action: Action);
    pub fn selected(&self) -> Option<usize>;
    pub fn poll_background_tasks(&mut self);
    pub fn expire_flash_message(&mut self);
}
```

### event

```rust
pub fn start() -> Result<()>;
```

### keymap

```rust
pub fn map_key_to_action(key: KeyEvent, app: &App) -> Action;
impl Mode {
    pub fn label(&self) -> &str;
    pub fn hint(&self) -> &str;
}
```

### editor

```rust
pub enum EditorOutcome { Continue, Save, Cancel }
pub enum HostFormMode { Add, Edit { index: usize, original_alias: String } }

pub struct HostFormState { /* ... */ }
impl HostFormState {
    pub fn new_add() -> Self;
    pub fn new_edit(index: usize, host: &HostBlock) -> Self;
    pub fn handle_key(&mut self, key: KeyEvent) -> EditorOutcome;
    pub fn build_host(&self) -> anyhow::Result<HostBlock>;
}
```

## cmd 模块

位于 [`qssh/src/cmd/`](/qssh/src/cmd/)。

```rust
pub mod ps;
    pub fn run(keyword: Option<String>) -> Result<()>;

pub mod add;
    pub fn run(alias: &str, user_at_host: &str, key: Option<&str>, port: u16) -> Result<()>;

pub mod rm;
    pub fn run(alias: &str) -> Result<()>;

pub mod connect;
    pub fn run(target: &str, ssh_args: &[String]) -> Result<()>;

pub mod export;
    pub fn run(file: Option<&str>) -> Result<()>;

pub mod import;
    pub fn run(file: &str) -> Result<()>;

pub mod help;
    pub fn run() -> Result<()>;

pub mod completions;
    pub fn run(shell: &str) -> Result<()>;
```
