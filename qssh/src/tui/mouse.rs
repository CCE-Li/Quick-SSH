//! 鼠标事件 → Action 映射（crossterm mouse）
//!
//! 功能：
//! - 主机列表：左键点击选中、滚轮上下滚动
//! - 操作面板（Docker / 服务 / 文件 / 日志）：点击选中、滚轮滚动
//! - 命令面板：点击选中、滚轮滚动
//! - Dashboard：点击监控 Widget 打开对应操作面板
//!
//! 坐标系：crossterm 0.28 的 `column` / `row` 与 ratatui 一致，均为 0 起始。

use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::{Margin, Position, Rect};

use super::action::{Action, Mode, View};
use super::app::App;
use crate::tui::dashboard::filter_palette;
use crate::tui::dashboard::WidgetId;
use crate::tui::widgets::centered_rect;

/// 操作面板弹窗尺寸（与 ui.rs 渲染保持一致）
const OPS_POPUP_W: u16 = 70;
const OPS_POPUP_H: u16 = 60;
const LOG_POPUP_W: u16 = 80;
const LOG_POPUP_H: u16 = 70;
const PALETTE_POPUP_W: u16 = 60;
const PALETTE_POPUP_H: u16 = 60;

/// 将鼠标事件映射为 Action（无对应操作时返回 `Action::None`）
pub fn map_mouse_to_action(event: MouseEvent, app: &App) -> Action {
    match event.kind {
        MouseEventKind::Down(MouseButton::Left) => left_click(app, event.column, event.row),
        MouseEventKind::ScrollUp => scroll(app, -1),
        MouseEventKind::ScrollDown => scroll(app, 1),
        _ => Action::None,
    }
}

/// 左键单击
fn left_click(app: &App, col: u16, row: u16) -> Action {
    match app.mode {
        Mode::Normal if app.view == View::HostList => {
            host_index_at(app, col, row).map_or(Action::None, Action::SelectListItem)
        }
        Mode::Normal => widget_click(app, col, row),
        Mode::DockerOps => panel_index_at(row, col, OPS_POPUP_W, OPS_POPUP_H, app.docker_count())
            .map_or(Action::None, Action::SelectListItem),
        Mode::ServiceOps => panel_index_at(row, col, OPS_POPUP_W, OPS_POPUP_H, app.service_count())
            .map_or(Action::None, Action::SelectListItem),
        Mode::FileOps => panel_index_at(row, col, OPS_POPUP_W, OPS_POPUP_H, app.file_count())
            .map_or(Action::None, Action::SelectListItem),
        Mode::LogOps => {
            panel_index_at(row, col, LOG_POPUP_W, LOG_POPUP_H, app.log_selected_count())
                .map_or(Action::None, Action::SelectListItem)
        }
        Mode::Palette => {
            palette_index_at(app, col, row).map_or(Action::None, Action::SelectListItem)
        }
        _ => Action::None,
    }
}

/// 滚轮滚动：按当前模式导航对应列表
fn scroll(app: &App, delta: isize) -> Action {
    let up = delta < 0;
    match app.mode {
        Mode::Normal if app.view == View::HostList => {
            if up {
                Action::MoveUp
            } else {
                Action::MoveDown
            }
        }
        Mode::DockerOps => panel_scroll(Action::DockerOpsMove(-1), Action::DockerOpsMove(1), up),
        Mode::ServiceOps => panel_scroll(Action::ServiceOpsMove(-1), Action::ServiceOpsMove(1), up),
        Mode::FileOps => panel_scroll(Action::FileOpsMove(-1), Action::FileOpsMove(1), up),
        Mode::LogOps => panel_scroll(Action::LogOpsMove(-1), Action::LogOpsMove(1), up),
        Mode::Palette => panel_scroll(Action::PaletteMove(-1), Action::PaletteMove(1), up),
        _ => Action::None,
    }
}

fn panel_scroll(up: Action, down: Action, go_up: bool) -> Action {
    if go_up {
        up
    } else {
        down
    }
}

/// 主机列表命中索引：主体左半区，边框下方每行一项
fn host_index_at(app: &App, col: u16, row: u16) -> Option<usize> {
    let area = full_area();
    let list_width = (area.width / 2).max(1);
    if col >= list_width || row <= 1 {
        return None;
    }
    let idx = (row - 2) as usize;
    (idx < app.hosts.len()).then_some(idx)
}

/// 操作面板列表命中索引：弹窗边框内每行一项
fn panel_index_at(row: u16, col: u16, w: u16, h: u16, count: usize) -> Option<usize> {
    let popup = centered_rect(w, h, full_area());
    let inner = popup.inner(Margin::new(1, 1));
    if !inner.contains(Position::new(col, row)) {
        return None;
    }
    let idx = (row - inner.y) as usize;
    (idx < count).then_some(idx)
}

/// 命令面板列表命中索引（需考虑顶部 Query 输入区 + 过滤后的可见条目）
fn palette_index_at(app: &App, col: u16, row: u16) -> Option<usize> {
    let popup = centered_rect(PALETTE_POPUP_W, PALETTE_POPUP_H, full_area());
    let inner = popup.inner(Margin::new(1, 1));
    // inner 纵向切分：Length(3) 查询输入 + Min(0) 动作列表（列表边框内每行一项）
    let list_top = inner.y + 3;
    if row <= list_top || !inner.contains(Position::new(col, row)) {
        return None;
    }
    let idx = (row - list_top - 1) as usize;
    let visible = filter_palette(crate::tui::dashboard::PALETTE_ACTIONS, &app.palette_query).len();
    (idx < visible).then_some(idx)
}

/// Dashboard 上点击 Widget → 打开对应操作面板
fn widget_click(app: &App, col: u16, row: u16) -> Action {
    let Some(profile) = app
        .dashboard_config
        .profiles
        .get(&app.dashboard_config.active_profile)
    else {
        return Action::None;
    };
    let pos = Position::new(col, row);
    for (id, rect) in crate::tui::dashboard::compute_layout(&profile.layout, full_area()) {
        if !rect.contains(pos) {
            continue;
        }
        return match id {
            WidgetId::Docker => Action::OpenDockerOps,
            WidgetId::Services => Action::OpenServiceOps,
            WidgetId::Files => Action::OpenFileOps,
            WidgetId::Logs => Action::OpenLogOps,
            WidgetId::Agent => Action::OpenAgentOps,
            _ => Action::None,
        };
    }
    Action::None
}

/// 当前终端区域（以 0,0 为原点）
fn full_area() -> Rect {
    let (w, h) = crossterm::terminal::size().unwrap_or((0, 0));
    Rect::new(0, 0, w, h)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crossterm::event::{MouseEvent, MouseEventKind};

    use super::*;
    use crate::config::types::SshConfig;
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
    fn mouse_move_is_ignored() {
        let action = map_mouse_to_action(
            MouseEvent {
                kind: MouseEventKind::Moved,
                column: 10,
                row: 10,
                modifiers: crossterm::event::KeyModifiers::NONE,
            },
            &app(),
        );
        assert!(matches!(action, Action::None));
    }

    #[test]
    fn scroll_in_palette_moves_down() {
        let mut a = app();
        a.mode = Mode::Palette;
        let action = map_mouse_to_action(
            MouseEvent {
                kind: MouseEventKind::ScrollDown,
                column: 0,
                row: 0,
                modifiers: crossterm::event::KeyModifiers::NONE,
            },
            &a,
        );
        assert!(matches!(action, Action::PaletteMove(1)));
    }
}
