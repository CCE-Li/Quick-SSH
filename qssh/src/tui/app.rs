use std::collections::HashMap;
use std::path::PathBuf;

use ratatui::widgets::ListState;

use crate::config::types::HostBlock;
use crate::tui::action::{Action, Mode};

// ── 应用状态 ─────────────────────────────────────────────

/// TUI 应用状态
pub struct App {
    /// 主机列表
    pub hosts: Vec<HostBlock>,
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
    /// 闪烁消息
    pub flash_message: Option<(String, String)>,
    /// 是否运行中
    pub running: bool,
}

impl App {
    pub fn new(hosts: Vec<HostBlock>, config_path: PathBuf) -> Self {
        let mut list_state = ListState::default();
        if !hosts.is_empty() {
            list_state.select(Some(0));
        }
        Self {
            hosts,
            config_path,
            list_state,
            scroll_offset: 0,
            mode: Mode::Normal,
            input_buffer: String::new(),
            search_keyword: String::new(),
            marked: Vec::new(),
            host_status: HashMap::new(),
            flash_message: None,
            running: true,
        }
    }

    /// 获取当前选中索引
    pub fn selected(&self) -> Option<usize> {
        self.list_state.selected()
    }

    /// 将内存中的 hosts 写回 SSH 配置文件
    fn save_config(&self) -> anyhow::Result<()> {
        let config = crate::config::SshConfig {
            hosts: self.hosts.clone(),
            preamble: String::new(),
        };
        let content = crate::config::render_config(&config);
        std::fs::write(&self.config_path, content)?;
        Ok(())
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
                self.input_buffer.push_str(&ch);
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
                self.input_buffer.clear();
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
                                self.flash_message =
                                    Some((format!("❌ 保存失败: {}", e), "red".into()));
                            } else {
                                self.flash_message =
                                    Some((format!("🗑️ 已删除主机 \"{}\"", alias), "green".into()));
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
                        let hostname = host.hostname().unwrap_or("");
                        let user = host.user().unwrap_or("");
                        let port = host.port();
                        // 预填: user@hostname:port
                        self.input_buffer = if user.is_empty() {
                            format!("{}:{}", hostname, port)
                        } else {
                            format!("{}@{}:{}", user, hostname, port)
                        };
                        self.mode = Mode::Edit;
                    }
                }
            }
            Action::DoEdit(input) => {
                // 恢复到 Normal 模式
                self.mode = Mode::Normal;
                if input.trim().is_empty() {
                    return;
                }
                if let Some(idx) = self.selected() {
                    if idx >= self.hosts.len() {
                        return;
                    }
                    // 解析 user@hostname:port 格式
                    let input = input.trim().to_string();
                    let (user, rest) = if let Some(pos) = input.find('@') {
                        (Some(input[..pos].to_string()), input[pos + 1..].to_string())
                    } else {
                        (None, input.clone())
                    };
                    let (hostname, port) = if let Some(pos) = rest.rfind(':') {
                        let p: u16 = rest[pos + 1..].parse().unwrap_or(22);
                        (rest[..pos].to_string(), p)
                    } else {
                        (rest, 22)
                    };
                    // 克隆别名，避免借用冲突
                    let alias = self.hosts[idx].alias.clone();
                    // 更新选中的 HostBlock
                    let host = &mut self.hosts[idx];
                    // 重建 directives
                    let mut new_directives = Vec::new();
                    new_directives.push(crate::config::types::SshDirective::HostName(hostname));
                    if let Some(u) = user {
                        new_directives.push(crate::config::types::SshDirective::User(u));
                    }
                    if port != 22 {
                        new_directives.push(crate::config::types::SshDirective::Port(port));
                    }
                    // 保留原有的 IdentityFile
                    if let Some(key_path) = host.identity_file() {
                        new_directives.push(crate::config::types::SshDirective::IdentityFile(
                            key_path.clone(),
                        ));
                    }
                    host.directives = new_directives;
                    // 保存到 SSH 配置文件
                    if let Err(e) = self.save_config() {
                        self.flash_message = Some((format!("❌ 保存失败: {}", e), "red".into()));
                    } else {
                        self.flash_message =
                            Some((format!("✏️ 已更新主机 \"{}\"", alias), "green".into()));
                    }
                }
            }
            Action::DoAdd(_input) => {
                // DoAdd 在 Add 模式下按 Enter 触发，由 keymap 构造
                // 这里留空，实际添加由 cmd::add 处理
                self.mode = Mode::Normal;
            }
            Action::Ping => {}
            Action::PingAll => {}
            Action::ToggleSelect => {
                if let Some(idx) = self.selected() {
                    if self.marked.contains(&idx) {
                        self.marked.retain(|&i| i != idx);
                    } else {
                        self.marked.push(idx);
                    }
                }
            }
            _ => {}
        }
    }
}
