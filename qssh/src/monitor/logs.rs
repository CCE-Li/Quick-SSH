//! 系统日志模型 + `journalctl` 输出解析
//!
//! Phase 7：Logs 日志模块。
//! - 远程命令：`journalctl -o short-iso`（结构化输出）
//! - 解析为 [`LogEntry`]，支持按 unit 筛选与日志限流

use crate::monitor::files::shell_quote;

/// 交互拉取时最大日志行数（日志限流）
pub const MAX_LOG_LINES: usize = 200;

/// 单条系统日志
#[derive(Debug, Clone, PartialEq)]
pub struct LogEntry {
    /// ISO 时间戳（本地时区）
    pub timestamp: String,
    /// 来源（unit 名 / kernel 等）
    pub unit: String,
    /// 日志内容
    pub message: String,
}

/// 从 `journalctl -o short-iso` 输出解析日志列表
///
/// 行格式：`<ISO ts> <host> <unit>: <message>`
pub fn parse_journalctl(text: &str) -> Vec<LogEntry> {
    text.lines()
        .filter_map(|line| {
            let line = line.trim_end();
            if line.is_empty() {
                return None;
            }
            let mut parts = line.splitn(3, ' ');
            let timestamp = parts.next()?.to_string();
            let _host = parts.next()?;
            let rest = parts.next()?;
            // unit 与 message 以 ": " 分界（如 `sshd[1234]: xxx`）
            let (unit, message) = match rest.split_once(": ") {
                Some((u, m)) => (u.to_string(), m.to_string()),
                None => ("system".to_string(), rest.to_string()),
            };
            if message.is_empty() {
                return None;
            }
            Some(LogEntry {
                timestamp,
                unit,
                message,
            })
        })
        .collect()
}

/// 构建 journalctl 命令（支持 unit 筛选 + 日志限流）
pub fn journalctl_command(unit: Option<&str>, tail: usize) -> String {
    let tail = tail.max(1);
    match unit {
        Some(u) if !u.is_empty() => format!(
            "journalctl -n {tail} --no-pager -o short-iso -u {} 2>/dev/null | tail -n {tail}",
            shell_quote(u)
        ),
        _ => format!("journalctl -n {tail} --no-pager -o short-iso 2>/dev/null | tail -n {tail}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_journalctl_output() {
        let out = "2026-08-13T16:00:00.123456+0800 web01 sshd[1234]: Server listening on 0.0.0.0 port 22\n2026-08-13T16:00:01.000001+0800 web01 systemd[1]: Started Session 42 of user root.\n2026-08-13T16:00:02.500000+0800 web01 kernel: audit: type=1400 audit(1755.100:1)";
        let logs = parse_journalctl(out);
        assert_eq!(logs.len(), 3);
        assert_eq!(logs[0].timestamp, "2026-08-13T16:00:00.123456+0800");
        assert_eq!(logs[0].unit, "sshd[1234]");
        assert_eq!(logs[0].message, "Server listening on 0.0.0.0 port 22");
        assert_eq!(logs[1].unit, "systemd[1]");
        assert_eq!(logs[2].unit, "kernel");
    }

    #[test]
    fn empty_input_yields_empty_list() {
        assert!(parse_journalctl("").is_empty());
        assert!(parse_journalctl("   \n\n").is_empty());
    }

    #[test]
    fn journalctl_command_no_unit() {
        let cmd = journalctl_command(None, 50);
        assert!(cmd.contains("-n 50"));
        assert!(cmd.contains("short-iso"));
        assert!(!cmd.contains("-u"));
    }

    #[test]
    fn journalctl_command_with_unit() {
        let cmd = journalctl_command(Some("sshd"), 100);
        assert!(cmd.contains("-u 'sshd'"));
        assert!(cmd.contains("-n 100"));
    }

    #[test]
    fn journalctl_command_tail_is_at_least_one() {
        let cmd = journalctl_command(None, 0);
        assert!(cmd.contains("-n 1"));
    }

    #[test]
    fn journalctl_command_empty_unit_falls_back() {
        let cmd = journalctl_command(Some(""), 20);
        assert!(!cmd.contains("-u"));
    }
}
