use crossterm::event::KeyEvent;

use super::action::{Action, Mode};
use crate::tui::app::App;

/// 键盘事件 → Action 映射
pub fn map_key_to_action(key: KeyEvent, app: &App) -> Action {
    use crossterm::event::KeyCode;

    match app.mode {
        Mode::Normal => match key.code {
            KeyCode::Char('j') | KeyCode::Down => Action::MoveDown,
            KeyCode::Char('k') | KeyCode::Up => Action::MoveUp,
            KeyCode::Char('g') => Action::MoveTop,
            KeyCode::Char('G') => Action::MoveBottom,
            KeyCode::Enter => Action::Connect,
            KeyCode::Char(' ') => Action::ToggleSelect,
            KeyCode::Char('d') => Action::Delete,
            KeyCode::Char('a') => Action::StartAdd,
            KeyCode::Char('p') => Action::Ping,
            KeyCode::Char('P') => Action::PingAll,
            KeyCode::Char('/') => Action::StartSearch,
            KeyCode::Char('q') | KeyCode::Esc => Action::Quit,
            KeyCode::Char('?') => Action::ShowHelp,
            _ => Action::None,
        },
        Mode::Search => match key.code {
            KeyCode::Esc => Action::CancelSearch,
            KeyCode::Enter => Action::SearchSubmit,
            KeyCode::Backspace => {
                let mut s = app.input_buffer.clone();
                s.pop();
                Action::SearchInput(s)
            }
            KeyCode::Char(c) => {
                let mut s = app.input_buffer.clone();
                s.push(c);
                Action::SearchInput(s)
            }
            _ => Action::None,
        },
        Mode::Add => match key.code {
            KeyCode::Esc => Action::HideHelp,
            KeyCode::Enter => Action::DoAdd(app.input_buffer.clone()),
            KeyCode::Char(c) => Action::DoAdd(c.to_string()),
            KeyCode::Backspace => {
                let mut s = app.input_buffer.clone();
                s.pop();
                Action::DoAdd(s)
            }
            _ => Action::None,
        },
        Mode::Confirm => match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => Action::ConfirmDelete(true),
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                Action::ConfirmDelete(false)
            }
            _ => Action::None,
        },
        Mode::Help => match key.code {
            KeyCode::Char('q') | KeyCode::Esc => Action::HideHelp,
            _ => Action::None,
        },
        _ => Action::None,
    }
}

// ── Mode 的 UI 方法 ─────────────────────────────────────

impl Mode {
    /// 模式对应的状态栏标签
    pub fn label(&self) -> &str {
        match self {
            Mode::Normal => " NORMAL ",
            Mode::Search => " SEARCH ",
            Mode::Add => " ADD ",
            Mode::Rename => " RENAME ",
            Mode::Export => " EXPORT ",
            Mode::Import => " IMPORT ",
            Mode::Confirm => " CONFIRM ",
            Mode::Help => " HELP ",
        }
    }

    /// 模式对应的提示信息
    pub fn hint(&self) -> &str {
        match self {
            Mode::Normal => "j↓ k↑ gg↕ G↕ /搜索 a添加 d删除 p检测 P全检 Enter连接 q退出 ?帮助",
            Mode::Search => "输入搜索关键词，Enter 确认，Esc 取消",
            Mode::Add => "格式: user@hostname[:port] 可选 --key <path>",
            Mode::Rename => "输入新别名，Enter 确认，Esc 取消",
            Mode::Export => "输入导出文件路径，Enter 确认，Esc 取消",
            Mode::Import => "输入导入文件路径，Enter 确认，Esc 取消",
            Mode::Confirm => "确认删除？y/Y 确认，n/N/Esc 取消",
            Mode::Help => "按 q/Esc 关闭帮助",
        }
    }
}
