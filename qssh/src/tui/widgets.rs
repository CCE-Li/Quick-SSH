use ratatui::layout::{Constraint, Direction, Flex, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::tui::app::App;

// ── 自定义 TUI 组件 ──────────────────────────────────────

/// 居中弹窗辅助函数（`Flex::Center` 自动分配两侧剩余空间）
#[allow(dead_code)]
pub fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let rows = Layout::vertical([Constraint::Percentage(percent_y)])
        .flex(Flex::Center)
        .split(area);
    Layout::horizontal([Constraint::Percentage(percent_x)])
        .flex(Flex::Center)
        .split(rows[0])[0]
}

/// 渲染确认对话框
#[allow(dead_code)]
pub fn render_confirm_dialog(frame: &mut Frame, area: Rect, title: &str, message: &str) {
    let popup_area = centered_rect(40, 20, area);
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(Style::default().bg(Color::DarkGray));

    let paragraph = Paragraph::new(message).block(block);
    frame.render_widget(Clear, popup_area);
    frame.render_widget(paragraph, popup_area);
}

/// 帮助弹窗的分组标题
fn help_section(title: &'static str) -> Line<'static> {
    Line::from(Span::styled(
        title,
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    ))
}

/// 渲染帮助弹窗
#[allow(dead_code)]
pub fn render_help_popup(frame: &mut Frame, area: Rect) {
    let popup_area = centered_rect(60, 80, area);
    let help_lines = vec![
        help_section("键盘快捷键"),
        Line::from(""),
        Line::from("  j / ↓       向下移动"),
        Line::from("  k / ↑       向上移动"),
        Line::from("  gg          跳到顶部"),
        Line::from("  G           跳到底部"),
        Line::from("  Space       选择/取消选择"),
        Line::from("  a           字段弹窗添加"),
        Line::from("  e           字段弹窗编辑"),
        Line::from("  d           删除主机"),
        Line::from("  /           搜索"),
        Line::from("  p           检测当前主机"),
        Line::from("  P           检测所有主机"),
        Line::from("  Enter       连接主机"),
        Line::from("  q / Esc     退出/取消"),
        Line::from("  ?           帮助"),
        Line::from(""),
        help_section("鼠标操作"),
        Line::from(""),
        Line::from("  点击主机      选中"),
        Line::from("  滚轮          滚动列表"),
        Line::from("  点击 Widget   打开对应操作面板"),
    ];

    let block = Block::default()
        .title("帮助")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(Style::default().bg(Color::Black));

    let paragraph = Paragraph::new(help_lines).block(block);
    frame.render_widget(Clear, popup_area);
    frame.render_widget(paragraph, popup_area);
}

pub fn render_host_form_popup(frame: &mut Frame, area: Rect, app: &App) {
    let Some(popup) = app.host_form.as_ref() else {
        return;
    };

    let popup_area = centered_rect(78, 72, area);
    let title = format!("{}  |  当前字段: {}", popup.title(), popup.active_label());
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(Style::default().bg(Color::Black));

    let inner = block.inner(popup_area);
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(6),
            Constraint::Length(2),
        ])
        .split(inner);
    let row1 = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
        .split(sections[0]);
    let row2 = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(sections[1]);
    let row3 = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(sections[2]);
    let row4 = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(sections[3]);

    let hint = Paragraph::new(popup.footer_hint()).style(Style::default().fg(Color::Gray));

    frame.render_widget(Clear, popup_area);
    frame.render_widget(block, popup_area);
    frame.render_widget(popup.field(0), row1[0]);
    frame.render_widget(popup.field(1), row1[1]);
    frame.render_widget(popup.field(2), row2[0]);
    frame.render_widget(popup.field(3), row2[1]);
    frame.render_widget(popup.field(4), row3[0]);
    frame.render_widget(popup.field(5), row3[1]);
    frame.render_widget(popup.field(6), row4[0]);
    frame.render_widget(popup.field(7), row4[1]);
    frame.render_widget(hint, sections[4]);
}
