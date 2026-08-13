//! Widget 抽象：`WidgetModule` trait + Mock 数据实现
//!
//! Phase 1 使用 Mock 数据展示 Dashboard 布局骨架；后续 Phase 将每个
//! 模块替换为真实监控数据源（见 `docs/ai-migration-plan.md`）。

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::Widget;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use super::config::WidgetId;

/// 布局中的最小渲染单元
pub trait WidgetModule {
    /// 模块标识（用于从配置中查找）
    fn id(&self) -> WidgetId;
    /// 模块标题（默认取 [`WidgetId::title`]）
    fn title(&self) -> &'static str;
    /// 渲染到给定屏幕区域
    fn render(&self, area: Rect, buf: &mut Buffer);
}

/// Mock 数据 Widget：Phase 1 用于展示 Dashboard 布局骨架
pub struct MockWidget {
    id: WidgetId,
    lines: Vec<&'static str>,
}

impl MockWidget {
    fn new(id: WidgetId) -> Self {
        Self {
            id,
            lines: mock_lines(id),
        }
    }
}

impl WidgetModule for MockWidget {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn title(&self) -> &'static str {
        self.id.title()
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::default().borders(Borders::ALL).title(self.title());
        let lines: Vec<Line> = self.lines.iter().map(|line| Line::from(*line)).collect();
        let paragraph = Paragraph::new(lines).wrap(Wrap { trim: true }).block(block);
        paragraph.render(area, buf);
    }
}

/// 生成指定模块的 Mock Widget（用于布局预览）
pub fn mock_widget(id: WidgetId) -> MockWidget {
    MockWidget::new(id)
}

fn mock_lines(id: WidgetId) -> Vec<&'static str> {
    match id {
        WidgetId::Cpu => vec![
            "Usage: 23.5%",
            "Load: 1.42 / 1.10 / 1.05",
            "Cores: 8 @ 3.60 GHz",
        ],
        WidgetId::Memory => vec!["Total: 32.0 GB", "Used: 18.6 GB (58%)", "Free: 13.4 GB"],
        WidgetId::Disk => vec!["/       118 GB / 256 GB", "/data   612 GB / 2.0 TB"],
        WidgetId::Network => vec!["RX: 1.2 MB/s", "TX: 340 KB/s", "eth0: 192.168.1.100"],
        WidgetId::Docker => vec![
            "Containers: 3 running / 4 total",
            "Images: 12 (2.1 GB cache)",
        ],
        WidgetId::Process => vec![
            "ssh       1234   0.2%   12 MB",
            "node      2345   1.1%  240 MB",
            "docker    3456   0.4%   80 MB",
        ],
        WidgetId::Services => vec!["sshd      active", "docker    active", "nginx     active"],
        WidgetId::Files => vec![
            "拖拽文件到输入框可上传（SCP）",
            "最近上传：qssh-uploader 独立工具",
        ],
        WidgetId::Logs => vec![
            "09:12:03 sshd[1112] Accepted publickey",
            "09:14:47 docker[3456] container started",
        ],
        WidgetId::SystemInfo => vec![
            "OS: Linux 6.6 (x86_64)",
            "Host: server-01",
            "Kernel uptime: 12d 4h",
        ],
        WidgetId::Agent => vec![
            "AI Agent 占位（Phase 3 接入）",
            "当前权限模式：READ_ONLY",
            "用法：Ctrl+Space 打开 Agent 输入",
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_widget_matches_id_and_title() {
        let widget = mock_widget(WidgetId::Docker);
        assert_eq!(widget.id(), WidgetId::Docker);
        assert_eq!(widget.title(), "Docker");
    }

    #[test]
    fn all_widgets_have_mock_lines() {
        for id in crate::tui::dashboard::config::ALL_WIDGETS {
            let widget = mock_widget(id);
            assert!(!widget.lines.is_empty(), "missing mock lines for {id:?}");
        }
    }
}
