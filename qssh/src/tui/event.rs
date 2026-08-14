use std::io::Write;
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{
    EnableMouseCapture, Event as CrosstermEvent, KeyCode, KeyEventKind, KeyModifiers,
};
use crossterm::execute;
use ratatui::DefaultTerminal;

use super::action::{Action, Mode};
use super::keymap::map_key_to_action;
use super::mouse::map_mouse_to_action;
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
    execute!(std::io::stdout(), EnableMouseCapture)?;

    // 创建应用状态
    let mut app = App::new(config, config_path);

    // 进入事件循环
    let result = run_event_loop(terminal, &mut app);

    // 恢复终端
    let _ = execute!(std::io::stdout(), crossterm::event::DisableMouseCapture);
    ratatui::try_restore()?;

    // 最终保障：确保光标可见（防止备用屏幕切换后主屏幕光标状态丢失）
    let _ = std::io::stdout().write_all(b"\x1b[?25h");
    let _ = std::io::stdout().flush();

    result
}

/// TUI 事件循环：Event → Action → State → Render
fn run_event_loop(mut terminal: DefaultTerminal, app: &mut App) -> Result<()> {
    let tick_rate = Duration::from_millis(100);

    while app.running {
        app.poll_background_tasks();
        app.expire_flash_message();

        // Render
        terminal.draw(|frame| render(frame, app))?;

        // 等待事件（带超时）
        let has_event = crossterm::event::poll(tick_rate)?;

        if has_event {
            let event = crossterm::event::read()?;
            // 鼠标事件：嵌入式终端模式直接忽略（无滚动缓冲），其余模式映射为 Action
            if let CrosstermEvent::Mouse(mouse) = event {
                if app.mode != Mode::Terminal {
                    let action = map_mouse_to_action(mouse, app);
                    app.apply(action);
                }
                continue;
            }

            if let CrosstermEvent::Key(key) = event {
                // 仅在按下时处理（忽略重复和释放）
                if key.kind == KeyEventKind::Press {
                    // 嵌入式终端模式：键盘直接转发给 PTY
                    if app.mode == Mode::Terminal {
                        // 断开快捷键：Esc / Ctrl+Shift+C
                        let close = key.code == KeyCode::Esc
                            || (key.code == KeyCode::Char('c')
                                && key.modifiers.contains(KeyModifiers::CONTROL)
                                && key.modifiers.contains(KeyModifiers::SHIFT));
                        if close {
                            app.apply(Action::CloseTerminal);
                        } else {
                            app.term_write_key(key);
                        }
                        continue;
                    }

                    if matches!(
                        app.mode,
                        crate::tui::action::Mode::Add
                            | crate::tui::action::Mode::Edit
                            | crate::tui::action::Mode::AgentConfig
                    ) {
                        app.handle_form_key(key);
                        continue;
                    }

                    let action = map_key_to_action(key, app);

                    // 先应用状态变更
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
