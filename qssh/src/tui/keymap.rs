use crossterm::event::{KeyEvent, KeyModifiers};

use super::action::{Action, Mode};
use crate::tui::app::App;

/// 键盘事件 → Action 映射
pub fn map_key_to_action(key: KeyEvent, app: &App) -> Action {
    use crossterm::event::KeyCode;

    match app.mode {
        Mode::Normal => {
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                match key.code {
                    KeyCode::Char('p' | 'P') => return Action::MoveUp,
                    KeyCode::Char('n' | 'N') => return Action::MoveDown,
                    _ => {}
                }
            }

            match key.code {
                KeyCode::Char('j') | KeyCode::Down => Action::MoveDown,
                KeyCode::Char('k') | KeyCode::Up => Action::MoveUp,
                KeyCode::Char('g') => Action::MoveTop,
                KeyCode::Char('G') => Action::MoveBottom,
                KeyCode::Enter => Action::Connect,
                KeyCode::Char(' ') => Action::ToggleSelect,
                KeyCode::Char('d') => Action::Delete,
                KeyCode::Char('e') => Action::StartEdit,
                KeyCode::Char('a') => Action::StartAdd,
                KeyCode::Char('p') => Action::Ping,
                KeyCode::Char('P') => Action::PingAll,
                KeyCode::Char('/') => Action::StartSearch,
                KeyCode::Char('.') => Action::ToggleAddress,
                KeyCode::Char('q') | KeyCode::Esc => Action::Quit,
                KeyCode::Char('?') => Action::ShowHelp,
                _ => Action::None,
            }
        }
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
        Mode::Add => Action::None,
        Mode::Edit => Action::None,
        Mode::Confirm => match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => Action::ConfirmDelete(true),
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => Action::ConfirmDelete(false),
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
            Mode::Edit => " EDIT ",
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
            Mode::Normal => {
                "j↓ k↑ Ctrl+N↓ Ctrl+P↑ gg↕ G↕ /搜索 a添加 e编辑 d删除 p检测 P全检 Enter连接 空格标记 .地址 q退出 ?帮助"
            }
            Mode::Search => "输入搜索关键词，Enter 确认，Esc 取消",
            Mode::Add => "字段添加弹窗: Tab 切换字段，Enter 下一项，Ctrl+S 保存，Esc 取消",
            Mode::Edit => "字段编辑弹窗: Tab 切换字段，Enter 下一项，Ctrl+S 保存，Esc 取消",
            Mode::Rename => "输入新别名，Enter 确认，Esc 取消",
            Mode::Export => "输入导出文件路径，Enter 确认，Esc 取消",
            Mode::Import => "输入导入文件路径，Enter 确认，Esc 取消",
            Mode::Confirm => "确认删除？y/Y 确认，n/N/Esc 取消",
            Mode::Help => "按 q/Esc 关闭帮助",
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    use super::map_key_to_action;
    use crate::config::types::SshConfig;
    use crate::tui::action::Action;
    use crate::tui::app::App;

    fn app() -> App {
        App::new(
            SshConfig {
                hosts: vec![],
                preamble: String::new(),
            },
            PathBuf::from("dummy"),
        )
    }

    #[test]
    fn ctrl_p_moves_up_instead_of_ping() {
        let action = map_key_to_action(
            KeyEvent::new(KeyCode::Char('p'), KeyModifiers::CONTROL),
            &app(),
        );

        assert!(matches!(action, Action::MoveUp));
    }

    #[test]
    fn ctrl_n_moves_down() {
        let action = map_key_to_action(
            KeyEvent::new(KeyCode::Char('n'), KeyModifiers::CONTROL),
            &app(),
        );

        assert!(matches!(action, Action::MoveDown));
    }

    #[test]
    fn plain_p_still_pings() {
        let action = map_key_to_action(
            KeyEvent::new(KeyCode::Char('p'), KeyModifiers::NONE),
            &app(),
        );

        assert!(matches!(action, Action::Ping));
    }
}
