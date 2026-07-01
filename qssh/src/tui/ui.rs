use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Frame;

use crate::tui::app::App;

/// 渲染主界面
pub fn render(frame: &mut Frame, app: &mut App) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // 标题栏
            Constraint::Min(0),    // 主体
            Constraint::Length(1), // 状态栏
        ])
        .split(area);

    render_header(frame, chunks[0], app);
    render_body(frame, chunks[1], app);
    render_status_bar(frame, chunks[2], app);
}

fn render_header(frame: &mut Frame, area: Rect, app: &App) {
    let title = format!(
        " Quick-SSH v{}  |  共 {} 台主机  |  模式: {}",
        env!("CARGO_PKG_VERSION"),
        app.hosts.len(),
        app.mode.label()
    );

    let header = Paragraph::new(title).style(
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    );
    frame.render_widget(header, area);
}

fn render_body(frame: &mut Frame, area: Rect, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
        .split(area);

    render_detail(frame, chunks[1], &*app);
    render_host_list(frame, chunks[0], app);
}

fn render_host_list(frame: &mut Frame, area: Rect, app: &mut App) {
    let items: Vec<ListItem> = app
        .hosts
        .iter()
        .enumerate()
        .map(|(i, host)| {
            let prefix = if app.marked.contains(&i) {
                " > "
            } else {
                "   "
            };

            let status_icon = match app.host_status.get(&host.alias) {
                Some(true) => "(*)",
                Some(false) => "(x)",
                None => "( )",
            };

            let content = Line::from(vec![
                Span::raw(prefix),
                Span::raw(status_icon),
                Span::raw(" "),
                Span::styled(&host.alias, Style::default().add_modifier(Modifier::BOLD)),
            ]);

            ListItem::new(content)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("主机列表"))
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("│ ");

    frame.render_stateful_widget(list, area, &mut app.list_state);
}

fn render_detail(frame: &mut Frame, area: Rect, app: &App) {
    let detail = if let Some(idx) = app.selected() {
        if let Some(host) = app.hosts.get(idx) {
            let hostname = host.hostname().unwrap_or("-");
            let user = host.user().map(|u| format!("{}@", u)).unwrap_or_default();
            let port = host.port();
            let key = host
                .identity_file()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "(agent)".to_string());

            format!(
                "别名: {}\n地址: {}{}:{}\n密钥: {}\n状态: {}",
                host.alias,
                user,
                hostname,
                port,
                key,
                match app.host_status.get(&host.alias) {
                    Some(true) => "在线",
                    Some(false) => "离线",
                    None => "未检测",
                }
            )
        } else {
            "选择主机查看详情".to_string()
        }
    } else {
        "选择主机查看详情".to_string()
    };

    let detail = Paragraph::new(detail).block(Block::default().borders(Borders::ALL).title("详情"));
    frame.render_widget(detail, area);
}

fn render_status_bar(frame: &mut Frame, area: Rect, app: &App) {
    let hint = app.mode.hint();
    let status = Paragraph::new(hint).style(Style::default().fg(Color::White).bg(Color::DarkGray));
    frame.render_widget(status, area);
}
