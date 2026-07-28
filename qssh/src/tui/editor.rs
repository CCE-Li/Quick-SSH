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
const FIELD_PASSWORD: usize = 5;
const FIELD_COMMENT: usize = 6;
const FIELD_EXTRA: usize = 7;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PasswordStorageAction {
    Unchanged,
    Keep,
    Set(String),
    Clear,
}

#[derive(Debug, Clone)]
pub struct HostFormSubmission {
    pub host: HostBlock,
    pub password_action: PasswordStorageAction,
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

    fn password(label: &'static str) -> Self {
        let mut field = Self::new(label, "", false);
        field.textarea.set_mask_char('*');
        field
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
    saved_password_exists: bool,
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
                FormField::password("Password（留空则不保存）"),
                FormField::new("注释（无需输入 #）", "", true),
                FormField::new("其他 SSH 指令", "", true),
            ],
            active_field: 0,
            saved_password_exists: false,
        };
        state.refresh_field_styles();
        state
    }

    pub fn new_edit(index: usize, host: &HostBlock, saved_password_exists: bool) -> Self {
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
                FormField::password(password_label(saved_password_exists)),
                FormField::new("注释（无需输入 #）", &extract_comment_lines(host), true),
                FormField::new("其他 SSH 指令", &extract_extra_lines(host), true),
            ],
            active_field: 0,
            saved_password_exists,
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
        "↑↓/Tab 切换字段，←→移动光标，Enter 下一项，Password 留空保留/不保存，!clear 清除，Ctrl+S 保存，Esc 取消"
    }

    pub fn active_label(&self) -> &str {
        self.fields[self.active_field].label
    }

    pub fn field(&self, index: usize) -> &TextArea<'static> {
        &self.fields[index].textarea
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> EditorOutcome {
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            if let KeyCode::Char('s') = key.code {
                return EditorOutcome::Save;
            }
        }

        match key.code {
            KeyCode::Esc => EditorOutcome::Cancel,
            KeyCode::Tab | KeyCode::BackTab => {
                self.cycle_field(!matches!(key.code, KeyCode::BackTab));
                EditorOutcome::Continue
            }
            KeyCode::Up | KeyCode::Down => {
                self.move_focus(key.code);
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

    pub fn build_submission(&self) -> anyhow::Result<HostFormSubmission> {
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

        for line in self.fields[FIELD_COMMENT].textarea.lines() {
            let trimmed = line.trim_end_matches('\r').trim();
            let comment = trimmed.strip_prefix('#').map(str::trim).unwrap_or(trimmed);
            if !comment.is_empty() {
                raw_lines.push(format!("    # {}", comment));
            }
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

        let host = HostBlock {
            alias,
            directives,
            raw_text: raw_lines.join("\n"),
        };

        Ok(HostFormSubmission {
            host,
            password_action: self.build_password_action(),
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

    fn move_focus(&mut self, direction: KeyCode) {
        self.active_field = match (direction, self.active_field) {
            (KeyCode::Up, FIELD_USER) => FIELD_ALIAS,
            (KeyCode::Up, FIELD_PORT) => FIELD_HOSTNAME,
            (KeyCode::Up, FIELD_IDENTITY) => FIELD_USER,
            (KeyCode::Up, FIELD_PASSWORD) => FIELD_PORT,
            (KeyCode::Up, FIELD_COMMENT) => FIELD_IDENTITY,
            (KeyCode::Up, FIELD_EXTRA) => FIELD_PASSWORD,
            (KeyCode::Down, FIELD_ALIAS) => FIELD_USER,
            (KeyCode::Down, FIELD_HOSTNAME) => FIELD_PORT,
            (KeyCode::Down, FIELD_USER) => FIELD_IDENTITY,
            (KeyCode::Down, FIELD_PORT) => FIELD_PASSWORD,
            (KeyCode::Down, FIELD_IDENTITY) => FIELD_COMMENT,
            (KeyCode::Down, FIELD_PASSWORD) => FIELD_EXTRA,
            _ => self.active_field,
        };
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

    fn build_password_action(&self) -> PasswordStorageAction {
        let value = self.field_text(FIELD_PASSWORD);

        if value.eq_ignore_ascii_case("!clear") {
            return PasswordStorageAction::Clear;
        }

        if value.is_empty() {
            if self.saved_password_exists {
                PasswordStorageAction::Keep
            } else {
                PasswordStorageAction::Unchanged
            }
        } else {
            PasswordStorageAction::Set(value)
        }
    }
}

fn extract_comment_lines(host: &HostBlock) -> String {
    host.comment_lines().join("\n")
}

fn extract_extra_lines(host: &HostBlock) -> String {
    if !host.raw_text.trim().is_empty() {
        let mut extras = Vec::new();
        for (index, line) in host.raw_text.lines().enumerate() {
            if index == 0 {
                continue;
            }

            let trimmed = line.trim();
            if trimmed.is_empty() {
                extras.push(trimmed.to_string());
                continue;
            }
            if trimmed.starts_with('#') {
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

fn password_label(saved_password_exists: bool) -> &'static str {
    if saved_password_exists {
        "Password（已保存，留空保留，!clear 清除）"
    } else {
        "Password（留空则不保存）"
    }
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
    use super::{
        default_identity_file_value, HostFormState, PasswordStorageAction, FIELD_ALIAS,
        FIELD_COMMENT, FIELD_EXTRA, FIELD_HOSTNAME, FIELD_PASSWORD, FIELD_PORT,
    };
    use crate::config::types::{HostBlock, SshDirective};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use tui_textarea::{CursorMove, TextArea};

    fn press(form: &mut HostFormState, code: KeyCode) {
        form.handle_key(KeyEvent::new(code, KeyModifiers::NONE));
    }

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

        let popup = HostFormState::new_edit(0, &host, false);
        assert_eq!(popup.field_text(FIELD_COMMENT), "keep me");
        assert_eq!(popup.field_text(FIELD_EXTRA), "ProxyJump bastion");
        let rebuilt = popup.build_submission().expect("host should rebuild");

        assert_eq!(rebuilt.host.alias, "demo");
        assert_eq!(rebuilt.host.hostname(), Some("example.com"));
        assert_eq!(rebuilt.host.user(), Some("root"));
        assert_eq!(rebuilt.host.port(), 2222);
        assert!(rebuilt.host.raw_text.contains("ProxyJump bastion"));
        assert!(rebuilt.host.raw_text.contains("# keep me"));
        assert_eq!(rebuilt.password_action, PasswordStorageAction::Unchanged);
    }

    #[test]
    fn rejects_managed_directive_inside_extra_field() {
        let host = HostBlock {
            alias: "demo".into(),
            directives: vec![],
            raw_text: "Host demo".into(),
        };

        let mut popup = HostFormState::new_edit(0, &host, false);
        popup.fields[FIELD_EXTRA].textarea = TextArea::from(["HostName another.example.com"]);

        let err = popup
            .build_submission()
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

    #[test]
    fn edit_form_keeps_existing_password_when_left_empty() {
        let host = HostBlock {
            alias: "demo".into(),
            directives: vec![],
            raw_text: "Host demo".into(),
        };

        let form = HostFormState::new_edit(0, &host, true);
        let submission = form
            .build_submission()
            .expect("submission should build successfully");

        assert_eq!(submission.password_action, PasswordStorageAction::Keep);
    }

    #[test]
    fn edit_form_can_clear_saved_password() {
        let host = HostBlock {
            alias: "demo".into(),
            directives: vec![],
            raw_text: "Host demo".into(),
        };

        let mut form = HostFormState::new_edit(0, &host, true);
        form.fields[FIELD_PASSWORD].textarea = TextArea::from(["!clear"]);

        let submission = form
            .build_submission()
            .expect("submission should build successfully");

        assert_eq!(submission.password_action, PasswordStorageAction::Clear);
    }

    #[test]
    fn password_field_is_masked_and_preserves_spaces() {
        let mut form = HostFormState::new_add();
        assert_eq!(form.fields[FIELD_PASSWORD].textarea.mask_char(), Some('*'));

        form.fields[FIELD_ALIAS].textarea = TextArea::from(["demo"]);
        form.fields[FIELD_HOSTNAME].textarea = TextArea::from(["example.com"]);
        form.fields[FIELD_PASSWORD].textarea = TextArea::from([" secret "]);
        let submission = form
            .build_submission()
            .expect("submission should build successfully");

        assert_eq!(
            submission.password_action,
            PasswordStorageAction::Set(" secret ".into())
        );
    }

    #[test]
    fn comment_field_adds_a_single_hash_prefix() {
        let mut form = HostFormState::new_add();
        form.fields[FIELD_ALIAS].textarea = TextArea::from(["demo"]);
        form.fields[FIELD_HOSTNAME].textarea = TextArea::from(["example.com"]);
        form.fields[FIELD_COMMENT].textarea =
            TextArea::from(["production server", "# owner: ops", "#"]);

        let submission = form
            .build_submission()
            .expect("submission should build successfully");

        assert_eq!(
            submission.host.comment_lines(),
            ["production server", "owner: ops"]
        );
        assert!(submission.host.raw_text.contains("    # production server"));
        assert!(submission.host.raw_text.contains("    # owner: ops"));
        assert!(!submission.host.raw_text.contains("# #"));
    }

    #[test]
    fn up_down_keys_move_focus_within_a_form_column() {
        let mut form = HostFormState::new_add();

        press(&mut form, KeyCode::Tab);
        assert_eq!(form.active_field, FIELD_HOSTNAME);
        press(&mut form, KeyCode::Down);
        assert_eq!(form.active_field, FIELD_PORT);
        press(&mut form, KeyCode::Down);
        assert_eq!(form.active_field, FIELD_PASSWORD);
        press(&mut form, KeyCode::Down);
        assert_eq!(form.active_field, FIELD_EXTRA);

        press(&mut form, KeyCode::Up);
        assert_eq!(form.active_field, FIELD_PASSWORD);
        press(&mut form, KeyCode::Up);
        assert_eq!(form.active_field, FIELD_PORT);
        press(&mut form, KeyCode::Up);
        assert_eq!(form.active_field, FIELD_HOSTNAME);
    }

    #[test]
    fn arrow_focus_does_not_wrap_at_form_edges() {
        let mut form = HostFormState::new_add();

        press(&mut form, KeyCode::Left);
        press(&mut form, KeyCode::Up);
        assert_eq!(form.active_field, FIELD_ALIAS);

        press(&mut form, KeyCode::Down);
        press(&mut form, KeyCode::Down);
        press(&mut form, KeyCode::Down);
        assert_eq!(form.active_field, FIELD_COMMENT);
        press(&mut form, KeyCode::Down);
        assert_eq!(form.active_field, FIELD_COMMENT);
    }

    #[test]
    fn left_right_keys_move_the_input_cursor_without_changing_focus() {
        let mut form = HostFormState::new_add();
        form.fields[FIELD_ALIAS].textarea = TextArea::from(["demo"]);
        form.fields[FIELD_ALIAS]
            .textarea
            .move_cursor(CursorMove::End);
        assert_eq!(form.fields[FIELD_ALIAS].textarea.cursor(), (0, 4));

        press(&mut form, KeyCode::Left);
        assert_eq!(form.active_field, FIELD_ALIAS);
        assert_eq!(form.fields[FIELD_ALIAS].textarea.cursor(), (0, 3));

        press(&mut form, KeyCode::Right);
        assert_eq!(form.active_field, FIELD_ALIAS);
        assert_eq!(form.fields[FIELD_ALIAS].textarea.cursor(), (0, 4));
    }
}
