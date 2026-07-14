use std::io::Write;
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{Event as CrosstermEvent, KeyEventKind};
use ratatui::DefaultTerminal;

use super::action::Action;
use super::keymap::map_key_to_action;
use super::ui::render;
use crate::config::types;
use crate::ssh::session::SshTarget;
use crate::ssh::spawn::start_interactive_session;
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
    let mut app = App::new(config, config_path);

    // 进入事件循环
    let result = run_event_loop(terminal, &mut app);

    // 恢复终端
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
            if let CrosstermEvent::Key(key) = crossterm::event::read()? {
                // 仅在按下时处理（忽略重复和释放）
                if key.kind == KeyEventKind::Press {
                    if matches!(
                        app.mode,
                        crate::tui::action::Mode::Add | crate::tui::action::Mode::Edit
                    ) {
                        app.handle_form_key(key);
                        continue;
                    }

                    let action = map_key_to_action(key, app);

                    // 先应用状态变更
                    app.apply(action.clone());

                    // Connect 需要终端控制，在事件循环中处理
                    if let Action::Connect = action {
                        if let Some(idx) = app.selected() {
                            if let Some(host) = app.hosts.get(idx) {
                                let target = SshTarget::from_host(host);
                                // 恢复终端，让 SSH 接管
                                ratatui::try_restore()?;
                                // 离开备用屏幕后，先确保主屏幕光标可见
                                // 某些终端在 LeaveAlternateScreen 后会隐藏光标，
                                // 而 SSH 远程 PTY 初始化不一定发 \x1b[?25h 来显示
                                let _ = std::io::stdout().write_all(b"\x1b[?25h");
                                let _ = std::io::stdout().flush();
                                let _ = start_interactive_session(&target, &[]);
                                // 重新初始化终端（进入备用屏幕）
                                terminal = ratatui::try_init()?;
                            }
                        }
                    }
                }
            }
        } else {
            // Tick 事件（可用于定时刷新等）
            app.apply(Action::None);
        }
    }

    Ok(())
}
