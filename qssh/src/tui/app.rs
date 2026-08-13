use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

use crossterm::event::KeyEvent;
use ratatui::widgets::ListState;

use crate::config::credentials;
use crate::config::types::{HostBlock, SshConfig};
use crate::monitor::docker::{container_action_command, DockerAction};
use crate::monitor::executor::RemoteExecutor;
use crate::monitor::files::{
    dir_entries, join_dir, ls_l_command, parse_ls_l, shell_quote, FileAction,
};
use crate::monitor::scheduler::{BackgroundEvent, MonitorScheduler};
use crate::monitor::services::{service_action_command, ServiceAction};
use crate::monitor::snapshot::ServerSnapshot;
use crate::monitor::{Collector, SshProcessExecutor};
use crate::ssh::session::SshTarget;
use crate::tui::action::{Action, Mode, View};
use crate::tui::dashboard::palette::{filter_palette, PaletteAction, PALETTE_ACTIONS};
use crate::tui::dashboard::widgets::WidgetModule;
use crate::tui::dashboard::{load_dashboard_config, save_dashboard_config, DashboardConfig};
use crate::tui::editor::{
    EditorOutcome, HostFormMode, HostFormState, HostFormSubmission, PasswordStorageAction,
};

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

pub struct FlashMessage {
    pub message: String,
    pub color: String,
    expires_at: Option<Instant>,
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
    /// 已保存密码的主机别名
    pub remembered_password_aliases: HashSet<String>,
    /// 是否运行中
    pub running: bool,
    /// 地址显示/隐藏（默认隐藏）
    pub show_address: bool,
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
            remembered_password_aliases,
            running: true,
            show_address: false,
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

    /// 获取当前选中索引
    pub fn selected(&self) -> Option<usize> {
        self.list_state.selected()
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

    pub fn poll_background_tasks(&mut self) {
        while let Ok(event) = self.ping_rx.try_recv() {
            self.handle_ping_event(event);
        }
        while let Ok(event) = self.monitor_rx.try_recv() {
            self.handle_monitor_event(event);
        }
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
                let i = self.selected().unwrap_or(0);
                if i > 0 {
                    self.list_state.select(Some(i - 1));
                }
            }
            Action::MoveDown => {
                let i = self.selected().unwrap_or(0);
                if i + 1 < self.hosts.len() {
                    self.list_state.select(Some(i + 1));
                }
            }
            Action::MoveTop => {
                if !self.hosts.is_empty() {
                    self.list_state.select(Some(0));
                }
            }
            Action::MoveBottom => {
                if !self.hosts.is_empty() {
                    self.list_state.select(Some(self.hosts.len() - 1));
                }
            }
            Action::Quit => self.running = false,
            Action::ShowHelp => self.mode = Mode::Help,
            Action::HideHelp | Action::CancelSearch => {
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
                self.mode = Mode::Normal;
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
                self.mode = Mode::Normal;
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
                self.mode = Mode::Normal;
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
                self.mode = Mode::Normal;
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
                self.mode = Mode::Normal;
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
                self.mode = Mode::Normal;
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
                self.mode = Mode::Normal;
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
            // ── 命令面板 ─────────────────────────────────
            Action::OpenPalette => {
                self.mode = Mode::Palette;
                self.palette_query.clear();
                self.palette_selected = 0;
            }
            Action::ClosePalette => {
                self.mode = Mode::Normal;
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
            }
            Action::SearchInput(ch) => {
                self.input_buffer = ch;
            }
            Action::SearchSubmit => {
                self.search_keyword = self.input_buffer.clone();
                self.mode = Mode::Normal;
                if !self.hosts.is_empty() {
                    self.list_state.select(Some(0));
                }
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
            Action::Connect => {}
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

    /// 待确认的服务操作（None = 未处于确认流程）
    pub fn service_pending_action(&self) -> Option<ServiceAction> {
        self.service_action
    }

    // ── 文件浏览面板访问器（UI 层只读）───────────────────

    /// 文件列表当前选中索引
    pub fn file_selected_index(&self) -> usize {
        self.file_selected
    }

    /// 待确认的文件操作（None = 未处于确认流程）
    pub fn file_pending_action(&self) -> Option<FileAction> {
        self.file_action
    }

    fn set_flash_message(&mut self, message: impl Into<String>, color: impl Into<String>) {
        self.flash_message = Some(FlashMessage {
            message: message.into(),
            color: color.into(),
            expires_at: None,
        });
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
}
