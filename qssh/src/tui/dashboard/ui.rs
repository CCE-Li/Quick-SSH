//! Dashboard 渲染：布局 + Widget + 命令面板弹窗

use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::Widget;
use ratatui::style::{Color, Style};
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
    /// 权限级别中文标签
    pub permission: String,
    /// 对话历史条数
    pub history: usize,
    /// 对话线程（用户/助手消息 + 工具步骤）
    pub thread: Vec<crate::tui::app::AgentChatEntry>,
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

/// 渲染 AI Agent 面板：opencode 风格的对话线程 + 底部输入
fn render_agent_widget(view: &AgentWidgetView, area: Rect, buf: &mut Buffer) {
    use crate::tui::app::AgentChatEntry;
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

    // 输入行（底部固定 1 行，opencode 风格 `> ` 提示符）
    let input_area = Rect {
        x: inner.x,
        y: inner.y + inner.height.saturating_sub(1),
        width: inner.width,
        height: 1,
    };
    let input_style = if view.focused {
        Style::default().bg(Color::Rgb(40, 44, 52))
    } else {
        Style::default()
    };
    let input_line = Line::from(vec![
        Span::styled(
            "> ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(view.input.clone()),
    ]);
    Paragraph::new(input_line)
        .style(input_style)
        .render(input_area, buf);

    // 输入光标：聚焦时在输入末尾绘制高对比块状光标
    if view.focused {
        use unicode_width::UnicodeWidthStr;
        let col = 2 + view.input.width();
        if col < inner.width as usize {
            let x = inner.x + col as u16;
            let y = input_area.y;
            if let Some(cell) = buf.cell_mut((x, y)) {
                // 块状光标：白底黑字，与输入行背景形成鲜明对比
                cell.set_symbol(" ");
                cell.set_style(Style::default().fg(Color::Black).bg(Color::White));
            }
        }
    }

    // 对话线程 + 状态（上方区域，展示最近的对话）
    let body_height = inner.height.saturating_sub(1) as usize;
    if body_height == 0 {
        return;
    }
    let body_area = Rect {
        x: inner.x,
        y: inner.y,
        width: inner.width,
        height: body_height as u16,
    };

    let status_color = match view.status {
        crate::agent::timeline::AgentStatus::Ready => Color::Green,
        crate::agent::timeline::AgentStatus::Thinking => Color::Cyan,
        crate::agent::timeline::AgentStatus::Executing => Color::Cyan,
        crate::agent::timeline::AgentStatus::Approval => Color::Yellow,
        crate::agent::timeline::AgentStatus::Error => Color::Red,
        crate::agent::timeline::AgentStatus::Done => Color::Green,
    };

    let mut lines: Vec<Line> = Vec::new();
    for entry in &view.thread {
        match entry {
            AgentChatEntry::User(text) => {
                lines.push(Line::from(vec![Span::styled(
                    "  You",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )]));
                push_wrapped(&mut lines, text, inner.width as usize);
            }
            AgentChatEntry::Assistant(text) => {
                lines.push(Line::from(vec![Span::styled(
                    "  agent",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                )]));
                push_wrapped(&mut lines, text, inner.width as usize);
            }
            AgentChatEntry::Tool(step) => {
                push_tool_step_lines(&mut lines, step, inner.width as usize);
            }
        }
        lines.push(Line::from(""));
    }

    // 进行中的时间线步骤（尚未固化到线程）
    for step in view.steps.iter().rev().take(body_height) {
        push_tool_step_lines(&mut lines, step, inner.width as usize);
    }
    // 状态行（底部、输入框上方）
    lines.push(Line::from(vec![Span::styled(
        format!(" {} — 已启动 {} 步", view.status.label(), view.started),
        Style::default().fg(status_color),
    )]));

    if view.thread.is_empty() && view.steps.is_empty() {
        lines.push(Line::from(Span::styled(
            "  输入指令回车发送（Ctrl+A 聚焦）",
            Style::default().fg(Color::DarkGray),
        )));
    }

    // 保留最近 body_height 行（模拟自动滚动到底部）
    if lines.len() > body_height {
        lines = lines.split_off(lines.len() - body_height);
    }
    Paragraph::new(lines).render(body_area, buf);
}

/// 工具步骤行：状态符号 + 名称 + 摘要；摘要按显示宽度换行，续行缩进对齐头部
fn push_tool_step_lines(
    lines: &mut Vec<Line<'static>>,
    step: &crate::agent::timeline::TimelineStep,
    width: usize,
) {
    use unicode_width::UnicodeWidthStr;

    let color = step_status_color(step.status);
    let head = format!(
        " {} {} {}{} — ",
        step.status.symbol(),
        step.status.label(),
        step.name,
        if step.read_only { " [只读]" } else { "" }
    );
    let head_w = UnicodeWidthStr::width(head.as_str());
    let detail = if step.detail.is_empty() {
        step.description.clone()
    } else {
        step.detail.clone()
    };
    // 首行：彩色头部
    lines.push(Line::from(vec![Span::styled(
        head.clone(),
        Style::default().fg(color),
    )]));
    // 摘要换行：首段接在头部后，续段缩进到头部宽度
    let avail = width.saturating_sub(head_w).max(1);
    let chunks = wrap_cells(&detail, avail);
    if let Some((first, rest)) = chunks.split_first() {
        if let Some(line) = lines.last_mut() {
            line.spans.push(Span::styled(
                first.clone(),
                Style::default().fg(Color::DarkGray),
            ));
        }
        for chunk in rest {
            lines.push(Line::from(vec![
                Span::styled(" ".repeat(head_w), Style::default()),
                Span::styled(chunk.clone(), Style::default().fg(Color::DarkGray)),
            ]));
        }
    }
}

/// 步骤状态 → 颜色
fn step_status_color(status: crate::agent::timeline::StepStatus) -> Color {
    match status {
        crate::agent::timeline::StepStatus::Pending => Color::DarkGray,
        crate::agent::timeline::StepStatus::Running => Color::Cyan,
        crate::agent::timeline::StepStatus::Done => Color::Green,
        crate::agent::timeline::StepStatus::Skipped => Color::Yellow,
        crate::agent::timeline::StepStatus::Failed => Color::Red,
    }
}

/// 按显示宽度追加文本行（CJK 全角按 2 列），首行与续行均缩进 4 格
fn push_wrapped(lines: &mut Vec<Line<'static>>, text: &str, width: usize) {
    let wrap_width = width.saturating_sub(4).max(1);
    for chunk in wrap_cells(text, wrap_width) {
        if chunk.is_empty() {
            lines.push(Line::from(""));
        } else {
            lines.push(Line::from(format!("    {chunk}")));
        }
    }
}

/// 按显示宽度把文本切分为不超过 `max_w` 列的片段（硬换行，保留空行）
fn wrap_cells(text: &str, max_w: usize) -> Vec<String> {
    use unicode_width::UnicodeWidthChar;

    let mut out = Vec::new();
    let mut buf = String::new();
    let mut buf_w = 0usize;
    for ch in text.chars() {
        if ch == '\n' {
            out.push(std::mem::take(&mut buf));
            buf_w = 0;
            continue;
        }
        let w = UnicodeWidthChar::width(ch).unwrap_or(0);
        if !buf.is_empty() && buf_w + w > max_w {
            out.push(std::mem::take(&mut buf));
            buf_w = 0;
        }
        buf.push(ch);
        buf_w += w;
    }
    if !buf.is_empty() {
        out.push(buf);
    }
    out
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::timeline::{AgentStatus, TimelineStep};
    use crate::tui::app::AgentChatEntry;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    fn view_with(input: &str, focused: bool, thread: Vec<AgentChatEntry>) -> AgentWidgetView {
        AgentWidgetView {
            target: "web".to_string(),
            input: input.to_string(),
            status: AgentStatus::Ready,
            steps: vec![TimelineStep::new("docker.list", "容器列表", true)],
            started: 1,
            permission: "读写".to_string(),
            history: 1,
            thread,
            focused,
        }
    }

    /// 聚焦时输入行末尾应出现块状光标（白底黑字），含中文输入宽度
    #[test]
    fn agent_cursor_renders_at_input_end() {
        for input in ["", "help", "查看 docker 容器", "重启 nginx"] {
            let view = view_with(input, true, vec![]);
            let backend = TestBackend::new(60, 20);
            let mut terminal = Terminal::new(backend).unwrap();
            terminal
                .draw(|frame| {
                    let area = frame.area();
                    let buf = frame.buffer_mut();
                    render_agent_widget(&view, area, buf);
                })
                .unwrap();
            let buf = terminal.backend().buffer();
            // 输入行位于内区域最后一行（外框 1 行边界内）
            let y = area_bottom_inner_y(buf);
            let expected_col = 2 + unicode_width::UnicodeWidthStr::width(input);
            // 光标单元格应有白底（块状光标）
            let cell = buf.cell((1 + expected_col as u16, y)).unwrap();
            assert_eq!(
                cell.style().bg,
                Some(Color::White),
                "input={input:?} 光标白底缺失"
            );
        }
    }

    fn area_bottom_inner_y(buf: &Buffer) -> u16 {
        buf.area.height.saturating_sub(2)
    }

    /// 未聚焦时不画光标
    #[test]
    fn agent_cursor_absent_when_not_focused() {
        let view = view_with("help", false, vec![]);
        let backend = TestBackend::new(60, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                let area = frame.area();
                let buf = frame.buffer_mut();
                render_agent_widget(&view, area, buf);
            })
            .unwrap();
        let buf = terminal.backend().buffer();
        let y = area_bottom_inner_y(buf);
        let expected_col = 2 + unicode_width::UnicodeWidthStr::width("help");
        let cell = buf.cell((1 + expected_col as u16, y)).unwrap();
        assert_ne!(cell.style().bg, Some(Color::White));
    }

    /// 对话线程：用户/助手消息 + 工具步骤按序渲染
    #[test]
    fn agent_thread_renders_roles_and_tools() {
        use crate::agent::timeline::{StepStatus, Timeline};
        let mut step = TimelineStep::new("docker.list", "容器列表", true);
        Timeline::new().set_result(0, StepStatus::Done, "ok");
        step.status = StepStatus::Done;
        step.detail = "ok".to_string();
        let thread = vec![
            AgentChatEntry::User("查看容器".to_string()),
            AgentChatEntry::Tool(step),
            AgentChatEntry::Assistant("有 3 个容器".to_string()),
        ];
        let view = view_with("", true, thread);
        let backend = TestBackend::new(60, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                let area = frame.area();
                let buf = frame.buffer_mut();
                render_agent_widget(&view, area, buf);
            })
            .unwrap();
        let rendered = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();
        assert!(rendered.contains("You"));
        assert!(rendered.contains("agent"));
        assert!(rendered.contains("docker.list"));
        assert!(rendered.contains("查"));
        assert!(rendered.contains("有"));
        assert!(rendered.contains("3"));
    }

    /// 按显示宽度换行：CJK 全角按 2 列计，超宽片段被正确切分
    #[test]
    fn wrap_cells_counts_cjk_as_two_columns() {
        assert_eq!(
            wrap_cells("你好世界abcdef", 6),
            vec!["你好世", "界abcd", "ef"]
        );
        // 空行保留
        assert_eq!(wrap_cells("ab\n\ncd", 10), vec!["ab", "", "cd"]);
        // 超长 ASCII 也按列切分
        let chunks = wrap_cells(&"x".repeat(20), 8);
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks.iter().map(|c| c.len()).sum::<usize>(), 20);
    }

    /// 长 Agent 回复按边界换行展示，尾部文本不被裁剪吞掉
    #[test]
    fn long_agent_reply_wraps_tail_not_swallowed() {
        let text = format!(
            "{}。",
            "服务器当前运行健康，CPU 与内存负载均处于低位，磁盘空间充足".repeat(4)
        );
        let thread = vec![AgentChatEntry::Assistant(text.clone())];
        let view = view_with("", false, thread);
        let backend = TestBackend::new(60, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                let area = frame.area();
                let buf = frame.buffer_mut();
                render_agent_widget(&view, area, buf);
            })
            .unwrap();
        let rendered = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();
        // 去掉换行缩进产生的空格后，消息尾部应连续出现（换行展示而非被裁剪吞掉）
        let compact: String = rendered.chars().filter(|c| !c.is_whitespace()).collect();
        let tail: String = text
            .chars()
            .rev()
            .take(4)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        assert!(
            compact.contains(&tail),
            "尾部 {tail:?} 被吞；渲染内容: {compact:?}"
        );
    }
}
