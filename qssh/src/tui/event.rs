use std::time::Duration;

use anyhow::Result;
use crossterm::event::{Event as CrosstermEvent, KeyEventKind};
use ratatui::DefaultTerminal;

use super::action::Action;
use super::keymap::map_key_to_action;
use super::ui::render;
use crate::config::types;
use crate::tui::app::App;

// ── TUI 主入口 ───────────────────────────────────────────

/// 启动 TUI 界面
pub fn start() -> Result<()> {
    // 加载 SSH 配置
    let config_path = types::default_config_path();
    types::ensure_config(&config_path)?;
    let config = crate::config::parser::parse_config(&config_path)?;

    // 创建终端
    let terminal = ratatui::try_init()?;

    // 创建应用状态
    let mut app = App::new(config.hosts, config_path);

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
