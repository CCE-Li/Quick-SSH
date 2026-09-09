//! Schema 驱动的 WebUI 设置表单模型
//!
//! 目标：加一个新的可设置项时，前端 HTML/JS **无需改动** —— 后端把每个分组
//! （[`GroupSpec`]）的字段描述（控件类型 / 选项 / 校验 / 提示）与当前值一起下发，
//! 前端按元数据通用渲染。每个分组对应一个配置文件存储（agent.json / `~/.qsshrc`
//! / dashboard.json / 未来新增），由 [`SettingsCatalog::load_values`] /
//! [`SettingsCatalog::apply`] 读写。
//!
//! 新增一个可设置项的流程：
//! 1. 在对应分组的 `fields` 里加一条 [`FieldSpec`]；
//! 2. 在对应的 `apply_*_to` 里加一条读写分支（校验 + 落盘映射）。
//!
//! 前后端页面无需再改。

use serde_json::{json, Map, Value};

use crate::agent::config::{load_agent_config, save_agent_config, AgentConfig};
use crate::agent::permissions::PermissionLevel;
use crate::agent::provider::ProviderKind;
use crate::config::settings::{load_settings, save_settings, QsshSettings};
use crate::tui::dashboard::config::{
    default_layout_for, load_dashboard_config, save_dashboard_config, DashboardConfig, WidgetId,
};

/// 字段控件类型
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FieldKind {
    /// 单行文本
    Text,
    /// 密码（不回显）
    Password,
    /// 数字（可带 min/max）
    Number,
    /// 下拉选择（单选）
    Select,
    /// 多选（复选框组）
    MultiSelect,
}

impl FieldKind {
    fn as_str(self) -> &'static str {
        match self {
            FieldKind::Text => "text",
            FieldKind::Password => "password",
            FieldKind::Number => "number",
            FieldKind::Select => "select",
            FieldKind::MultiSelect => "multiselect",
        }
    }
}

/// 下拉选项
pub struct FieldChoice {
    pub value: &'static str,
    pub label: &'static str,
}

/// 字段的选项来源
pub enum FieldOptions {
    /// 静态选项（编译期确定，如模块清单 / 权限级别）
    Static(&'static [FieldChoice]),
    /// 动态选项：从 dashboard 配置读取 profile 名称
    Profiles,
}

/// 单个可设置项的描述
pub struct FieldSpec {
    /// 传给后端的字段键（保存时用）
    pub key: &'static str,
    /// 中文标签
    pub label: &'static str,
    pub kind: FieldKind,
    pub placeholder: &'static str,
    pub hint: &'static str,
    pub options: FieldOptions,
    pub min: Option<i64>,
    pub max: Option<i64>,
    /// 该字段变化时前端是否需要联动刷新（如选择 Profile 后重填模块复选框）
    pub reload_on_change: bool,
}

/// 分组的「测试动作」（可选）：渲染成一个按钮，把本组当前字段值 POST 到 `endpoint`
pub struct TestActionSpec {
    pub label: &'static str,
    pub endpoint: &'static str,
}

/// 一个设置分组（对应一个配置文件存储）
pub struct GroupSpec {
    pub id: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    /// 配置文件路径（前端展示用）
    pub path: &'static str,
    pub fields: &'static [FieldSpec],
    pub test: Option<TestActionSpec>,
}

impl GroupSpec {
    /// 解析某字段的选项（支持动态来源）
    fn resolve_options(f: &FieldSpec) -> Vec<Value> {
        match f.options {
            FieldOptions::Static(choices) => choices
                .iter()
                .map(|o| json!({ "value": o.value, "label": o.label }))
                .collect(),
            FieldOptions::Profiles => load_dashboard_config()
                .profiles
                .keys()
                .map(|name| json!({ "value": name, "label": name }))
                .collect(),
        }
    }

    /// 输出字段元数据（不含当前值），供前端通用渲染
    fn to_meta(&self) -> Value {
        json!({
            "id": self.id,
            "title": self.title,
            "description": self.description,
            "path": self.path,
            "fields": self.fields.iter().map(|f| json!({
                "key": f.key,
                "label": f.label,
                "kind": f.kind.as_str(),
                "placeholder": f.placeholder,
                "hint": f.hint,
                "min": f.min,
                "max": f.max,
                "reload_on_change": f.reload_on_change,
                "options": Self::resolve_options(f),
            })).collect::<Vec<_>>(),
            "test": self.test.as_ref()
                .map(|t| json!({ "label": t.label, "endpoint": t.endpoint })),
        })
    }
}

// ── 分组目录 ─────────────────────────────────────────────

const PROVIDER_CHOICES: &[FieldChoice] = &[
    FieldChoice {
        value: "opencode",
        label: "opencode-go（推荐）",
    },
    FieldChoice {
        value: "openai",
        label: "OpenAI Compatible",
    },
    FieldChoice {
        value: "ollama",
        label: "Ollama",
    },
];

const PERMISSION_CHOICES: &[FieldChoice] = &[
    FieldChoice {
        value: "ask_before_execute",
        label: "执行前确认（默认）",
    },
    FieldChoice {
        value: "read_only",
        label: "只读",
    },
    FieldChoice {
        value: "auto_safe",
        label: "自动安全",
    },
    FieldChoice {
        value: "full_access",
        label: "完全访问",
    },
];

static WIDGET_CHOICES: &[FieldChoice] = &[
    FieldChoice {
        value: "cpu",
        label: "CPU",
    },
    FieldChoice {
        value: "memory",
        label: "Memory",
    },
    FieldChoice {
        value: "disk",
        label: "Disk",
    },
    FieldChoice {
        value: "network",
        label: "Network",
    },
    FieldChoice {
        value: "docker",
        label: "Docker",
    },
    FieldChoice {
        value: "process",
        label: "Processes",
    },
    FieldChoice {
        value: "services",
        label: "Services",
    },
    FieldChoice {
        value: "files",
        label: "Files",
    },
    FieldChoice {
        value: "logs",
        label: "Logs",
    },
    FieldChoice {
        value: "system-info",
        label: "System Info",
    },
    FieldChoice {
        value: "agent",
        label: "AI Agent",
    },
    FieldChoice {
        value: "terminal",
        label: "Terminal",
    },
];

static AGENT_FIELDS: &[FieldSpec] = &[
    FieldSpec {
        key: "provider",
        label: "Provider（供应商）",
        kind: FieldKind::Select,
        placeholder: "",
        hint: "",
        options: FieldOptions::Static(PROVIDER_CHOICES),
        min: None,
        max: None,
        reload_on_change: false,
    },
    FieldSpec {
        key: "api_key",
        label: "API Key",
        kind: FieldKind::Password,
        placeholder: "留空则使用环境变量 / opencode auth.json",
        hint: "",
        options: FieldOptions::Static(&[]),
        min: None,
        max: None,
        reload_on_change: false,
    },
    FieldSpec {
        key: "base_url",
        label: "Base URL",
        kind: FieldKind::Text,
        placeholder: "https://api.openai.com/v1",
        hint: "",
        options: FieldOptions::Static(&[]),
        min: None,
        max: None,
        reload_on_change: false,
    },
    FieldSpec {
        key: "model",
        label: "Model（连接成功后自动获取）",
        kind: FieldKind::Text,
        placeholder: "点击测试连接后自动填充模型列表",
        hint: "",
        options: FieldOptions::Static(&[]),
        min: None,
        max: None,
        reload_on_change: false,
    },
    FieldSpec {
        key: "permission",
        label: "权限级别",
        kind: FieldKind::Select,
        placeholder: "",
        hint: "",
        options: FieldOptions::Static(PERMISSION_CHOICES),
        min: None,
        max: None,
        reload_on_change: false,
    },
    FieldSpec {
        key: "timeout_secs",
        label: "请求超时（秒）",
        kind: FieldKind::Number,
        placeholder: "",
        hint: "",
        options: FieldOptions::Static(&[]),
        min: Some(5),
        max: Some(600),
        reload_on_change: false,
    },
];

static PROGRAM_FIELDS: &[FieldSpec] = &[
    FieldSpec {
        key: "default_port",
        label: "默认 SSH 端口",
        kind: FieldKind::Number,
        placeholder: "",
        hint: "",
        options: FieldOptions::Static(&[]),
        min: Some(1),
        max: Some(65535),
        reload_on_change: false,
    },
    FieldSpec {
        key: "ping_timeout_secs",
        label: "TCP 超时（秒）",
        kind: FieldKind::Number,
        placeholder: "",
        hint: "",
        options: FieldOptions::Static(&[]),
        min: Some(1),
        max: Some(60),
        reload_on_change: false,
    },
    FieldSpec {
        key: "upload_concurrency",
        label: "上传并发数",
        kind: FieldKind::Number,
        placeholder: "",
        hint: "",
        options: FieldOptions::Static(&[]),
        min: Some(1),
        max: Some(16),
        reload_on_change: false,
    },
];

static DASHBOARD_FIELDS: &[FieldSpec] = &[
    FieldSpec {
        key: "active_profile",
        label: "活动 Profile",
        kind: FieldKind::Select,
        placeholder: "",
        hint: "切换 Profile 后，下方「启用的模块」会随之更新",
        options: FieldOptions::Profiles,
        min: None,
        max: None,
        reload_on_change: true,
    },
    FieldSpec {
        key: "enabled",
        label: "启用的模块（决定布局）",
        kind: FieldKind::MultiSelect,
        placeholder: "",
        hint: "保存后按启用列表自动重建布局（如 Terminal 作为中央主区域）。",
        options: FieldOptions::Static(WIDGET_CHOICES),
        min: None,
        max: None,
        reload_on_change: false,
    },
];

/// 设置目录：所有分组在这里注册
pub struct SettingsCatalog;

impl SettingsCatalog {
    pub fn groups() -> &'static [GroupSpec] {
        static GROUPS: &[GroupSpec] = &[
            GroupSpec {
                id: "agent",
                title: "Agent 配置",
                description: "配置 AI Agent 的供应商、模型与权限级别。",
                path: "~/.config/quick-ssh/agent.json",
                fields: AGENT_FIELDS,
                test: Some(TestActionSpec {
                    label: "测试连接",
                    endpoint: "/api/test",
                }),
            },
            GroupSpec {
                id: "program",
                title: "程序设置",
                description: "全局程序运行参数（预留，写入 ~/.qsshrc）。",
                path: "~/.qsshrc",
                fields: PROGRAM_FIELDS,
                test: None,
            },
            GroupSpec {
                id: "dashboard",
                title: "Dashboard 布局",
                description: "选择活动 Profile，并配置该 Profile 启用的监控 / 终端 / Agent 模块。",
                path: "~/.config/quick-ssh/dashboard.json",
                fields: DASHBOARD_FIELDS,
                test: None,
            },
        ];
        GROUPS
    }

    /// 读取所有分组：元数据 + 当前值
    pub fn load_all() -> Value {
        let groups = Self::groups()
            .iter()
            .map(|g| {
                let mut meta = g.to_meta();
                meta["values"] = Self::load_values(g.id);
                meta
            })
            .collect::<Vec<_>>();
        json!({ "groups": groups })
    }

    /// 读取某分组的当前值（只包含已注册字段）
    fn load_values(id: &str) -> Value {
        match id {
            "agent" => agent_values(),
            "program" => program_values(),
            "dashboard" => dashboard_values(),
            _ => json!({}),
        }
    }

    /// 应用某个分组的字段更新并落盘
    pub fn apply(id: &str, values: &Value) -> Result<(), String> {
        match id {
            "agent" => {
                let mut config = load_agent_config();
                apply_agent_to(&mut config, values)?;
                save_agent_config(&config).map_err(|e| format!("保存失败: {e}"))
            }
            "program" => {
                let mut settings = load_settings().unwrap_or_default();
                apply_program_to(&mut settings, values)?;
                save_settings(&settings).map_err(|e| format!("保存失败: {e}"))
            }
            "dashboard" => {
                let mut config = load_dashboard_config();
                apply_dashboard_to(&mut config, values)?;
                save_dashboard_config(&config).map_err(|e| format!("保存失败: {e}"))
            }
            _ => Err(format!("未知分组: {id}")),
        }
    }
}

// ── Agent 分组 ───────────────────────────────────────────

fn agent_values() -> Value {
    let c = load_agent_config();
    json!({
        "provider": ProviderKind::from_str(&c.provider).unwrap_or_default().as_str(),
        "base_url": c.base_url,
        "model": c.model,
        "permission": c.permission,
        "timeout_secs": c.timeout_secs,
        "api_key": c.api_key,
    })
}

/// 把请求里的字段更新应用到 `config`（只做校验与赋值，不落盘）
fn apply_agent_to(config: &mut AgentConfig, v: &Value) -> Result<(), String> {
    if let Some(val) = v.get("provider").and_then(Value::as_str) {
        if ProviderKind::from_str(val).is_none() {
            return Err("Provider 无效（支持 openai-compatible / opencode / ollama）".to_string());
        }
        config.provider = val.to_string();
    }
    if let Some(val) = v.get("base_url").and_then(Value::as_str) {
        config.base_url = val.to_string();
    }
    if let Some(val) = v.get("model").and_then(Value::as_str) {
        config.model = val.to_string();
    }
    if let Some(val) = v.get("permission").and_then(Value::as_str) {
        if PermissionLevel::from_str(val).is_none() {
            return Err(
                "权限级别无效（read_only / ask_before_execute / auto_safe / full_access)"
                    .to_string(),
            );
        }
        config.permission = val.to_string();
    }
    if let Some(val) = v.get("timeout_secs").and_then(Value::as_u64) {
        config.timeout_secs = val.clamp(5, 600);
    }
    if let Some(val) = v.get("api_key").and_then(Value::as_str) {
        config.api_key = val.trim().to_string();
    }
    Ok(())
}

// ── 程序设置分组 ─────────────────────────────────────────

fn program_values() -> Value {
    let s = load_settings().unwrap_or_default();
    json!({
        "default_port": s.default_port,
        "ping_timeout_secs": s.ping_timeout_secs,
        "upload_concurrency": s.upload_concurrency,
    })
}

/// 把请求里的字段更新应用到 `settings`（只做校验与赋值，不落盘）
fn apply_program_to(settings: &mut QsshSettings, v: &Value) -> Result<(), String> {
    if let Some(val) = v.get("default_port").and_then(Value::as_u64) {
        settings.default_port = val.clamp(1, 65535) as u16;
    }
    if let Some(val) = v.get("ping_timeout_secs").and_then(Value::as_u64) {
        settings.ping_timeout_secs = val.clamp(1, 60);
    }
    if let Some(val) = v.get("upload_concurrency").and_then(Value::as_u64) {
        settings.upload_concurrency = val.clamp(1, 16) as usize;
    }
    Ok(())
}

// ── Dashboard 布局分组 ───────────────────────────────────

/// Widget 的 kebab-case 键（与 `serde` 序列化一致）
fn widget_key(w: WidgetId) -> String {
    serde_json::to_string(&w)
        .unwrap_or_default()
        .trim_matches('"')
        .to_string()
}

/// 从 kebab-case 键解析 Widget
fn widget_from_key(s: &str) -> Option<WidgetId> {
    serde_json::from_str::<WidgetId>(&format!("\"{s}\"")).ok()
}

fn dashboard_values() -> Value {
    let cfg = load_dashboard_config();
    let active = cfg.active_profile.clone();
    // 每个 Profile 的启用模块，供前端在选择 Profile 时本地切换复选框
    let mut profiles: Map<String, Value> = Map::new();
    for (name, p) in &cfg.profiles {
        let enabled: Vec<Value> = p.enabled.iter().map(|&w| json!(widget_key(w))).collect();
        profiles.insert(name.clone(), Value::Array(enabled));
    }
    let enabled: Vec<Value> = cfg
        .profiles
        .get(&active)
        .map(|p| p.enabled.iter().map(|&w| json!(widget_key(w))).collect())
        .unwrap_or_default();
    json!({
        "active_profile": active,
        "enabled": enabled,
        "_profiles": Value::Object(profiles),
    })
}

/// 把请求里的字段更新应用到 `config`（只做校验与赋值，不落盘）
fn apply_dashboard_to(config: &mut DashboardConfig, v: &Value) -> Result<(), String> {
    if let Some(profile) = v.get("active_profile").and_then(Value::as_str) {
        if !config.profiles.contains_key(profile) {
            return Err(format!("Profile 不存在: {profile}"));
        }
        config.active_profile = profile.to_string();
    }
    if let Some(arr) = v.get("enabled").and_then(Value::as_array) {
        let mut enabled = Vec::new();
        for val in arr {
            let s = val
                .as_str()
                .ok_or_else(|| "enabled 元素必须是字符串".to_string())?;
            let w = widget_from_key(s).ok_or_else(|| format!("未知模块: {s}"))?;
            enabled.push(w);
        }
        let active = config.active_profile.clone();
        let profile = config
            .profiles
            .get_mut(&active)
            .ok_or_else(|| format!("Profile 不存在: {active}"))?;
        profile.enabled = enabled;
        // 布局始终按启用列表重建，保证一致（见 dashboard/config.rs）
        profile.layout = default_layout_for(&profile.enabled);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::dashboard::config::{default_dashboard_config, ALL_WIDGETS};

    /// 全部可用 Widget 键（与 `ALL_WIDGETS` 一致）
    fn all_widget_keys() -> Vec<String> {
        ALL_WIDGETS.iter().map(|&w| widget_key(w)).collect()
    }

    #[test]
    fn agent_values_contains_all_fields() {
        let v = agent_values();
        for key in [
            "provider",
            "base_url",
            "model",
            "permission",
            "timeout_secs",
            "api_key",
        ] {
            assert!(v.get(key).is_some(), "缺少字段 {key}");
        }
    }

    #[test]
    fn agent_apply_rejects_invalid_provider_without_saving() {
        let mut c = AgentConfig::default();
        let err = apply_agent_to(&mut c, &json!({ "provider": "bogus" })).unwrap_err();
        assert!(err.contains("Provider 无效"));
        // 校验失败时不应改动已有配置
        assert_eq!(c.provider, AgentConfig::default().provider);
    }

    #[test]
    fn agent_apply_rejects_invalid_permission() {
        let mut c = AgentConfig::default();
        assert!(apply_agent_to(&mut c, &json!({ "permission": "hack" })).is_err());
    }

    #[test]
    fn agent_apply_mutates_fields() {
        let mut c = AgentConfig::default();
        apply_agent_to(
            &mut c,
            &json!({
                "provider": "ollama",
                "model": "qwen2.5:7b",
                "timeout_secs": 30,
                "api_key": "  sk-123  ",
            }),
        )
        .unwrap();
        assert_eq!(c.provider, "ollama");
        assert_eq!(c.model, "qwen2.5:7b");
        assert_eq!(c.timeout_secs, 30);
        assert_eq!(c.api_key, "sk-123");
    }

    #[test]
    fn agent_apply_clamps_timeout() {
        let mut c = AgentConfig::default();
        apply_agent_to(&mut c, &json!({ "timeout_secs": 99999 })).unwrap();
        assert_eq!(c.timeout_secs, 600);
    }

    #[test]
    fn program_apply_mutates_and_clamps() {
        let mut s = QsshSettings::default();
        apply_program_to(
            &mut s,
            &json!({ "default_port": 2222, "ping_timeout_secs": 5, "upload_concurrency": 99 }),
        )
        .unwrap();
        assert_eq!(s.default_port, 2222);
        assert_eq!(s.ping_timeout_secs, 5);
        assert_eq!(s.upload_concurrency, 16);
    }

    #[test]
    fn widget_key_roundtrip() {
        let keys = all_widget_keys();
        assert_eq!(keys.len(), ALL_WIDGETS.len());
        for k in keys {
            let w = widget_from_key(&k).unwrap_or_else(|| panic!("无法解析 {k}"));
            assert_eq!(widget_key(w), k);
        }
        assert!(widget_from_key("bogus").is_none());
    }

    #[test]
    fn dashboard_apply_sets_enabled_and_rebuilds_layout() {
        let mut cfg = default_dashboard_config();
        apply_dashboard_to(
            &mut cfg,
            &json!({ "enabled": ["cpu", "memory", "network"] }),
        )
        .unwrap();
        let profile = &cfg.profiles["Default"];
        assert_eq!(
            profile.enabled,
            vec![WidgetId::Cpu, WidgetId::Memory, WidgetId::Network]
        );
        // 布局应包含所有启用模块
        let mut found = Vec::new();
        crate::tui::dashboard::layout::collect_widget_ids(&profile.layout, &mut found);
        assert_eq!(found, profile.enabled);
    }

    #[test]
    fn dashboard_apply_switches_active_profile() {
        let mut cfg = default_dashboard_config();
        apply_dashboard_to(&mut cfg, &json!({ "active_profile": "AI" })).unwrap();
        assert_eq!(cfg.active_profile, "AI");
    }

    #[test]
    fn dashboard_apply_rejects_unknown_profile() {
        let mut cfg = default_dashboard_config();
        assert!(apply_dashboard_to(&mut cfg, &json!({ "active_profile": "Nope" })).is_err());
        assert!(apply_dashboard_to(&mut cfg, &json!({ "enabled": ["nope"] })).is_err());
    }

    #[test]
    fn load_all_returns_meta_and_values() {
        let v = SettingsCatalog::load_all();
        let groups = v["groups"].as_array().unwrap();
        assert!(
            groups.len() >= 3,
            "至少应有 agent / program / dashboard 分组"
        );
        let agent = groups
            .iter()
            .find(|g| g["id"] == "agent")
            .expect("应有 agent 分组");
        assert_eq!(agent["test"]["label"], "测试连接");
        // 字段元数据里含类型
        let fields = agent["fields"].as_array().unwrap();
        let provider = fields.iter().find(|f| f["key"] == "provider").unwrap();
        assert_eq!(provider["kind"], "select");
        assert!(agent["values"]["provider"].is_string());

        // dashboard 分组：动态 profile 选项 + 多选模块 + 本地切换数据
        let dash = groups
            .iter()
            .find(|g| g["id"] == "dashboard")
            .expect("应有 dashboard 分组");
        let dash_fields = dash["fields"].as_array().unwrap();
        let profile_field = dash_fields
            .iter()
            .find(|f| f["key"] == "active_profile")
            .unwrap();
        assert_eq!(profile_field["kind"], "select");
        assert_eq!(profile_field["reload_on_change"], true);
        assert!(
            profile_field["options"].as_array().unwrap().len() >= 1,
            "profile 选项应动态生成"
        );
        let enabled_field = dash_fields.iter().find(|f| f["key"] == "enabled").unwrap();
        assert_eq!(enabled_field["kind"], "multiselect");
        assert_eq!(
            enabled_field["options"].as_array().unwrap().len(),
            ALL_WIDGETS.len()
        );
        assert!(dash["values"]["active_profile"].is_string());
        assert!(dash["values"]["enabled"].is_array());
        assert!(dash["values"]["_profiles"].is_object());
    }
}
