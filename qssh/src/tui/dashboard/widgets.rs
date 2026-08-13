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
    /// 注入最新一次采集的监控快照（无数据时传 `None`）
    fn set_snapshot(&mut self, snapshot: Option<&crate::monitor::snapshot::ServerSnapshot>);
    /// 渲染到给定屏幕区域
    fn render(&self, area: Rect, buf: &mut Buffer);
}

/// Widget：优先消费真实监控快照，缺失时回退到 Mock 数据
pub struct MockWidget {
    id: WidgetId,
    lines: Vec<&'static str>,
    snapshot: Option<crate::monitor::snapshot::ServerSnapshot>,
}

impl MockWidget {
    fn new(id: WidgetId) -> Self {
        Self {
            id,
            lines: mock_lines(id),
            snapshot: None,
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

    fn set_snapshot(&mut self, snapshot: Option<&crate::monitor::snapshot::ServerSnapshot>) {
        self.snapshot = snapshot.cloned();
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::default().borders(Borders::ALL).title(self.title());
        let lines = self.render_lines();
        let paragraph = Paragraph::new(lines).wrap(Wrap { trim: true }).block(block);
        paragraph.render(area, buf);
    }
}

impl MockWidget {
    /// 根据 WidgetId 选择真实数据渲染；无数据或非监控指标时回退到 Mock
    fn render_lines(&self) -> Vec<Line<'static>> {
        let Some(snap) = &self.snapshot else {
            return self.lines.iter().map(|line| Line::from(*line)).collect();
        };
        // 致命错误：所有 Widget 统一显示错误原因，便于定位问题
        if snap.has_fatal_error() && !snap.warnings.is_empty() {
            return snap
                .warnings
                .iter()
                .map(|w| Line::from(w.clone()))
                .collect();
        }
        let lines: Vec<String> = match self.id {
            WidgetId::Cpu => snap.cpu.as_ref().map(render_cpu).unwrap_or_default(),
            WidgetId::Memory => snap.memory.as_ref().map(render_memory).unwrap_or_default(),
            WidgetId::Disk => {
                if snap.disks.is_empty() {
                    snap.warnings.clone()
                } else {
                    render_disks(&snap.disks)
                }
            }
            WidgetId::Network => snap
                .network
                .as_ref()
                .map(render_network)
                .unwrap_or_default(),
            WidgetId::SystemInfo => snap.system.as_ref().map(render_system).unwrap_or_default(),
            WidgetId::Process => render_processes(&snap.processes),
            // 尚未接入真实数据源的模块：保留 Mock 展示
            _ => self.lines.iter().map(|line| line.to_string()).collect(),
        };
        if lines.is_empty() {
            return vec![Line::from("暂无数据")];
        }
        lines.into_iter().map(Line::from).collect()
    }
}

fn render_cpu(cpu: &crate::monitor::snapshot::CpuInfo) -> Vec<String> {
    let load = format!(
        "Load: {:.2} / {:.2} / {:.2}",
        cpu.load.0, cpu.load.1, cpu.load.2
    );
    vec![
        format!("Usage: {:.1}%", cpu.usage_percent),
        load,
        format!("Cores: {} @ {:.2} GHz", cpu.cores, cpu.freq_ghz),
    ]
}

fn render_memory(mem: &crate::monitor::snapshot::MemoryInfo) -> Vec<String> {
    vec![
        format!("Total: {:.1} GB", mem.total_gb),
        format!("Used: {:.1} GB ({:.0}%)", mem.used_gb, mem.usage_percent),
        format!("Free: {:.1} GB", mem.total_gb - mem.used_gb),
    ]
}

fn render_disks(disks: &[crate::monitor::snapshot::DiskInfo]) -> Vec<String> {
    disks
        .iter()
        .map(|disk| {
            format!(
                "{:<12} {:>7.1} GB / {:>7.1} GB ({:.0}%)",
                disk.mount, disk.used_gb, disk.total_gb, disk.usage_percent
            )
        })
        .collect()
}

fn render_network(net: &crate::monitor::snapshot::NetworkInfo) -> Vec<String> {
    let ip = net.ip.clone().unwrap_or_else(|| "未知".to_string());
    vec![
        format!("RX: {:.2} MB/s", net.rx_mbps),
        format!("TX: {:.2} MB/s", net.tx_mbps),
        format!("IP: {}", ip),
    ]
}

fn render_system(sys: &crate::monitor::snapshot::SystemInfo) -> Vec<String> {
    vec![
        format!("OS: {}", sys.os),
        format!("Host: {}", sys.hostname),
        format!("Uptime: {}s", sys.uptime_secs),
    ]
}

fn render_processes(processes: &[crate::monitor::snapshot::ProcessInfo]) -> Vec<String> {
    processes
        .iter()
        .map(|proc| {
            format!(
                "{:<16} {:>7} {:>6.1}% {:>8.1} MB",
                proc.name, proc.pid, proc.cpu_percent, proc.mem_mb
            )
        })
        .collect()
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

    #[test]
    fn cpu_widget_renders_real_snapshot_data() {
        let mut widget = mock_widget(WidgetId::Cpu);
        let mut snap = crate::monitor::snapshot::ServerSnapshot::new("srv");
        snap.cpu = Some(crate::monitor::snapshot::CpuInfo {
            usage_percent: 42.5,
            load: (1.0, 0.8, 0.6),
            cores: 4,
            freq_ghz: 2.4,
        });
        widget.set_snapshot(Some(&snap));
        let lines = widget.render_lines();
        let text: String = lines
            .iter()
            .map(|line| line.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(text.contains("42.5%"), "text: {text}");
        assert!(text.contains("1.00 / 0.80 / 0.60"), "text: {text}");
        assert!(text.contains("4 @ 2.40 GHz"), "text: {text}");
    }

    #[test]
    fn widget_shows_warning_on_fatal_snapshot() {
        let mut widget = mock_widget(WidgetId::Memory);
        let mut snap = crate::monitor::snapshot::ServerSnapshot::new("srv");
        snap.warnings = vec!["连接超时".to_string()];
        widget.set_snapshot(Some(&snap));
        let lines = widget.render_lines();
        let text: String = lines
            .iter()
            .map(|line| line.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(text.contains("连接超时"), "text: {text}");
    }
}
