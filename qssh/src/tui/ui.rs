use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Frame;

use crate::agent::timeline::AgentStatus;
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
        // AgentOps：AI Agent 面板内嵌输入框已由 Dashboard widget 渲染，
        // 此处仅处理 Agent 设置/确认弹窗
        Mode::AgentOps => {}
        Mode::AgentConfirm => {
            render_agent_confirm_popup(frame, area, app);
        }
        Mode::AgentConfig => {
            render_agent_config_popup(frame, area, app);
        }
        Mode::Search => {
            render_search_popup(frame, area, app);
        }
        Mode::Command => {
            render_command_popup(frame, area, app);
        }
        Mode::CommandResult => {
            render_command_result_popup(frame, area, app);
        }
        _ => {}
    }

    // 拖动主机项时绘制跟随光标的浮动标签（拖拽反馈）
    render_drag_ghost(frame, app);
}

/// 拖动主机项时，在光标处绘制浮动标签（高对比样式），作为「正在拖动」的视觉反馈
fn render_drag_ghost(frame: &mut Frame, app: &App) {
    use unicode_width::UnicodeWidthStr;

    if app.host_drag.is_none() {
        return;
    }
    let Some((col, row)) = app.host_drag_pos else {
        return;
    };
    let Some(host) = app.selected().and_then(|idx| app.hosts.get(idx)) else {
        return;
    };

    // 贴边时给出「松手即开新窗口」的提示（绿色 + 文字）
    let label = if app.host_drag_outside {
        format!(" {} → 新窗口 ", host.alias)
    } else {
        format!(" {} ", host.alias)
    };
    let width = label.as_str().width() as u16;
    let area = frame.area();
    if width == 0 || area.width == 0 || area.height == 0 {
        return;
    }
    // 标签始终保持在窗口内
    let x = col.min(area.width.saturating_sub(width));
    let y = row.min(area.height.saturating_sub(1));
    let rect = Rect::new(x, y, width.min(area.width), 1);

    let bg = if app.host_drag_outside {
        Color::Green
    } else {
        Color::Cyan
    };
    let style = Style::default()
        .fg(Color::Black)
        .bg(bg)
        .add_modifier(Modifier::BOLD);
    frame.render_widget(ratatui::widgets::Clear, rect);
    frame.render_widget(Paragraph::new(Line::from(Span::styled(label, style))), rect);
}

/// Dashboard 主体：按活动 Profile 布局渲染各 Widget
fn render_dashboard_body(frame: &mut Frame, area: Rect, app: &mut App) {
    let Some(profile) = app
        .dashboard_config
        .profiles
        .get(&app.dashboard_config.active_profile)
        .cloned()
    else {
        return;
    };
    // 嵌入式终端：按布局面板尺寸同步 PTY 尺寸
    if app.term_session.is_some() {
        for (id, rect) in crate::tui::dashboard::compute_layout(&profile.layout, area) {
            if id == crate::tui::dashboard::WidgetId::Terminal {
                let margin = ratatui::layout::Margin::new(1, 1);
                let inner = rect.inner(margin);
                if inner.width > 0 && inner.height > 0 {
                    app.term_resize(inner.width, inner.height);
                }
                break;
            }
        }
    }
    let timeline = app.agent_timeline();
    let started = if timeline.is_empty() {
        0
    } else {
        timeline.started_count()
    };
    let agent_view = crate::tui::dashboard::ui::AgentWidgetView {
        target: app.monitor_target_alias().unwrap_or("-").to_string(),
        input: app.agent_input().to_string(),
        status: app.agent_status(),
        steps: timeline.steps().to_vec(),
        started,
        permission: app.agent_permission_label().to_string(),
        history: app.agent_history_count(),
        thread: app.agent_thread().to_vec(),
        focused: matches!(app.mode, Mode::AgentOps),
    };
    crate::tui::dashboard::ui::render_dashboard(
        frame,
        area,
        &profile,
        &mut app.dashboard_widgets,
        focused_widget_from_mode(app.mode),
        app.term_session.as_ref(),
        Some(&agent_view),
    );
}

/// 当前操作面板对应的 Dashboard 模块（打开面板时该模块高亮）
fn focused_widget_from_mode(mode: Mode) -> Option<crate::tui::dashboard::WidgetId> {
    use crate::tui::dashboard::WidgetId;
    match mode {
        Mode::DockerOps | Mode::DockerConfirm => Some(WidgetId::Docker),
        Mode::ServiceOps | Mode::ServiceConfirm => Some(WidgetId::Services),
        Mode::FileOps | Mode::FileConfirm => Some(WidgetId::Files),
        Mode::LogOps | Mode::LogFilter => Some(WidgetId::Logs),
        Mode::AgentOps | Mode::AgentConfig | Mode::AgentConfirm => Some(WidgetId::Agent),
        Mode::Terminal => Some(WidgetId::Terminal),
        _ => None,
    }
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

/// `/` 主机搜索悬浮框：输入即时过滤主机列表
fn render_search_popup(frame: &mut Frame, area: Rect, app: &App) {
    use crate::tui::widgets::centered_rect;

    let popup = centered_rect(70, 15, area);
    frame.render_widget(ratatui::widgets::Clear, popup);

    let total = app.hosts.len();
    let matched = app.visible_indices().len();
    let lines: Vec<Line> = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("  / {}", app.input_buffer),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            format!("  匹配 {} / {} 台主机", matched, total),
            Style::default().fg(if matched > 0 {
                Color::Green
            } else {
                Color::Red
            }),
        )),
        Line::from(""),
        Line::from("  Enter 确认    Esc 取消"),
    ];

    let block = Block::default().borders(Borders::ALL).title(" 搜索主机 ");
    frame.render_widget(Paragraph::new(lines).block(block), popup);
}

/// `:` 远程命令输入悬浮框：在选中主机上执行命令
fn render_command_popup(frame: &mut Frame, area: Rect, app: &App) {
    use crate::tui::widgets::centered_rect;

    let popup = centered_rect(70, 18, area);
    frame.render_widget(ratatui::widgets::Clear, popup);

    let target = app
        .selected()
        .and_then(|idx| app.hosts.get(idx))
        .map(|h| h.alias.clone())
        .unwrap_or_else(|| "-".to_string());
    let lines: Vec<Line> = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("  : {}", app.command_input()),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!("  目标主机: {}", target),
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
        Line::from("  Enter 在选中主机上执行    Esc 取消"),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" 远程命令执行 ");
    frame.render_widget(Paragraph::new(lines).block(block), popup);
}

/// `:` 远程命令执行结果弹窗
fn render_command_result_popup(frame: &mut Frame, area: Rect, app: &App) {
    use crate::tui::widgets::centered_rect;

    let Some(result) = app.command_result() else {
        return;
    };

    let popup = centered_rect(80, 75, area);
    frame.render_widget(ratatui::widgets::Clear, popup);

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(""));
    if let Some(err) = &result.error {
        lines.push(Line::from(Span::styled(
            format!("  执行失败: {}", err),
            Style::default().fg(Color::Red),
        )));
    } else {
        lines.push(Line::from(Span::styled(
            format!(
                "  退出码: {}",
                result
                    .exit_code
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| "-".to_string())
            ),
            Style::default().fg(Color::Yellow),
        )));
    }
    lines.push(Line::from(""));
    if !result.stdout.is_empty() {
        lines.push(Line::from(Span::styled(
            "  ── stdout ──",
            Style::default().fg(Color::Cyan),
        )));
        for line in result.stdout.lines().take(40) {
            lines.push(Line::from(format!("  {line}")));
        }
        lines.push(Line::from(""));
    }
    if !result.stderr.is_empty() {
        lines.push(Line::from(Span::styled(
            "  ── stderr ──",
            Style::default().fg(Color::Cyan),
        )));
        for line in result.stderr.lines().take(20) {
            lines.push(Line::from(format!("  {line}")));
        }
        lines.push(Line::from(""));
    }
    if result.stdout.is_empty() && result.stderr.is_empty() && result.error.is_none() {
        lines.push(Line::from(Span::styled(
            "  （命令无输出）",
            Style::default().fg(Color::DarkGray),
        )));
    }
    lines.push(Line::from(""));
    lines.push(Line::from("  q/Esc 关闭"));

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" 执行结果 — {}: {} ", result.alias, result.command));
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
    let weight = app.main_split_weight.clamp(20, 80) as u32;
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Ratio(weight, 100),
            Constraint::Ratio(100 - weight, 100),
        ])
        .split(area);

    render_detail(frame, chunks[1], &*app);
    render_host_list(frame, chunks[0], app);
}

fn render_host_list(frame: &mut Frame, area: Rect, app: &mut App) {
    let visible = app.visible_indices();
    let items: Vec<ListItem> = visible
        .iter()
        .map(|&i| {
            let host = &app.hosts[i];
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

/// 详情面板：对长文本按显示宽度硬换行（CJK 全角按 2 列，超长单词也折行），
/// 避免 `Paragraph::wrap`（WordWrapper）对无空格长 token 不折行而横向溢出。
fn render_detail(frame: &mut Frame, area: Rect, app: &App) {
    let text = if let Some(idx) = app.selected() {
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

    // 内区宽度（去掉 1 行边框 ×2），按此宽度硬换行
    let inner_width = area.width.saturating_sub(2) as usize;
    if inner_width == 0 {
        return;
    }
    let lines = wrap_detail_lines(&text, inner_width);
    let detail = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title("详情")
            .title_alignment(ratatui::layout::Alignment::Left),
    );
    frame.render_widget(detail, area);
}

/// 把文本按显示宽度硬折行：先按 `\n` 分逻辑行，再对每一行按 Unicode 显示宽度
/// 切分（CJK 全角占 2 列），保证长 token 也被折行而不横向溢出。
fn wrap_detail_lines(text: &str, max_width: usize) -> Vec<Line<'static>> {
    use unicode_width::UnicodeWidthChar;
    use unicode_width::UnicodeWidthStr;

    let mut out = Vec::new();
    for logical in text.split('\n') {
        if logical.width() <= max_width {
            out.push(Line::from(logical.to_string()));
            continue;
        }
        let mut buf = String::new();
        let mut buf_w = 0usize;
        for ch in logical.chars() {
            let w = UnicodeWidthChar::width(ch).unwrap_or(0);
            if !buf.is_empty() && buf_w + w > max_width {
                out.push(Line::from(std::mem::take(&mut buf)));
                buf_w = 0;
            }
            buf.push(ch);
            buf_w += w;
        }
        if !buf.is_empty() {
            out.push(Line::from(buf));
        }
    }
    out
}

fn render_status_bar(frame: &mut Frame, area: Rect, app: &App) {
    let (message, style) = if app.split_drag.is_some() || app.main_split_drag.is_some() {
        (
            "拖动分隔线调整窗口大小（松开保存）".to_string(),
            Style::default().fg(Color::Yellow).bg(Color::DarkGray),
        )
    } else if let Some(flash_message) = &app.flash_message {
        let fg = match flash_message.color.as_str() {
            "green" => Color::Green,
            "red" => Color::Red,
            "yellow" => Color::Yellow,
            _ => Color::White,
        };
        (
            flash_message.message.clone(),
            Style::default().fg(fg).bg(Color::DarkGray),
        )
    } else {
        (
            app.mode.hint(app.view).to_string(),
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

/// Agent 设置弹窗（编辑 agent.json：provider/base_url/model/permission/timeout）
fn render_agent_config_popup(frame: &mut Frame, area: Rect, app: &App) {
    use crate::tui::widgets::centered_rect;

    let Some(form) = app.agent_form.as_ref() else {
        return;
    };

    let popup = centered_rect(70, 55, area);
    frame.render_widget(ratatui::widgets::Clear, popup);

    let title = format!(
        " Agent 设置  |  当前字段: {}  |  Ctrl+S 保存  Esc 取消 ",
        form.active_label()
    );
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(Style::default().bg(Color::Black));

    let inner = block.inner(popup);
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(inner);

    let hint = Paragraph::new(form.footer_hint()).style(Style::default().fg(Color::Gray));

    frame.render_widget(block, popup);
    frame.render_widget(form.field(0), sections[0]);
    frame.render_widget(form.field(1), sections[1]);
    frame.render_widget(form.field(2), sections[2]);
    frame.render_widget(form.field(3), sections[3]);
    frame.render_widget(form.field(4), sections[4]);
    frame.render_widget(hint, sections[5]);
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

    /// 详情面板对超长文本应自适应换行：`wrap_detail_lines` 按显示宽度硬折行，
    /// 长 token（无空格连续串 / CJK）也被折行，拼接后文本完整不丢。
    #[test]
    fn detail_long_text_wraps() {
        use unicode_width::UnicodeWidthStr;

        let long_comment = format!(
            "生产环境服务器，用于部署应用 {}",
            "与数据库集群和缓存节点保持心跳，出现异常时自动告警".repeat(8)
        );
        let detail = format!(
            "别名: demo\n地址: ********\n密钥: (agent)\n认证: 密钥登录\n状态: ● 在线\n注释: {long_comment}"
        );
        let max_width = 24;
        let lines = super::wrap_detail_lines(&detail, max_width);
        assert!(!lines.is_empty(), "wrap_detail_lines 不应产生空结果");

        // 每行显示宽度不得超过 max_width（CJK 全角按 2 列）
        for line in &lines {
            assert!(
                line.width() <= max_width,
                "wrap 后仍有超宽行: {:?}（宽度 {}）",
                line,
                line.width()
            );
        }

        // 拼接后内容完整：去掉断行边界与空白后，头尾都应保留
        let joined = lines.iter().map(|l| l.to_string()).collect::<String>();
        let compact: String = joined.chars().filter(|c| !c.is_whitespace()).collect();
        assert!(compact.contains("生产环境服务器"), "换行后开头丢失");
        assert!(compact.contains("自动告警"), "换行后结尾丢失");

        // 纯 ASCII 长 token 也被折行（原 WordWrapper 不折无空格串）
        let long_ascii = "a".repeat(60);
        let lines = super::wrap_detail_lines(&long_ascii, 24);
        assert!(lines.len() > 1, "长 ASCII token 应被折行");
        assert!(lines.iter().all(|l| l.width() <= 24));
    }

    /// 拖动主机项时，在光标处绘制高对比浮动标签（拖拽反馈）
    #[test]
    fn drag_ghost_follows_cursor() {
        let config = SshConfig {
            hosts: vec![HostBlock {
                alias: "web-01".into(),
                directives: vec![SshDirective::HostName("example.com".into())],
                raw_text: String::new(),
            }],
            preamble: String::new(),
        };
        let mut app = App::new(config, PathBuf::from("unused-test-config"));
        app.list_state.select(Some(0));

        let backend = TestBackend::new(40, 12);
        let mut terminal = Terminal::new(backend).expect("test terminal should initialize");

        // 未拖动：光标处没有浮动标签（不是青色背景）
        terminal
            .draw(|frame| render(frame, &mut app))
            .expect("TUI should render");
        assert_ne!(
            terminal
                .backend()
                .buffer()
                .cell((7, 5))
                .map(|cell| cell.style().bg),
            Some(Some(ratatui::style::Color::Cyan)),
            "未拖动时不应有浮动标签"
        );

        // 拖动中：标签出现在光标处（青色背景，含别名首字符）
        app.host_drag = Some(0);
        app.host_drag_pos = Some((6, 5));
        terminal
            .draw(|frame| render(frame, &mut app))
            .expect("TUI should render");
        let cell = terminal
            .backend()
            .buffer()
            .cell((7, 5))
            .expect("光标处应有单元格");
        assert_eq!(
            cell.style().bg,
            Some(ratatui::style::Color::Cyan),
            "拖动时应绘制浮动标签"
        );
        assert_eq!(cell.symbol(), "w");

        // 贴边：标签变绿并附带「新窗口」提示（松手才开新窗口）
        app.host_drag_outside = true;
        terminal
            .draw(|frame| render(frame, &mut app))
            .expect("TUI should render");
        let cell = terminal
            .backend()
            .buffer()
            .cell((7, 5))
            .expect("光标处应有单元格");
        assert_eq!(
            cell.style().bg,
            Some(ratatui::style::Color::Green),
            "贴边时标签应变为绿色"
        );
    }
}
