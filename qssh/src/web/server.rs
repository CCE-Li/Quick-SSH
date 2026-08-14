//! 极简 HTTP 服务器 + WebUI 路由（零新增依赖）
//!
//! 使用 `std::net::TcpListener` 手写 HTTP/1.1 解析，避免引入重型框架。
//! Agent 会话复用 [`crate::agent`] 现有能力：`AgentRunner` + `SshProcessExecutor` + `Collector`。

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;

use serde_json::{json, Value};

use crate::agent::config::load_agent_config;
use crate::agent::permissions::PermissionLevel;
use crate::agent::provider::{ChatMessage, ProviderKind};
use crate::agent::timeline::Timeline;
use crate::agent::tools::ToolCall;
use crate::agent::{AgentEvent, AgentRunner, AgentSession};
use crate::config::{default_config_path, parser};
use crate::monitor::executor::SshProcessExecutor;
use crate::monitor::platform::Collector;
use crate::ssh::session::SshTarget;

use super::WEBUI_VERSION;

/// 内嵌前端页面（src/web/index.html）
const HTML: &str = include_str!("index.html");

/// 审批等待的最长时长
const APPROVAL_TIMEOUT: Duration = Duration::from_secs(600);

/// 全局服务状态
struct AppState {
    /// 审批等待表：token → 审批响应通道
    pending: Mutex<HashMap<String, mpsc::Sender<bool>>>,
    /// 审批 token 计数器
    counter: AtomicU64,
}

/// 启动 WebUI 服务（阻塞）
pub fn serve(port: u16) -> anyhow::Result<()> {
    let addr = format!("127.0.0.1:{port}");
    let listener = TcpListener::bind(&addr)?;
    println!("🌐 Quick-SSH WebUI 已启动：http://{addr}");
    println!("   按 Ctrl+C 停止服务");
    let state = Arc::new(AppState {
        pending: Mutex::new(HashMap::new()),
        counter: AtomicU64::new(1),
    });
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let state = Arc::clone(&state);
                std::thread::spawn(move || handle_connection(stream, state));
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
        "HTTP/1.1 {status} {}\r\nContent-Type: {ctype}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        reason_phrase(status),
        body.len()
    );
    w.write_all(head.as_bytes())?;
    w.write_all(body.as_bytes())?;
    w.flush()
}

fn write_json(w: &mut TcpStream, status: u16, body: &Value) -> std::io::Result<()> {
    let payload = body.to_string();
    write_simple(w, status, "application/json; charset=utf-8", &payload)
}

/// 写入一条 SSE 事件：`data: {json}\n\n`
fn write_sse(w: &mut TcpStream, data: &Value) -> std::io::Result<()> {
    let payload = data.to_string();
    let block = format!("data: {payload}\n\n");
    w.write_all(block.as_bytes())?;
    w.flush()
}

// ── 连接处理与路由 ────────────────────────────────────────

fn handle_connection(stream: TcpStream, state: Arc<AppState>) {
    let mut writer = match stream.try_clone() {
        Ok(w) => w,
        Err(_) => return,
    };
    let _ = stream.set_read_timeout(Some(Duration::from_secs(120)));
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
        ("GET", "/api/hosts") => handle_hosts(&mut writer),
        ("POST", "/api/chat") => handle_chat(&mut writer, &req, &state),
        ("POST", "/api/approve") => handle_approve(&mut writer, &req, &state),
        ("GET", "/api/config") => handle_config_get(&mut writer),
        ("POST", "/api/config") => handle_config_post(&mut writer, &req),
        _ => {
            let _ = write_simple(&mut writer, 404, "text/plain", "Not Found");
        }
    }
}

// ── API Handlers ──────────────────────────────────────────

/// GET /api/status — 运行状态（provider / 权限 / 主机数）
fn handle_status(w: &mut TcpStream) {
    let config = load_agent_config();
    let host_count = parser::parse_config(&default_config_path())
        .map(|c| c.hosts.len())
        .unwrap_or(0);
    let kind = ProviderKind::from_str(&config.provider).unwrap_or_default();
    let provider_label = match kind {
        ProviderKind::OpenAI => "OpenAI 兼容",
        ProviderKind::OpenCodeGo => "opencode-go",
        ProviderKind::Ollama => "Ollama",
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
            "host_count": host_count,
        }),
    );
}

/// GET /api/hosts — SSH 主机列表
fn handle_hosts(w: &mut TcpStream) {
    let ssh_config = match parser::parse_config(&default_config_path()) {
        Ok(c) => c,
        Err(e) => {
            let _ = write_json(
                w,
                500,
                &json!({ "error": format!("读取 SSH 配置失败: {e}") }),
            );
            return;
        }
    };
    let hosts: Vec<Value> = ssh_config
        .hosts
        .iter()
        .map(|h| {
            json!({
                "alias": h.alias,
                "hostname": h.hostname().unwrap_or(""),
                "user": h.user().unwrap_or(""),
                "port": h.port(),
                "identity_file": h.identity_file().map(|p| p.display().to_string()).unwrap_or_default(),
            })
        })
        .collect();
    let _ = write_json(w, 200, &json!({ "hosts": hosts }));
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
                &json!({ "error": "Provider 无效（支持 openai / opencode / ollama）" }),
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
    match crate::agent::config::save_agent_config(&config) {
        Ok(_) => {
            let saved = load_agent_config();
            let _ = write_json(
                w,
                200,
                &json!({ "ok": true, "config": serde_json::to_value(&saved).unwrap_or(Value::Null) }),
            );
        }
        Err(e) => {
            let _ = write_json(w, 500, &json!({ "error": format!("保存失败: {e}") }));
        }
    }
}

/// POST /api/approve — 危险操作审批回调
fn handle_approve(w: &mut TcpStream, req: &Request, state: &Arc<AppState>) {
    let body: Value = match serde_json::from_slice(&req.body) {
        Ok(v) => v,
        Err(e) => {
            let _ = write_json(w, 400, &json!({ "error": format!("请求格式错误: {e}") }));
            return;
        }
    };
    let token = body["token"].as_str().unwrap_or("");
    let approved = body["approve"].as_bool().unwrap_or(false);
    let sender = {
        let mut guard = match state.pending.lock() {
            Ok(g) => g,
            Err(_) => {
                let _ = write_json(w, 500, &json!({ "error": "服务状态不可用" }));
                return;
            }
        };
        guard.remove(token)
    };
    match sender {
        Some(tx) => {
            let _ = tx.send(approved);
            let _ = write_json(w, 200, &json!({ "ok": true }));
        }
        None => {
            let _ = write_json(w, 404, &json!({ "error": "审批请求不存在或已过期" }));
        }
    }
}

/// POST /api/chat — 运行 Agent 会话，以 SSE 流式回报事件
fn handle_chat(w: &mut TcpStream, req: &Request, state: &Arc<AppState>) {
    let body: Value = match serde_json::from_slice(&req.body) {
        Ok(v) => v,
        Err(e) => {
            let _ = write_json(w, 400, &json!({ "error": format!("请求格式错误: {e}") }));
            return;
        }
    };
    let message = body["message"].as_str().unwrap_or("").trim().to_string();
    let alias = body["alias"].as_str().unwrap_or("").trim().to_string();
    if message.is_empty() {
        let _ = write_json(w, 400, &json!({ "error": "消息不能为空" }));
        return;
    }

    // 解析目标主机
    let ssh_config = match parser::parse_config(&default_config_path()) {
        Ok(c) => c,
        Err(e) => {
            let _ = write_json(
                w,
                500,
                &json!({ "error": format!("读取 SSH 配置失败: {e}") }),
            );
            return;
        }
    };
    let host = ssh_config.hosts.iter().find(|h| h.alias == alias).cloned();
    let Some(host) = host else {
        let _ = write_json(w, 400, &json!({ "error": format!("未找到主机: {alias}") }));
        return;
    };

    // 对话历史
    let mut history: Vec<ChatMessage> = Vec::new();
    if let Some(arr) = body["history"].as_array() {
        for item in arr {
            let role = item["role"].as_str().unwrap_or("user").to_string();
            let content = item["content"].as_str().unwrap_or("").to_string();
            history.push(ChatMessage { role, content });
        }
    }

    // SSE 响应头（允许跨域，便于本地调试）
    let _ = w.write_all(
        b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\nConnection: keep-alive\r\nAccess-Control-Allow-Origin: *\r\n\r\n",
    );

    // 执行器 + 快照采集
    let executor = SshProcessExecutor::new(SshTarget::from_host(&host));
    let collector = Collector::new(Box::new(SshProcessExecutor::new(SshTarget::from_host(
        &host,
    ))));
    let snapshot = collector.collect(&alias);

    // 后台线程运行 Agent 会话
    let agent_config = load_agent_config();
    let (event_tx, event_rx) = mpsc::channel::<AgentEvent>();
    let (session_tx, session_rx) = mpsc::channel::<AgentSession>();
    let state_clone = Arc::clone(state);
    let runner = AgentRunner::new(agent_config, event_tx);
    let (signal_tx, signal_rx) = mpsc::channel::<(String, String, String, String)>();
    let executor_for_thread = executor;

    std::thread::spawn(move || {
        let approve = move |call: &ToolCall| -> bool {
            let token = state_clone
                .counter
                .fetch_add(1, Ordering::Relaxed)
                .to_string();
            let (resp_tx, resp_rx) = mpsc::channel::<bool>();
            if let Ok(mut guard) = state_clone.pending.lock() {
                guard.insert(token.clone(), resp_tx);
            }
            let reason = format!(
                "工具 {}（{}）请求执行 {} 操作",
                call.name(),
                call.tool.description(),
                call.tool.danger().label()
            );
            let _ = signal_tx.send((
                token.clone(),
                call.name().to_string(),
                call.tool.description().to_string(),
                reason,
            ));
            resp_rx.recv_timeout(APPROVAL_TIMEOUT).unwrap_or(false)
        };
        let session = runner.run(&history, &message, &executor_for_thread, &snapshot, approve);
        let _ = session_tx.send(session);
    });

    // 消费事件 → SSE 流
    loop {
        match event_rx.recv_timeout(Duration::from_millis(200)) {
            Ok(event) => match event {
                AgentEvent::Status(status) => {
                    let _ = write_sse(w, &json!({ "type": "status", "status": status.label() }));
                }
                AgentEvent::Timeline(timeline) => {
                    let _ = write_sse(
                        w,
                        &json!({ "type": "timeline", "timeline": timeline_json(&timeline) }),
                    );
                }
                AgentEvent::ApprovalNeeded { .. } => {
                    // token 由 approve 回调经 signal 通道送达（见下方 Timeout 分支）
                }
                AgentEvent::Started => {}
                AgentEvent::Finished { reply, error } => {
                    let _ = write_sse(
                        w,
                        &json!({ "type": "partial_finish", "reply": reply, "error": error }),
                    );
                }
            },
            Err(mpsc::RecvTimeoutError::Timeout) => {
                // 有新的审批待处理（approve 回调注册后经 signal 通知）
                while let Ok((token, name, description, reason)) = signal_rx.try_recv() {
                    let _ = write_sse(
                        w,
                        &json!({
                            "type": "approval",
                            "token": token,
                            "tool": name,
                            "description": description,
                            "reason": reason,
                        }),
                    );
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }

    // 收取最终会话结果
    match session_rx.recv_timeout(Duration::from_secs(10)) {
        Ok(session) => {
            let _ = write_sse(
                w,
                &json!({
                    "type": "finished",
                    "reply": session.reply,
                    "error": session.error,
                    "timeline": timeline_json(&session.timeline),
                }),
            );
        }
        Err(_) => {
            let _ = write_sse(
                w,
                &json!({ "type": "finished", "reply": "", "error": "会话结果丢失", "timeline": {} }),
            );
        }
    }
    let _ = w.flush();
}

/// 时间线序列化为 JSON（Timeline 无 Serialize derive，手动构造）
fn timeline_json(timeline: &Timeline) -> Value {
    json!({
        "steps": timeline.steps().iter().map(|s| {
            json!({
                "name": s.name,
                "description": s.description,
                "read_only": s.read_only,
                "status": s.status.label(),
                "symbol": s.status.symbol(),
                "detail": s.detail,
            })
        }).collect::<Vec<_>>()
    })
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
    fn timeline_json_serializes_steps() {
        let mut timeline = Timeline::new();
        timeline.push(crate::agent::timeline::TimelineStep::new(
            "server.status",
            "服务器状态",
            true,
        ));
        let value = timeline_json(&timeline);
        let steps = value["steps"].as_array().expect("steps array");
        assert_eq!(steps.len(), 1);
        assert_eq!(steps[0]["name"], "server.status");
        assert_eq!(steps[0]["status"], "等待");
        assert_eq!(steps[0]["symbol"], "○");
    }

    #[test]
    fn sse_format_contains_data_prefix() {
        // write_sse 需要 TcpStream，这里仅验证 JSON 结构
        let payload = json!({ "type": "status", "status": "◇ Thinking" });
        let s = format!("data: {payload}\n\n");
        assert!(s.starts_with("data: "));
        assert!(s.ends_with("\n\n"));
    }

    #[test]
    fn request_parse_drops_query() {
        // 用内存缓冲模拟请求行解析逻辑
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
}
