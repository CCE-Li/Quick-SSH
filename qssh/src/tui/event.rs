use std::io::Write;
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{
    DisableMouseCapture, EnableMouseCapture, Event as CrosstermEvent, KeyCode, KeyEventKind,
    KeyModifiers,
};
use crossterm::execute;
use ratatui::DefaultTerminal;

use super::action::{Action, Mode, View};
use super::keymap::{map_key_to_action, map_terminal_prefix_key};
use super::mouse::map_mouse_to_action;
use super::ui::render;
use crate::config::types;
use crate::ssh::session::SshTarget;
use crate::ssh::spawn::start_interactive_session;
use crate::tui::app::App;

// ── TUI 主入口 ───────────────────────────────────────────

/// 安全轮询终端事件：捕获 crossterm_winapi 对无效 `INPUT_RECORD` 的 panic。
///
/// Windows 终端偶发写入 `EventType=0` 的异常控制台记录，上游 crossterm_winapi
/// 在 `From<INPUT_RECORD>` 中直接 `panic!`（见 crossterm-winapi#25）。这里用
/// `catch_unwind` 拦截：坏记录已被读取消费，后续轮询不受影响。返回 `false`
/// 表示无事件（含发生 panic 被忽略的情况），真实的 I/O 错误照常上抛。
fn poll_event_safely(timeout: Duration) -> Result<bool> {
    match std::panic::catch_unwind(|| crossterm::event::poll(timeout)) {
        Ok(Ok(has_event)) => Ok(has_event),
        Ok(Err(e)) => Err(e.into()),
        Err(_) => Ok(false),
    }
}

/// 安全读取单个终端事件（配合 `poll_event_safely` 使用）。
///
/// 捕获解析无效记录时的 panic，此时返回 `Ok(None)` 跳过该事件；
/// 真实的 I/O 错误照常上抛。
fn read_event_safely() -> Result<Option<CrosstermEvent>> {
    match std::panic::catch_unwind(crossterm::event::read) {
        Ok(Ok(event)) => Ok(Some(event)),
        Ok(Err(e)) => Err(e.into()),
        Err(_) => Ok(None),
    }
}

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

        // 等待事件（带超时）。捕获 crossterm_winapi 对异常 INPUT_RECORD 的 panic
        // （Windows 偶发 EventType=0 的无效控制台记录，见 crossterm-winapi#25），
        // 避免单个无效记录导致整个 TUI 崩溃；坏记录已被消费，下一轮继续正常读取。
        let has_event = poll_event_safely(tick_rate)?;
        if !has_event {
            continue;
        }
        // read_event_safely 在解析到无效记录时返回 Ok(None)，跳过该事件继续循环；
        // 真实的 I/O 错误照常上抛。
        let Some(event) = read_event_safely()? else {
            continue;
        };
        // 鼠标事件：全部映射为 Action（含终端/Agent 模式下的标签页式聚焦切换）。
        // 终端面板本身不转发鼠标给 PTY（无滚动缓冲），点击终端/Agent 只是切换焦点。
        if let CrosstermEvent::Mouse(mouse) = event {
            // 鼠标点击会取消待命中的 Ctrl+B 前缀（避免误把后续按键解释为 TUI 命令）
            if app.term_prefix_active() {
                app.set_term_prefix(false);
            }
            let action = map_mouse_to_action(mouse, app);
            app.apply(action);
            continue;
        }

        if let CrosstermEvent::Key(key) = event {
            // 仅在按下时处理（忽略重复和释放）
            if key.kind == KeyEventKind::Press {
                // 分隔线拖动期间按键盘：取消拖动
                if app.split_drag.is_some() || app.main_split_drag.is_some() {
                    app.cancel_split_drag();
                }
                // 嵌入式终端模式：键盘直接转发给 PTY
                if app.mode == Mode::Terminal {
                    // Ctrl+B 前缀等待状态：按键映射为 TUI 命令，而非转发给 PTY
                    if app.term_prefix_active() {
                        let action = map_terminal_prefix_key(key);
                        app.set_term_prefix(false);
                        if let Some(action) = action {
                            app.apply(action);
                        } else {
                            app.term_write_key(key);
                        }
                        continue;
                    }

                    // 前缀键：Ctrl+B 进入命令等待状态（不转发给 PTY）
                    let is_prefix = key.code == KeyCode::Char('b')
                        && key.modifiers.contains(KeyModifiers::CONTROL)
                        && !key.modifiers.contains(KeyModifiers::ALT)
                        && !key.modifiers.contains(KeyModifiers::SHIFT);
                    if is_prefix {
                        app.set_term_prefix(true);
                        continue;
                    }

                    // 断开快捷键：仅 Ctrl+Shift+C。
                    // Esc 必须放行给远端（vim 等程序依赖 Esc 退出插入模式），
                    // 因此不作为断开键；断开走 Ctrl+Shift+C 或 Ctrl+B 前缀的 q/x。
                    let close = key.code == KeyCode::Char('c')
                        && key.modifiers.contains(KeyModifiers::CONTROL)
                        && key.modifiers.contains(KeyModifiers::SHIFT);
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
                app.apply(action.clone());

                // 主机列表主界面 Enter 连接：退出 TUI 让 SSH 全屏接管（添加 AI 之前的界面），
                // 会话结束后重新初始化终端回到 TUI。Dashboard 控制台的连接由 apply 内的
                // `start_terminal_session`（嵌入式终端）处理。
                if matches!(action, Action::Connect) && app.view == View::HostList {
                    if let Some(idx) = app.selected() {
                        if let Some(host) = app.hosts.get(idx) {
                            let target = SshTarget::from_host(host);
                            // 恢复终端，让 SSH 接管
                            ratatui::try_restore()?;
                            // 关键：必须先关闭鼠标捕获，再进入 SSH 会话。
                            // 否则 Windows 控制台残留的鼠标上报（EnableMouseCapture 在
                            // start() 中开启、try_restore() 不会自动关闭）会作为字节流
                            // 进入 SSH stdin 并被转发到远端，shell 提示符处出现
                            // `35;99;25M…` 之类的鼠标坐标垃圾。
                            let _ = execute!(std::io::stdout(), DisableMouseCapture);
                            // 确保光标可见（防止备用屏幕切换后主屏幕光标状态丢失）
                            let _ = std::io::stdout().write_all(b"\x1b[?25h");
                            let _ = std::io::stdout().flush();
                            let _ = start_interactive_session(&target, &[], true);
                            // SSH 退出后，重新初始化终端
                            terminal = ratatui::try_init()?;
                            execute!(std::io::stdout(), EnableMouseCapture)?;
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
