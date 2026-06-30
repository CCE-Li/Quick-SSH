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
        out.push_str(&host.render());
    }

    out
}
