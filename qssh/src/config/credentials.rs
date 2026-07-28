use std::io::Write;

use anyhow::{bail, Context, Result};
use keyring::{Entry, Error as KeyringError};

const SERVICE_NAME: &str = "quick-ssh";
pub const ASKPASS_ACTIVE_ENV: &str = "QSSH_ASKPASS_ACTIVE";
pub const ASKPASS_ALIAS_ENV: &str = "QSSH_ASKPASS_ALIAS";

fn entry_for_alias(alias: &str) -> Result<Entry> {
    Entry::new(SERVICE_NAME, alias).with_context(|| format!("无法打开系统凭据项: {}", alias))
}

pub fn load_password(alias: &str) -> Result<Option<String>> {
    let entry = entry_for_alias(alias)?;
    match entry.get_password() {
        Ok(password) => Ok(Some(password)),
        Err(KeyringError::NoEntry) => Ok(None),
        Err(err) => Err(anyhow::Error::new(err))
            .with_context(|| format!("无法读取主机 {} 的已保存密码", alias)),
    }
}

pub fn has_password(alias: &str) -> Result<bool> {
    Ok(load_password(alias)?.is_some())
}

pub fn store_password(alias: &str, password: &str) -> Result<()> {
    let entry = entry_for_alias(alias)?;
    entry
        .set_password(password)
        .with_context(|| format!("无法保存主机 {} 的密码到系统凭据库", alias))
}

pub fn delete_password(alias: &str) -> Result<()> {
    let entry = entry_for_alias(alias)?;
    match entry.delete_credential() {
        Ok(_) | Err(KeyringError::NoEntry) => Ok(()),
        Err(err) => Err(anyhow::Error::new(err))
            .with_context(|| format!("无法删除主机 {} 的已保存密码", alias)),
    }
}

pub fn move_password(old_alias: &str, new_alias: &str) -> Result<bool> {
    if old_alias == new_alias {
        return Ok(false);
    }

    let Some(password) = load_password(old_alias)? else {
        return Ok(false);
    };

    store_password(new_alias, &password)?;
    delete_password(old_alias)?;
    Ok(true)
}

/// OpenSSH invokes the current qssh executable as its AskPass helper.
/// Returns true when the invocation was handled and normal CLI parsing should stop.
pub fn handle_askpass_request() -> Result<bool> {
    if std::env::var(ASKPASS_ACTIVE_ENV).as_deref() != Ok("1") {
        return Ok(false);
    }

    let prompt = std::env::args().nth(1).unwrap_or_default();
    if !is_secret_prompt(&prompt) {
        bail!("OpenSSH 当前请求的不是登录密码，Quick-SSH 未自动填写");
    }

    let alias = std::env::var(ASKPASS_ALIAS_ENV).context("AskPass 请求缺少主机别名")?;
    let password =
        load_password(&alias)?.ok_or_else(|| anyhow::anyhow!("主机 {} 没有已保存密码", alias))?;

    let mut stdout = std::io::stdout().lock();
    stdout
        .write_all(password.as_bytes())
        .context("无法向 OpenSSH 提供已保存密码")?;
    stdout.flush().context("无法刷新 AskPass 输出")?;
    Ok(true)
}

fn is_secret_prompt(prompt: &str) -> bool {
    let prompt = prompt.to_lowercase();
    prompt.contains("password") || prompt.contains("密码") || prompt.contains("口令")
}

#[cfg(test)]
mod tests {
    use super::is_secret_prompt;

    #[test]
    fn recognizes_password_prompts_only() {
        assert!(is_secret_prompt("root@example.com's password:"));
        assert!(is_secret_prompt("请输入密码："));
        assert!(!is_secret_prompt("Enter passphrase for key 'id_ed25519':"));
        assert!(!is_secret_prompt(
            "Are you sure you want to continue connecting (yes/no)?"
        ));
    }
}
