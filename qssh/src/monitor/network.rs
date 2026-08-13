//! 网络连接统计模型 + `ss` 输出解析
//!
//! Phase 5：Network 模块增强。
//! - 远程命令：`ss -tan state all`（结构化统计各 TCP 状态连接数）
//! - 解析为 [`ConnCounts`]，随 `ServerSnapshot` 一起采集展示

/// TCP 连接状态统计
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ConnCounts {
    /// LISTEN 监听数
    pub listening: u32,
    /// ESTAB 已建立数
    pub established: u32,
    /// TIME_WAIT 数
    pub time_wait: u32,
    /// CLOSE_WAIT 数
    pub close_wait: u32,
    /// FIN_WAIT1/2 数
    pub fin_wait: u32,
    /// SYN_SENT 数
    pub syn_sent: u32,
    /// 全部状态连接总数
    pub total: u32,
}

impl ConnCounts {
    /// 是否为健康的连接状态（无大量 CLOSE_WAIT / 异常堆积）
    pub fn is_healthy(&self) -> bool {
        self.close_wait < 100
    }
}

/// 从 `ss -tan state all` 输出解析连接状态计数。
///
/// 期望命令：
/// `ss -tan state all | awk 'NR>1 {print $1}' | sort | uniq -c | awk '{print $2"|"$1}'`
pub fn parse_ss_tan(text: &str) -> ConnCounts {
    let mut counts = ConnCounts::default();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let mut parts = line.split('|');
        let state = parts.next().unwrap_or_default().trim();
        let count: u32 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
        match state {
            "LISTEN" => counts.listening = count,
            "ESTAB" => counts.established = count,
            "TIME-WAIT" => counts.time_wait = count,
            "CLOSE-WAIT" => counts.close_wait = count,
            "FIN-WAIT-1" | "FIN-WAIT-2" => counts.fin_wait = count,
            "SYN-SENT" => counts.syn_sent = count,
            _ => {}
        }
        counts.total += count;
    }
    counts
}

/// 构建 `ss -tan` 连接统计采集命令（结构化、`|` 分隔）
///
/// 预留独立采集 API（后续拆 crate / 按需调用时使用）
#[allow(dead_code)]
pub fn ss_connections_command() -> &'static str {
    "ss -tan state all | awk 'NR>1 {print $1}' | sort | uniq -c | awk '{print $2\"|\"$1}'"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_empty_output() {
        let counts = parse_ss_tan("");
        assert_eq!(counts.total, 0);
        assert!(counts.is_healthy());
    }

    #[test]
    fn parses_connection_states() {
        let out = "LISTEN|5\nESTAB|23\nTIME-WAIT|12\nCLOSE-WAIT|3\nFIN-WAIT-1|2\nSYN-SENT|1";
        let counts = parse_ss_tan(out);
        assert_eq!(counts.listening, 5);
        assert_eq!(counts.established, 23);
        assert_eq!(counts.time_wait, 12);
        assert_eq!(counts.close_wait, 3);
        assert_eq!(counts.fin_wait, 2);
        assert_eq!(counts.syn_sent, 1);
        assert_eq!(counts.total, 46);
        assert!(counts.is_healthy());
    }

    #[test]
    fn unhealthy_when_many_close_wait() {
        let mut counts = ConnCounts::default();
        counts.close_wait = 200;
        assert!(!counts.is_healthy());
    }
}
