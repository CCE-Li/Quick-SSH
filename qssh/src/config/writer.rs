use super::types::SshConfig;

/// 将 Host 块列表渲染回完整的 SSH 配置文件内容
pub fn render_config(config: &SshConfig) -> String {
    let mut out = String::new();

    if !config.preamble.is_empty() {
        out.push_str(&config.preamble);
        out.push('\n');
    }

    for host in &config.hosts {
        if !out.is_empty() && !out.ends_with('\n') {
            out.push('\n');
        }
        if host.raw_text.trim().is_empty() {
            out.push_str(&host.render());
        } else {
            out.push_str(&host.raw_text);
            if !host.raw_text.ends_with('\n') {
                out.push('\n');
            }
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::types::{HostBlock, SshConfig, SshDirective};

    #[test]
    fn preserves_preamble_and_raw_host_text() {
        let config = SshConfig {
            preamble: "Host *\n    ServerAliveInterval 60".into(),
            hosts: vec![HostBlock {
                alias: "demo".into(),
                directives: vec![SshDirective::HostName("example.com".into())],
                raw_text: "Host demo\n    HostName example.com\n    # keep this comment".into(),
            }],
        };

        let rendered = render_config(&config);

        assert!(rendered.starts_with("Host *\n    ServerAliveInterval 60\n"));
        assert!(rendered.contains("Host demo\n    HostName example.com\n    # keep this comment"));
    }
}
