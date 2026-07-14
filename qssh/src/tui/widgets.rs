use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::tui::app::App;

// ── 自定义 TUI 组件 ──────────────────────────────────────

/// 居中弹窗辅助函数
#[allow(dead_code)]
pub fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            ratatui::layout::Constraint::Percentage((100 - percent_y) / 2),
            ratatui::layout::Constraint::Percentage(percent_y),
            ratatui::layout::Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints([
            ratatui::layout::Constraint::Percentage((100 - percent_x) / 2),
            ratatui::layout::Constraint::Percentage(percent_x),
            ratatui::layout::Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// 渲染确认对话框
#[allow(dead_code)]
pub fn render_confirm_dialog(frame: &mut Frame, area: Rect, title: &str, message: &str) {
    let popup_area = centered_rect(40, 20, area);
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::DarkGray));

    let paragraph = Paragraph::new(message).block(block);
    frame.render_widget(Clear, popup_area);
    frame.render_widget(paragraph, popup_area);
}

/// 渲染帮助弹窗
#[allow(dead_code)]
pub fn render_help_popup(frame: &mut Frame, area: Rect) {
    let popup_area = centered_rect(60, 70, area);
    let help_text = "\
┌─ 键盘快捷键 ──────────────────────┐
│                                    │
│  j / ↓       向下移动              │
│  k / ↑       向上移动              │
│  gg          跳到顶部              │
│  G           跳到底部              │
│  Space       选择/取消选择          │
│  a           字段弹窗添加           │
│  e           字段弹窗编辑           │
│  d           删除主机              │
│  /           搜索                  │
│  p           检测当前主机          │
│  P           检测所有主机          │
│  Enter       连接主机              │
│  q / Esc     退出/取消             │
│  ?           帮助                  │
│                                    │
└────────────────────────────────────┘";

    let block = Block::default()
        .title("帮助")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black));

    let paragraph = Paragraph::new(help_text).block(block);
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
        .style(Style::default().bg(Color::Black));

    let inner = block.inner(popup_area);
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(7),
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

    let hint = Paragraph::new(popup.footer_hint()).style(Style::default().fg(Color::Gray));

    frame.render_widget(Clear, popup_area);
    frame.render_widget(block, popup_area);
    frame.render_widget(popup.field(0), row1[0]);
    frame.render_widget(popup.field(1), row1[1]);
    frame.render_widget(popup.field(2), row2[0]);
    frame.render_widget(popup.field(3), row2[1]);
    frame.render_widget(popup.field(4), sections[2]);
    frame.render_widget(popup.field(5), sections[3]);
    frame.render_widget(hint, sections[4]);
}
