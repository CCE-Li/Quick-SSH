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

/// Agent 面板渲染所需的最小只读视图（从 App 提取，避免 dashboard 依赖 App）。
/// 字段为 owned 类型，避免渲染期间持有对 App 的借用。
pub struct AgentWidgetView {
    /// 监控目标别名（未连接时为 "-"）
    pub target: String,
    /// 输入缓冲
    pub input: String,
    /// Agent 状态
    pub status: crate::agent::timeline::AgentStatus,
    /// 执行时间线步骤
    pub steps: Vec<crate::agent::timeline::TimelineStep>,
    /// 已启动的步骤数
    pub started: usize,
    /// 最近一次回复
    pub last_reply: String,
    /// 权限级别中文标签
    pub permission: String,
    /// 对话历史条数
    pub history: usize,
    /// 是否聚焦（AgentOps 模式）
    pub focused: bool,
}

/// 渲染 Dashboard 主视图：按 Profile 布局树绘制各 Widget
///
/// 当布局包含 `WidgetId::Terminal` 且存在嵌入式终端会话时，优先渲染真实
/// PTY 屏幕内容；否则回退到普通 Widget 渲染。
/// `WidgetId::Agent` 提供 `agent_view` 时渲染内嵌输入框面板（聚焦时允许输入）。
/// `focused` 为当前聚焦的模块（操作面板打开时对应 Widget 边框高亮）。
pub fn render_dashboard(
    frame: &mut Frame,
    area: Rect,
    profile: &ProfileConfig,
    widgets: &mut [Box<dyn WidgetModule>],
    focused: Option<WidgetId>,
    term_session: Option<&crate::tui::term::TermSession>,
    agent_view: Option<&AgentWidgetView>,
) {
    for (id, rect) in compute_layout(&profile.layout, area) {
        // 嵌入式终端：使用真实会话渲染
        if id == WidgetId::Terminal {
            if let Some(session) = term_session {
                let buf = frame.buffer_mut();
                render_terminal_widget(session, rect, buf, focused == Some(WidgetId::Terminal));
                continue;
            }
        }
        // AI Agent：内嵌输入框面板
        if id == WidgetId::Agent {
            if let Some(view) = agent_view {
                let buf = frame.buffer_mut();
                render_agent_widget(view, rect, buf);
                continue;
            }
        }
        if let Some(widget) = widgets.iter_mut().find(|widget| widget.id() == id) {
            widget.set_focused(focused == Some(id));
            let buf = frame.buffer_mut();
            widget.render(rect, buf);
        }
    }
}

/// 渲染 AI Agent 面板：输入框 + 状态 + 时间线摘要 + 最近回复
fn render_agent_widget(view: &AgentWidgetView, area: Rect, buf: &mut Buffer) {
    use ratatui::prelude::Modifier;
    use ratatui::style::Style;
    use ratatui::widgets::{Block, Borders, Paragraph};

    let border_style = if view.focused {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let title = format!(
        " AI Agent — {} / 权限: {} / 历史 {} ",
        view.target, view.permission, view.history
    );
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(border_style);
    let inner = block.inner(area);
    block.render(area, buf);

    if inner.width == 0 || inner.height == 0 {
        return;
    }

    // 输入行（顶部固定 1 行，聚焦时带底色）
    let input_area = Rect {
        x: inner.x,
        y: inner.y,
        width: inner.width,
        height: 1,
    };
    let input_style = if view.focused {
        Style::default().bg(Color::Rgb(40, 44, 52))
    } else {
        Style::default()
    };
    let input_line = Line::from(vec![
        Span::styled("> ", Style::default().fg(Color::Cyan)),
        Span::raw(view.input.clone()),
    ]);
    Paragraph::new(input_line)
        .style(input_style)
        .render(input_area, buf);

    // 输入光标：聚焦时在输入末尾叠加反色块
    if view.focused {
        let col = 2 + view.input.chars().count();
        if col < inner.width as usize {
            let x = inner.x + col as u16;
            let y = inner.y;
            if let Some(cell) = buf.cell_mut((x, y)) {
                let style = cell.style();
                let fg = match style.fg {
                    Some(Color::Reset) | None => Color::White,
                    Some(c) => c,
                };
                let bg = match style.bg {
                    Some(Color::Reset) | None => Color::Black,
                    Some(c) => c,
                };
                let modifiers = cell.modifier;
                cell.set_style(Style::default().fg(bg).bg(fg).add_modifier(modifiers));
            }
        }
    }

    // 状态 + 时间线 + 最近回复（下方区域）
    let body_area = Rect {
        x: inner.x,
        y: inner.y + 1,
        width: inner.width,
        height: inner.height.saturating_sub(1),
    };
    if body_area.height == 0 {
        return;
    }
    let status_color = match view.status {
        crate::agent::timeline::AgentStatus::Ready => Color::Green,
        crate::agent::timeline::AgentStatus::Thinking => Color::Cyan,
        crate::agent::timeline::AgentStatus::Executing => Color::Cyan,
        crate::agent::timeline::AgentStatus::Approval => Color::Yellow,
        crate::agent::timeline::AgentStatus::Error => Color::Red,
        crate::agent::timeline::AgentStatus::Done => Color::Green,
    };
    let mut lines: Vec<Line> = vec![Line::from(vec![Span::styled(
        format!(" {} — 已启动 {} 步", view.status.label(), view.started),
        Style::default().fg(status_color),
    )])];
    for step in view
        .steps
        .iter()
        .rev()
        .take(body_area.height.saturating_sub(1) as usize)
    {
        let color = match step.status {
            crate::agent::timeline::StepStatus::Pending => Color::DarkGray,
            crate::agent::timeline::StepStatus::Running => Color::Cyan,
            crate::agent::timeline::StepStatus::Done => Color::Green,
            crate::agent::timeline::StepStatus::Skipped => Color::Yellow,
            crate::agent::timeline::StepStatus::Failed => Color::Red,
        };
        let detail = if step.detail.is_empty() {
            step.description.clone()
        } else {
            step.detail.clone()
        };
        let mut spans = vec![
            Span::styled(
                format!(" {} {} ", step.status.symbol(), step.status.label()),
                Style::default().fg(color),
            ),
            Span::styled(step.name.clone(), Style::default().fg(color)),
        ];
        if step.read_only {
            spans.push(Span::styled(
                " [只读]",
                Style::default().fg(Color::DarkGray),
            ));
        }
        spans.push(Span::raw(" — "));
        spans.push(Span::styled(detail, Style::default().fg(Color::DarkGray)));
        lines.push(Line::from(spans));
    }
    if !view.last_reply.is_empty() {
        lines.push(Line::from(vec![
            Span::styled(" 回复: ", Style::default().fg(Color::DarkGray)),
            Span::raw(view.last_reply.clone()),
        ]));
    }
    if view.steps.is_empty() && view.last_reply.is_empty() {
        lines.push(Line::from(Span::styled(
            "  输入指令回车发送（Ctrl+A 聚焦）",
            Style::default().fg(Color::DarkGray),
        )));
    }
    Paragraph::new(lines).render(body_area, buf);
}

/// 渲染嵌入式终端面板（带边框 + PTY 屏幕内容 + 状态标题）
/// `focused` 为 true 时边框黄色加粗高亮（Terminal 模式下聚焦）
fn render_terminal_widget(
    session: &crate::tui::term::TermSession,
    area: Rect,
    buf: &mut Buffer,
    focused: bool,
) {
    use ratatui::prelude::Modifier;
    use ratatui::style::Style;
    use ratatui::widgets::Block;

    let title = format!(" {} — {} ", session.target.alias, session.status.label());
    let border_style = if focused {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(border_style);
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
