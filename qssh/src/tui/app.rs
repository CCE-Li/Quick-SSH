use std::collections::HashMap;

use ratatui::widgets::ListState;

use crate::config::types::HostBlock;
use crate::tui::action::{Action, Mode};

// ── 应用状态 ─────────────────────────────────────────────

/// TUI 应用状态
pub struct App {
    /// 主机列表
    pub hosts: Vec<HostBlock>,
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
    pub fn new(hosts: Vec<HostBlock>) -> Self {
        let mut list_state = ListState::default();
        if !hosts.is_empty() {
            list_state.select(Some(0));
        }
        Self {
            hosts,
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
            Action::ConfirmDelete(confirmed) => {
                self.mode = Mode::Normal;
                if confirmed {
                    self.flash_message = Some(("已删除".into(), "green".into()));
                }
            }
            Action::Ping => {}
            Action::PingAll => {}
            _ => {}
        }
    }
}
