//! Dashboard 渲染：布局 + Widget + 命令面板弹窗

use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::Widget;
use ratatui::style::Color;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph};
use ratatui::Frame;

use super::config::{ProfileConfig, WidgetId};
use super::layout::compute_layout;
use super::palette::{filter_palette, PALETTE_ACTIONS};
use super::widgets::WidgetModule;
use crate::tui::widgets::centered_rect;

/// 渲染 Dashboard 主视图：按 Profile 布局树绘制各 Widget
///
/// 当布局包含 `WidgetId::Terminal` 且存在嵌入式终端会话时，优先渲染真实
/// PTY 屏幕内容；否则回退到普通 Widget 渲染。
pub fn render_dashboard(
    frame: &mut Frame,
    area: Rect,
    profile: &ProfileConfig,
    widgets: &[Box<dyn WidgetModule>],
    term_session: Option<&crate::tui::term::TermSession>,
) {
    for (id, rect) in compute_layout(&profile.layout, area) {
        // 嵌入式终端：使用真实会话渲染
        if id == WidgetId::Terminal {
            if let Some(session) = term_session {
                let buf = frame.buffer_mut();
                render_terminal_widget(session, rect, buf);
                continue;
            }
        }
        if let Some(widget) = widgets.iter().find(|widget| widget.id() == id) {
            let buf = frame.buffer_mut();
            widget.render(rect, buf);
        }
    }
}

/// 渲染嵌入式终端面板（带边框 + PTY 屏幕内容 + 状态标题）
fn render_terminal_widget(session: &crate::tui::term::TermSession, area: Rect, buf: &mut Buffer) {
    use ratatui::widgets::Block;

    let title = format!(" {} — {} ", session.target.alias, session.status.label());
    let block = Block::default().borders(Borders::ALL).title(title);
    let inner_area = block.inner(area);
    // 先画边框，再在内区域渲染 PTY 屏幕（内区域不覆盖边框）
    block.render(area, buf);
    if inner_area.height > 0 && inner_area.width > 0 {
        session.render(inner_area, buf);
    }
}

/// 渲染命令面板弹窗（Ctrl+K）
pub fn render_palette_popup(frame: &mut Frame, query: &str, selected: usize) {
    let area = frame.area();
    let popup = centered_rect(60, 60, area);
    frame.render_widget(Clear, popup);

    let outer = Block::default()
        .borders(Borders::ALL)
        .title(" Command Palette ");
    let inner = outer.inner(popup);
    outer.render(popup, frame.buffer_mut());

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(inner);

    let input =
        Paragraph::new(query).block(Block::default().borders(Borders::ALL).title(" Query "));
    frame.render_widget(input, chunks[0]);

    let items: Vec<_> = filter_palette(PALETTE_ACTIONS, query);
    let list_items: Vec<ListItem> = items
        .iter()
        .enumerate()
        .map(|(idx, item)| {
            let title = if idx == selected {
                Span::styled(
                    item.title,
                    ratatui::style::Style::default().fg(Color::Yellow),
                )
            } else {
                Span::raw(item.title)
            };
            ListItem::new(Line::from(title))
        })
        .collect();
    let list = List::new(list_items)
        .block(Block::default().borders(Borders::ALL).title(" Actions "))
        .highlight_symbol("> ");
    let mut state = ListState::default();
    state.select(Some(selected.min(items.len().saturating_sub(1))));
    frame.render_stateful_widget(list, chunks[1], &mut state);
}
