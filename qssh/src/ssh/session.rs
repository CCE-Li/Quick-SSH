use std::path::PathBuf;

use crate::config::types::{self, HostBlock, SshConfig};

// ── SSH 连接目标（已解析） ───────────────────────────────

#[derive(Debug, Clone)]
pub struct SshTarget {
    pub alias: String,
    pub hostname: String,
    pub user: Option<String>,
    pub port: u16,
    pub identity_file: Option<PathBuf>,
}

impl SshTarget {
    /// 从 HostBlock 构建
    pub fn from_host(host: &HostBlock) -> Self {
        Self {
            alias: host.alias.clone(),
            hostname: host.hostname().unwrap_or(&host.alias).to_string(),
            user: host.user().map(|s| s.to_string()),
            port: host.port(),
            identity_file: host.identity_file().cloned(),
        }
    }

    /// 从 user@hostname 字符串解析
    pub fn from_user_at_host(input: &str, port: u16) -> Self {
        let (user, hostname) = if let Some((u, h)) = input.split_once('@') {
            (Some(u.to_string()), h.to_string())
        } else {
            (None, input.to_string())
        };

        Self {
            alias: hostname.clone(),
            hostname,
            user,
            port,
            identity_file: None,
        }
    }

    /// 构建 SSH 命令行参数
    pub fn build_ssh_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        if let Some(ref user) = self.user {
            args.push("-l".into());
            args.push(user.clone());
        }

        if self.port != 22 {
            args.push("-p".into());
            args.push(self.port.to_string());
        }

        if let Some(ref key) = self.identity_file {
            let key_str = key.display().to_string();
            let expanded = shellexpand::full(&key_str)
                .map(|c| c.to_string())
                .unwrap_or(key_str);
            args.push("-i".into());
            args.push(expanded);
        }

        // 禁用 SSH 连接复用检测，确保每次都建立新连接
        args.push("-o".into());
        args.push("ControlMaster=no".into());

        args.push(self.hostname.clone());
        args
    }
}

/// 解析连接目标：优先作为别名查找，否则视为 user@host
pub fn resolve_target(config: &SshConfig, input: &str) -> SshTarget {
    if let Some(host) = types::find_host(config, input) {
        SshTarget::from_host(host)
    } else {
        SshTarget::from_user_at_host(input, 22)
    }
}
