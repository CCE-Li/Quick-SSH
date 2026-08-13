//! AI Provider：LLM 通信层（OpenAI 兼容 HTTP via `curl` / 本地 ollama CLI）
//!
//! Phase 8：零新增依赖。双后端（决策 D3 选项 A/B 兼容）：
//! - [`ProviderKind::OpenAI`]：`curl -s` 调 OpenAI 兼容 `/chat/completions`
//! - [`ProviderKind::Ollama`]：优先走 `ollama run` CLI（本地优先 / 离线可用），
//!   也支持 `curl` 直连本地 11434 端口
//!
//! API Key 从环境变量 `QSSH_OPENAI_API_KEY` 读取（不落盘）。

use std::process::Command;

use serde_json::json;

use super::tools::ToolCall;

/// Provider 类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProviderKind {
    /// OpenAI 兼容 HTTP（DeepSeek / Qwen / OpenAI ...）
    #[default]
    OpenAI,
    /// 本地 ollama（离线可用）
    Ollama,
}

impl ProviderKind {
    /// 从配置字符串解析
    pub fn from_str(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "openai" | "openai-compatible" | "deepseek" | "qwen" => Some(ProviderKind::OpenAI),
            "ollama" | "local" => Some(ProviderKind::Ollama),
            _ => None,
        }
    }

    /// 配置字符串（与 [`from_str`] 互逆）
    pub fn as_str(self) -> &'static str {
        match self {
            ProviderKind::OpenAI => "openai",
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
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            kind: ProviderKind::OpenAI,
            base_url: "https://api.deepseek.com/v1".to_string(),
            model: "deepseek-chat".to_string(),
            timeout_secs: 60,
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
        ProviderKind::OpenAI => chat_openai(config, messages, available_tools),
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
    let api_key = std::env::var("QSSH_OPENAI_API_KEY").map_err(|_| ProviderError::MissingApiKey)?;

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
        .args(["-s", "-m", &config.timeout_secs.to_string()])
        .args(["-X", "POST"])
        .args(["-H", "Content-Type: application/json"])
        .args(["-H", &format!("Authorization: Bearer {}", api_key)])
        .args(["-d", &body])
        .arg(&url);

    let output = command
        .output()
        .map_err(|e| ProviderError::Spawn(e.to_string()))?;

    if !output.status.success() {
        return Err(ProviderError::Http(format!(
            "curl 退出码 {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
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
        .args(["-s", "-m", &config.timeout_secs.to_string()])
        .args(["-X", "POST"])
        .args(["-H", "Content-Type: application/json"])
        .args(["-d", &body])
        .arg(&url);

    let output = command
        .output()
        .map_err(|e| ProviderError::Spawn(e.to_string()))?;

    if !output.status.success() {
        return Err(ProviderError::Http(format!(
            "curl 退出码 {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
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
                write!(f, "未设置环境变量 QSSH_OPENAI_API_KEY")
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
        assert_eq!(ProviderKind::from_str("bogus"), None);
    }
}
