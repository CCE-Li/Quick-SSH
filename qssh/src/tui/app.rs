use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

use crossterm::event::KeyEvent;
use ratatui::layout::Position;
use ratatui::widgets::ListState;

use crate::agent::config::{load_agent_config, AgentConfig};
use crate::agent::provider::ChatMessage;
use crate::agent::timeline::{AgentStatus, Timeline};
use crate::agent::tools::ToolCall;
use crate::agent::AgentEvent;
use crate::config::credentials;
use crate::config::types::{HostBlock, SshConfig};
use crate::monitor::docker::{container_action_command, DockerAction};
use crate::monitor::executor::{ExecError, ExecOutput, RemoteExecutor};
use crate::monitor::files::{
    dir_entries, join_dir, ls_l_command, parse_ls_l, shell_quote, FileAction,
};
use crate::monitor::logs::{journalctl_command, parse_journalctl, LogEntry, MAX_LOG_LINES};
use crate::monitor::scheduler::{BackgroundEvent, MonitorScheduler};
use crate::monitor::services::{service_action_command, ServiceAction};
use crate::monitor::snapshot::ServerSnapshot;
use crate::monitor::{Collector, SshProcessExecutor};
use crate::ssh::session::SshTarget;
use crate::tui::action::{Action, Mode, View};
use crate::tui::dashboard::palette::{filter_palette, PaletteAction, PALETTE_ACTIONS};
use crate::tui::dashboard::widgets::WidgetModule;
use crate::tui::dashboard::{
    load_dashboard_config, save_dashboard_config, DashboardConfig, LayoutNode,
};
use crate::tui::editor::{
    AgentFormState, EditorOutcome, HostFormMode, HostFormState, HostFormSubmission,
    PasswordStorageAction,
};
use crate::tui::term::{TermEvent, TermSession};

#[derive(Debug)]
enum PingEvent {
    SingleFinished {
        alias: String,
        online: bool,
        error: Option<String>,
    },
    BatchHostFinished {
        alias: String,
        online: bool,
        error: Option<String>,
    },
    BatchCompleted {
        online_count: usize,
        total: usize,
    },
}

/// `:` 远程命令后台执行事件
#[derive(Debug)]
enum CmdEvent {
    Finished {
        alias: String,
        output: Result<ExecOutput, ExecError>,
    },
}

/// `:` 远程命令执行结果（供结果弹窗展示）
#[derive(Debug, Clone)]
pub struct CommandResult {
    pub alias: String,
    pub command: String,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub error: Option<String>,
}

pub struct FlashMessage {
    pub message: String,
    pub color: String,
    expires_at: Option<Instant>,
}

/// Dashboard 分隔线拖动状态（鼠标拖动调整相邻窗口大小）
pub struct SplitDrag {
    /// 从布局根到目标 Split 节点的子节点下标路径
    path: Vec<usize>,
    /// 分隔线左侧/上方子节点下标（拖动其右/下边界）
    pivot: usize,
    /// 拖动开始时各子节点的像素尺寸（Horizontal 为宽，Vertical 为高）
    fixed: Vec<u16>,
}

impl SplitDrag {
    /// 构造拖动状态（仅测试使用）
    #[cfg(test)]
    pub fn new(path: Vec<usize>, pivot: usize, fixed: Vec<u16>) -> Self {
        Self { path, pivot, fixed }
    }
}

/// Agent 对话线程条目（opencode 风格：消息 + 内联工具步骤）
#[derive(Debug, Clone)]
pub enum AgentChatEntry {
    /// 用户发送的消息
    User(String),
    /// 助手回复（自然语言 / 错误摘要）
    Assistant(String),
    /// 工具调用步骤（含最终状态）
    Tool(crate::agent::timeline::TimelineStep),
}

// ── 应用状态 ─────────────────────────────────────────────

/// TUI 应用状态
pub struct App {
    /// 主机列表
    pub hosts: Vec<HostBlock>,
    /// Host 之外的全局配置
    pub preamble: String,
    /// SSH 配置文件路径
    pub config_path: PathBuf,
    /// 列表状态（选中索引等）
    pub list_state: ListState,
    /// 滚动偏移（预留）
    #[allow(dead_code)]
    pub scroll_offset: usize,
    /// 当前模式
    pub mode: Mode,
    /// 当前视图（主机列表 / Dashboard）
    pub view: View,
    /// 输入缓存
    pub input_buffer: String,
    /// 搜索结果过滤
    pub search_keyword: String,
    /// 已选择（标记）的主机
    pub marked: Vec<usize>,
    /// 主机在线状态: alias → online
    pub host_status: HashMap<String, bool>,
    /// 正在后台检测中的主机
    pub pending_pings: HashSet<String>,
    /// 闪烁消息
    pub flash_message: Option<FlashMessage>,
    /// 新增 / 编辑表单弹窗状态
    pub host_form: Option<HostFormState>,
    /// Dashboard 配置（多 Profile）
    pub dashboard_config: DashboardConfig,
    /// Dashboard 当前渲染的 Widget 实例
    pub dashboard_widgets: Vec<Box<dyn WidgetModule>>,
    /// 命令面板输入
    pub palette_query: String,
    /// 命令面板选中项索引
    pub palette_selected: usize,
    /// 后台检测结果通道
    ping_rx: Receiver<PingEvent>,
    ping_tx: Sender<PingEvent>,
    /// 后台监控调度器（Dashboard 视图活跃时采集）
    monitor_scheduler: MonitorScheduler,
    /// 监控事件接收端
    monitor_rx: Receiver<BackgroundEvent>,
    /// 当前正在监控的主机别名（None = 未监控）
    monitor_target: Option<String>,
    /// 最近一次监控快照（Docker/服务操作面板使用）
    latest_snapshot: Option<ServerSnapshot>,
    /// Docker 容器列表选中索引
    docker_selected: usize,
    /// 待确认的 Docker 操作（None = 未处于确认流程）
    docker_action: Option<DockerAction>,
    /// 服务列表选中索引
    service_selected: usize,
    /// 待确认的服务操作（None = 未处于确认流程）
    service_action: Option<ServiceAction>,
    /// 文件列表选中索引
    file_selected: usize,
    /// 待确认的文件操作（None = 未处于确认流程）
    file_action: Option<FileAction>,
    /// 日志列表选中索引
    log_selected: usize,
    /// 日志 unit 筛选输入缓冲
    log_unit: String,
    /// 日志原始（未筛选）列表
    log_raw: Vec<LogEntry>,
    /// Agent 设置表单弹窗状态（None = 未打开）
    pub agent_form: Option<AgentFormState>,
    /// AI Agent 配置
    agent_config: AgentConfig,
    /// Agent 事件接收端
    agent_rx: Receiver<AgentEvent>,
    /// Agent 发送端（后台线程持有）
    agent_tx: Sender<AgentEvent>,
    /// 危险操作批准回传通道（后台会话阻塞等待 UI 决定）
    agent_approval_tx: Option<Sender<bool>>,
    /// Agent 输入缓冲
    agent_input: String,
    /// Agent 状态（状态栏 AI 指示）
    agent_status: AgentStatus,
    /// Agent 时间线（执行步骤）
    agent_timeline: Timeline,
    /// Agent 对话历史（注入 LLM 上下文）
    agent_history: Vec<ChatMessage>,
    /// Agent 对话线程（展示用，含用户/助手消息与工具步骤）
    agent_thread: Vec<AgentChatEntry>,
    /// 待批准的 Agent 工具调用
    agent_pending_call: Option<ToolCall>,
    /// `:` 远程命令输入缓冲
    command_input: String,
    /// `:` 远程命令执行结果（None = 无结果）
    command_result: Option<CommandResult>,
    /// 远程命令后台执行事件接收端
    cmd_rx: Receiver<CmdEvent>,
    /// 远程命令后台执行事件发送端
    cmd_tx: Sender<CmdEvent>,
    /// 已保存密码的主机别名
    pub remembered_password_aliases: HashSet<String>,
    /// 是否运行中
    pub running: bool,
    /// 地址显示/隐藏（默认隐藏）
    pub show_address: bool,
    /// 嵌入式终端会话（None = 未连接）
    pub term_session: Option<TermSession>,
    /// 终端事件接收端
    term_rx: Receiver<TermEvent>,
    /// 终端事件发送端（会话持有克隆）
    term_tx: Sender<TermEvent>,
    /// 嵌入式终端前缀键等待状态（Ctrl+B 按下后等待下一个 TUI 命令）
    term_prefix: bool,
    /// Dashboard 分隔线拖动状态（None = 未在拖动）
    pub split_drag: Option<SplitDrag>,
    /// 主界面（主机列表/详情）左栏宽度权重（0-100，默认 50）
    pub main_split_weight: u16,
    /// 主界面分隔线拖动中（记录起始权重，None = 未在拖动）
    pub main_split_drag: Option<u16>,
}

impl App {
    pub fn new(config: SshConfig, config_path: PathBuf) -> Self {
        let hosts = config.hosts;
        let (ping_tx, ping_rx) = mpsc::channel();
        let mut list_state = ListState::default();
        if !hosts.is_empty() {
            list_state.select(Some(0));
        }
        let remembered_password_aliases = load_saved_password_aliases(&hosts);
        let dashboard_config = load_dashboard_config();
        let dashboard_widgets = Self::build_widgets(&dashboard_config);
        let (monitor_scheduler, monitor_rx) = MonitorScheduler::with_channel();
        let (agent_tx, agent_rx) = mpsc::channel();
        let (term_tx, term_rx) = mpsc::channel();
        let (cmd_tx, cmd_rx) = mpsc::channel();
        let agent_config = load_agent_config();
        let main_split_weight = load_main_split_weight().clamp(20, 80);
        Self {
            hosts,
            preamble: config.preamble,
            config_path,
            list_state,
            scroll_offset: 0,
            mode: Mode::Normal,
            view: View::HostList,
            input_buffer: String::new(),
            search_keyword: String::new(),
            marked: Vec::new(),
            host_status: HashMap::new(),
            pending_pings: HashSet::new(),
            flash_message: None,
            host_form: None,
            dashboard_config,
            dashboard_widgets,
            palette_query: String::new(),
            palette_selected: 0,
            ping_rx,
            ping_tx,
            monitor_scheduler,
            monitor_rx,
            monitor_target: None,
            latest_snapshot: None,
            docker_selected: 0,
            docker_action: None,
            service_selected: 0,
            service_action: None,
            file_selected: 0,
            file_action: None,
            log_selected: 0,
            log_unit: String::new(),
            log_raw: Vec::new(),
            agent_form: None,
            agent_config,
            agent_rx,
            agent_tx,
            agent_approval_tx: None,
            agent_input: String::new(),
            agent_status: AgentStatus::Ready,
            agent_timeline: Timeline::new(),
            agent_history: Vec::new(),
            agent_thread: Vec::new(),
            agent_pending_call: None,
            command_input: String::new(),
            command_result: None,
            cmd_rx,
            cmd_tx,
            remembered_password_aliases,
            running: true,
            show_address: false,
            term_session: None,
            term_rx,
            term_tx,
            term_prefix: false,
            split_drag: None,
            main_split_weight,
            main_split_drag: None,
        }
    }

    /// 根据配置的活动 Profile 重建 Widget 实例（Mock 数据）
    fn build_widgets(config: &DashboardConfig) -> Vec<Box<dyn WidgetModule>> {
        let profile = config
            .profiles
            .get(&config.active_profile)
            .map(|profile| &profile.enabled)
            .cloned()
            .unwrap_or_default();
        profile
            .into_iter()
            .map(|id| {
                Box::new(crate::tui::dashboard::widgets::mock_widget(id)) as Box<dyn WidgetModule>
            })
            .collect()
    }

    /// 保存 Dashboard 配置到磁盘
    pub fn save_dashboard(&mut self) {
        if let Err(e) = save_dashboard_config(&self.dashboard_config) {
            self.set_flash_message(format!("Dashboard 配置保存失败: {}", e), "red");
        } else {
            self.dashboard_widgets = Self::build_widgets(&self.dashboard_config);
            self.set_flash_message("Dashboard 配置已保存", "green");
        }
    }

    /// 获取当前选中索引（映射为 hosts 中的真实下标）
    pub fn selected(&self) -> Option<usize> {
        self.visible_indices()
            .get(self.list_state.selected()?)
            .copied()
    }

    /// 当前可见主机下标列表（搜索过滤后；无过滤时返回全部）
    pub fn visible_indices(&self) -> Vec<usize> {
        let kw = self.search_keyword.trim();
        if kw.is_empty() {
            return (0..self.hosts.len()).collect();
        }
        let kw = kw.to_lowercase();
        self.hosts
            .iter()
            .enumerate()
            .filter(|(_, h)| {
                h.alias.to_lowercase().contains(&kw)
                    || h.hostname()
                        .map(|n| n.to_lowercase().contains(&kw))
                        .unwrap_or(false)
            })
            .map(|(i, _)| i)
            .collect()
    }

    /// 将内存中的 hosts 写回 SSH 配置文件
    fn save_config(&self) -> anyhow::Result<()> {
        let config = SshConfig {
            hosts: self.hosts.clone(),
            preamble: self.preamble.clone(),
        };
        let content = crate::config::render_config(&config);
        std::fs::write(&self.config_path, content)?;
        Ok(())
    }

    pub fn handle_form_key(&mut self, key: KeyEvent) {
        if self.agent_form.is_some() {
            self.handle_agent_form_key(key);
            return;
        }
        let Some(outcome) = self.host_form.as_mut().map(|form| form.handle_key(key)) else {
            return;
        };

        match outcome {
            EditorOutcome::Continue => {}
            EditorOutcome::Cancel => {
                let cancelled_mode = self.host_form.as_ref().map(|form| form.mode().clone());
                self.host_form = None;
                self.mode = Mode::Normal;
                let message = match cancelled_mode {
                    Some(HostFormMode::Add) => "已取消添加".to_string(),
                    Some(HostFormMode::Edit { .. }) => "已取消编辑".to_string(),
                    None => "已取消操作".to_string(),
                };
                self.set_timed_flash_message(message, "yellow", Duration::from_secs(1));
            }
            EditorOutcome::Save => {
                if let Err(e) = self.commit_host_form() {
                    self.set_flash_message(format!("保存失败: {}", e), "red");
                }
            }
        }
    }

    /// Agent 设置表单键盘处理
    fn handle_agent_form_key(&mut self, key: KeyEvent) {
        let Some(outcome) = self.agent_form.as_mut().map(|form| form.handle_key(key)) else {
            return;
        };
        match outcome {
            EditorOutcome::Continue => {}
            EditorOutcome::Cancel => {
                self.agent_form = None;
                self.mode = Mode::Normal;
                self.set_timed_flash_message("已取消 Agent 设置", "yellow", Duration::from_secs(1));
            }
            EditorOutcome::Save => {
                let config = match self
                    .agent_form
                    .as_ref()
                    .map(|form| form.build_config())
                    .transpose()
                {
                    Ok(Some(config)) => config,
                    Ok(None) => return,
                    Err(e) => {
                        self.set_flash_message(format!("设置无效: {}", e), "red");
                        return;
                    }
                };
                if let Err(e) = crate::agent::config::save_agent_config(&config) {
                    self.set_flash_message(format!("保存失败: {}", e), "red");
                    return;
                }
                self.agent_config = config.clone();
                self.agent_form = None;
                self.mode = Mode::Normal;
                self.set_flash_message(
                    format!(
                        "Agent 设置已保存: {} / {}（权限: {}）",
                        config.provider,
                        config.model,
                        config.permission_level().label()
                    ),
                    "green",
                );
            }
        }
    }

    pub fn poll_background_tasks(&mut self) {
        while let Ok(event) = self.ping_rx.try_recv() {
            self.handle_ping_event(event);
        }
        while let Ok(event) = self.monitor_rx.try_recv() {
            self.handle_monitor_event(event);
        }
        while let Ok(event) = self.agent_rx.try_recv() {
            self.handle_agent_event(event);
        }
        while let Ok(event) = self.cmd_rx.try_recv() {
            self.handle_cmd_event(event);
        }
        self.poll_term_events();
    }

    /// 消费嵌入式终端事件（新输出 → 触发重绘；会话结束 → 更新状态）
    fn poll_term_events(&mut self) {
        while let Ok(event) = self.term_rx.try_recv() {
            match event {
                TermEvent::Output => {
                    if let Some(session) = self.term_session.as_mut() {
                        session.mark_running();
                    }
                }
                TermEvent::Exited { code } => {
                    if let Some(session) = self.term_session.as_mut() {
                        session.handle_exit(code);
                    }
                    // 会话结束后同样回到主机列表主界面（与主动断开一致），
                    // 使 q 一次即可退出程序；仅在焦点仍在终端时切换，避免打断已打开的操作面板。
                    if self.mode == Mode::Terminal {
                        self.return_to_host_list();
                    }
                    self.set_flash_message("SSH 会话已结束", "yellow");
                }
            }
        }
    }

    /// 向嵌入式终端写入键盘输入（转发给 PTY）
    pub fn term_write_key(&mut self, key: KeyEvent) {
        if let Some(session) = self.term_session.as_mut() {
            session.write_key(key);
        }
    }

    /// 调整嵌入式终端尺寸（面板 resize 时调用）
    pub fn term_resize(&mut self, cols: u16, rows: u16) {
        if let Some(session) = self.term_session.as_mut() {
            session.resize(cols, rows);
        }
    }

    /// 是否处于嵌入式终端前缀键等待状态（Ctrl+B 已按下，等待 TUI 命令）
    pub fn term_prefix_active(&self) -> bool {
        self.term_prefix
    }

    /// 设置/清除嵌入式终端前缀键等待状态
    pub fn set_term_prefix(&mut self, active: bool) {
        self.term_prefix = active;
    }

    /// 取消分隔线拖动（放弃本次调整，不保存）
    pub fn cancel_split_drag(&mut self) {
        self.split_drag = None;
        self.main_split_drag = None;
    }

    /// 开始拖动分隔线：命中主界面或 Dashboard 布局中的分隔线则记录拖动状态
    fn start_split_drag(&mut self, col: u16, row: u16) {
        // Dashboard 视图下，Terminal / AgentOps 模式同样允许拖动分隔线；
        // 主机列表视图及弹窗类模式不允许拖动。
        let allow = matches!(self.mode, Mode::Normal)
            || (self.view == View::Dashboard
                && matches!(self.mode, Mode::Terminal | Mode::AgentOps));
        if !allow {
            return;
        }
        // 主界面（主机列表 / 详情）：命中垂直分隔线 → 记录左栏起始权重
        if self.view == View::HostList {
            let area = terminal_area();
            let boundary = main_split_boundary(self.main_split_weight, area.width);
            let on_line = col == boundary.saturating_sub(1) || col == boundary;
            if on_line {
                self.main_split_drag = Some(self.main_split_weight);
            }
            return;
        }
        if self.view != View::Dashboard {
            return;
        }
        let Some(profile) = self
            .dashboard_config
            .profiles
            .get(&self.dashboard_config.active_profile)
        else {
            return;
        };
        let area = terminal_area();
        let Some((path, pivot)) = crate::tui::dashboard::layout::hit_test_divider(
            &profile.layout,
            area,
            Position::new(col, row),
        ) else {
            return;
        };
        let Some(fixed) = crate::tui::dashboard::layout::split_sizes(&profile.layout, area, &path)
        else {
            return;
        };
        self.split_drag = Some(SplitDrag { path, pivot, fixed });
    }

    /// 拖动中更新分隔线两侧子节点的权重
    fn move_split_drag(&mut self, col: u16, row: u16) {
        // 主界面分隔线拖动：按列位置换算左栏权重
        if let Some(start) = self.main_split_drag {
            let area = terminal_area();
            if area.width == 0 {
                return;
            }
            let new_weight = (col as u32 * 100 / area.width as u32) as u16;
            self.main_split_weight = new_weight.clamp(20, 80);
            let _ = start;
            return;
        }
        let (path, pivot, fixed) = match &self.split_drag {
            Some(drag) => (drag.path.clone(), drag.pivot, drag.fixed.clone()),
            None => return,
        };
        let Some(profile) = self
            .dashboard_config
            .profiles
            .get_mut(&self.dashboard_config.active_profile)
        else {
            return;
        };
        let area = terminal_area();
        let Some((node, split_area, _)) =
            crate::tui::dashboard::layout::split_metrics(&mut profile.layout, area, &path)
        else {
            return;
        };
        let LayoutNode::Split {
            direction,
            children,
        } = node
        else {
            return;
        };
        let Some(weights) = crate::tui::dashboard::layout::apply_split_resize(
            children.len(),
            *direction,
            split_area,
            pivot,
            &fixed,
            col,
            row,
        ) else {
            return;
        };
        for (child, w) in children.iter_mut().zip(weights) {
            child.weight = w.max(1);
        }
    }

    /// 结束拖动并持久化调整后的布局
    fn end_split_drag(&mut self) {
        // 主界面分隔线：保存权重到配置目录下的 ui.json（不写 ~/.qsshrc）
        if self.main_split_drag.take().is_some() {
            if let Err(e) = save_main_split_weight(self.main_split_weight) {
                self.set_flash_message(format!("布局保存失败: {}", e), "red");
            } else {
                self.set_flash_message("布局已调整并保存", "green");
            }
            return;
        }
        if self.split_drag.take().is_none() {
            return;
        }
        if let Err(e) = save_dashboard_config(&self.dashboard_config) {
            self.set_flash_message(format!("布局保存失败: {}", e), "red");
        } else {
            self.set_flash_message("布局已调整并保存", "green");
        }
    }

    /// 启动嵌入式 SSH 终端会话（连接当前选中主机）
    pub fn start_terminal_session(&mut self) {
        // 若已有会话，先断开
        if let Some(mut session) = self.term_session.take() {
            session.kill();
        }
        let Some(idx) = self.selected() else {
            self.set_flash_message("未选择主机", "yellow");
            return;
        };
        let Some(host) = self.hosts.get(idx) else {
            return;
        };
        let target = SshTarget::from_host(host);
        // 每次启动使用全新事件通道，避免旧会话遗留事件干扰
        let (tx, rx) = mpsc::channel();
        match TermSession::spawn(target.clone(), tx.clone(), 80, 24) {
            Ok(session) => {
                self.term_session = Some(session);
                self.term_tx = tx;
                self.term_rx = rx;
                // 切换到 Dashboard 视图以显示嵌入式终端面板，并同步开启监控
                self.view = View::Dashboard;
                self.mode = Mode::Terminal;
                if self.monitor_target.is_none() {
                    self.start_monitoring();
                }
                self.set_flash_message(
                    format!("已连接 {}（Esc 或 Ctrl+Shift+C 断开）", target.alias),
                    "green",
                );
            }
            Err(e) => {
                self.set_flash_message(format!("连接失败: {}", e), "red");
            }
        }
    }

    /// 关闭嵌入式终端会话（断开 SSH）
    ///
    /// 断开后回到主机列表主界面（而非停留在 Dashboard），这样用户直接按一次 `q`
    /// 即可退出程序，符合轻量化应用的操作直觉。
    pub fn close_terminal_session(&mut self) {
        if let Some(mut session) = self.term_session.take() {
            session.kill();
        }
        self.return_to_host_list();
        self.set_flash_message("已断开 SSH 会话", "yellow");
    }

    /// 返回主机列表主界面：切换视图、复位模式并停止后台监控
    fn return_to_host_list(&mut self) {
        self.view = View::HostList;
        self.mode = Mode::Normal;
        self.stop_monitoring();
    }

    /// 启动对当前选中主机的后台监控（进入 Dashboard 视图时调用）
    pub fn start_monitoring(&mut self) {
        if self.monitor_target.is_some() {
            return;
        }
        let Some(idx) = self.selected() else {
            return;
        };
        let Some(host) = self.hosts.get(idx) else {
            return;
        };
        let target = SshTarget::from_host(host);
        let executor =
            SshProcessExecutor::new(target.clone()).with_timeouts(5, Duration::from_secs(10));
        let collector = Collector::new(Box::new(executor));
        let alias = target.alias.clone();
        self.monitor_scheduler
            .add_target(alias.clone(), collector, Duration::from_secs(2));
        self.monitor_target = Some(alias.clone());
        self.set_flash_message(format!("开始监控 {}", alias), "green");
    }

    /// 停止后台监控（离开 Dashboard 视图时调用）
    pub fn stop_monitoring(&mut self) {
        if let Some(alias) = self.monitor_target.take() {
            self.monitor_scheduler.stop_target(&alias);
            self.set_flash_message(format!("停止监控 {}", alias), "yellow");
        }
    }

    /// 消费监控事件：将最新快照分发到各 Widget
    fn handle_monitor_event(&mut self, event: BackgroundEvent) {
        match event {
            BackgroundEvent::MonitorSnapshot(snapshot) => {
                for widget in &mut self.dashboard_widgets {
                    widget.set_snapshot(Some(&snapshot));
                }
                self.latest_snapshot = Some(snapshot.clone());
                if snapshot.warnings.is_empty() {
                    return;
                }
                let warning = snapshot.warnings.join("; ");
                // 致命错误（连接失败 / 无任何数据）红色常驻提示；普通警告黄色
                let (message, color) = if snapshot.has_fatal_error() {
                    (
                        format!("监控失败 {}: {}", snapshot.alias, warning),
                        "red".to_string(),
                    )
                } else {
                    (
                        format!("{}: {}", snapshot.alias, warning),
                        "yellow".to_string(),
                    )
                };
                self.set_flash_message(message, color);
            }
        }
    }

    pub fn expire_flash_message(&mut self) {
        let should_clear = self
            .flash_message
            .as_ref()
            .and_then(|message| message.expires_at)
            .is_some_and(|expires_at| Instant::now() >= expires_at);

        if should_clear {
            self.flash_message = None;
        }
    }

    /// 应用 Action 更新状态（核心方法）
    pub fn apply(&mut self, action: Action) {
        match action {
            Action::None => {}
            Action::MoveUp => {
                let pos = self.list_state.selected().unwrap_or(0);
                if pos > 0 {
                    self.list_state.select(Some(pos - 1));
                }
            }
            Action::MoveDown => {
                let pos = self.list_state.selected().unwrap_or(0);
                if pos + 1 < self.visible_indices().len() {
                    self.list_state.select(Some(pos + 1));
                }
            }
            Action::MoveTop => {
                if !self.visible_indices().is_empty() {
                    self.list_state.select(Some(0));
                }
            }
            Action::MoveBottom => {
                let len = self.visible_indices().len();
                if len > 0 {
                    self.list_state.select(Some(len - 1));
                }
            }
            Action::SelectListItem(idx) => match self.mode {
                Mode::Normal => {
                    if idx < self.visible_indices().len() {
                        self.list_state.select(Some(idx));
                    }
                }
                Mode::DockerOps => {
                    let len = self.docker_snapshot().map(|s| s.docker.len()).unwrap_or(0);
                    if idx < len {
                        self.docker_selected = idx;
                    }
                }
                Mode::ServiceOps => {
                    let len = self
                        .docker_snapshot()
                        .map(|s| s.services.len())
                        .unwrap_or(0);
                    if idx < len {
                        self.service_selected = idx;
                    }
                }
                Mode::FileOps => {
                    let len = self.docker_snapshot().map(|s| s.files.len()).unwrap_or(0);
                    if idx < len {
                        self.file_selected = idx;
                    }
                }
                Mode::LogOps => {
                    if idx < self.log_selected_count() {
                        self.log_selected = idx;
                    }
                }
                Mode::Palette if idx < PALETTE_ACTIONS.len() => {
                    self.palette_selected = idx;
                }
                _ => {}
            },
            Action::Quit => {
                // 退出前终止嵌入式 SSH 会话，避免残留子进程
                if let Some(mut session) = self.term_session.take() {
                    session.kill();
                }
                self.running = false;
            }
            Action::ShowHelp => self.mode = Mode::Help,
            Action::HideHelp => {
                self.mode = Mode::Normal;
            }
            Action::CancelSearch => {
                self.input_buffer.clear();
                self.search_keyword.clear();
                self.mode = Mode::Normal;
            }
            // ── Dashboard 视图切换 ────────────────────────
            Action::ShowDashboard => {
                self.view = View::Dashboard;
                self.mode = Mode::Normal;
                self.start_monitoring();
            }
            Action::ShowHostList => {
                self.view = View::HostList;
                self.mode = Mode::Normal;
                self.stop_monitoring();
            }
            Action::EditDashboard => {
                self.mode = Mode::DashboardConfig;
            }
            // ── Dashboard 配置弹窗 ────────────────────────
            Action::CloseDashboardConfig => {
                self.restore_mode_after_overlay();
            }
            Action::ToggleDashboardModule(id) => {
                if let Some(profile) = self
                    .dashboard_config
                    .profiles
                    .get_mut(&self.dashboard_config.active_profile)
                {
                    if let Some(pos) = profile.enabled.iter().position(|w| *w == id) {
                        profile.enabled.remove(pos);
                    } else {
                        profile.enabled.push(id);
                    }
                    profile.layout = crate::tui::dashboard::default_layout_for(&profile.enabled);
                }
            }
            Action::SaveDashboardConfig => {
                self.mode = Mode::Normal;
                self.save_dashboard();
            }
            // ── Docker 容器操作 ──────────────────────────
            Action::OpenDockerOps => {
                if self.latest_snapshot.is_some() {
                    self.docker_selected = 0;
                    self.mode = Mode::DockerOps;
                } else {
                    self.set_flash_message(
                        "暂无容器数据，请先在 Dashboard 视图等待监控采集",
                        "yellow",
                    );
                }
            }
            Action::CloseDockerOps => {
                self.restore_mode_after_overlay();
                self.docker_action = None;
            }
            Action::DockerOpsMove(delta) => {
                let count = self
                    .latest_snapshot
                    .as_ref()
                    .map(|s| s.docker.len())
                    .unwrap_or(0);
                if count > 0 {
                    let len = count as isize;
                    let next = self.docker_selected as isize + delta;
                    self.docker_selected = next.rem_euclid(len) as usize;
                }
            }
            Action::DockerActionSelected(action) => {
                self.docker_action = Some(action);
                self.mode = Mode::DockerConfirm;
            }
            Action::ConfirmDocker(confirmed) => {
                let pending = self.docker_action.take();
                self.restore_mode_after_overlay();
                if !confirmed {
                    return;
                }
                let Some(action) = pending else { return };
                let Some(container) = self
                    .latest_snapshot
                    .as_ref()
                    .and_then(|s| s.docker.get(self.docker_selected))
                    .cloned()
                else {
                    self.set_flash_message("容器列表已刷新，请重新选择", "yellow");
                    return;
                };
                // 复用当前监控目标的执行器执行危险操作
                let Some(alias) = self.monitor_target.clone() else {
                    self.set_flash_message("未在监控状态，无法执行容器操作", "red");
                    return;
                };
                let Some(host) = self.hosts.iter().find(|h| h.alias == alias).cloned() else {
                    self.set_flash_message("主机已不存在", "red");
                    return;
                };
                let target = SshTarget::from_host(&host);
                let executor =
                    SshProcessExecutor::new(target).with_timeouts(5, Duration::from_secs(10));
                let command = container_action_command(action, &container.id);
                let flash = format!(
                    "执行 docker {} {} ({})…",
                    action.subcommand(),
                    container.name,
                    alias
                );
                self.set_flash_message(flash, "yellow");
                let (result_tx, result_rx) = mpsc::channel();
                let _ = thread::spawn(move || {
                    let outcome = match executor.exec(&command) {
                        Ok(out) if out.exit_code == Some(0) => {
                            format!("{} 成功", action.label())
                        }
                        Ok(out) => format!(
                            "{} 失败: {}",
                            action.label(),
                            out.stderr.trim_end().lines().next().unwrap_or("未知错误")
                        ),
                        Err(e) => format!("{} 失败: {}", action.label(), e),
                    };
                    let _ = result_tx.send(outcome);
                });
                if let Ok(outcome) = result_rx.recv_timeout(Duration::from_secs(15)) {
                    self.set_flash_message(outcome, "green");
                } else {
                    self.set_flash_message("操作超时", "red");
                }
            }
            // ── 系统服务操作 ────────────────────────────
            Action::OpenServiceOps => {
                if self.latest_snapshot.is_some() {
                    self.service_selected = 0;
                    self.mode = Mode::ServiceOps;
                } else {
                    self.set_flash_message(
                        "暂无服务数据，请先在 Dashboard 视图等待监控采集",
                        "yellow",
                    );
                }
            }
            Action::CloseServiceOps => {
                self.restore_mode_after_overlay();
                self.service_action = None;
            }
            Action::ServiceOpsMove(delta) => {
                let count = self
                    .latest_snapshot
                    .as_ref()
                    .map(|s| s.services.len())
                    .unwrap_or(0);
                if count > 0 {
                    let len = count as isize;
                    let next = self.service_selected as isize + delta;
                    self.service_selected = next.rem_euclid(len) as usize;
                }
            }
            Action::ServiceActionSelected(action) => {
                self.service_action = Some(action);
                self.mode = Mode::ServiceConfirm;
            }
            Action::ConfirmService(confirmed) => {
                let pending = self.service_action.take();
                self.restore_mode_after_overlay();
                if !confirmed {
                    return;
                }
                let Some(action) = pending else { return };
                let Some(service) = self
                    .latest_snapshot
                    .as_ref()
                    .and_then(|s| s.services.get(self.service_selected))
                    .cloned()
                else {
                    self.set_flash_message("服务列表已刷新，请重新选择", "yellow");
                    return;
                };
                let Some(alias) = self.monitor_target.clone() else {
                    self.set_flash_message("未在监控状态，无法执行服务操作", "red");
                    return;
                };
                let Some(host) = self.hosts.iter().find(|h| h.alias == alias).cloned() else {
                    self.set_flash_message("主机已不存在", "red");
                    return;
                };
                let target = SshTarget::from_host(&host);
                let executor =
                    SshProcessExecutor::new(target).with_timeouts(5, Duration::from_secs(10));
                let command = service_action_command(action, &service.name);
                let flash = format!(
                    "执行 systemctl {} {} ({})…",
                    action.subcommand(),
                    service.name,
                    alias
                );
                self.set_flash_message(flash, "yellow");
                let (result_tx, result_rx) = mpsc::channel();
                let _ = thread::spawn(move || {
                    let outcome = match executor.exec(&command) {
                        Ok(out) if out.exit_code == Some(0) => {
                            format!("{} 成功", action.label())
                        }
                        Ok(out) => format!(
                            "{} 失败: {}",
                            action.label(),
                            out.stderr.trim_end().lines().next().unwrap_or("未知错误")
                        ),
                        Err(e) => format!("{} 失败: {}", action.label(), e),
                    };
                    let _ = result_tx.send(outcome);
                });
                if let Ok(outcome) = result_rx.recv_timeout(Duration::from_secs(15)) {
                    self.set_flash_message(outcome, "green");
                } else {
                    self.set_flash_message("操作超时", "red");
                }
            }
            // ── 文件浏览操作 ────────────────────────────
            Action::OpenFileOps => {
                if self.latest_snapshot.is_some() {
                    self.file_selected = 0;
                    self.mode = Mode::FileOps;
                } else {
                    self.set_flash_message(
                        "暂无文件数据，请先在 Dashboard 视图等待监控采集",
                        "yellow",
                    );
                }
            }
            Action::CloseFileOps => {
                self.restore_mode_after_overlay();
                self.file_action = None;
            }
            Action::FileOpsMove(delta) => {
                let count = self
                    .latest_snapshot
                    .as_ref()
                    .map(|s| s.files.len())
                    .unwrap_or(0);
                if count > 0 {
                    let len = count as isize;
                    let next = self.file_selected as isize + delta;
                    self.file_selected = next.rem_euclid(len) as usize;
                }
            }
            Action::FileOpsEnter => {
                let Some(alias) = self.monitor_target.clone() else {
                    self.set_flash_message("未在监控状态，无法浏览文件", "red");
                    return;
                };
                let Some(host) = self.hosts.iter().find(|h| h.alias == alias).cloned() else {
                    self.set_flash_message("主机已不存在", "red");
                    return;
                };
                let Some(file) = self
                    .latest_snapshot
                    .as_ref()
                    .and_then(|s| s.files.get(self.file_selected))
                    .cloned()
                else {
                    self.set_flash_message("文件列表已刷新，请重新选择", "yellow");
                    return;
                };
                if !file.is_dir {
                    return;
                }
                let new_cwd = join_dir(
                    &self
                        .latest_snapshot
                        .as_ref()
                        .map(|s| s.file_cwd.clone())
                        .unwrap_or_else(|| "~".to_string()),
                    &file.name,
                );
                let target = SshTarget::from_host(&host);
                let executor =
                    SshProcessExecutor::new(target).with_timeouts(5, Duration::from_secs(10));
                let command = ls_l_command(&new_cwd);
                let (result_tx, result_rx) = mpsc::channel();
                let _ = thread::spawn(move || {
                    let outcome = executor
                        .exec(&command)
                        .map(|out| (out.stdout, out.stderr, out.exit_code));
                    let _ = result_tx.send(outcome);
                });
                if let Ok(Ok((stdout, stderr, exit_code))) =
                    result_rx.recv_timeout(Duration::from_secs(15))
                {
                    if let Some(snap) = self.latest_snapshot.as_mut() {
                        if exit_code == Some(0) {
                            snap.files = dir_entries(parse_ls_l(&stdout));
                            snap.file_cwd = new_cwd.clone();
                            self.file_selected = 0;
                        } else {
                            self.set_flash_message(
                                format!(
                                    "进入目录失败: {}",
                                    stderr.trim_end().lines().next().unwrap_or("未知错误")
                                ),
                                "red",
                            );
                        }
                    }
                } else {
                    self.set_flash_message("进入目录超时", "red");
                }
            }
            Action::FileOpsUp => {
                let Some(alias) = self.monitor_target.clone() else {
                    self.set_flash_message("未在监控状态，无法浏览文件", "red");
                    return;
                };
                let Some(host) = self.hosts.iter().find(|h| h.alias == alias).cloned() else {
                    self.set_flash_message("主机已不存在", "red");
                    return;
                };
                let Some(cwd) = self.latest_snapshot.as_ref().map(|s| s.file_cwd.clone()) else {
                    return;
                };
                let new_cwd = crate::monitor::files::parent_dir(&cwd);
                if new_cwd == cwd {
                    return;
                }
                let target = SshTarget::from_host(&host);
                let executor =
                    SshProcessExecutor::new(target).with_timeouts(5, Duration::from_secs(10));
                let command = ls_l_command(&new_cwd);
                let (result_tx, result_rx) = mpsc::channel();
                let _ = thread::spawn(move || {
                    let outcome = executor
                        .exec(&command)
                        .map(|out| (out.stdout, out.stderr, out.exit_code));
                    let _ = result_tx.send(outcome);
                });
                if let Ok(Ok((stdout, stderr, exit_code))) =
                    result_rx.recv_timeout(Duration::from_secs(15))
                {
                    if let Some(snap) = self.latest_snapshot.as_mut() {
                        if exit_code == Some(0) {
                            snap.files = dir_entries(parse_ls_l(&stdout));
                            snap.file_cwd = new_cwd.clone();
                            self.file_selected = 0;
                        } else {
                            self.set_flash_message(
                                format!(
                                    "返回上级失败: {}",
                                    stderr.trim_end().lines().next().unwrap_or("未知错误")
                                ),
                                "red",
                            );
                        }
                    }
                } else {
                    self.set_flash_message("返回上级超时", "red");
                }
            }
            Action::FileActionSelected(action) => {
                self.file_action = Some(action);
                self.mode = Mode::FileConfirm;
            }
            Action::ConfirmFile(confirmed) => {
                let pending = self.file_action.take();
                self.restore_mode_after_overlay();
                if !confirmed {
                    return;
                }
                let Some(action) = pending else { return };
                let Some(file) = self
                    .latest_snapshot
                    .as_ref()
                    .and_then(|s| s.files.get(self.file_selected))
                    .cloned()
                else {
                    self.set_flash_message("文件列表已刷新，请重新选择", "yellow");
                    return;
                };
                let Some(alias) = self.monitor_target.clone() else {
                    self.set_flash_message("未在监控状态，无法下载文件", "red");
                    return;
                };
                let Some(host) = self.hosts.iter().find(|h| h.alias == alias).cloned() else {
                    self.set_flash_message("主机已不存在", "red");
                    return;
                };
                let target = SshTarget::from_host(&host);
                let remote_path = format!(
                    "{}/{}",
                    self.latest_snapshot
                        .as_ref()
                        .map(|s| s.file_cwd.as_str())
                        .unwrap_or("~"),
                    file.name
                );
                match action {
                    FileAction::Download => {
                        let user_part = match target.user {
                            Some(ref u) => format!("{}@", u),
                            None => String::new(),
                        };
                        let remote_target = format!(
                            "{}{}:{}",
                            user_part,
                            target.hostname,
                            shell_quote(&remote_path)
                        );
                        let dest = std::env::current_dir()
                            .map(|d| d.join(&file.name))
                            .unwrap_or_else(|_| PathBuf::from(&file.name));
                        let flash = format!("下载 {} → {}…", remote_path, dest.display());
                        self.set_flash_message(flash, "yellow");
                        let (result_tx, result_rx) = mpsc::channel();
                        let _ = thread::spawn(move || {
                            let mut cmd = std::process::Command::new("scp");
                            if target.port != 22 {
                                cmd.arg("-P").arg(target.port.to_string());
                            }
                            if let Some(ref key_path) = target.identity_file {
                                cmd.arg("-i").arg(key_path.as_os_str());
                            }
                            cmd.arg("-o").arg("ControlMaster=no");
                            cmd.arg("-q");
                            cmd.arg(&remote_target);
                            cmd.arg(&dest);
                            let outcome = match cmd.status() {
                                Ok(st) if st.success() => "下载成功".to_string(),
                                Ok(st) => format!("下载失败: 退出码 {:?}", st.code().unwrap_or(-1)),
                                Err(e) => format!("下载失败: {}", e),
                            };
                            let _ = result_tx.send(outcome);
                        });
                        if let Ok(outcome) = result_rx.recv_timeout(Duration::from_secs(60)) {
                            self.set_flash_message(outcome, "green");
                        } else {
                            self.set_flash_message("下载超时", "red");
                        }
                    }
                }
            }
            // ── 日志面板 ────────────────────────────────
            Action::OpenLogOps => {
                if self.latest_snapshot.is_some() {
                    self.log_raw = self
                        .latest_snapshot
                        .as_ref()
                        .map(|s| s.logs.clone())
                        .unwrap_or_default();
                    self.log_selected = 0;
                    self.mode = Mode::LogOps;
                } else {
                    self.set_flash_message(
                        "暂无日志数据，请先在 Dashboard 视图等待监控采集",
                        "yellow",
                    );
                }
            }
            Action::CloseLogOps => {
                self.restore_mode_after_overlay();
            }
            Action::LogOpsMove(delta) => {
                let count = self.log_selected_count();
                if count > 0 {
                    let len = count as isize;
                    let next = self.log_selected as isize + delta;
                    self.log_selected = next.rem_euclid(len) as usize;
                }
            }
            Action::LogOpsRefresh => {
                self.mode = Mode::LogOps;
                let Some(alias) = self.monitor_target.clone() else {
                    self.set_flash_message("未在监控状态，无法获取日志", "red");
                    return;
                };
                let Some(host) = self.hosts.iter().find(|h| h.alias == alias).cloned() else {
                    self.set_flash_message("主机已不存在", "red");
                    return;
                };
                let target = SshTarget::from_host(&host);
                let executor =
                    SshProcessExecutor::new(target).with_timeouts(5, Duration::from_secs(15));
                let unit = if self.log_unit.trim().is_empty() {
                    None
                } else {
                    Some(self.log_unit.trim().to_string())
                };
                let command = journalctl_command(unit.as_deref(), MAX_LOG_LINES);
                let (result_tx, result_rx) = mpsc::channel();
                let _ = thread::spawn(move || {
                    let outcome = executor
                        .exec(&command)
                        .map(|out| (out.stdout, out.stderr, out.exit_code));
                    let _ = result_tx.send(outcome);
                });
                if let Ok(Ok((stdout, stderr, exit_code))) =
                    result_rx.recv_timeout(Duration::from_secs(20))
                {
                    if exit_code == Some(0) {
                        let parsed = parse_journalctl(&stdout);
                        if let Some(snap) = self.latest_snapshot.as_mut() {
                            snap.logs = parsed.clone();
                            snap.log_unit = self.log_unit.clone();
                        }
                        self.log_raw = parsed;
                        self.log_selected = 0;
                        self.set_flash_message(
                            format!(
                                "已刷新日志 {} 条{}",
                                self.log_raw.len(),
                                if self.log_unit.trim().is_empty() {
                                    String::new()
                                } else {
                                    format!("（unit: {}）", self.log_unit.trim())
                                }
                            ),
                            "green",
                        );
                    } else {
                        self.set_flash_message(
                            format!(
                                "获取日志失败: {}",
                                stderr.trim_end().lines().next().unwrap_or("未知错误")
                            ),
                            "red",
                        );
                    }
                } else {
                    self.set_flash_message("获取日志超时", "red");
                }
            }
            Action::LogUnitFilter(input) => {
                self.log_unit = input;
            }
            // ── AI Agent 面板 ────────────────────────────
            Action::OpenAgentOps => {
                // 打开面板时刷新配置，使 WebUI 修改的权限级别即时反映到状态栏/面板
                self.refresh_agent_config();
                self.mode = Mode::AgentOps;
                self.agent_input.clear();
                self.agent_status = AgentStatus::Ready;
            }
            Action::CloseAgentOps => {
                self.restore_mode_after_overlay();
            }
            Action::FocusAgent => {
                // 鼠标点击聚焦 Agent 面板：不重置输入，保留对话现场
                if self.view == View::Dashboard {
                    self.mode = Mode::AgentOps;
                }
            }
            Action::FocusTerminal => {
                // 鼠标点击聚焦终端面板：SSH 会话仍连接时切回嵌入式终端；
                // 会话不存在或已断开时（如经主机列表全屏 SSH 后回到 Dashboard），
                // 直接发起连接，使终端标签页具备“点击即激活”的标签页式体验。
                if self.view != View::Dashboard {
                    return;
                }
                let alive = self
                    .term_session
                    .as_ref()
                    .map(|s| s.status != crate::tui::term::TermStatus::Exited)
                    .unwrap_or(false);
                if alive {
                    self.mode = Mode::Terminal;
                } else {
                    self.start_terminal_session();
                }
            }
            Action::OpenAgentConfig => {
                self.agent_form = Some(AgentFormState::new(&self.agent_config));
                self.mode = Mode::AgentConfig;
            }
            Action::CloseAgentConfig => {
                self.agent_form = None;
                self.restore_mode_after_overlay();
            }
            Action::AgentInput(input) => {
                self.agent_input = input;
            }
            Action::AgentSubmit => {
                let input = self.agent_input.trim().to_string();
                if input.is_empty() {
                    self.set_flash_message("请输入指令", "yellow");
                    return;
                }
                self.agent_input.clear();
                self.agent_thread.push(AgentChatEntry::User(input.clone()));
                // 用户直接输入工具调用（无 LLM 时可用）
                if let Some(call) = crate::agent::tools::parse_tool_call(&input) {
                    self.run_agent_tool(call, input);
                    return;
                }
                self.start_agent_chat(input);
            }
            Action::AgentRespond(approved) => {
                self.agent_pending_call = None;
                if let Some(tx) = self.agent_approval_tx.take() {
                    let _ = tx.send(approved);
                }
                if approved {
                    self.set_flash_message("已批准执行", "green");
                } else {
                    self.set_flash_message("已拒绝执行", "yellow");
                }
                self.mode = Mode::AgentOps;
                self.agent_status = AgentStatus::Executing;
            }
            // ── 命令面板 ─────────────────────────────────
            Action::OpenPalette => {
                self.mode = Mode::Palette;
                self.palette_query.clear();
                self.palette_selected = 0;
            }
            Action::ClosePalette => {
                self.restore_mode_after_overlay();
            }
            Action::PaletteInput(query) => {
                self.palette_query = query;
                self.palette_selected = 0;
            }
            Action::PaletteMove(delta) => {
                let count = filter_palette(PALETTE_ACTIONS, &self.palette_query).len();
                if count > 0 {
                    let len = count as isize;
                    let next = self.palette_selected as isize + delta;
                    self.palette_selected = next.rem_euclid(len) as usize;
                }
            }
            Action::PaletteSelect => {
                let items = filter_palette(PALETTE_ACTIONS, &self.palette_query);
                if let Some(item) =
                    items.get(self.palette_selected.min(items.len().saturating_sub(1)))
                {
                    self.mode = Mode::Normal;
                    match item.action {
                        PaletteAction::ShowDashboard => {
                            self.view = View::Dashboard;
                            self.start_monitoring();
                        }
                        PaletteAction::ShowHostList => {
                            self.view = View::HostList;
                            self.stop_monitoring();
                        }
                        PaletteAction::EditDashboard => {
                            self.mode = Mode::DashboardConfig;
                        }
                        PaletteAction::EditAgentConfig => {
                            // 打开设置前刷新配置，确保表单展示的是磁盘最新值
                            self.refresh_agent_config();
                            self.agent_form = Some(AgentFormState::new(&self.agent_config));
                            self.mode = Mode::AgentConfig;
                        }
                        PaletteAction::OpenAgent => {
                            // 打开面板前刷新配置，使 WebUI 修改的权限级别即时生效
                            self.refresh_agent_config();
                            self.mode = Mode::AgentOps;
                            self.agent_input.clear();
                            self.agent_status = AgentStatus::Ready;
                        }
                        PaletteAction::RefreshAll => {
                            self.start_ping_all();
                        }
                        PaletteAction::AddHost => {
                            self.mode = Mode::Add;
                            self.host_form = Some(HostFormState::new_add());
                            self.flash_message = None;
                        }
                        PaletteAction::SearchHost => {
                            self.mode = Mode::Search;
                            self.input_buffer.clear();
                        }
                        PaletteAction::Quit => {
                            self.running = false;
                        }
                    }
                }
            }
            Action::StartSearch => {
                self.mode = Mode::Search;
                self.input_buffer.clear();
                self.search_keyword.clear();
            }
            Action::SearchInput(ch) => {
                self.input_buffer = ch.clone();
                self.search_keyword = ch;
            }
            Action::SearchSubmit => {
                self.search_keyword = self.input_buffer.clone();
                self.mode = Mode::Normal;
                if !self.visible_indices().is_empty() {
                    self.list_state.select(Some(0));
                }
            }
            // ── `:` 远程命令 ─────────────────────────────
            Action::OpenCommand => {
                self.mode = Mode::Command;
                self.command_input.clear();
            }
            Action::CloseCommand => {
                self.restore_mode_after_overlay();
                self.command_input.clear();
            }
            Action::CommandInput(input) => {
                self.command_input = input;
            }
            Action::CommandSubmit => {
                let command = self.command_input.trim().to_string();
                if command.is_empty() {
                    self.set_flash_message("请输入要执行的命令", "yellow");
                    return;
                }
                self.start_remote_command(command);
            }
            Action::CloseCommandResult => {
                self.command_result = None;
                self.restore_mode_after_overlay();
            }
            Action::StartAdd => {
                self.mode = Mode::Add;
                self.host_form = Some(HostFormState::new_add());
                self.flash_message = None;
            }
            // ── 删除 ──────────────────────────────────────
            Action::Delete => {
                if self.selected().is_some() && !self.hosts.is_empty() {
                    self.mode = Mode::Confirm;
                }
            }
            Action::ConfirmDelete(confirmed) => {
                if confirmed {
                    // 批量删除：如果有标记的主机，删除所有标记项
                    if !self.marked.is_empty() {
                        let marked_aliases: Vec<String> = self
                            .marked
                            .iter()
                            .filter_map(|&i| self.hosts.get(i).map(|h| h.alias.clone()))
                            .collect();
                        let count = marked_aliases.len();

                        // 从后往前删除，避免索引漂移
                        let mut sorted = self.marked.clone();
                        sorted.sort_unstable_by(|a, b| b.cmp(a));
                        sorted.dedup();
                        for &i in &sorted {
                            if i < self.hosts.len() {
                                self.hosts.remove(i);
                            }
                        }
                        // 清除所有标记中的主机在线状态和已保存密码
                        for alias in &marked_aliases {
                            self.host_status.remove(alias);
                            self.remembered_password_aliases.remove(alias);
                            let _ = credentials::delete_password(alias);
                        }
                        self.marked.clear();

                        // 调整选中位置
                        let new_len = self.hosts.len();
                        if new_len > 0 {
                            let selected = self.selected().unwrap_or(0);
                            if selected >= new_len {
                                self.list_state.select(Some(new_len - 1));
                            }
                        } else {
                            self.list_state.select(None);
                        }

                        // 保存到 SSH 配置文件
                        if let Err(e) = self.save_config() {
                            self.set_flash_message(format!("保存失败: {}", e), "red");
                        } else {
                            self.set_flash_message(format!("已批量删除 {} 台主机", count), "green");
                        }
                    } else if let Some(idx) = self.selected() {
                        // 单台删除（原有逻辑）
                        if idx < self.hosts.len() {
                            let alias = self.hosts[idx].alias.clone();
                            self.hosts.remove(idx);
                            self.host_status.remove(&alias);
                            // 调整选中位置
                            let new_len = self.hosts.len();
                            if new_len > 0 {
                                let new_idx = if idx >= new_len { new_len - 1 } else { idx };
                                self.list_state.select(Some(new_idx));
                            } else {
                                self.list_state.select(None);
                            }
                            // 保存到 SSH 配置文件
                            if let Err(e) = self.save_config() {
                                self.set_flash_message(format!("保存失败: {}", e), "red");
                            } else {
                                self.remembered_password_aliases.remove(&alias);
                                match credentials::delete_password(&alias) {
                                    Ok(_) => {
                                        self.set_flash_message(
                                            format!("已删除主机 \"{}\"", alias),
                                            "green",
                                        );
                                    }
                                    Err(err) => {
                                        self.set_flash_message(
                                            format!(
                                                "已删除主机 \"{}\"，但清理已保存密码失败: {}",
                                                alias, err
                                            ),
                                            "yellow",
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
                self.mode = Mode::Normal;
            }
            // ── 编辑 ──────────────────────────────────────
            Action::StartEdit => {
                if let Some(idx) = self.selected() {
                    if idx < self.hosts.len() {
                        let host = &self.hosts[idx];
                        let has_saved_password =
                            self.remembered_password_aliases.contains(&host.alias);
                        self.host_form =
                            Some(HostFormState::new_edit(idx, host, has_saved_password));
                        self.mode = Mode::Edit;
                        self.flash_message = None;
                    }
                }
            }
            Action::DoAdd(_input) => {
                // DoAdd 在 Add 模式下按 Enter 触发，由 keymap 构造
                // 这里留空，实际添加由 cmd::add 处理
                self.mode = Mode::Normal;
            }
            Action::Connect => {
                // 主机列表主界面的连接由事件循环负责（退出 TUI 全屏 SSH）；
                // 此处仅处理 Dashboard 控制台内的嵌入式终端连接。
                if self.view == View::Dashboard {
                    self.start_terminal_session();
                }
            }
            Action::CloseTerminal => {
                self.close_terminal_session();
            }
            Action::TermScroll(delta) => {
                if let Some(session) = self.term_session.as_mut() {
                    session.scroll(delta);
                }
            }
            Action::TermScrollToBottom => {
                if let Some(session) = self.term_session.as_mut() {
                    session.scroll_to_bottom();
                }
            }
            // ── Dashboard 分隔线拖动 ─────────────────────
            Action::StartSplitDrag { col, row } => {
                self.start_split_drag(col, row);
            }
            Action::MoveSplitDrag { col, row } => {
                self.move_split_drag(col, row);
            }
            Action::EndSplitDrag => {
                self.end_split_drag();
            }
            Action::Ping => {
                if let Some(idx) = self.selected() {
                    if let Some(host) = self.hosts.get(idx) {
                        if let Some(hostname) = host.hostname() {
                            self.start_single_ping(
                                host.alias.clone(),
                                hostname.to_string(),
                                host.port(),
                            );
                        }
                    }
                }
            }
            Action::PingAll => {
                self.start_ping_all();
            }
            Action::ToggleSelect => {
                if let Some(idx) = self.selected() {
                    if self.marked.contains(&idx) {
                        self.marked.retain(|&i| i != idx);
                    } else {
                        self.marked.push(idx);
                    }
                }
            }
            Action::ToggleAddress => {
                self.show_address = !self.show_address;
            }
            _ => {}
        }
    }

    fn commit_host_form(&mut self) -> anyhow::Result<()> {
        let (form_mode, submission) = {
            let form = self
                .host_form
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("表单未初始化"))?;
            (form.mode().clone(), form.build_submission()?)
        };

        match form_mode {
            HostFormMode::Add => self.commit_new_host(submission),
            HostFormMode::Edit {
                index,
                original_alias,
            } => self.commit_existing_host(index, original_alias, submission),
        }
    }

    fn commit_new_host(&mut self, submission: HostFormSubmission) -> anyhow::Result<()> {
        let HostFormSubmission {
            host: new_host,
            password_action,
        } = submission;

        if self.hosts.iter().any(|host| host.alias == new_host.alias) {
            anyhow::bail!("主机别名 \"{}\" 已存在", new_host.alias);
        }

        let new_alias = new_host.alias.clone();
        self.hosts.push(new_host);
        let new_index = self.hosts.len() - 1;
        self.list_state.select(Some(new_index));

        if let Err(err) = self.save_config() {
            self.hosts.pop();
            if self.hosts.is_empty() {
                self.list_state.select(None);
            } else {
                self.list_state.select(Some(self.hosts.len() - 1));
            }
            return Err(err);
        }

        if let Err(err) = self.apply_new_password_action(&new_alias, &password_action) {
            self.hosts.pop();
            if self.hosts.is_empty() {
                self.list_state.select(None);
            } else {
                self.list_state.select(Some(self.hosts.len() - 1));
            }
            let _ = self.save_config();
            return Err(err);
        }

        self.refresh_password_cache_for_alias(&new_alias);

        self.mode = Mode::Normal;
        self.host_form = None;
        self.set_flash_message(format!("已新增主机 \"{}\"", new_alias), "green");
        Ok(())
    }

    fn commit_existing_host(
        &mut self,
        idx: usize,
        original_alias: String,
        submission: HostFormSubmission,
    ) -> anyhow::Result<()> {
        let HostFormSubmission {
            host: updated_host,
            password_action,
        } = submission;

        if idx >= self.hosts.len() {
            anyhow::bail!("当前选中的主机不存在");
        }

        if self
            .hosts
            .iter()
            .enumerate()
            .any(|(host_idx, host)| host_idx != idx && host.alias == updated_host.alias)
        {
            anyhow::bail!("主机别名 \"{}\" 已存在", updated_host.alias);
        }

        let previous_host = self.hosts[idx].clone();
        let new_alias = updated_host.alias.clone();
        self.hosts[idx] = updated_host;

        if let Err(err) = self.save_config() {
            self.hosts[idx] = previous_host;
            return Err(err);
        }

        if let Err(err) =
            self.apply_existing_password_action(&original_alias, &new_alias, &password_action)
        {
            self.hosts[idx] = previous_host;
            let _ = self.save_config();
            return Err(err);
        }

        self.host_status.remove(&original_alias);
        self.refresh_password_cache_for_alias(&original_alias);
        self.refresh_password_cache_for_alias(&new_alias);
        self.mode = Mode::Normal;
        self.host_form = None;
        self.set_flash_message(format!("已更新主机 \"{}\"", new_alias), "green");
        Ok(())
    }

    fn apply_new_password_action(
        &self,
        alias: &str,
        password_action: &PasswordStorageAction,
    ) -> anyhow::Result<()> {
        match password_action {
            PasswordStorageAction::Unchanged | PasswordStorageAction::Keep => Ok(()),
            PasswordStorageAction::Set(password) => credentials::store_password(alias, password),
            PasswordStorageAction::Clear => credentials::delete_password(alias),
        }
    }

    fn apply_existing_password_action(
        &self,
        original_alias: &str,
        new_alias: &str,
        password_action: &PasswordStorageAction,
    ) -> anyhow::Result<()> {
        match password_action {
            PasswordStorageAction::Unchanged => Ok(()),
            PasswordStorageAction::Keep => {
                if original_alias != new_alias {
                    credentials::move_password(original_alias, new_alias)?;
                }
                Ok(())
            }
            PasswordStorageAction::Set(password) => {
                credentials::store_password(new_alias, password)?;
                if original_alias != new_alias {
                    credentials::delete_password(original_alias)?;
                }
                Ok(())
            }
            PasswordStorageAction::Clear => {
                credentials::delete_password(original_alias)?;
                if original_alias != new_alias {
                    credentials::delete_password(new_alias)?;
                }
                Ok(())
            }
        }
    }

    fn refresh_password_cache_for_alias(&mut self, alias: &str) {
        match credentials::has_password(alias) {
            Ok(true) => {
                self.remembered_password_aliases.insert(alias.to_string());
            }
            Ok(false) | Err(_) => {
                self.remembered_password_aliases.remove(alias);
            }
        }
    }

    /// 在选中的主机上后台执行远程命令（`:` 快捷命令）
    fn start_remote_command(&mut self, command: String) {
        let Some(host_idx) = self.selected() else {
            self.set_flash_message("请先选择一台主机", "yellow");
            return;
        };
        let Some(host) = self.hosts.get(host_idx).cloned() else {
            return;
        };
        let alias = host.alias.clone();
        let target = SshTarget::from_host(&host);
        let executor = SshProcessExecutor::new(target).with_timeouts(5, Duration::from_secs(30));

        self.mode = Mode::Normal;
        self.set_flash_message(format!("{} 上执行: {}", alias, command), "yellow");

        let tx = self.cmd_tx.clone();
        thread::spawn(move || {
            let output = executor.exec(&command);
            let _ = tx.send(CmdEvent::Finished { alias, output });
        });
    }

    /// 处理远程命令执行结果
    fn handle_cmd_event(&mut self, event: CmdEvent) {
        let CmdEvent::Finished { alias, output } = event;
        match output {
            Ok(out) => {
                self.command_result = Some(CommandResult {
                    alias,
                    command: self.command_input.clone(),
                    stdout: out.stdout,
                    stderr: out.stderr,
                    exit_code: out.exit_code,
                    error: None,
                });
            }
            Err(err) => {
                self.command_result = Some(CommandResult {
                    alias,
                    command: self.command_input.clone(),
                    stdout: String::new(),
                    stderr: String::new(),
                    exit_code: None,
                    error: Some(err.to_string()),
                });
            }
        }
        self.command_input.clear();
        self.mode = Mode::CommandResult;
    }

    fn start_single_ping(&mut self, alias: String, hostname: String, port: u16) {
        if self.pending_pings.contains(&alias) {
            self.set_flash_message(format!("{} 正在检测中…", alias), "yellow");
            return;
        }

        self.pending_pings.insert(alias.clone());
        self.set_flash_message(format!("开始检测 {}…", alias), "yellow");

        let tx = self.ping_tx.clone();
        thread::spawn(move || {
            let (online, error) = run_ping_check(&hostname, port);
            let _ = tx.send(PingEvent::SingleFinished {
                alias,
                online,
                error,
            });
        });
    }

    fn start_ping_all(&mut self) {
        let targets: Vec<(String, String, u16)> = self
            .hosts
            .iter()
            .filter_map(|host| {
                host.hostname()
                    .map(|hostname| (host.alias.clone(), hostname.to_string(), host.port()))
            })
            .filter(|(alias, _, _)| !self.pending_pings.contains(alias))
            .collect();

        if targets.is_empty() {
            self.set_flash_message("没有可检测的主机，或都在检测中", "yellow");
            return;
        }

        let total = targets.len();
        for (alias, _, _) in &targets {
            self.pending_pings.insert(alias.clone());
        }
        self.set_flash_message(format!("开始全量检测 {} 台主机…", total), "yellow");

        let tx = self.ping_tx.clone();
        thread::spawn(move || {
            let (result_tx, result_rx) = mpsc::channel();

            for (alias, hostname, port) in targets {
                let result_tx = result_tx.clone();
                thread::spawn(move || {
                    let (online, error) = run_ping_check(&hostname, port);
                    let _ = result_tx.send((alias, online, error));
                });
            }
            drop(result_tx);

            let mut online_count = 0usize;
            for (alias, online, error) in result_rx {
                if online {
                    online_count += 1;
                }
                let _ = tx.send(PingEvent::BatchHostFinished {
                    alias,
                    online,
                    error,
                });
            }

            let _ = tx.send(PingEvent::BatchCompleted {
                online_count,
                total,
            });
        });
    }

    fn handle_ping_event(&mut self, event: PingEvent) {
        let ping_msg_duration = Duration::from_secs(1);

        match event {
            PingEvent::SingleFinished {
                alias,
                online,
                error,
            } => {
                self.pending_pings.remove(&alias);
                if !self.host_exists(&alias) {
                    return;
                }

                self.host_status.insert(alias.clone(), online);
                match error {
                    Some(error) => {
                        self.set_timed_flash_message(
                            format!("检测失败: {}: {}", alias, error),
                            "red",
                            ping_msg_duration,
                        );
                    }
                    None => {
                        let (status, color) = if online {
                            ("在线", "green")
                        } else {
                            ("离线", "yellow")
                        };
                        self.set_timed_flash_message(
                            format!("{}: {}", alias, status),
                            color,
                            ping_msg_duration,
                        );
                    }
                }
            }
            PingEvent::BatchHostFinished {
                alias,
                online,
                error,
            } => {
                self.pending_pings.remove(&alias);
                if !self.host_exists(&alias) {
                    return;
                }

                self.host_status.insert(alias, online);
                if error.is_some() {
                    // 对全量检测不逐条刷错误提示，统一在最终状态里给结果。
                }
            }
            PingEvent::BatchCompleted {
                online_count,
                total,
            } => {
                let color = if online_count == total {
                    "green"
                } else {
                    "yellow"
                };
                self.set_timed_flash_message(
                    format!("全量检测完成: {}/{} 在线", online_count, total),
                    color,
                    ping_msg_duration,
                );
            }
        }
    }

    fn host_exists(&self, alias: &str) -> bool {
        self.hosts.iter().any(|host| host.alias == alias)
    }

    // ── Docker 操作面板访问器（UI 层只读）─────────────────

    /// 最近一次监控快照（含容器列表）
    pub fn docker_snapshot(&self) -> Option<&ServerSnapshot> {
        self.latest_snapshot.as_ref()
    }

    /// 容器列表当前选中索引
    pub fn docker_selected_index(&self) -> usize {
        self.docker_selected
    }

    /// 容器条目数
    #[allow(dead_code)]
    pub fn docker_count(&self) -> usize {
        self.docker_snapshot().map(|s| s.docker.len()).unwrap_or(0)
    }

    /// 待确认的 Docker 操作（None = 未处于确认流程）
    pub fn docker_pending_action(&self) -> Option<DockerAction> {
        self.docker_action
    }

    /// 当前监控目标别名
    pub fn monitor_target_alias(&self) -> Option<&str> {
        self.monitor_target.as_deref()
    }

    // ── 系统服务操作面板访问器（UI 层只读）───────────────

    /// 服务列表当前选中索引
    pub fn service_selected_index(&self) -> usize {
        self.service_selected
    }

    /// 服务条目数
    pub fn service_count(&self) -> usize {
        self.docker_snapshot()
            .map(|s| s.services.len())
            .unwrap_or(0)
    }

    /// 待确认的服务操作（None = 未处于确认流程）
    pub fn service_pending_action(&self) -> Option<ServiceAction> {
        self.service_action
    }

    // ── 文件浏览面板访问器（UI 层只读）───────────────────

    /// 文件列表当前选中索引
    pub fn file_selected_index(&self) -> usize {
        self.file_selected
    }

    /// 文件条目数
    pub fn file_count(&self) -> usize {
        self.docker_snapshot().map(|s| s.files.len()).unwrap_or(0)
    }

    /// 待确认的文件操作（None = 未处于确认流程）
    pub fn file_pending_action(&self) -> Option<FileAction> {
        self.file_action
    }

    // ── 日志面板访问器（UI 层只读）──────────────────────

    /// 日志列表当前选中索引
    pub fn log_selected_index(&self) -> usize {
        self.log_selected
    }

    /// 当前筛选 unit（空 = 全部）
    pub fn log_unit(&self) -> &str {
        &self.log_unit
    }

    /// 日志条目数（已按 unit 过滤）
    pub fn log_selected_count(&self) -> usize {
        self.filtered_logs().len()
    }

    /// 按当前 unit 过滤后的日志
    pub fn filtered_logs(&self) -> Vec<LogEntry> {
        let unit = self.log_unit.trim();
        if unit.is_empty() {
            self.log_raw.clone()
        } else {
            self.log_raw
                .iter()
                .filter(|log| log.unit.contains(unit))
                .cloned()
                .collect()
        }
    }

    // ── AI Agent 访问器（UI 层只读）─────────────────────

    /// Agent 输入缓冲
    pub fn agent_input(&self) -> &str {
        &self.agent_input
    }

    /// `:` 远程命令输入缓冲
    pub fn command_input(&self) -> &str {
        &self.command_input
    }

    /// `:` 远程命令执行结果（None = 无结果）
    pub fn command_result(&self) -> Option<&CommandResult> {
        self.command_result.as_ref()
    }

    /// Agent 状态（状态栏 AI 指示）
    pub fn agent_status(&self) -> AgentStatus {
        self.agent_status
    }

    /// Agent 时间线
    pub fn agent_timeline(&self) -> &Timeline {
        &self.agent_timeline
    }

    /// 待批准的 Agent 工具调用（None = 未处于确认流程）
    pub fn agent_pending_call(&self) -> Option<&ToolCall> {
        self.agent_pending_call.as_ref()
    }

    /// 对话历史条数
    pub fn agent_history_count(&self) -> usize {
        self.agent_history.len()
    }

    /// 当前权限级别中文标签
    pub fn agent_permission_label(&self) -> &'static str {
        self.agent_config.permission_level().label()
    }

    /// 重新从磁盘加载 Agent 配置（WebUI / 设置面板修改后即时生效）。
    ///
    /// `agent.json` 可能被 WebUI（`qssh web`）或外部编辑，而 TUI 启动时只加载一次；
    /// 每次发起 Agent 会话前刷新，保证权限级别等设置与磁盘一致。
    fn refresh_agent_config(&mut self) {
        self.agent_config = load_agent_config();
    }

    /// Agent 对话线程（用户/助手消息 + 工具步骤）
    pub fn agent_thread(&self) -> &[AgentChatEntry] {
        &self.agent_thread
    }

    // ── AI Agent 后台会话 ──────────────────────────────

    /// 消费 Agent 后台事件，更新 UI 状态
    fn handle_agent_event(&mut self, event: AgentEvent) {
        match event {
            AgentEvent::Started => {
                self.agent_status = AgentStatus::Thinking;
            }
            AgentEvent::Status(status) => {
                self.agent_status = status;
            }
            AgentEvent::Timeline(timeline) => {
                self.agent_timeline = timeline;
            }
            AgentEvent::ApprovalNeeded { call, reason } => {
                self.agent_pending_call = Some(call);
                if !reason.is_empty() {
                    self.set_flash_message(reason, "yellow");
                }
                self.agent_status = AgentStatus::Approval;
                self.mode = Mode::AgentConfirm;
            }
            AgentEvent::Finished { reply, error } => {
                // 把本轮时间线步骤固化为线程条目（工具调用内联展示）
                let steps = self.agent_timeline.steps().to_vec();
                for step in steps {
                    self.agent_thread.push(AgentChatEntry::Tool(step));
                }
                if let Some(error) = error {
                    self.agent_status = AgentStatus::Error;
                    self.agent_thread
                        .push(AgentChatEntry::Assistant(format!("⚠ {}", error)));
                    self.set_flash_message(format!("Agent 错误: {}", error), "red");
                } else {
                    self.agent_status = AgentStatus::Done;
                    self.agent_thread
                        .push(AgentChatEntry::Assistant(reply.clone()));
                    self.set_flash_message("Agent 执行完成", "green");
                }
                self.agent_approval_tx = None;
            }
        }
    }

    /// 启动 LLM 会话（后台线程，阻塞执行；UI 通过事件接收进度）
    fn start_agent_chat(&mut self, input: String) {
        // 每次会话前刷新配置，使 WebUI 修改的权限级别立即生效
        self.refresh_agent_config();
        let Some(alias) = self.monitor_target.clone() else {
            self.set_flash_message("未在监控状态，无法使用 Agent", "red");
            return;
        };
        let Some(host) = self.hosts.iter().find(|h| h.alias == alias).cloned() else {
            self.set_flash_message("主机已不存在", "red");
            return;
        };
        let target = SshTarget::from_host(&host);
        let executor = SshProcessExecutor::new(target).with_timeouts(5, Duration::from_secs(15));
        let snapshot = self.latest_snapshot.clone();
        let history = self.agent_history.clone();
        let config = self.agent_config.clone();
        let tx = self.agent_tx.clone();
        let approval_tx = self.agent_tx.clone();

        // 构建审批通道：后台线程创建，等待 UI 回传
        let (approve_channel_tx, approve_channel_rx) = mpsc::channel::<bool>();
        self.agent_approval_tx = Some(approve_channel_tx);
        let _ = tx.send(AgentEvent::Started);
        let _ = tx.send(AgentEvent::Timeline(Timeline::new()));

        thread::spawn(move || {
            use crate::agent::AgentRunner;
            let runner = AgentRunner::new(config, tx);
            let approve = move |_call: &ToolCall| {
                // 阻塞等待 UI 决定（App::AgentRespond 回传）
                approve_channel_rx
                    .recv_timeout(Duration::from_secs(300))
                    .unwrap_or(false)
            };
            // 构造快照引用（在闭包内持有默认值，避免临时借用）
            let default_snapshot = ServerSnapshot::new("");
            let snapshot_ref = snapshot.as_ref().unwrap_or(&default_snapshot);
            let session = runner.run(&history, &input, &executor, snapshot_ref, approve);
            // 最终时间线（与 run() 内部事件重复，幂等无害）
            let _ = approval_tx.send(AgentEvent::Timeline(session.timeline.clone()));
            let _ = approval_tx.send(AgentEvent::Finished {
                reply: session.reply,
                error: session.error,
            });
        });
    }

    /// 直接执行用户手动输入的工具调用（无 LLM 时可用）
    fn run_agent_tool(&mut self, call: ToolCall, _input: String) {
        // 每次执行前刷新配置，使 WebUI 修改的权限级别立即生效
        self.refresh_agent_config();
        let Some(alias) = self.monitor_target.clone() else {
            self.set_flash_message("未在监控状态，无法执行工具", "red");
            return;
        };
        let Some(host) = self.hosts.iter().find(|h| h.alias == alias).cloned() else {
            self.set_flash_message("主机已不存在", "red");
            return;
        };
        let target = SshTarget::from_host(&host);
        let executor = SshProcessExecutor::new(target).with_timeouts(5, Duration::from_secs(15));
        let tx = self.agent_tx.clone();
        let permission = self.agent_config.permission_level();

        // 构建审批通道：后台线程阻塞等待 UI 决定（App::AgentRespond 回传）
        let (approve_channel_tx, approve_channel_rx) = mpsc::channel::<bool>();
        self.agent_approval_tx = Some(approve_channel_tx);

        let _ = tx.send(AgentEvent::Status(AgentStatus::Executing));
        let _ = tx.send(AgentEvent::Timeline(Timeline::new()));

        thread::spawn(move || {
            use crate::agent::timeline::{StepStatus, TimelineStep};
            use crate::agent::tools::execute_tool;
            let mut timeline = Timeline::new();
            let step_index = timeline.len();
            timeline.push(TimelineStep::new(
                call.name(),
                call.tool.description(),
                call.is_read_only(),
            ));
            let _ = tx.send(AgentEvent::Timeline(timeline.clone()));

            let approval = permission.decide(call.tool.danger());
            let approved = match approval {
                crate::agent::permissions::Approval::Allowed => true,
                crate::agent::permissions::Approval::Denied => {
                    timeline.set_result(
                        step_index,
                        StepStatus::Skipped,
                        format!("被权限策略拒绝（{}）", permission.label()),
                    );
                    let _ = tx.send(AgentEvent::Status(AgentStatus::Ready));
                    let _ = tx.send(AgentEvent::Timeline(timeline.clone()));
                    let _ = tx.send(AgentEvent::Finished {
                        reply: String::new(),
                        error: Some(format!("工具 {} 被权限拒绝", call.name())),
                    });
                    return;
                }
                crate::agent::permissions::Approval::NeedsApproval => {
                    let _ = tx.send(AgentEvent::ApprovalNeeded {
                        call: call.clone(),
                        reason: format!(
                            "工具 {}（{}）请求执行 {} 操作",
                            call.name(),
                            call.tool.description(),
                            call.tool.danger().label()
                        ),
                    });
                    // 阻塞等待 UI 决定（App::AgentRespond 回传，超时视为拒绝）
                    approve_channel_rx
                        .recv_timeout(Duration::from_secs(300))
                        .unwrap_or(false)
                }
            };

            if !approved {
                timeline.set_result(step_index, StepStatus::Skipped, "用户取消".to_string());
                let _ = tx.send(AgentEvent::Status(AgentStatus::Ready));
                let _ = tx.send(AgentEvent::Timeline(timeline.clone()));
                let _ = tx.send(AgentEvent::Finished {
                    reply: String::new(),
                    error: None,
                });
                return;
            }

            let result = execute_tool(&executor, &call);
            let (status, detail) = if result.ok {
                (StepStatus::Done, result.output.clone())
            } else {
                (StepStatus::Failed, result.output.clone())
            };
            timeline.set_result(step_index, status, detail);
            let _ = tx.send(AgentEvent::Timeline(timeline.clone()));
            let _ = tx.send(AgentEvent::Status(AgentStatus::Done));
            let _ = tx.send(AgentEvent::Finished {
                reply: if result.ok {
                    format!("工具 {} 执行成功:\n{}", call.name(), result.output)
                } else {
                    format!("工具 {} 执行失败:\n{}", call.name(), result.output)
                },
                error: if result.ok { None } else { Some(result.output) },
            });
        });
    }

    fn set_flash_message(&mut self, message: impl Into<String>, color: impl Into<String>) {
        self.flash_message = Some(FlashMessage {
            message: message.into(),
            color: color.into(),
            expires_at: None,
        });
    }

    /// 关闭操作面板/弹窗后应进入的模式：
    /// SSH 会话仍连接且在 Dashboard 视图时，把焦点还给嵌入式终端（键盘继续转发给 PTY）；
    /// 否则回到普通模式。
    fn restore_mode_after_overlay(&mut self) {
        if self.term_session.is_some() && self.view == View::Dashboard {
            self.mode = Mode::Terminal;
        } else {
            self.mode = Mode::Normal;
        }
    }

    fn set_timed_flash_message(
        &mut self,
        message: impl Into<String>,
        color: impl Into<String>,
        duration: Duration,
    ) {
        self.flash_message = Some(FlashMessage {
            message: message.into(),
            color: color.into(),
            expires_at: Some(Instant::now() + duration),
        });
    }
}

fn run_ping_check(hostname: &str, port: u16) -> (bool, Option<String>) {
    match crate::network::ping::check_host(hostname, port, 3) {
        Ok(online) => (online, None),
        Err(error) => (false, Some(error.to_string())),
    }
}

/// 当前 Dashboard 渲染区域（以 0,0 为原点；与 mouse.rs 的坐标系约定一致）。
/// 主体位于 1 行标题栏与 1 行状态栏之间，与 ui.rs 渲染及鼠标命中保持一致。
fn terminal_area() -> ratatui::layout::Rect {
    let (w, h) = crossterm::terminal::size().unwrap_or((0, 0));
    ratatui::layout::Rect::new(0, 1, w, h.saturating_sub(2))
}

/// 主界面左栏（主机列表）宽度权重对应的垂直分隔线 x 坐标（绝对终端列）
fn main_split_boundary(weight: u16, width: u16) -> u16 {
    let weight = weight.clamp(0, 100);
    (width as u32 * weight as u32 / 100) as u16
}

/// 主界面分隔权重配置文件路径（与 dashboard.json 同目录，独立 JSON 文件）
fn ui_config_path() -> PathBuf {
    crate::tui::dashboard::config::config_path()
        .parent()
        .map(|p| p.join("ui.json"))
        .unwrap_or_else(|| PathBuf::from("ui.json"))
}

/// 加载主界面主机列表左栏权重（默认 50）
fn load_main_split_weight() -> u16 {
    let path = ui_config_path();
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str::<UiConfig>(&s).ok())
        .and_then(|c| c.main_split_weight)
        .unwrap_or(50)
}

/// 保存主界面主机列表左栏权重
fn save_main_split_weight(weight: u16) -> anyhow::Result<()> {
    let path = ui_config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let config = UiConfig {
        main_split_weight: Some(weight),
    };
    let content = serde_json::to_string_pretty(&config)?;
    std::fs::write(&path, content)?;
    Ok(())
}

/// 主界面 UI 配置（独立 JSON，不触碰手写的 ~/.qsshrc）
#[derive(serde::Serialize, serde::Deserialize)]
struct UiConfig {
    main_split_weight: Option<u16>,
}

fn load_saved_password_aliases(hosts: &[HostBlock]) -> HashSet<String> {
    hosts
        .iter()
        .filter_map(|host| match credentials::has_password(&host.alias) {
            Ok(true) => Some(host.alias.clone()),
            Ok(false) | Err(_) => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::App;
    use crate::config::types::SshConfig;
    use crate::tui::action::{Action, Mode, View};
    use std::path::PathBuf;
    use std::time::{Duration, Instant};

    #[test]
    fn timed_flash_message_expires_and_clears() {
        let mut app = App::new(
            SshConfig {
                hosts: vec![],
                preamble: String::new(),
            },
            PathBuf::from("dummy"),
        );

        app.set_timed_flash_message("已取消编辑", "yellow", Duration::from_secs(1));
        assert!(app.flash_message.is_some());

        if let Some(message) = app.flash_message.as_mut() {
            message.expires_at = Some(Instant::now() - Duration::from_millis(10));
        }

        app.expire_flash_message();
        assert!(app.flash_message.is_none());
    }

    #[test]
    fn focus_agent_switches_to_agent_ops_without_resetting_input() {
        let mut app = App::new(
            SshConfig {
                hosts: vec![],
                preamble: String::new(),
            },
            PathBuf::from("dummy"),
        );
        app.view = View::Dashboard;
        app.agent_input = String::from("查看日志");
        app.apply(Action::FocusAgent);
        assert_eq!(app.mode, Mode::AgentOps);
        assert_eq!(app.agent_input, "查看日志");
    }

    #[test]
    fn focus_agent_only_in_dashboard_view() {
        let mut app = App::new(
            SshConfig {
                hosts: vec![],
                preamble: String::new(),
            },
            PathBuf::from("dummy"),
        );
        app.view = View::HostList;
        app.apply(Action::FocusAgent);
        assert_ne!(app.mode, Mode::AgentOps);
    }

    #[test]
    fn focus_terminal_requires_live_session() {
        let mut app = App::new(
            SshConfig {
                hosts: vec![],
                preamble: String::new(),
            },
            PathBuf::from("dummy"),
        );
        app.view = View::Dashboard;
        app.apply(Action::FocusTerminal);
        // 无会话时不应进入 Terminal 模式
        assert_ne!(app.mode, Mode::Terminal);
    }

    #[test]
    fn close_terminal_session_returns_to_host_list() {
        let mut app = App::new(
            SshConfig {
                hosts: vec![],
                preamble: String::new(),
            },
            PathBuf::from("dummy"),
        );
        app.view = View::Dashboard;
        app.mode = Mode::Terminal;
        app.close_terminal_session();
        // 断开后回到主机列表主界面，按一次 q 即可退出程序
        assert_eq!(app.view, View::HostList);
        assert_eq!(app.mode, Mode::Normal);
    }

    #[test]
    fn main_split_drag_updates_weight() {
        let mut app = App::new(
            SshConfig {
                hosts: vec![],
                preamble: String::new(),
            },
            PathBuf::from("dummy"),
        );
        app.view = View::HostList;
        app.mode = Mode::Normal;
        app.main_split_drag = Some(50);
        // 拖动到第 70 列（假设终端宽 100 → 权重 70），应用后应 clamp 在 20..80
        app.apply(Action::MoveSplitDrag { col: 70, row: 5 });
        assert!(app.main_split_weight >= 20 && app.main_split_weight <= 80);
        // 拖到极端左（col=2）应 clamp 到 20
        app.apply(Action::MoveSplitDrag { col: 2, row: 5 });
        assert_eq!(app.main_split_weight, 20);
        // 结束拖动清除状态
        app.apply(Action::EndSplitDrag);
        assert!(app.main_split_drag.is_none());
    }

    /// refresh_agent_config 会把磁盘上的 agent.json 权限刷新进内存（与 WebUI 修改保持一致）
    #[test]
    fn refresh_agent_config_reloads_disk_permission() {
        use crate::agent::config::{config_path, AgentConfig};
        use crate::agent::permissions::PermissionLevel;

        // 备份原文件，测试后恢复
        let path = config_path();
        let original = std::fs::read_to_string(&path).ok();

        // 写一个 full_access 配置到磁盘
        let test_config = AgentConfig {
            permission: PermissionLevel::FullAccess.as_str().to_string(),
            ..AgentConfig::default()
        };
        crate::agent::config::save_agent_config(&test_config).expect("save test config");

        // App 构造时加载的是磁盘上的 full_access
        let mut app = App::new(
            SshConfig {
                hosts: vec![],
                preamble: String::new(),
            },
            PathBuf::from("dummy"),
        );
        assert_eq!(
            app.agent_config.permission_level(),
            PermissionLevel::FullAccess
        );

        // 模拟 WebUI 把磁盘配置改为 read_only，再调用 refresh
        let ro_config = AgentConfig {
            permission: PermissionLevel::ReadOnly.as_str().to_string(),
            ..AgentConfig::default()
        };
        crate::agent::config::save_agent_config(&ro_config).expect("save ro config");
        app.refresh_agent_config();
        assert_eq!(
            app.agent_config.permission_level(),
            PermissionLevel::ReadOnly
        );

        // 恢复原文件
        match original {
            Some(content) => {
                let _ = std::fs::write(&path, content);
            }
            None => {
                let _ = std::fs::remove_file(&path);
            }
        }
    }
}
