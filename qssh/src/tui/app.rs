use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

use crossterm::event::KeyEvent;
use ratatui::widgets::ListState;

use crate::config::types::{HostBlock, SshConfig};
use crate::tui::action::{Action, Mode};
use crate::tui::editor::{EditorOutcome, HostFormMode, HostFormState};

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
    /// 后台检测结果通道
    ping_rx: Receiver<PingEvent>,
    ping_tx: Sender<PingEvent>,
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
        Self {
            hosts,
            preamble: config.preamble,
            config_path,
            list_state,
            scroll_offset: 0,
            mode: Mode::Normal,
            input_buffer: String::new(),
            search_keyword: String::new(),
            marked: Vec::new(),
            host_status: HashMap::new(),
            pending_pings: HashSet::new(),
            flash_message: None,
            host_form: None,
            ping_rx,
            ping_tx,
            running: true,
            show_address: false,
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
                    if let Some(idx) = self.selected() {
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
                                self.set_flash_message(
                                    format!("已删除主机 \"{}\"", alias),
                                    "green",
                                );
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
                        self.host_form = Some(HostFormState::new_edit(idx, host));
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
        let (form_mode, built_host) = {
            let form = self
                .host_form
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("表单未初始化"))?;
            (form.mode().clone(), form.build_host()?)
        };

        match form_mode {
            HostFormMode::Add => self.commit_new_host(built_host),
            HostFormMode::Edit {
                index,
                original_alias,
            } => self.commit_existing_host(index, original_alias, built_host),
        }
    }

    fn commit_new_host(&mut self, new_host: HostBlock) -> anyhow::Result<()> {
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

        self.mode = Mode::Normal;
        self.host_form = None;
        self.set_flash_message(format!("已新增主机 \"{}\"", new_alias), "green");
        Ok(())
    }

    fn commit_existing_host(
        &mut self,
        idx: usize,
        original_alias: String,
        updated_host: HostBlock,
    ) -> anyhow::Result<()> {
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

        self.host_status.remove(&original_alias);
        self.mode = Mode::Normal;
        self.host_form = None;
        self.set_flash_message(format!("已更新主机 \"{}\"", new_alias), "green");
        Ok(())
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
