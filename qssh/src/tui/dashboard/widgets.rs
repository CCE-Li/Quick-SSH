//! Widget 抽象：`WidgetModule` trait + Mock 数据实现
//!
//! Phase 1 使用 Mock 数据展示 Dashboard 布局骨架；后续 Phase 将每个
//! 模块替换为真实监控数据源（见 `docs/ai-migration-plan.md`）。

use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::prelude::Widget;
use ratatui::style::{Color, Modifier, Style};
use ratatui::symbols;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Cell, LineGauge, Paragraph, Row, Table, Wrap};

use super::config::WidgetId;

/// 布局中的最小渲染单元
pub trait WidgetModule {
    /// 模块标识（用于从配置中查找）
    fn id(&self) -> WidgetId;
    /// 模块标题（默认取 [`WidgetId::title`]）
    fn title(&self) -> &'static str;
    /// 注入最新一次采集的监控快照（无数据时传 `None`）
    fn set_snapshot(&mut self, snapshot: Option<&crate::monitor::snapshot::ServerSnapshot>);
    /// 标记是否聚焦（操作面板打开时高亮边框）
    fn set_focused(&mut self, focused: bool);
    /// 渲染到给定屏幕区域
    fn render(&self, area: Rect, buf: &mut Buffer);
}

/// Widget：优先消费真实监控快照，缺失时回退到 Mock 数据
pub struct MockWidget {
    id: WidgetId,
    lines: Vec<&'static str>,
    snapshot: Option<crate::monitor::snapshot::ServerSnapshot>,
    focused: bool,
}

impl MockWidget {
    fn new(id: WidgetId) -> Self {
        Self {
            id,
            lines: mock_lines(id),
            snapshot: None,
            focused: false,
        }
    }
}

/// 指标类模块的进度条规格（CPU / 内存）
struct GaugeSpec {
    /// 进度比（0.0 - 1.0）
    ratio: f64,
    /// 进度条标签
    label: String,
    /// 被进度条替代的文本行下标（渲染时跳过，避免重复展示）
    text_index: usize,
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

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        let block = self.block();
        // 表格类模块（磁盘/容器/进程/服务/文件）：列对齐 + 表头
        if let Some(table) = self.table() {
            table.block(block).render(area, buf);
            return;
        }
        let inner = block.inner(area);
        block.render(area, buf);
        if inner.width == 0 || inner.height == 0 {
            return;
        }
        // 指标类模块（CPU/内存）：首行进度条，其余文本行
        if let Some(gauge) = self.gauge() {
            let rows = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).split(inner);
            LineGauge::default()
                .ratio(gauge.ratio)
                .label(gauge.label)
                .line_set(symbols::line::THICK)
                .filled_style(Style::default().fg(Color::Cyan))
                .render(rows[0], buf);
            let lines: Vec<Line> = self
                .render_lines()
                .into_iter()
                .enumerate()
                .filter(|(idx, _)| *idx != gauge.text_index)
                .map(|(_, line)| line)
                .collect();
            Paragraph::new(lines)
                .wrap(Wrap { trim: true })
                .render(rows[1], buf);
            return;
        }
        let lines = self.render_lines();
        Paragraph::new(lines)
            .wrap(Wrap { trim: true })
            .render(inner, buf);
    }
}

impl MockWidget {
    /// 模块外框（聚焦高亮；容器/服务模块标题附带运行数汇总）
    fn block(&self) -> Block<'static> {
        let border_style = if self.focused {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(self.dynamic_title())
            .border_style(border_style)
    }

    /// 标题：容器 / 服务模块附带运行数汇总
    fn dynamic_title(&self) -> String {
        let Some(snap) = self.snapshot.as_ref() else {
            return self.title().to_string();
        };
        match self.id {
            WidgetId::Docker if !snap.docker.is_empty() => {
                let running = snap.docker.iter().filter(|c| c.running).count();
                format!(
                    "{} — {} running / {} total",
                    self.title(),
                    running,
                    snap.docker.len()
                )
            }
            WidgetId::Services if !snap.services.is_empty() => {
                let running = snap.services.iter().filter(|s| s.is_running()).count();
                format!(
                    "{} — {} running / {} total",
                    self.title(),
                    running,
                    snap.services.len()
                )
            }
            WidgetId::Files if !snap.files.is_empty() => {
                format!("{} — {}", self.title(), snap.file_cwd)
            }
            _ => self.title().to_string(),
        }
    }

    /// 表格类模块（磁盘/容器/进程/服务/文件）：有真实数据时返回带表头的 Table
    fn table(&self) -> Option<Table<'static>> {
        let snap = self.snapshot.as_ref()?;
        if snap.has_fatal_error() {
            return None;
        }
        let header = |cells: Vec<&'static str>| {
            Row::new(cells).style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
        };
        match self.id {
            WidgetId::Disk if !snap.disks.is_empty() => {
                let rows = snap.disks.iter().map(|disk| {
                    Row::new(vec![
                        Cell::from(disk.mount.clone()),
                        Cell::from(format!("{:.1} GB", disk.used_gb)),
                        Cell::from(format!("{:.1} GB", disk.total_gb)),
                        Cell::from(format!("{:.0}%", disk.usage_percent)),
                    ])
                });
                Some(
                    Table::new(
                        rows,
                        [
                            Constraint::Percentage(40),
                            Constraint::Percentage(20),
                            Constraint::Percentage(20),
                            Constraint::Percentage(20),
                        ],
                    )
                    .header(header(vec!["挂载点", "已用", "总量", "使用率"])),
                )
            }
            WidgetId::Docker if !snap.docker.is_empty() => {
                let rows = snap.docker.iter().take(8).map(|container| {
                    let (icon, color) = if container.running {
                        ("●", Color::Green)
                    } else {
                        ("○", Color::Red)
                    };
                    Row::new(vec![
                        Cell::from(Span::styled(icon, Style::default().fg(color))),
                        Cell::from(container.name.clone()),
                        Cell::from(container.status.clone()),
                        Cell::from(container.image.clone()),
                    ])
                });
                Some(
                    Table::new(
                        rows,
                        [
                            Constraint::Length(3),
                            Constraint::Percentage(30),
                            Constraint::Percentage(30),
                            Constraint::Percentage(40),
                        ],
                    )
                    .header(header(vec!["", "名称", "状态", "镜像"])),
                )
            }
            WidgetId::Process if !snap.processes.is_empty() => {
                let rows = snap.processes.iter().map(|process| {
                    Row::new(vec![
                        Cell::from(process.name.clone()),
                        Cell::from(process.pid.to_string()),
                        Cell::from(format!("{:.1}%", process.cpu_percent)),
                        Cell::from(format!("{:.1} MB", process.mem_mb)),
                    ])
                });
                Some(
                    Table::new(
                        rows,
                        [
                            Constraint::Percentage(45),
                            Constraint::Percentage(15),
                            Constraint::Percentage(20),
                            Constraint::Percentage(20),
                        ],
                    )
                    .header(header(vec!["进程", "PID", "CPU", "内存"])),
                )
            }
            WidgetId::Services if !snap.services.is_empty() => {
                let rows = snap.services.iter().take(10).map(|service| {
                    let (icon, color) = if service.active == "failed" {
                        ("○", Color::Red)
                    } else if service.is_running() {
                        ("●", Color::Green)
                    } else {
                        ("○", Color::DarkGray)
                    };
                    let status = if service.active == "failed" {
                        "failed"
                    } else if service.is_running() {
                        "running"
                    } else {
                        service.sub.as_str()
                    };
                    Row::new(vec![
                        Cell::from(Span::styled(icon, Style::default().fg(color))),
                        Cell::from(service.name.clone()),
                        Cell::from(status.to_string()).style(Style::default().fg(color)),
                    ])
                });
                Some(
                    Table::new(
                        rows,
                        [
                            Constraint::Length(3),
                            Constraint::Percentage(60),
                            Constraint::Percentage(40),
                        ],
                    )
                    .header(header(vec!["", "服务", "状态"])),
                )
            }
            WidgetId::Files if !snap.files.is_empty() => {
                let rows = crate::monitor::files::dir_entries(snap.files.to_vec())
                    .into_iter()
                    .take(10)
                    .map(|file| {
                        let (icon, size) = if file.is_dir {
                            ("📁", "-".to_string())
                        } else {
                            ("📄", crate::monitor::files::format_file_size(file.size))
                        };
                        Row::new(vec![
                            Cell::from(icon),
                            Cell::from(file.name.clone()),
                            Cell::from(size),
                            Cell::from(file.mtime.clone()),
                        ])
                    });
                Some(
                    Table::new(
                        rows,
                        [
                            Constraint::Length(3),
                            Constraint::Percentage(50),
                            Constraint::Percentage(20),
                            Constraint::Percentage(30),
                        ],
                    )
                    .header(header(vec!["", "名称", "大小", "修改时间"])),
                )
            }
            _ => None,
        }
    }

    /// 指标类模块的进度条规格
    fn gauge(&self) -> Option<GaugeSpec> {
        let snap = self.snapshot.as_ref()?;
        if snap.has_fatal_error() {
            return None;
        }
        match self.id {
            WidgetId::Cpu => snap.cpu.as_ref().map(|cpu| GaugeSpec {
                ratio: (cpu.usage_percent as f64 / 100.0).clamp(0.0, 1.0),
                label: format!(" Usage {:.1}% ", cpu.usage_percent),
                text_index: 0,
            }),
            WidgetId::Memory => snap.memory.as_ref().map(|mem| GaugeSpec {
                ratio: (mem.usage_percent as f64 / 100.0).clamp(0.0, 1.0),
                label: format!(
                    " Used {:.1} / {:.1} GB ({:.0}%) ",
                    mem.used_gb, mem.total_gb, mem.usage_percent
                ),
                text_index: 1,
            }),
            _ => None,
        }
    }

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
            WidgetId::Network => {
                let mut lines = snap
                    .network
                    .as_ref()
                    .map(render_network)
                    .unwrap_or_default();
                if let Some(conns) = &snap.conn_counts {
                    lines.extend(render_conn_counts(conns));
                }
                lines
            }
            WidgetId::SystemInfo => snap.system.as_ref().map(render_system).unwrap_or_default(),
            WidgetId::Process => render_processes(&snap.processes),
            WidgetId::Docker => render_docker(&snap.docker),
            WidgetId::Services => render_services(&snap.services),
            WidgetId::Files => render_files(&snap.files, &snap.file_cwd),
            WidgetId::Logs => render_logs(&snap.logs),
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

/// TCP 连接状态统计行（Phase 5）
fn render_conn_counts(conns: &crate::monitor::network::ConnCounts) -> Vec<String> {
    let health = if conns.is_healthy() {
        "正常".to_string()
    } else {
        "⚠ CLOSE_WAIT 堆积".to_string()
    };
    vec![
        format!(
            "Conn: LISTEN {} ESTAB {} TW {}",
            conns.listening, conns.established, conns.time_wait
        ),
        format!(
            "      CLOSE_WAIT {} FIN {} SYN {}  [{}]",
            conns.close_wait, conns.fin_wait, conns.syn_sent, health
        ),
    ]
}

fn render_system(sys: &crate::monitor::snapshot::SystemInfo) -> Vec<String> {
    vec![
        format!("OS: {}", sys.os),
        format!("Host: {}", sys.hostname),
        format!("Uptime: {}s", sys.uptime_secs),
    ]
}

fn render_services(services: &[crate::monitor::services::ServiceInfo]) -> Vec<String> {
    if services.is_empty() {
        return vec!["systemd 不可用或无服务".to_string()];
    }
    let running = services.iter().filter(|s| s.is_running()).count();
    let mut lines = vec![format!(
        "Services: {} running / {} total",
        running,
        services.len()
    )];
    for service in services.iter().take(10) {
        let marker = if service.is_running() { "●" } else { "○" };
        let status = if service.active == "failed" {
            "failed"
        } else if service.is_running() {
            "running"
        } else {
            service.sub.as_str()
        };
        lines.push(format!("{marker} {:<24} {:>10}", service.name, status));
    }
    lines
}

fn render_docker(containers: &[crate::monitor::docker::DockerContainer]) -> Vec<String> {
    if containers.is_empty() {
        return vec!["Docker 未安装或不可用".to_string()];
    }
    let running = containers.iter().filter(|c| c.running).count();
    let mut lines = vec![format!(
        "Containers: {} running / {} total",
        running,
        containers.len()
    )];
    for container in containers.iter().take(8) {
        let marker = if container.running { "●" } else { "○" };
        lines.push(format!(
            "{marker} {:<16} {:>12}  {}",
            container.name, container.status, container.image
        ));
    }
    lines
}

/// 远程文件列表（Phase 6）
fn render_files(files: &[crate::monitor::files::RemoteFile], cwd: &str) -> Vec<String> {
    if files.is_empty() {
        return vec![format!("Files: {cwd} — 目录为空或 ls 不可用")];
    }
    let mut lines = vec![format!("Files: {cwd}  ({} 项)", files.len())];
    for file in crate::monitor::files::dir_entries(files.to_vec())
        .into_iter()
        .take(10)
    {
        let marker = if file.is_dir { "d" } else { "-" };
        let size = if file.is_dir {
            "-".to_string()
        } else {
            crate::monitor::files::format_file_size(file.size)
        };
        lines.push(format!(
            "{marker} {:<20} {:>8} {}",
            file.name, size, file.mtime
        ));
    }
    lines
}

/// 系统日志列表（Phase 7）
fn render_logs(logs: &[crate::monitor::logs::LogEntry]) -> Vec<String> {
    if logs.is_empty() {
        return vec!["journalctl 不可用或无日志".to_string()];
    }
    logs.iter()
        .take(8)
        .map(|log| format!("{} {} {}", log.timestamp, log.unit, log.message))
        .collect()
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
            "AI Agent 已接入（工具调用 + 权限）",
            "Ctrl+A / Ctrl+Space 打开 Agent 输入",
            "面板内按 C 打开设置",
        ],
        WidgetId::Terminal => vec![
            "嵌入式终端（嵌入式 SSH）",
            "按 Enter 连接到所选主机",
            "Ctrl+Shift+C 退出终端模式",
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

    /// 把 widget 渲染到内存缓冲并返回全部单元格文本
    fn render_to_text(widget: &MockWidget, width: u16, height: u16) -> String {
        let area = Rect::new(0, 0, width, height);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf);
        buf.content().iter().map(|cell| cell.symbol()).collect()
    }

    /// 磁盘模块用 Table 渲染：含中文表头、挂载点与使用率列
    #[test]
    fn disk_widget_renders_table_with_header() {
        let mut widget = mock_widget(WidgetId::Disk);
        let mut snap = crate::monitor::snapshot::ServerSnapshot::new("srv");
        snap.disks = vec![crate::monitor::snapshot::DiskInfo {
            mount: "/data".into(),
            used_gb: 10.0,
            total_gb: 40.0,
            usage_percent: 25.0,
        }];
        widget.set_snapshot(Some(&snap));

        let text = render_to_text(&widget, 50, 6);
        // 宽字符（CJK）占两列、尾随单元格为空白，比较前先去空白
        let compact: String = text.chars().filter(|c| !c.is_whitespace()).collect();
        assert!(compact.contains("挂载点"), "缺少表头: {text}");
        assert!(compact.contains("/data"), "缺少挂载点: {text}");
        assert!(compact.contains("25%"), "缺少使用率: {text}");
    }

    /// CPU 模块：使用率渲染为 LineGauge 进度条，原文本行不再重复展示
    #[test]
    fn cpu_widget_renders_usage_gauge() {
        let mut widget = mock_widget(WidgetId::Cpu);
        let mut snap = crate::monitor::snapshot::ServerSnapshot::new("srv");
        snap.cpu = Some(crate::monitor::snapshot::CpuInfo {
            usage_percent: 42.5,
            load: (1.0, 0.8, 0.6),
            cores: 4,
            freq_ghz: 2.4,
        });
        widget.set_snapshot(Some(&snap));

        let text = render_to_text(&widget, 40, 6);
        assert!(text.contains("Usage 42.5%"), "缺少进度条标签: {text}");
        assert!(
            !text.contains("Usage: 42.5%"),
            "文本行应被进度条替代: {text}"
        );
        assert!(text.contains("Load: 1.00 / 0.80 / 0.60"), "text: {text}");
    }
}
