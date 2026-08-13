use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Frame;

use crate::agent::timeline::{AgentStatus, StepStatus};
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
        Mode::ServiceOps => {
            render_service_ops_popup(frame, area, app);
        }
        Mode::ServiceConfirm => {
            render_service_confirm_popup(frame, area, app);
        }
        Mode::FileOps => {
            render_file_ops_popup(frame, area, app);
        }
        Mode::FileConfirm => {
            render_file_confirm_popup(frame, area, app);
        }
        Mode::LogOps => {
            render_log_ops_popup(frame, area, app);
        }
        Mode::LogFilter => {
            render_log_filter_popup(frame, area, app);
        }
        Mode::AgentOps => {
            render_agent_ops_popup(frame, area, app);
        }
        Mode::AgentConfirm => {
            render_agent_confirm_popup(frame, area, app);
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

/// 系统服务操作面板弹窗：列出服务 + 高亮选中项 + 操作提示
fn render_service_ops_popup(frame: &mut Frame, area: Rect, app: &App) {
    use crate::tui::widgets::centered_rect;

    let popup = centered_rect(70, 60, area);
    frame.render_widget(ratatui::widgets::Clear, popup);

    let Some(snapshot) = app.docker_snapshot() else {
        let paragraph = Paragraph::new("暂无服务数据").block(
            Block::default()
                .borders(Borders::ALL)
                .title(" 系统服务操作 "),
        );
        frame.render_widget(paragraph, popup);
        return;
    };

    let mut items: Vec<ListItem> = Vec::new();
    for (idx, s) in snapshot.services.iter().enumerate() {
        let marker = if idx == app.service_selected_index() {
            "┃ "
        } else {
            "  "
        };
        let status_icon = if s.is_running() { "●" } else { "○" };
        let status_color = if s.active == "failed" {
            Color::Red
        } else if s.is_running() {
            Color::Green
        } else {
            Color::DarkGray
        };
        let status = if s.active == "failed" {
            "failed"
        } else if s.is_running() {
            "running"
        } else {
            s.sub.as_str()
        };
        let line = Line::from(vec![
            Span::raw(marker),
            Span::styled(status_icon, Style::default().fg(status_color)),
            Span::raw(" "),
            Span::styled(&s.name, Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(
                format!("  {} / {}", s.active, status),
                Style::default().fg(status_color),
            ),
        ]);
        items.push(ListItem::new(line));
    }

    let title = format!(
        " 系统服务操作 ({}) — {} ",
        snapshot.services.len(),
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

/// 服务危险操作确认弹窗：显示将要执行的操作 + 目标服务
fn render_service_confirm_popup(frame: &mut Frame, area: Rect, app: &App) {
    use crate::tui::widgets::centered_rect;

    let popup = centered_rect(50, 25, area);
    frame.render_widget(ratatui::widgets::Clear, popup);

    let action = app.service_pending_action();
    let service = app
        .docker_snapshot()
        .and_then(|s| s.services.get(app.service_selected_index()));

    let (action_text, color) = match action {
        Some(a) => (a.label(), Color::Yellow),
        None => ("未知操作", Color::Red),
    };
    let service_name = service.map(|s| s.name.as_str()).unwrap_or("-");

    let lines: Vec<Line> = vec![
        Line::from(""),
        Line::from(format!("  将要 {} 服务: {}", action_text, service_name)),
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
        .title(" 服务危险操作确认 ");
    frame.render_widget(Paragraph::new(lines).block(block), popup);
}

/// 文件浏览弹窗：列出当前目录文件，支持目录导航
fn render_file_ops_popup(frame: &mut Frame, area: Rect, app: &App) {
    use crate::tui::widgets::centered_rect;

    let popup = centered_rect(80, 65, area);
    frame.render_widget(ratatui::widgets::Clear, popup);

    let Some(snapshot) = app.docker_snapshot() else {
        let paragraph = Paragraph::new("暂无文件数据")
            .block(Block::default().borders(Borders::ALL).title(" 文件浏览 "));
        frame.render_widget(paragraph, popup);
        return;
    };

    let mut items: Vec<ListItem> = Vec::new();
    for (idx, file) in snapshot.files.iter().enumerate() {
        let marker = if idx == app.file_selected_index() {
            "┃ "
        } else {
            "  "
        };
        let dir_icon = if file.is_dir { "📁" } else { "📄" };
        let size = if file.is_dir {
            "DIR".to_string()
        } else {
            crate::monitor::files::format_file_size(file.size)
        };
        let line = Line::from(vec![
            Span::raw(marker),
            Span::raw(dir_icon),
            Span::raw(" "),
            Span::styled(&file.name, Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(
                format!("  {:>9}  {}", size, file.mtime),
                Style::default().fg(Color::DarkGray),
            ),
        ]);
        items.push(ListItem::new(line));
    }

    let title = format!(
        " 文件浏览 {} — {} ",
        snapshot.file_cwd,
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

/// 文件操作确认弹窗（下载）
fn render_file_confirm_popup(frame: &mut Frame, area: Rect, app: &App) {
    use crate::tui::widgets::centered_rect;

    let popup = centered_rect(50, 25, area);
    frame.render_widget(ratatui::widgets::Clear, popup);

    let action = app.file_pending_action();
    let file = app
        .docker_snapshot()
        .and_then(|s| s.files.get(app.file_selected_index()));

    let (action_text, color) = match action {
        Some(a) => (a.label(), Color::Yellow),
        None => ("未知操作", Color::Red),
    };
    let file_name = file.map(|f| f.name.as_str()).unwrap_or("-");

    let lines: Vec<Line> = vec![
        Line::from(""),
        Line::from(format!("  将要 {} 文件: {}", action_text, file_name)),
        Line::from(""),
        Line::from("  下载到当前目录（本地工作目录）"),
        Line::from(""),
        Line::from("  y/Y 确认执行    n/N/Esc 取消"),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(color))
        .title(" 文件操作确认 ");
    frame.render_widget(Paragraph::new(lines).block(block), popup);
}

/// 日志面板弹窗：展示 journalctl 日志，支持 unit 筛选
fn render_log_ops_popup(frame: &mut Frame, area: Rect, app: &App) {
    use crate::tui::widgets::centered_rect;

    let popup = centered_rect(80, 70, area);
    frame.render_widget(ratatui::widgets::Clear, popup);

    let logs = app.filtered_logs();
    let mut items: Vec<ListItem> = Vec::new();
    for (idx, log) in logs.iter().enumerate() {
        let marker = if idx == app.log_selected_index() {
            "┃ "
        } else {
            "  "
        };
        let line = Line::from(vec![
            Span::raw(marker),
            Span::styled(log.timestamp.clone(), Style::default().fg(Color::DarkGray)),
            Span::styled(format!(" [{}]", log.unit), Style::default().fg(Color::Cyan)),
            Span::raw(" "),
            Span::styled(log.message.clone(), Style::default()),
        ]);
        items.push(ListItem::new(line));
    }

    let unit_label = if app.log_unit().trim().is_empty() {
        "全部".to_string()
    } else {
        app.log_unit().trim().to_string()
    };
    let title = format!(
        " 系统日志 ({}) unit: {} — {} ",
        logs.len(),
        unit_label,
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

/// 日志 unit 筛选输入弹窗
fn render_log_filter_popup(frame: &mut Frame, area: Rect, app: &App) {
    use crate::tui::widgets::centered_rect;

    let popup = centered_rect(50, 25, area);
    frame.render_widget(ratatui::widgets::Clear, popup);

    let lines: Vec<Line> = vec![
        Line::from(""),
        Line::from("  输入要筛选的 unit 名称（留空 = 全部）："),
        Line::from(""),
        Line::from(Span::styled(
            format!("  > {}", app.log_unit()),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("  Enter 应用    Esc 取消"),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" 日志筛选 unit ");
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

    // 追加 AI Agent 状态指示（右侧，Layout 分栏避免字符宽度计算问题）
    let ai_label = match app.agent_status() {
        AgentStatus::Ready => "AI:● Ready".to_string(),
        AgentStatus::Thinking => "AI:◇ Thinking".to_string(),
        AgentStatus::Executing => "AI:● Executing".to_string(),
        AgentStatus::Approval => "AI:! Approval".to_string(),
        AgentStatus::Error => "AI:▲ Error".to_string(),
        AgentStatus::Done => "AI:✓ Done".to_string(),
    };
    let ai_style = match app.agent_status() {
        AgentStatus::Ready | AgentStatus::Done => {
            Style::default().fg(Color::Green).bg(Color::DarkGray)
        }
        AgentStatus::Thinking | AgentStatus::Executing => {
            Style::default().fg(Color::Cyan).bg(Color::DarkGray)
        }
        AgentStatus::Approval => Style::default().fg(Color::Yellow).bg(Color::DarkGray),
        AgentStatus::Error => Style::default().fg(Color::Red).bg(Color::DarkGray),
    };

    let bar = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(ai_label.len() as u16),
        ])
        .split(area);
    frame.render_widget(Paragraph::new(message).style(style), bar[0]);
    frame.render_widget(Paragraph::new(ai_label).style(ai_style), bar[1]);
}

/// Agent 面板弹窗：输入 + 时间线 + 最近回复
fn render_agent_ops_popup(frame: &mut Frame, area: Rect, app: &App) {
    use crate::tui::widgets::centered_rect;

    let popup = centered_rect(80, 80, area);
    frame.render_widget(ratatui::widgets::Clear, popup);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // 输入行
            Constraint::Min(0),    // 时间线
            Constraint::Length(1), // 状态
        ])
        .split(popup);

    let title = format!(
        " AI Agent — {} / 权限: {} / 历史 {} ",
        app.monitor_target_alias().unwrap_or("-"),
        app.agent_permission_label(),
        app.agent_history_count()
    );
    let block = Block::default().borders(Borders::ALL).title(title);
    frame.render_widget(block.clone(), popup);

    // 输入行
    let input_line = Line::from(vec![
        Span::styled("> ", Style::default().fg(Color::Cyan)),
        Span::raw(app.agent_input()),
    ]);
    frame.render_widget(
        Paragraph::new(input_line),
        chunks[0].inner(ratatui::layout::Margin {
            horizontal: 1,
            vertical: 0,
        }),
    );

    // 时间线
    let timeline = app.agent_timeline();
    let steps = timeline.steps();
    let mut items: Vec<ListItem> = Vec::new();
    if steps.is_empty() {
        items.push(ListItem::new(Line::from(Span::styled(
            "  还没有执行步骤 — 输入指令回车，或直接输入 tool:server.status 手动执行",
            Style::default().fg(Color::DarkGray),
        ))));
    } else {
        for step in steps {
            let color = match step.status {
                StepStatus::Pending => Color::DarkGray,
                StepStatus::Running => Color::Cyan,
                StepStatus::Done => Color::Green,
                StepStatus::Skipped => Color::Yellow,
                StepStatus::Failed => Color::Red,
            };
            let read_only = if step.read_only {
                Span::styled("[只读] ", Style::default().fg(Color::DarkGray))
            } else {
                Span::styled("[写] ", Style::default().fg(Color::Yellow))
            };
            let mut spans = vec![
                Span::styled(
                    format!(" {} ", step.status.symbol()),
                    Style::default().fg(color),
                ),
                Span::styled(step.name.clone(), Style::default().fg(color)),
                Span::raw(" "),
                read_only,
                Span::styled(
                    step.description.clone(),
                    Style::default().fg(Color::DarkGray),
                ),
            ];
            if !step.detail.is_empty() {
                spans.push(Span::raw(" — "));
                spans.push(Span::styled(
                    step.detail.clone(),
                    Style::default().fg(Color::White),
                ));
            }
            items.push(ListItem::new(Line::from(spans)));
        }
    }
    let progress = if timeline.is_empty() {
        String::new()
    } else {
        // 最后一步状态标签（如 "成功" / "执行中" / "失败"）
        let last_label = steps.last().map(|step| step.status.label()).unwrap_or("");
        format!(
            "执行 {} / {} · {}",
            timeline.started_count(),
            steps.len(),
            last_label
        )
    };
    let status_text = format!(
        " {} {}  |  {}  |  Enter 发送  Esc 关闭",
        app.agent_status().label(),
        progress,
        app.agent_last_reply(),
    );
    let status_line = Line::from(vec![Span::styled(
        status_text,
        Style::default().fg(Color::DarkGray),
    )]);
    frame.render_widget(
        List::new(items),
        chunks[1].inner(ratatui::layout::Margin {
            horizontal: 1,
            vertical: 0,
        }),
    );
    frame.render_widget(Paragraph::new(status_line), chunks[2]);
}

/// Agent 危险操作确认弹窗
fn render_agent_confirm_popup(frame: &mut Frame, area: Rect, app: &App) {
    use crate::tui::widgets::centered_rect;

    let popup = centered_rect(60, 30, area);
    frame.render_widget(ratatui::widgets::Clear, popup);

    let Some(call) = app.agent_pending_call() else {
        let block = Block::default().borders(Borders::ALL).title(" Agent 确认 ");
        frame.render_widget(Paragraph::new("  没有待确认的操作").block(block), popup);
        return;
    };

    let danger = call.tool.danger();
    let danger_color = match danger {
        crate::agent::permissions::DangerLevel::Read => Color::Green,
        crate::agent::permissions::DangerLevel::SafeWrite => Color::Yellow,
        crate::agent::permissions::DangerLevel::Dangerous => Color::Red,
    };
    let lines: Vec<Line> = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  工具: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                call.name(),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("  说明: ", Style::default().fg(Color::DarkGray)),
            Span::styled(call.tool.description(), Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::styled("  危险等级: ", Style::default().fg(Color::DarkGray)),
            Span::styled(danger.label(), Style::default().fg(danger_color)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  目标: ", Style::default().fg(Color::DarkGray)),
            Span::raw(if call.args.target.is_empty() {
                "-".to_string()
            } else {
                call.args.target.clone()
            }),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "  y / Y 执行    n / N / Esc 拒绝",
            Style::default().fg(Color::Yellow),
        )),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" ⚠ 危险操作确认 — Agent ");
    frame.render_widget(Paragraph::new(lines).block(block), popup);
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
