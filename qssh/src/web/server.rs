//! 极简 HTTP 服务器 + WebUI 路由（零新增依赖）
//!
//! `qssh web` 提供本地配置页：查看 / 修改 Agent 配置，并一键测试
//! Provider 连接是否成功（复用 [`crate::agent::provider::chat`]）。

use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::time::Instant;

use serde_json::{json, Value};

use crate::agent::config::{load_agent_config, save_agent_config};
use crate::agent::permissions::PermissionLevel;
use crate::agent::provider::{chat, ChatMessage, ProviderKind};

use super::WEBUI_VERSION;

/// 内嵌前端页面（src/web/index.html）
const HTML: &str = include_str!("index.html");

// ── 服务入口 ──────────────────────────────────────────────

/// 启动 WebUI 服务（阻塞）
pub fn serve(port: u16) -> anyhow::Result<()> {
    let addr = format!("127.0.0.1:{port}");
    let listener = TcpListener::bind(&addr)?;
    println!("🌐 Quick-SSH WebUI 已启动：http://{addr}");
    println!("   按 Ctrl+C 停止服务");
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                std::thread::spawn(|| handle_connection(stream));
            }
            Err(e) => eprintln!("连接错误: {e}"),
        }
    }
    Ok(())
}

// ── HTTP 请求解析 ─────────────────────────────────────────

struct Request {
    method: String,
    path: String,
    body: Vec<u8>,
}

fn read_request(reader: &mut impl BufRead) -> std::io::Result<Request> {
    let mut request_line = String::new();
    reader.read_line(&mut request_line)?;
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("").to_string();
    let raw_path = parts.next().unwrap_or("/").to_string();
    // 丢弃 query string
    let path = raw_path.split('?').next().unwrap_or("/").to_string();

    let mut content_length = 0usize;
    let mut line = String::new();
    loop {
        line.clear();
        if reader.read_line(&mut line)? == 0 {
            break;
        }
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            break;
        }
        if let Some(value) = trimmed.to_ascii_lowercase().strip_prefix("content-length:") {
            content_length = value.trim().parse().unwrap_or(0);
        }
    }

    let mut body = vec![0u8; content_length];
    if content_length > 0 {
        reader.read_exact(&mut body)?;
    }
    Ok(Request { method, path, body })
}

// ── 响应辅助 ──────────────────────────────────────────────

fn reason_phrase(status: u16) -> &'static str {
    match status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        500 => "Internal Server Error",
        _ => "OK",
    }
}

fn write_simple(w: &mut TcpStream, status: u16, ctype: &str, body: &str) -> std::io::Result<()> {
    let head = format!(
        "HTTP/1.1 {status} {}\r\nContent-Type: {ctype}\r\nContent-Length: {}\r\nConnection: close\r\nAccess-Control-Allow-Origin: *\r\n\r\n",
        reason_phrase(status),
        body.len()
    );
    w.write_all(head.as_bytes())?;
    w.write_all(body.as_bytes())?;
    w.flush()
}

fn write_json(w: &mut TcpStream, status: u16, value: &Value) -> std::io::Result<()> {
    let body = value.to_string();
    write_simple(w, status, "application/json; charset=utf-8", &body)
}

// ── 连接处理与路由 ────────────────────────────────────────

fn handle_connection(stream: TcpStream) {
    let mut writer = match stream.try_clone() {
        Ok(w) => w,
        Err(_) => return,
    };
    let _ = stream.set_read_timeout(Some(std::time::Duration::from_secs(30)));
    let mut reader = BufReader::new(stream);

    let req = match read_request(&mut reader) {
        Ok(r) => r,
        Err(_) => {
            let _ = write_simple(&mut writer, 400, "text/plain", "Bad Request");
            return;
        }
    };

    let (method, path) = (req.method.as_str(), req.path.as_str());
    match (method, path) {
        ("GET", "/") => {
            let _ = write_simple(&mut writer, 200, "text/html; charset=utf-8", HTML);
        }
        ("GET", "/api/status") => handle_status(&mut writer),
        ("GET", "/api/config") => handle_config_get(&mut writer),
        ("POST", "/api/config") => handle_config_post(&mut writer, &req),
        ("POST", "/api/test") => handle_test(&mut writer, &req),
        _ => {
            let _ = write_simple(&mut writer, 404, "text/plain", "Not Found");
        }
    }
}

// ── API Handlers ──────────────────────────────────────────

/// GET /api/status — 运行状态（provider / 权限 / key 提示）
fn handle_status(w: &mut TcpStream) {
    let config = load_agent_config();
    let kind = ProviderKind::from_str(&config.provider).unwrap_or_default();
    let (provider_label, key_hint) = match kind {
        ProviderKind::OpenAI => (
            "OpenAI Compatible",
            "环境变量 QSSH_OPENAI_API_KEY 或 OPENCODE_API_KEY",
        ),
        ProviderKind::OpenCodeGo => (
            "opencode-go",
            "复用 ~/.local/share/opencode/auth.json（opencode-go）",
        ),
        ProviderKind::Ollama => ("Ollama", "本地 ollama CLI（无需 key）"),
    };
    let _ = write_json(
        w,
        200,
        &json!({
            "version": WEBUI_VERSION,
            "provider": kind.as_str(),
            "provider_label": provider_label,
            "base_url": config.base_url,
            "model": config.model,
            "permission": config.permission,
            "permission_label": config.permission_level().label(),
            "timeout_secs": config.timeout_secs,
            "key_hint": key_hint,
        }),
    );
}

/// GET /api/config — 读取 Agent 配置
fn handle_config_get(w: &mut TcpStream) {
    let config = load_agent_config();
    let kind = ProviderKind::from_str(&config.provider).unwrap_or_default();
    let _ = write_json(
        w,
        200,
        &json!({
            "provider": kind.as_str(),
            "base_url": config.base_url,
            "model": config.model,
            "permission": config.permission,
            "timeout_secs": config.timeout_secs,
        }),
    );
}

/// POST /api/config — 保存 Agent 配置
fn handle_config_post(w: &mut TcpStream, req: &Request) {
    let mut config = load_agent_config();
    let body: Value = match serde_json::from_slice(&req.body) {
        Ok(v) => v,
        Err(e) => {
            let _ = write_json(w, 400, &json!({ "error": format!("请求格式错误: {e}") }));
            return;
        }
    };
    if let Some(v) = body["provider"].as_str() {
        if ProviderKind::from_str(v).is_none() {
            let _ = write_json(
                w,
                400,
                &json!({ "error": "Provider 无效（支持 openai-compatible / opencode / ollama）" }),
            );
            return;
        }
        config.provider = v.to_string();
    }
    if let Some(v) = body["base_url"].as_str() {
        config.base_url = v.to_string();
    }
    if let Some(v) = body["model"].as_str() {
        config.model = v.to_string();
    }
    if let Some(v) = body["permission"].as_str() {
        if PermissionLevel::from_str(v).is_none() {
            let _ = write_json(
                w,
                400,
                &json!({ "error": "权限级别无效（read_only / ask_before_execute / auto_safe / full_access）" }),
            );
            return;
        }
        config.permission = v.to_string();
    }
    if let Some(v) = body["timeout_secs"].as_u64() {
        config.timeout_secs = v.clamp(5, 600);
    }
    match save_agent_config(&config) {
        Ok(_) => {
            let _ = write_json(w, 200, &json!({ "ok": true }));
        }
        Err(e) => {
            let _ = write_json(w, 500, &json!({ "error": format!("保存失败: {e}") }));
        }
    }
}

/// POST /api/test — 用（表单）配置向 Provider 发最小请求，验证连接
fn handle_test(w: &mut TcpStream, req: &Request) {
    let mut config = load_agent_config();
    let body: Value = serde_json::from_slice(&req.body).unwrap_or(Value::Null);
    if let Some(v) = body["provider"].as_str() {
        if ProviderKind::from_str(v).is_some() {
            config.provider = v.to_string();
        }
    }
    if let Some(v) = body["base_url"].as_str() {
        config.base_url = v.to_string();
    }
    if let Some(v) = body["model"].as_str() {
        config.model = v.to_string();
    }
    if let Some(v) = body["timeout_secs"].as_u64() {
        config.timeout_secs = v.clamp(5, 60);
    }
    let provider_config = config.to_provider_config();
    let ping = ChatMessage {
        role: "user".to_string(),
        content: "回复 OK 即可，不要调用任何工具。".to_string(),
    };
    let start = Instant::now();
    match chat(&provider_config, &[ping], "") {
        Ok(reply) => {
            let elapsed_ms = start.elapsed().as_millis();
            let _ = write_json(
                w,
                200,
                &json!({
                    "ok": true,
                    "elapsed_ms": elapsed_ms,
                    "reply": truncate(&reply.text, 200),
                }),
            );
        }
        Err(e) => {
            let _ = write_json(w, 200, &json!({ "ok": false, "error": e.to_string() }));
        }
    }
}

/// 截断长文本（测试结果展示用）
fn truncate(text: &str, max: usize) -> String {
    let mut s = text.trim().to_string();
    if s.chars().count() > max {
        s = s.chars().take(max).collect::<String>();
        s.push('…');
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reason_phrase_mapping() {
        assert_eq!(reason_phrase(200), "OK");
        assert_eq!(reason_phrase(400), "Bad Request");
        assert_eq!(reason_phrase(404), "Not Found");
        assert_eq!(reason_phrase(500), "Internal Server Error");
        assert_eq!(reason_phrase(299), "OK");
    }

    #[test]
    fn request_parse_drops_query() {
        let raw = "GET /api/status?x=1 HTTP/1.1\r\nHost: localhost\r\n\r\n";
        let mut reader = BufReader::new(raw.as_bytes());
        let req = read_request(&mut reader).expect("parse ok");
        assert_eq!(req.method, "GET");
        assert_eq!(req.path, "/api/status");
        assert!(req.body.is_empty());
    }

    #[test]
    fn request_parse_reads_body() {
        let raw = "POST /api/config HTTP/1.1\r\nContent-Type: application/json\r\nContent-Length: 15\r\n\r\n{\"a\":\"b\",\"c\":1}";
        let mut reader = BufReader::new(raw.as_bytes());
        let req = read_request(&mut reader).expect("parse ok");
        assert_eq!(req.method, "POST");
        assert_eq!(req.path, "/api/config");
        let parsed: Value = serde_json::from_slice(&req.body).expect("json ok");
        assert_eq!(parsed["a"], "b");
        assert_eq!(parsed["c"], 1);
    }

    #[test]
    fn truncate_short_text_unchanged() {
        assert_eq!(truncate("hello", 10), "hello");
    }

    #[test]
    fn truncate_long_text_marks() {
        let s = truncate("你好世界，这是一个比较长的回复内容", 5);
        assert!(s.ends_with('…'));
    }
}
