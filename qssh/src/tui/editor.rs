use std::path::PathBuf;

use anyhow::{anyhow, bail};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders};
use tui_textarea::TextArea;

use crate::config::types::{HostBlock, SshDirective};

const FIELD_ALIAS: usize = 0;
const FIELD_HOSTNAME: usize = 1;
const FIELD_USER: usize = 2;
const FIELD_PORT: usize = 3;
const FIELD_IDENTITY: usize = 4;
const FIELD_EXTRA: usize = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorOutcome {
    Continue,
    Save,
    Cancel,
}

#[derive(Debug, Clone)]
pub enum HostFormMode {
    Add,
    Edit {
        index: usize,
        original_alias: String,
    },
}

struct FormField {
    label: &'static str,
    multiline: bool,
    textarea: TextArea<'static>,
}

impl FormField {
    fn new(label: &'static str, value: &str, multiline: bool) -> Self {
        let lines = if multiline {
            split_lines_preserve(value)
        } else {
            vec![value.replace(['\r', '\n'], " ")]
        };

        let mut textarea = TextArea::from(lines);
        textarea.set_style(Style::default().fg(Color::White));
        textarea.set_cursor_line_style(Style::default());
        textarea.set_cursor_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::LightCyan)
                .add_modifier(Modifier::BOLD),
        );

        Self {
            label,
            multiline,
            textarea,
        }
    }

    fn text(&self) -> String {
        self.textarea.lines().join("\n")
    }

    fn set_active(&mut self, active: bool) {
        let title = if active {
            format!("> {}", self.label)
        } else {
            self.label.to_string()
        };
        let border = if active {
            Color::LightCyan
        } else {
            Color::DarkGray
        };
        let cursor_line_style = if active && self.multiline {
            Style::default().bg(Color::DarkGray)
        } else {
            Style::default()
        };

        self.textarea.set_block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border)),
        );
        self.textarea.set_cursor_line_style(cursor_line_style);
    }
}

pub struct HostFormState {
    mode: HostFormMode,
    fields: Vec<FormField>,
    active_field: usize,
}

impl HostFormState {
    pub fn new_add() -> Self {
        let mut state = Self {
            mode: HostFormMode::Add,
            fields: vec![
                FormField::new("Host", "", false),
                FormField::new("HostName", "", false),
                FormField::new("User", "", false),
                FormField::new("Port", "", false),
                FormField::new("IdentityFile", &default_identity_file_value(), false),
                FormField::new("其他指令 / 注释", "", true),
            ],
            active_field: 0,
        };
        state.refresh_field_styles();
        state
    }

    pub fn new_edit(index: usize, host: &HostBlock) -> Self {
        let port_value = host
            .directives
            .iter()
            .find_map(|directive| match directive {
                SshDirective::Port(port) => Some(port.to_string()),
                _ => None,
            })
            .unwrap_or_default();

        let identity_value = host
            .identity_file()
            .map(|path| path.display().to_string())
            .unwrap_or_default();

        let mut state = Self {
            mode: HostFormMode::Edit {
                index,
                original_alias: host.alias.clone(),
            },
            fields: vec![
                FormField::new("Host", &host.alias, false),
                FormField::new("HostName", host.hostname().unwrap_or(""), false),
                FormField::new("User", host.user().unwrap_or(""), false),
                FormField::new("Port", &port_value, false),
                FormField::new("IdentityFile", &identity_value, false),
                FormField::new("其他指令 / 注释", &extract_extra_lines(host), true),
            ],
            active_field: 0,
        };
        state.refresh_field_styles();
        state
    }

    pub fn mode(&self) -> &HostFormMode {
        &self.mode
    }

    pub fn title(&self) -> String {
        match &self.mode {
            HostFormMode::Add => "新增主机".to_string(),
            HostFormMode::Edit { original_alias, .. } => format!("编辑主机 {}", original_alias),
        }
    }

    pub fn footer_hint(&self) -> &'static str {
        "Tab 切换字段，Enter 跳到下一项，最后一栏支持多行，Ctrl+S 保存，Esc 取消"
    }

    pub fn active_label(&self) -> &str {
        self.fields[self.active_field].label
    }

    pub fn field(&self, index: usize) -> &TextArea<'static> {
        &self.fields[index].textarea
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> EditorOutcome {
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('s') => return EditorOutcome::Save,
                _ => {}
            }
        }

        match key.code {
            KeyCode::Esc => EditorOutcome::Cancel,
            KeyCode::Tab | KeyCode::BackTab => {
                self.cycle_field(!matches!(key.code, KeyCode::BackTab));
                EditorOutcome::Continue
            }
            KeyCode::Enter if !self.fields[self.active_field].multiline => {
                self.cycle_field(true);
                EditorOutcome::Continue
            }
            _ => {
                self.fields[self.active_field].textarea.input(key);
                EditorOutcome::Continue
            }
        }
    }

    pub fn build_host(&self) -> anyhow::Result<HostBlock> {
        let alias = self.field_text(FIELD_ALIAS).trim().to_string();
        if alias.is_empty() {
            bail!("Host 别名不能为空");
        }

        let hostname = self.field_text(FIELD_HOSTNAME).trim().to_string();
        let user = self.field_text(FIELD_USER).trim().to_string();
        let port_text = self.field_text(FIELD_PORT).trim().to_string();
        let identity_text = self.field_text(FIELD_IDENTITY).trim().to_string();

        let port = if port_text.is_empty() {
            None
        } else {
            Some(
                port_text
                    .parse::<u16>()
                    .map_err(|_| anyhow!("Port 必须是 1-65535 的整数"))?,
            )
        };

        let mut directives = Vec::new();
        let mut raw_lines = vec![format!("Host {}", alias)];

        if !hostname.is_empty() {
            directives.push(SshDirective::HostName(hostname.clone()));
            raw_lines.push(format!("    HostName {}", hostname));
        }

        if !user.is_empty() {
            directives.push(SshDirective::User(user.clone()));
            raw_lines.push(format!("    User {}", user));
        }

        if let Some(port) = port {
            directives.push(SshDirective::Port(port));
            raw_lines.push(format!("    Port {}", port));
        }

        if !identity_text.is_empty() {
            let expanded = shellexpand::full(&identity_text)
                .map_err(|err| anyhow!("IdentityFile 路径无效: {}", err))?;
            directives.push(SshDirective::IdentityFile(PathBuf::from(expanded.as_ref())));
            raw_lines.push(format!("    IdentityFile {}", identity_text));
        }

        for line in self.fields[FIELD_EXTRA].textarea.lines() {
            let normalized = line.trim_end_matches('\r');
            let trimmed = normalized.trim();

            if trimmed.is_empty() {
                raw_lines.push(String::new());
                continue;
            }

            if trimmed.starts_with('#') {
                raw_lines.push(format!("    {}", trimmed));
                continue;
            }

            let key = trimmed
                .split_whitespace()
                .next()
                .ok_or_else(|| anyhow!("无法解析其他指令"))?;

            if is_managed_directive(key) {
                bail!("请通过专门字段修改 {}，不要写在“其他指令”里", key);
            }

            raw_lines.push(format!("    {}", trimmed));
            if let Some((directive_key, directive_value)) = trimmed.split_once(char::is_whitespace)
            {
                directives.push(SshDirective::Unknown(
                    directive_key.to_string(),
                    directive_value.trim().to_string(),
                ));
            } else {
                directives.push(SshDirective::Unknown(trimmed.to_string(), String::new()));
            }
        }

        Ok(HostBlock {
            alias,
            directives,
            raw_text: raw_lines.join("\n"),
        })
    }

    fn cycle_field(&mut self, forward: bool) {
        if forward {
            self.active_field = (self.active_field + 1) % self.fields.len();
        } else if self.active_field == 0 {
            self.active_field = self.fields.len() - 1;
        } else {
            self.active_field -= 1;
        }
        self.refresh_field_styles();
    }

    fn refresh_field_styles(&mut self) {
        for (index, field) in self.fields.iter_mut().enumerate() {
            field.set_active(index == self.active_field);
        }
    }

    fn field_text(&self, index: usize) -> String {
        self.fields[index].text()
    }
}

fn extract_extra_lines(host: &HostBlock) -> String {
    if !host.raw_text.trim().is_empty() {
        let mut extras = Vec::new();
        for (index, line) in host.raw_text.lines().enumerate() {
            if index == 0 {
                continue;
            }

            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                extras.push(trimmed.to_string());
                continue;
            }

            let key = trimmed.split_whitespace().next().unwrap_or("");
            if !is_managed_directive(key) {
                extras.push(trimmed.to_string());
            }
        }
        return extras.join("\n");
    }

    host.directives
        .iter()
        .filter_map(|directive| match directive {
            SshDirective::Unknown(key, value) if value.is_empty() => Some(key.clone()),
            SshDirective::Unknown(key, value) => Some(format!("{} {}", key, value)),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn is_managed_directive(key: &str) -> bool {
    matches!(
        key.to_ascii_lowercase().as_str(),
        "host" | "hostname" | "user" | "port" | "identityfile"
    )
}

fn split_lines_preserve(text: &str) -> Vec<String> {
    let mut lines: Vec<String> = text
        .split('\n')
        .map(|line| line.trim_end_matches('\r').to_string())
        .collect();
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

fn default_identity_file_value() -> String {
    let home = dirs::home_dir()
        .or_else(|| std::env::var_os("USERPROFILE").map(PathBuf::from))
        .or_else(|| std::env::var_os("HOME").map(PathBuf::from));

    home.map(|path| path.join(".ssh").join("id_rsa").display().to_string())
        .unwrap_or_else(|| {
            if cfg!(windows) {
                r"C:\Users\<user>\.ssh\id_rsa".to_string()
            } else {
                "~/.ssh/id_rsa".to_string()
            }
        })
}

#[cfg(test)]
mod tests {
    use super::{default_identity_file_value, HostFormState, FIELD_EXTRA};
    use crate::config::types::{HostBlock, SshDirective};
    use tui_textarea::TextArea;

    #[test]
    fn rebuilds_host_from_form_fields() {
        let host = HostBlock {
            alias: "demo".into(),
            directives: vec![
                SshDirective::HostName("example.com".into()),
                SshDirective::User("root".into()),
                SshDirective::Port(2222),
                SshDirective::Unknown("ProxyJump".into(), "bastion".into()),
            ],
            raw_text: "Host demo\n    HostName example.com\n    User root\n    Port 2222\n    ProxyJump bastion\n    # keep me".into(),
        };

        let popup = HostFormState::new_edit(0, &host);
        let rebuilt = popup.build_host().expect("host should rebuild");

        assert_eq!(rebuilt.alias, "demo");
        assert_eq!(rebuilt.hostname(), Some("example.com"));
        assert_eq!(rebuilt.user(), Some("root"));
        assert_eq!(rebuilt.port(), 2222);
        assert!(rebuilt.raw_text.contains("ProxyJump bastion"));
        assert!(rebuilt.raw_text.contains("# keep me"));
    }

    #[test]
    fn rejects_managed_directive_inside_extra_field() {
        let host = HostBlock {
            alias: "demo".into(),
            directives: vec![],
            raw_text: "Host demo".into(),
        };

        let mut popup = HostFormState::new_edit(0, &host);
        popup.fields[FIELD_EXTRA].textarea = TextArea::from(["HostName another.example.com"]);

        let err = popup
            .build_host()
            .expect_err("managed directive should fail");
        assert!(err.to_string().contains("专门字段"));
    }

    #[test]
    fn add_form_prefills_identity_file_with_platform_default() {
        let form = HostFormState::new_add();
        let identity_value = form.field(4).lines().join("\n");

        assert_eq!(identity_value, default_identity_file_value());
        assert!(
            identity_value.ends_with(".ssh\\id_rsa") || identity_value.ends_with(".ssh/id_rsa")
        );
    }
}
