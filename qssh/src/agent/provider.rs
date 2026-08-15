//! AI Provider：LLM 通信层（OpenAI 兼容 HTTP via `curl` / 本地 ollama CLI）
//!
//! Phase 8：零新增依赖。双后端（决策 D3 选项 A/B 兼容）：
//! - [`ProviderKind::OpenAI`]：`curl -s` 调 OpenAI 兼容 `/chat/completions`
//! - [`ProviderKind::Ollama`]：优先走 `ollama run` CLI（本地优先 / 离线可用），
//!   也支持 `curl` 直连本地 11434 端口
//!
//! API Key 解析顺序（OpenAI 兼容后端）：
//! 1. 环境变量 `QSSH_OPENAI_API_KEY`
//! 2. 环境变量 `OPENCODE_API_KEY`（opencode-go 的标准变量）
//! 3. opencode 登录凭证 `~/.local/share/opencode/auth.json`（key 不落盘在本项目配置中）

use std::process::Command;

use serde_json::json;

use super::tools::ToolCall;

/// Provider 类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProviderKind {
    /// OpenAI 兼容 HTTP（DeepSeek / Qwen / OpenAI ...）
    #[default]
    OpenAI,
    /// opencode-go（`https://opencode.ai/zen/go/v1`，OpenAI 兼容）
    OpenCodeGo,
    /// 本地 ollama（离线可用）
    Ollama,
}

/// opencode-go 的 API 端点（models.dev 注册表定义，OpenAI 兼容）
pub const OPENCODE_GO_BASE_URL: &str = "https://opencode.ai/zen/go/v1";

/// opencode-go 常用模型名
pub const OPENCODE_GO_MODEL: &str = "deepseek-v4-flash";

/// opencode 登录凭证文件（相对 data 目录）
pub const OPENCODE_AUTH_RELATIVE_PATH: &str = "opencode/auth.json";

/// opencode 登录凭证中的 provider 名
pub const OPENCODE_GO_AUTH_KEY: &str = "opencode-go";

/// opencode 的标准 API key 环境变量
pub const OPENCODE_API_KEY_ENV: &str = "OPENCODE_API_KEY";

impl ProviderKind {
    /// 从配置字符串解析
    pub fn from_str(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "openai" | "openai-compatible" | "deepseek" | "qwen" => Some(ProviderKind::OpenAI),
            "opencode" | "opencode-go" => Some(ProviderKind::OpenCodeGo),
            "ollama" | "local" => Some(ProviderKind::Ollama),
            _ => None,
        }
    }

    /// 配置字符串（与 [`from_str`] 互逆）
    pub fn as_str(self) -> &'static str {
        match self {
            ProviderKind::OpenAI => "openai",
            ProviderKind::OpenCodeGo => "opencode",
            ProviderKind::Ollama => "ollama",
        }
    }
}

/// Provider 配置
#[derive(Debug, Clone)]
pub struct ProviderConfig {
    pub kind: ProviderKind,
    /// OpenAI 兼容 base_url（如 `https://api.deepseek.com/v1`）
    pub base_url: String,
    /// 模型名（如 `deepseek-chat` / `qwen2.5:7b`）
    pub model: String,
    /// 请求超时（秒）
    pub timeout_secs: u64,
    /// 显式配置的 API Key（`None` 时回退到环境变量 / opencode auth.json）
    pub api_key: Option<String>,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            kind: ProviderKind::OpenAI,
            base_url: "https://api.deepseek.com/v1".to_string(),
            model: "deepseek-chat".to_string(),
            timeout_secs: 60,
            api_key: None,
        }
    }
}

/// 单条对话消息
#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// LLM 回复
#[derive(Debug, Clone)]
pub struct ChatReply {
    /// 自然语言回复
    pub text: String,
    /// 解析出的工具调用（可能为空）
    pub tool_call: Option<ToolCall>,
}

/// 调用 LLM（阻塞，带超时）。
///
/// 返回 [`ProviderError`]，由调用方决定降级策略。
pub fn chat(
    config: &ProviderConfig,
    messages: &[ChatMessage],
    available_tools: &str,
) -> Result<ChatReply, ProviderError> {
    match config.kind {
        ProviderKind::OpenAI | ProviderKind::OpenCodeGo => {
            chat_openai(config, messages, available_tools)
        }
        ProviderKind::Ollama => match chat_ollama_cli(config, messages, available_tools) {
            Ok(reply) => Ok(reply),
            Err(_) => chat_ollama_http(config, messages, available_tools),
        },
    }
}

/// OpenAI 兼容 API：`curl -s <base_url>/chat/completions`
fn chat_openai(
    config: &ProviderConfig,
    messages: &[ChatMessage],
    available_tools: &str,
) -> Result<ChatReply, ProviderError> {
    let api_key = resolve_api_key(config)?;

    let system = format!(
        "你是 Quick-SSH 的运维助手。你可以调用工具管理远程服务器。\n\
         可用工具（每次回复若需工具，请单独一行输出如下格式，不要夹杂其他内容）：\n\
         {available_tools}\n\
         若需要工具，仅输出 tool:name 目标=值 形式，例如：\n\
         tool:server.status\n\
         tool:service.restart 目标=cron\n\
         否则直接输出自然语言回复。"
    );
    let mut payload_messages = vec![json!({ "role": "system", "content": system })];
    for message in messages {
        payload_messages.push(json!({
            "role": message.role,
            "content": message.content,
        }));
    }

    let body = json!({
        "model": config.model,
        "messages": payload_messages,
        "temperature": 0.2,
        "max_tokens": 1024,
    })
    .to_string();

    let url = format!("{}/chat/completions", config.base_url.trim_end_matches('/'));
    let mut command = Command::new("curl");
    command
        .args(curl_https_args(config.timeout_secs))
        .args(["-X", "POST"])
        .args(["-H", "Content-Type: application/json"])
        .args(["-H", &format!("Authorization: Bearer {}", api_key)])
        .args(["-d", &body])
        .arg(&url);

    let output = command
        .output()
        .map_err(|e| ProviderError::Spawn(e.to_string()))?;

    if !output.status.success() {
        return Err(ProviderError::Http(curl_error(
            output.status.code(),
            &output.stderr,
        )));
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| ProviderError::Parse(format!("JSON 解析失败: {e}")))?;

    let content = parsed["choices"][0]["message"]["content"]
        .as_str()
        .ok_or_else(|| ProviderError::Parse("响应缺少 content".to_string()))?;

    Ok(build_reply(content))
}

/// 解析 API Key，优先级：
/// 1. 配置中显式设置的 `api_key`（`ProviderConfig.api_key`，WebUI 配置）
/// 2. 环境变量 `QSSH_OPENAI_API_KEY`（Quick-SSH 专属）
/// 3. 环境变量 `OPENCODE_API_KEY`（opencode-go 标准变量）
/// 4. opencode 登录凭证 `~/.local/share/opencode/auth.json` 中对应 provider 的 `key`
fn resolve_api_key(config: &ProviderConfig) -> Result<String, ProviderError> {
    // 1. 显式配置的 key 优先级最高
    if let Some(key) = config.api_key.as_deref() {
        if !key.trim().is_empty() {
            return Ok(key.trim().to_string());
        }
    }
    // 2. 环境变量
    if let Ok(key) = std::env::var("QSSH_OPENAI_API_KEY") {
        if !key.trim().is_empty() {
            return Ok(key);
        }
    }
    if let Ok(key) = std::env::var(OPENCODE_API_KEY_ENV) {
        if !key.trim().is_empty() {
            return Ok(key);
        }
    }
    // 3. opencode 登录凭证回退（仅对 opencode / opencode-go provider）
    let auth_name = match config.kind {
        ProviderKind::OpenCodeGo => Some(OPENCODE_GO_AUTH_KEY),
        _ => None,
    };
    if let Some(name) = auth_name {
        if let Ok(key) = read_opencode_auth_key(name) {
            if !key.trim().is_empty() {
                return Ok(key);
            }
        }
    }
    Err(ProviderError::MissingApiKey)
}

/// opencode 登录凭证路径：`~/.local/share/opencode/auth.json`
///（macOS / Linux 也可用 `~/.config/opencode/auth.json`，这里优先 data 目录）
fn opencode_auth_path() -> std::path::PathBuf {
    if let Some(data) = dirs::data_local_dir() {
        let candidate = data.join(OPENCODE_AUTH_RELATIVE_PATH);
        if candidate.exists() {
            return candidate;
        }
    }
    if let Some(config) = dirs::config_dir() {
        let candidate = config.join(OPENCODE_AUTH_RELATIVE_PATH);
        if candidate.exists() {
            return candidate;
        }
    }
    dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(OPENCODE_AUTH_RELATIVE_PATH)
}

/// 读取 opencode `auth.json` 中指定 provider 的 API key。
///
/// 结构示例：`{ "opencode-go": { "type": "api", "key": "sk-..." } }`
fn read_opencode_auth_key(provider_name: &str) -> Result<String, ProviderError> {
    let path = opencode_auth_path();
    let content = std::fs::read_to_string(&path).map_err(|_| ProviderError::MissingApiKey)?;
    let parsed: serde_json::Value =
        serde_json::from_str(&content).map_err(|_| ProviderError::MissingApiKey)?;
    let key = parsed[provider_name]["key"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or(ProviderError::MissingApiKey)?;
    Ok(key)
}

/// ollama CLI：`ollama run <model> "<prompt>"`
fn chat_ollama_cli(
    config: &ProviderConfig,
    messages: &[ChatMessage],
    available_tools: &str,
) -> Result<ChatReply, ProviderError> {
    // 拼接对话历史为单一提示（ollama run 单轮）
    let mut prompt = String::new();
    prompt.push_str(&format!(
        "你是 Quick-SSH 的运维助手。可用工具：\n{}\n\n",
        available_tools
    ));
    for message in messages {
        let role = match message.role.as_str() {
            "user" => "用户",
            "assistant" => "助手",
            _ => "系统",
        };
        prompt.push_str(&format!("{role}: {}\n", message.content));
    }
    prompt.push_str("\n请回复。若需调用工具，单独一行输出 tool:name 目标=值。");

    let mut command = Command::new("ollama");
    command
        .args(["run", &config.model, &prompt])
        .env("OLLAMA_HOST", config.base_url.clone());

    let output = command
        .output()
        .map_err(|e| ProviderError::Spawn(e.to_string()))?;

    if !output.status.success() {
        return Err(ProviderError::Http(format!(
            "ollama 退出码 {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    Ok(build_reply(&String::from_utf8_lossy(&output.stdout)))
}

/// ollama HTTP：`curl -s <base_url>/api/chat`
fn chat_ollama_http(
    config: &ProviderConfig,
    messages: &[ChatMessage],
    available_tools: &str,
) -> Result<ChatReply, ProviderError> {
    let system = format!(
        "你是 Quick-SSH 的运维助手。可用工具：\n{available_tools}\n\
         若需调用工具，单独一行输出 tool:name 目标=值。"
    );
    let mut payload_messages = vec![json!({ "role": "system", "content": system })];
    for message in messages {
        payload_messages.push(json!({ "role": message.role, "content": message.content }));
    }
    let body = json!({
        "model": config.model,
        "messages": payload_messages,
        "stream": false,
    })
    .to_string();

    let base = if config.base_url.is_empty() {
        "http://localhost:11434".to_string()
    } else {
        config.base_url.clone()
    };
    let url = format!("{}/api/chat", base.trim_end_matches('/'));

    let mut command = Command::new("curl");
    command
        .args(["-sS", "-m", &config.timeout_secs.to_string()])
        .args(["-X", "POST"])
        .args(["-H", "Content-Type: application/json"])
        .args(["-d", &body])
        .arg(&url);

    let output = command
        .output()
        .map_err(|e| ProviderError::Spawn(e.to_string()))?;

    if !output.status.success() {
        return Err(ProviderError::Http(curl_error(
            output.status.code(),
            &output.stderr,
        )));
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| ProviderError::Parse(format!("JSON 解析失败: {e}")))?;

    let content = parsed["message"]["content"]
        .as_str()
        .ok_or_else(|| ProviderError::Parse("响应缺少 content".to_string()))?;

    Ok(build_reply(content))
}

/// 获取 Provider 支持的模型列表（OpenAI 兼容 API）。
///
/// 通过 `curl -s <base_url>/models` 拉取 `data[].id` 列表；
/// Ollama 本地后端通过 `/api/tags` 获取。失败时返回 [`ProviderError`]。
pub fn list_models(config: &ProviderConfig) -> Result<Vec<String>, ProviderError> {
    match config.kind {
        ProviderKind::OpenAI | ProviderKind::OpenCodeGo => list_models_openai(config),
        ProviderKind::Ollama => list_models_ollama(config),
    }
}

/// OpenAI 兼容：`curl -s <base_url>/models` → `data[].id`
fn list_models_openai(config: &ProviderConfig) -> Result<Vec<String>, ProviderError> {
    let api_key = resolve_api_key(config)?;
    let base = if config.base_url.is_empty() {
        "https://api.deepseek.com/v1".to_string()
    } else {
        config.base_url.clone()
    };
    let url = format!("{}/models", base.trim_end_matches('/'));

    let mut command = Command::new("curl");
    command
        .args(curl_https_args(config.timeout_secs))
        .args(["-H", &format!("Authorization: Bearer {}", api_key)])
        .arg(&url);

    let output = command
        .output()
        .map_err(|e| ProviderError::Spawn(e.to_string()))?;

    if !output.status.success() {
        return Err(ProviderError::Http(curl_error(
            output.status.code(),
            &output.stderr,
        )));
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| ProviderError::Parse(format!("JSON 解析失败: {e}")))?;

    let mut models = Vec::new();
    if let Some(arr) = parsed["data"].as_array() {
        for item in arr {
            if let Some(id) = item["id"].as_str() {
                if !id.trim().is_empty() {
                    models.push(id.to_string());
                }
            }
        }
    }
    models.sort();
    models.dedup();
    Ok(models)
}

/// Ollama：`curl -s <base_url>/api/tags` → `models[].name`
fn list_models_ollama(config: &ProviderConfig) -> Result<Vec<String>, ProviderError> {
    let base = if config.base_url.is_empty() {
        "http://localhost:11434".to_string()
    } else {
        config.base_url.clone()
    };
    let url = format!("{}/api/tags", base.trim_end_matches('/'));

    let mut command = Command::new("curl");
    command
        .args(["-sS", "-m", &config.timeout_secs.to_string()])
        .arg(&url);

    let output = command
        .output()
        .map_err(|e| ProviderError::Spawn(e.to_string()))?;

    if !output.status.success() {
        return Err(ProviderError::Http(curl_error(
            output.status.code(),
            &output.stderr,
        )));
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| ProviderError::Parse(format!("JSON 解析失败: {e}")))?;

    let mut models = Vec::new();
    if let Some(arr) = parsed["models"].as_array() {
        for item in arr {
            if let Some(name) = item["name"].as_str() {
                if !name.trim().is_empty() {
                    models.push(name.to_string());
                }
            }
        }
    }
    models.sort();
    models.dedup();
    Ok(models)
}

/// 从 LLM 回复构建 [`ChatReply`]：提取首行 `tool:` 调用
fn build_reply(content: &str) -> ChatReply {
    // 优先整段匹配（LLM 可能在回复后附带工具调用）
    let tool_call = content
        .lines()
        .find_map(|line| super::tools::parse_tool_call(line.trim()))
        .or_else(|| super::tools::parse_tool_call(content));
    let text = if tool_call.is_some() {
        // 移除工具调用行，保留其余说明
        content
            .lines()
            .filter(|line| super::tools::parse_tool_call(line.trim()).is_none())
            .collect::<Vec<_>>()
            .join("\n")
            .trim()
            .to_string()
    } else {
        content.to_string()
    };
    ChatReply { text, tool_call }
}

/// 构建 HTTPS curl 基础参数：
/// - `-sS`：静默成功输出，但失败时保留 curl 的 stderr 诊断信息
///   （此前用 `-s` 会把 curl 35 等具体错误完全吞掉）
/// - `--ssl-no-revoke`：Windows Schannel 后端禁用证书吊销检查。
///   吊销检查在受限网络 / 代理下失败是 curl 35（`CURLE_SSL_CONNECT_ERROR`）
///   的最常见根因，禁用后证书链验证仍由 Schannel 完成。
fn curl_https_args(timeout_secs: u64) -> Vec<String> {
    vec![
        "-sS".to_string(),
        "-m".to_string(),
        timeout_secs.to_string(),
        "--ssl-no-revoke".to_string(),
    ]
}

/// 格式化 curl 失败错误，附带常见退出码诊断提示。
///
/// `code` 为 curl 进程的退出码（`ExitStatus::code()`，进程被信号终止时为 `None`）。
fn curl_error(code: Option<i32>, stderr: &[u8]) -> String {
    let detail = String::from_utf8_lossy(stderr).trim().to_string();
    let hint = match code {
        Some(35) => "；SSL 连接错误（已禁用证书吊销检查，若仍失败请检查网络 / 代理 / Base URL）",
        Some(6) => "；DNS 解析失败（请检查网络或 Base URL）",
        Some(7) => "；连接被拒绝（请检查 Base URL 与端口）",
        Some(60) => "；证书验证失败（自签名 / MITM / 过期证书）",
        _ => "",
    };
    let code_str = match code {
        Some(c) => c.to_string(),
        None => "unknown".to_string(),
    };
    if detail.is_empty() {
        format!("curl 退出码 {code_str}{hint}")
    } else {
        format!("curl 退出码 {code_str}: {detail}{hint}")
    }
}

/// Provider 错误
#[derive(Debug, Clone)]
pub enum ProviderError {
    /// 缺少 API Key（OpenAI 后端）
    MissingApiKey,
    /// 无法启动子进程（curl / ollama 未安装）
    Spawn(String),
    /// HTTP / 退出码错误
    Http(String),
    /// 响应解析失败
    Parse(String),
}

impl std::fmt::Display for ProviderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProviderError::MissingApiKey => {
                write!(
                    f,
                    "未找到 API Key（已依次检查 QSSH_OPENAI_API_KEY、OPENCODE_API_KEY 与 opencode auth.json）"
                )
            }
            ProviderError::Spawn(msg) => write!(f, "无法启动子进程: {msg}"),
            ProviderError::Http(msg) => write!(f, "HTTP 错误: {msg}"),
            ProviderError::Parse(msg) => write!(f, "解析失败: {msg}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_reply_plain_text() {
        let reply = build_reply("服务器状态正常。");
        assert_eq!(reply.text, "服务器状态正常。");
        assert!(reply.tool_call.is_none());
    }

    #[test]
    fn build_reply_extracts_tool_call() {
        let reply = build_reply("我来查看一下。\ntool:server.status");
        assert!(reply.tool_call.is_some());
        let call = reply.tool_call.unwrap();
        assert_eq!(call.tool.name(), "server.status");
        // 工具调用行被移除
        assert_eq!(reply.text, "我来查看一下。");
    }

    #[test]
    fn provider_kind_parsing() {
        assert_eq!(ProviderKind::from_str("openai"), Some(ProviderKind::OpenAI));
        assert_eq!(ProviderKind::from_str("OLLAMA"), Some(ProviderKind::Ollama));
        assert_eq!(ProviderKind::from_str("local"), Some(ProviderKind::Ollama));
        assert_eq!(
            ProviderKind::from_str("opencode"),
            Some(ProviderKind::OpenCodeGo)
        );
        assert_eq!(
            ProviderKind::from_str("opencode-go"),
            Some(ProviderKind::OpenCodeGo)
        );
        assert_eq!(ProviderKind::from_str("bogus"), None);
    }

    #[test]
    fn opencode_go_defaults() {
        assert_eq!(OPENCODE_GO_BASE_URL, "https://opencode.ai/zen/go/v1");
        assert_eq!(ProviderKind::OpenCodeGo.as_str(), "opencode");
    }

    #[test]
    fn resolve_api_key_uses_explicit_first() {
        let config = ProviderConfig {
            kind: ProviderKind::OpenAI,
            base_url: "https://example.com/v1".to_string(),
            model: "test".to_string(),
            timeout_secs: 10,
            api_key: Some("sk-explicit".to_string()),
        };
        unsafe {
            std::env::set_var("QSSH_OPENAI_API_KEY", "sk-env");
        }
        let key = resolve_api_key(&config).expect("explicit key should win");
        assert_eq!(key, "sk-explicit");
        unsafe {
            std::env::remove_var("QSSH_OPENAI_API_KEY");
        }
    }

    #[test]
    fn resolve_api_key_uses_env_fallback() {
        unsafe {
            std::env::remove_var("QSSH_OPENAI_API_KEY");
            std::env::set_var("OPENCODE_API_KEY", "sk-env-test");
        }
        let config = ProviderConfig {
            api_key: None,
            ..ProviderConfig::default()
        };
        let key = resolve_api_key(&config).expect("env key should be found");
        assert_eq!(key, "sk-env-test");
        unsafe {
            std::env::remove_var("OPENCODE_API_KEY");
        }
    }

    #[test]
    fn resolve_api_key_missing_returns_error() {
        unsafe {
            std::env::remove_var("QSSH_OPENAI_API_KEY");
            std::env::remove_var("OPENCODE_API_KEY");
        }
        let config = ProviderConfig {
            api_key: None,
            kind: ProviderKind::OpenAI,
            ..ProviderConfig::default()
        };
        assert!(matches!(
            resolve_api_key(&config),
            Err(ProviderError::MissingApiKey)
        ));
    }

    #[test]
    fn curl_https_args_includes_no_revoke_and_show_errors() {
        let args = curl_https_args(30);
        assert!(args.iter().any(|a| a == "-sS"));
        assert!(args.iter().any(|a| a == "--ssl-no-revoke"));
        assert!(args.iter().any(|a| a == "30"));
    }

    #[test]
    fn curl_error_formats_ssl_code_35() {
        let msg = curl_error(Some(35), b"schannel: failed to verify revocation");
        assert!(msg.contains("35"));
        assert!(msg.contains("schannel: failed to verify revocation"));
        assert!(msg.contains("SSL"));
    }

    #[test]
    fn curl_error_empty_detail_still_has_hint() {
        let msg = curl_error(Some(35), b"");
        assert!(msg.contains("35"));
        assert!(msg.contains("SSL"));
    }

    #[test]
    fn list_models_parses_openai_data() {
        let text = r#"{"object":"list","data":[{"id":"deepseek-chat"},{"id":"deepseek-reasoner"},{"id":"deepseek-chat"}]}"#;
        let parsed: serde_json::Value = serde_json::from_str(text).unwrap();
        let mut models = Vec::new();
        if let Some(arr) = parsed["data"].as_array() {
            for item in arr {
                if let Some(id) = item["id"].as_str() {
                    models.push(id.to_string());
                }
            }
        }
        models.sort();
        models.dedup();
        assert_eq!(models, vec!["deepseek-chat", "deepseek-reasoner"]);
    }

    #[test]
    fn list_models_parses_ollama_tags() {
        let text = r#"{"models":[{"name":"qwen2.5:7b"},{"name":"llama3:latest"}]}"#;
        let parsed: serde_json::Value = serde_json::from_str(text).unwrap();
        let mut models = Vec::new();
        if let Some(arr) = parsed["models"].as_array() {
            for item in arr {
                if let Some(name) = item["name"].as_str() {
                    models.push(name.to_string());
                }
            }
        }
        models.sort();
        models.dedup();
        assert_eq!(models, vec!["llama3:latest", "qwen2.5:7b"]);
    }

    #[test]
    fn read_opencode_auth_key_parses_json() {
        use std::io::Write;
        let tmp = std::env::temp_dir().join(format!("qssh-auth-test-{}", std::process::id()));
        std::fs::create_dir_all(tmp.join("opencode")).unwrap();
        let auth = tmp.join("opencode").join("auth.json");
        let mut f = std::fs::File::create(&auth).unwrap();
        write!(
            f,
            r#"{{"opencode-go": {{"type": "api", "key": "sk-auth-123"}}}}"#
        )
        .unwrap();
        let content = std::fs::read_to_string(&auth).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        let key = parsed["opencode-go"]["key"].as_str().unwrap();
        assert_eq!(key, "sk-auth-123");
        // 清理
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
