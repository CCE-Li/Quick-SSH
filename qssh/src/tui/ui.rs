use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Frame;

use crate::tui::action::Mode;
use crate::tui::app::App;
use crate::tui::dashboard::ui::render_palette_popup;
use crate::tui::widgets::render_host_form_popup;

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
    match app.view {
        crate::tui::action::View::HostList => render_body(frame, chunks[1], app),
        crate::tui::action::View::Dashboard => {
            render_dashboard_body(frame, chunks[1], app);
        }
    }
    render_status_bar(frame, chunks[2], app);

    match app.mode {
        Mode::Add | Mode::Edit => {
            render_host_form_popup(frame, area, app);
        }
        Mode::Help => {
            crate::tui::widgets::render_help_popup(frame, area);
        }
        Mode::Palette => {
            render_palette_popup(frame, &app.palette_query, app.palette_selected);
        }
        Mode::DashboardConfig => {
            render_dashboard_config_popup(frame, area, app);
        }
        Mode::DockerOps => {
            render_docker_ops_popup(frame, area, app);
        }
        Mode::DockerConfirm => {
            render_docker_confirm_popup(frame, area, app);
        }
        _ => {}
    }
}

/// Dashboard 主体：按活动 Profile 布局渲染各 Widget
fn render_dashboard_body(frame: &mut Frame, area: Rect, app: &mut App) {
    let Some(profile) = app
        .dashboard_config
        .profiles
        .get(&app.dashboard_config.active_profile)
    else {
        return;
    };
    crate::tui::dashboard::ui::render_dashboard(frame, area, profile, &app.dashboard_widgets);
}

/// Dashboard 配置弹窗：勾选模块（Phase 1.6）
fn render_dashboard_config_popup(frame: &mut Frame, area: Rect, app: &App) {
    use crate::tui::widgets::centered_rect;

    let popup = centered_rect(60, 70, area);
    frame.render_widget(ratatui::widgets::Clear, popup);

    let enabled = app
        .dashboard_config
        .profiles
        .get(&app.dashboard_config.active_profile)
        .map(|profile| &profile.enabled)
        .cloned()
        .unwrap_or_default();

    let mut lines: Vec<Line> = Vec::new();
    let active_profile = &app.dashboard_config.active_profile;
    lines.push(Line::from(format!(
        "  Profile: {}  (q/Esc 关闭，s 保存)",
        active_profile
    )));
    lines.push(Line::from(""));

    for (idx, id) in crate::tui::dashboard::ALL_WIDGETS.iter().enumerate() {
        let checked = enabled.contains(id);
        let marker = if checked { "[x]" } else { "[ ]" };
        let key = match idx {
            0..=8 => format!("{}", idx + 1),
            9 => "0".to_string(),
            _ => "a".to_string(),
        };
        let mut span = Span::raw(format!("  {} {}  {}", key, marker, id.title()));
        if checked {
            span = Span::styled(
                format!("  {} {}  {}", key, marker, id.title()),
                Style::default().fg(Color::Green),
            );
        }
        lines.push(Line::from(span));
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Dashboard Configuration ");
    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, popup);
}

/// Docker 容器操作面板弹窗：列出容器 + 高亮选中项 + 操作提示
fn render_docker_ops_popup(frame: &mut Frame, area: Rect, app: &App) {
    use crate::tui::widgets::centered_rect;

    let popup = centered_rect(70, 60, area);
    frame.render_widget(ratatui::widgets::Clear, popup);

    let Some(snapshot) = app.docker_snapshot() else {
        let paragraph = Paragraph::new("暂无容器数据").block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Docker 容器操作 "),
        );
        frame.render_widget(paragraph, popup);
        return;
    };

    let mut items: Vec<ListItem> = Vec::new();
    for (idx, c) in snapshot.docker.iter().enumerate() {
        let marker = if idx == app.docker_selected_index() {
            "┃ "
        } else {
            "  "
        };
        let status_icon = if c.running { "●" } else { "○" };
        let status_color = if c.running { Color::Green } else { Color::Red };
        let line = Line::from(vec![
            Span::raw(marker),
            Span::styled(status_icon, Style::default().fg(status_color)),
            Span::raw(" "),
            Span::styled(&c.name, Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(format!("  {}", c.image), Style::default().fg(Color::Cyan)),
            Span::styled(
                format!("  {}", c.status),
                Style::default().fg(Color::DarkGray),
            ),
        ]);
        items.push(ListItem::new(line));
    }

    let title = format!(
        " Docker 容器操作 ({}) — {} ",
        snapshot.docker.len(),
        app.monitor_target_alias().unwrap_or("-")
    );
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("");
    frame.render_widget(list, popup);
}

/// Docker 危险操作确认弹窗：显示将要执行的操作 + 目标容器
fn render_docker_confirm_popup(frame: &mut Frame, area: Rect, app: &App) {
    use crate::tui::widgets::centered_rect;

    let popup = centered_rect(50, 25, area);
    frame.render_widget(ratatui::widgets::Clear, popup);

    let action = app.docker_pending_action();
    let container = app
        .docker_snapshot()
        .and_then(|s| s.docker.get(app.docker_selected_index()));

    let (action_text, color) = match action {
        Some(a) => (a.label(), Color::Yellow),
        None => ("未知操作", Color::Red),
    };
    let container_name = container.map(|c| c.name.as_str()).unwrap_or("-");

    let lines: Vec<Line> = vec![
        Line::from(""),
        Line::from(format!("  将要 {} 容器: {}", action_text, container_name)),
        Line::from(""),
        Line::from(Span::styled(
            "  ⚠ 该操作将直接作用于远程主机，请确认",
            Style::default().fg(Color::Yellow),
        )),
        Line::from(""),
        Line::from("  y/Y 确认执行    n/N/Esc 取消"),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(color))
        .title(" Docker 危险操作确认 ");
    frame.render_widget(Paragraph::new(lines).block(block), popup);
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
            let prefix = if app.marked.contains(&i) { "> " } else { " " };

            let status_span = if app.pending_pings.contains(&host.alias) {
                Span::styled("◔", Style::default().fg(Color::Yellow))
            } else {
                match app.host_status.get(&host.alias) {
                    Some(true) => Span::styled("●", Style::default().fg(Color::Green)),
                    Some(false) => Span::styled("●", Style::default().fg(Color::Red)),
                    None => Span::styled("○", Style::default().fg(Color::DarkGray)),
                }
            };

            let mut spans = vec![
                Span::raw(prefix),
                status_span,
                Span::raw(" "),
                Span::styled(&host.alias, Style::default().add_modifier(Modifier::BOLD)),
            ];
            if let Some(comment) = host.comment_lines().into_iter().next() {
                // 注释最多显示 10 个字符
                let display_comment: String = comment.chars().take(10).collect();
                spans.push(Span::styled(
                    format!("  {}", display_comment),
                    Style::default().fg(Color::DarkGray),
                ));
            }

            let content = Line::from(spans);

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
        .highlight_symbol("┃ ");

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
            let has_saved_password = app.remembered_password_aliases.contains(&host.alias);
            let auth = match (host.identity_file().is_some(), has_saved_password) {
                (true, true) => "密钥优先",
                (true, false) => "密钥登录",
                (false, true) => "密码登录",
                (false, false) => "手动输入",
            };
            let comments = host.comment_lines();
            let comment_display = if comments.is_empty() {
                "-".to_string()
            } else {
                comments.join("\n      ")
            };

            let addr_display = if app.show_address {
                format!("{}{}:{}", user, hostname, port)
            } else {
                "********".to_string()
            };

            format!(
                "别名: {}\n地址: {}\n密钥: {}\n认证: {}\n状态: {}\n注释: {}",
                host.alias,
                addr_display,
                key,
                auth,
                if app.pending_pings.contains(&host.alias) {
                    "◔ 检测中"
                } else {
                    match app.host_status.get(&host.alias) {
                        Some(true) => "● 在线",
                        Some(false) => "● 离线",
                        None => "○ 未检测",
                    }
                },
                comment_display
            )
        } else {
            "选择主机查看详情".to_string()
        }
    } else {
        "选择主机查看详情".to_string()
    };

    let detail = Paragraph::new(detail)
        .block(Block::default().borders(Borders::ALL).title("详情"))
        .wrap(ratatui::widgets::Wrap { trim: false });
    frame.render_widget(detail, area);
}

fn render_status_bar(frame: &mut Frame, area: Rect, app: &App) {
    let (message, style) = if let Some(flash_message) = &app.flash_message {
        let fg = match flash_message.color.as_str() {
            "green" => Color::Green,
            "red" => Color::Red,
            "yellow" => Color::Yellow,
            _ => Color::White,
        };
        (
            flash_message.message.as_str(),
            Style::default().fg(fg).bg(Color::DarkGray),
        )
    } else {
        (
            app.mode.hint(),
            Style::default().fg(Color::White).bg(Color::DarkGray),
        )
    };
    let status = Paragraph::new(message).style(style);
    frame.render_widget(status, area);
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    use super::render;
    use crate::config::types::{HostBlock, SshConfig, SshDirective};
    use crate::tui::app::App;

    #[test]
    fn renders_host_comments_in_list_and_detail() {
        let config = SshConfig {
            hosts: vec![HostBlock {
                alias: "demo".into(),
                directives: vec![SshDirective::HostName("example.com".into())],
                raw_text:
                    "Host demo\n    HostName example.com\n    # production server\n    # owner: ops"
                        .into(),
            }],
            preamble: String::new(),
        };
        let mut app = App::new(config, PathBuf::from("unused-test-config"));
        let backend = TestBackend::new(100, 20);
        let mut terminal = Terminal::new(backend).expect("test terminal should initialize");

        terminal
            .draw(|frame| render(frame, &mut app))
            .expect("TUI should render");

        let rendered = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();
        assert!(rendered.contains("production server"));
        assert!(rendered.contains("owner: ops"));
    }
}
