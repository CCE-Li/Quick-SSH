use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

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
│  a           添加主机              │
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
