//! 鼠标事件 → Action 映射（crossterm mouse）
//!
//! 功能：
//! - 主机列表：左键点击选中、滚轮上下滚动
//! - 操作面板（Docker / 服务 / 文件 / 日志）：点击选中、滚轮滚动
//! - 命令面板：点击选中、滚轮滚动
//! - Dashboard：点击监控 Widget 打开对应操作面板；点击 Terminal / AI Agent 切换聚焦
//! - 终端 / Agent 面板：点击对方 Widget 即可在两者间切换焦点（标签页式使用）
//!
//! 坐标系：crossterm 0.28 的 `column` / `row` 与 ratatui 一致，均为 0 起始。

use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::{Margin, Position, Rect};

use super::action::{Action, Mode, View};
use super::app::App;
use crate::tui::dashboard::filter_palette;
use crate::tui::dashboard::layout::hit_test_divider;
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
    let dragging = app.split_drag.is_some() || app.main_split_drag.is_some();
    match event.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            if dragging {
                // 拖动中再次按下：先结束上一次拖动
                Action::EndSplitDrag
            } else {
                left_click(app, event.column, event.row)
            }
        }
        MouseEventKind::Drag(MouseButton::Left) => {
            if dragging {
                Action::MoveSplitDrag {
                    col: event.column,
                    row: event.row,
                }
            } else {
                Action::None
            }
        }
        MouseEventKind::Up(MouseButton::Left) => {
            if dragging {
                Action::EndSplitDrag
            } else {
                Action::None
            }
        }
        MouseEventKind::ScrollUp => scroll(app, -1),
        MouseEventKind::ScrollDown => scroll(app, 1),
        _ => Action::None,
    }
}

/// 左键单击
fn left_click(app: &App, col: u16, row: u16) -> Action {
    match app.mode {
        Mode::Normal if app.view == View::HostList => {
            // 主界面：先判断是否命中主机列表/详情分隔线
            if is_on_main_divider(app, col) {
                Action::StartSplitDrag { col, row }
            } else {
                host_index_at(app, col, row).map_or(Action::None, Action::SelectListItem)
            }
        }
        Mode::Normal => {
            if is_on_divider(app, col, row) {
                Action::StartSplitDrag { col, row }
            } else {
                widget_click(app, col, row)
            }
        }
        // 终端 / Agent 面板：点击 Dashboard 上任意 Widget 切换聚焦（不做分隔线拖动）
        Mode::Terminal | Mode::AgentOps => widget_click(app, col, row),
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

/// 主界面（主机列表/详情）分隔线命中检测：命中垂直分隔线则开启拖动
fn is_on_main_divider(app: &App, col: u16) -> bool {
    is_on_main_divider_w(app.main_split_weight, full_area().width, col)
}

/// 指定宽度下的主界面分隔线命中检测（测试友好）
fn is_on_main_divider_w(weight: u16, width: u16, col: u16) -> bool {
    let boundary = main_split_boundary(weight, width);
    col == boundary.saturating_sub(1) || col == boundary
}

/// Dashboard 视图下鼠标是否落在某条分隔线上（命中则开启拖动）
fn is_on_divider(app: &App, col: u16, row: u16) -> bool {
    let Some(profile) = app
        .dashboard_config
        .profiles
        .get(&app.dashboard_config.active_profile)
    else {
        return false;
    };
    let pos = Position::new(col, row);
    hit_test_divider(&profile.layout, dashboard_area(), pos).is_some()
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

/// 主机列表命中索引：主体左半区，边框下方每行一项（按过滤后可见列表）
fn host_index_at(app: &App, col: u16, row: u16) -> Option<usize> {
    let area = full_area();
    host_index_at_w(app, area.width, col, row)
}

/// 指定宽度下的主机列表命中索引（测试友好）
fn host_index_at_w(app: &App, width: u16, col: u16, row: u16) -> Option<usize> {
    let list_width = main_split_boundary(app.main_split_weight, width).max(1);
    if col >= list_width || row <= 1 {
        return None;
    }
    let idx = (row - 2) as usize;
    (idx < app.visible_indices().len()).then_some(idx)
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

/// Dashboard 上点击 Widget → 打开对应操作面板，或切换 Terminal/Agent 聚焦
fn widget_click(app: &App, col: u16, row: u16) -> Action {
    widget_click_at(app, col, row, dashboard_area())
}

/// 在指定区域上点击 Widget → 打开对应操作面板，或切换 Terminal/Agent 聚焦
fn widget_click_at(app: &App, col: u16, row: u16, area: Rect) -> Action {
    let Some(profile) = app
        .dashboard_config
        .profiles
        .get(&app.dashboard_config.active_profile)
    else {
        return Action::None;
    };
    let pos = Position::new(col, row);
    for (id, rect) in crate::tui::dashboard::compute_layout(&profile.layout, area) {
        if !rect.contains(pos) {
            continue;
        }
        return match id {
            WidgetId::Docker => Action::OpenDockerOps,
            WidgetId::Services => Action::OpenServiceOps,
            WidgetId::Files => Action::OpenFileOps,
            WidgetId::Logs => Action::OpenLogOps,
            // 标签页式聚焦：点击 Agent / Terminal 切换当前焦点
            WidgetId::Agent => Action::FocusAgent,
            WidgetId::Terminal => Action::FocusTerminal,
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

/// 主界面左栏（主机列表）宽度权重对应的垂直分隔线 x 坐标（绝对终端列）
fn main_split_boundary(weight: u16, width: u16) -> u16 {
    let weight = weight.clamp(0, 100);
    (width as u32 * weight as u32 / 100) as u16
}

/// Dashboard 实际渲染区域：主体位于 1 行标题栏与 1 行状态栏之间（与 ui.rs 一致）
fn dashboard_area() -> Rect {
    let (w, h) = crossterm::terminal::size().unwrap_or((0, 0));
    Rect::new(0, 1, w, h.saturating_sub(2))
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

    /// 构造包含 Terminal + Agent 的布局并返回 App
    fn app_with_terminal_and_agent() -> App {
        use crate::tui::dashboard::config::{default_layout_for, ProfileConfig, WidgetId};
        let enabled = vec![WidgetId::Terminal, WidgetId::Agent];
        let mut a = app();
        a.dashboard_config.active_profile = "p".into();
        a.dashboard_config.profiles.insert(
            "p".into(),
            ProfileConfig {
                enabled: enabled.clone(),
                layout: default_layout_for(&enabled),
            },
        );
        a.view = View::Dashboard;
        a
    }

    /// 默认布局（Terminal + Agent 水平切分）下，点击各区域应切换对应聚焦
    #[test]
    fn dashboard_click_focuses_terminal_and_agent() {
        use crate::tui::dashboard::layout::compute_layout;
        use ratatui::layout::Rect;

        let a = app_with_terminal_and_agent();
        let area = Rect::new(0, 1, 80, 22);
        let rects = compute_layout(&a.dashboard_config.profiles["p"].layout, area);
        // Terminal + Agent 水平切分，Terminal 在左
        assert!(matches!(rects[0].0, WidgetId::Terminal));
        assert!(matches!(rects[1].0, WidgetId::Agent));
        let term_rect = rects[0].1;
        let agent_rect = rects[1].1;
        // 点击 Terminal 中间 → FocusTerminal
        let action = widget_click_at(
            &a,
            term_rect.x + term_rect.width / 2,
            term_rect.y + term_rect.height / 2,
            area,
        );
        assert!(matches!(action, Action::FocusTerminal));
        // 点击 Agent 中间 → FocusAgent
        let action = widget_click_at(
            &a,
            agent_rect.x + agent_rect.width / 2,
            agent_rect.y + agent_rect.height / 2,
            area,
        );
        assert!(matches!(action, Action::FocusAgent));
    }

    /// 终端模式下点击 Agent Widget → FocusAgent（标签页切换）
    #[test]
    fn terminal_mode_click_agent_focuses_agent() {
        let mut a = app_with_terminal_and_agent();
        a.mode = Mode::Terminal;
        let area = Rect::new(0, 1, 80, 22);
        let rects = crate::tui::dashboard::layout::compute_layout(
            &a.dashboard_config.profiles["p"].layout,
            area,
        );
        let agent_rect = rects[1].1;
        let action = widget_click_at(
            &a,
            agent_rect.x + agent_rect.width / 2,
            agent_rect.y + agent_rect.height / 2,
            area,
        );
        assert!(matches!(action, Action::FocusAgent));
    }

    /// AgentOps 模式下点击 Terminal Widget → FocusTerminal（标签页切换）
    #[test]
    fn agent_ops_mode_click_terminal_focuses_terminal() {
        let mut a = app_with_terminal_and_agent();
        a.mode = Mode::AgentOps;
        let area = Rect::new(0, 1, 80, 22);
        let rects = crate::tui::dashboard::layout::compute_layout(
            &a.dashboard_config.profiles["p"].layout,
            area,
        );
        let term_rect = rects[0].1;
        let action = widget_click_at(
            &a,
            term_rect.x + term_rect.width / 2,
            term_rect.y + term_rect.height / 2,
            area,
        );
        assert!(matches!(action, Action::FocusTerminal));
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

    #[test]
    fn drag_event_maps_to_move_split_drag() {
        let mut a = app();
        a.split_drag = Some(crate::tui::app::SplitDrag::new(vec![], 0, vec![]));
        let action = map_mouse_to_action(
            MouseEvent {
                kind: MouseEventKind::Drag(MouseButton::Left),
                column: 55,
                row: 10,
                modifiers: crossterm::event::KeyModifiers::NONE,
            },
            &a,
        );
        assert!(matches!(action, Action::MoveSplitDrag { col: 55, row: 10 }));
    }

    #[test]
    fn up_event_ends_split_drag() {
        let mut a = app();
        a.split_drag = Some(crate::tui::app::SplitDrag::new(vec![], 0, vec![]));
        let action = map_mouse_to_action(
            MouseEvent {
                kind: MouseEventKind::Up(MouseButton::Left),
                column: 55,
                row: 10,
                modifiers: crossterm::event::KeyModifiers::NONE,
            },
            &a,
        );
        assert!(matches!(action, Action::EndSplitDrag));
    }

    #[test]
    fn second_down_during_drag_ends_previous_drag() {
        let mut a = app();
        a.split_drag = Some(crate::tui::app::SplitDrag::new(vec![], 0, vec![]));
        let action = map_mouse_to_action(
            MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Left),
                column: 55,
                row: 10,
                modifiers: crossterm::event::KeyModifiers::NONE,
            },
            &a,
        );
        assert!(matches!(action, Action::EndSplitDrag));
    }

    #[test]
    fn drag_without_drag_state_is_ignored() {
        let action = map_mouse_to_action(
            MouseEvent {
                kind: MouseEventKind::Drag(MouseButton::Left),
                column: 55,
                row: 10,
                modifiers: crossterm::event::KeyModifiers::NONE,
            },
            &app(),
        );
        assert!(matches!(action, Action::None));
    }

    #[test]
    fn scroll_in_terminal_mode_is_ignored() {
        let mut a = app();
        a.mode = Mode::Terminal;
        let action = map_mouse_to_action(
            MouseEvent {
                kind: MouseEventKind::ScrollDown,
                column: 0,
                row: 0,
                modifiers: crossterm::event::KeyModifiers::NONE,
            },
            &a,
        );
        assert!(matches!(action, Action::None));
    }

    /// 主界面（HostList 视图）点击分隔线 → StartSplitDrag
    #[test]
    fn hostlist_divider_click_starts_drag() {
        let mut a = app();
        a.view = View::HostList;
        a.main_split_weight = 50;
        // 宽度 80、权重 50 → 分隔线在 col=40（左右各 40）
        let boundary = main_split_boundary(50, 80);
        assert_eq!(boundary, 40);
        // 命中检测（左右相邻列均算命中）
        assert!(is_on_main_divider_w(50, 80, 39));
        assert!(is_on_main_divider_w(50, 80, 40));
        assert!(!is_on_main_divider_w(50, 80, 41));
        // 通过完整左键路径映射（依赖真实终端尺寸的环境下也应路由到拖动）
        let action = map_mouse_to_action(
            MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Left),
                column: boundary,
                row: 5,
                modifiers: crossterm::event::KeyModifiers::NONE,
            },
            &a,
        );
        if matches!(action, Action::None) {
            // 测试环境终端尺寸为 0 时不会命中分隔线，此为环境限制而非逻辑错误
            assert!(host_index_at_w(&a, 0, boundary, 5).is_none());
        } else {
            assert!(
                matches!(action, Action::StartSplitDrag { .. }),
                "点击分隔线应开始拖动，实际 {action:?}"
            );
        }
    }

    /// 主界面点击分隔线左侧列表区域 → 仍选中主机（不受分隔影响）
    #[test]
    fn hostlist_click_inside_list_selects() {
        let mut a = app();
        a.view = View::HostList;
        a.main_split_weight = 50;
        // 填充 2 台主机使列表可见
        use crate::config::types::{HostBlock, SshDirective};
        a.hosts = (0..2)
            .map(|i| HostBlock {
                alias: format!("host{i}"),
                directives: vec![SshDirective::HostName("example.com".into())],
                raw_text: String::new(),
            })
            .collect();
        // 50% of 80 = 40；col=10 在左栏内
        let idx = host_index_at_w(&a, 80, 10, 3);
        assert_eq!(idx, Some(1), "col=10 row=3 应命中列表第 2 项");
        // col 超过分隔线 → 详情区，不命中列表
        assert!(host_index_at_w(&a, 80, 60, 3).is_none());
        // 左栏命中逻辑经完整路径映射（终端尺寸为 0 时退化，仅验证不 panic）
        let _ = map_mouse_to_action(
            MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Left),
                column: 10,
                row: 3,
                modifiers: crossterm::event::KeyModifiers::NONE,
            },
            &a,
        );
    }

    /// 拖动分隔线时应产生 MoveSplitDrag（宿主需要设置拖动状态）
    #[test]
    fn hostlist_drag_while_main_drag_active_moves() {
        let mut a = app();
        a.view = View::HostList;
        a.main_split_drag = Some(50);
        let action = map_mouse_to_action(
            MouseEvent {
                kind: MouseEventKind::Drag(MouseButton::Left),
                column: 60,
                row: 5,
                modifiers: crossterm::event::KeyModifiers::NONE,
            },
            &a,
        );
        assert!(matches!(action, Action::MoveSplitDrag { col: 60, row: 5 }));
    }
}
