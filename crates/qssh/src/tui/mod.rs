mod app;
mod event;
mod modes;
mod ui;
mod widgets;

use std::time::Duration;

use anyhow::Result;
use crossterm::event::{Event as CrosstermEvent, KeyEventKind};
use ratatui::DefaultTerminal;

use crate::config::ssh_config;
use crate::tui::app::App;
use crate::tui::event::Action;
use crate::tui::ui::render;

// ── TUI 主入口 ───────────────────────────────────────────

/// 启动 TUI 界面
pub fn start() -> Result<()> {
    // 加载 SSH 配置
    let config_path = ssh_config::default_config_path();
    ssh_config::ensure_config(&config_path)?;
    let config = ssh_config::parse_config(&config_path)?;

    // 创建终端
    let terminal = ratatui::try_init()?;

    // 创建应用状态
    let mut app = App::new(config.hosts);

    // 进入事件循环
    let result = run_event_loop(terminal, &mut app);

    // 恢复终端
    ratatui::try_restore()?;

    result
}

/// TUI 事件循环：Event → Action → State → Render
fn run_event_loop(mut terminal: DefaultTerminal, app: &mut App) -> Result<()> {
    let tick_rate = Duration::from_millis(100);

    while app.running {
        // Render
        terminal.draw(|frame| render(frame, app))?;

        // 等待事件（带超时）
        let has_event = crossterm::event::poll(tick_rate)?;

        if has_event {
            if let CrosstermEvent::Key(key) = crossterm::event::read()? {
                // 仅在按下时处理（忽略重复和释放）
                if key.kind == KeyEventKind::Press {
                    let action = map_key_to_action(key, app);
                    app.apply(action);
                }
            }
        } else {
            // Tick 事件（可用于定时刷新等）
            app.apply(Action::None);
        }
    }

    Ok(())
}

/// 键盘事件 → Action 映射
fn map_key_to_action(key: crossterm::event::KeyEvent, app: &App) -> Action {
    use crossterm::event::KeyCode;
    use crate::tui::modes::Mode;

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
