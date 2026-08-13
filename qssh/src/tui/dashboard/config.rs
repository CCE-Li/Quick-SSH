//! Dashboard 配置：模块列表 + 布局树 + 多 Profile + 持久化
//!
//! 配置文件位于平台配置目录 `~/.config/quick-ssh/dashboard.json`
//! （Windows: `%APPDATA%\quick-ssh\dashboard.json`）。

use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Dashboard 可用的模块 ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WidgetId {
    Cpu,
    Memory,
    Disk,
    Network,
    Docker,
    Process,
    Services,
    Files,
    Logs,
    SystemInfo,
    Agent,
}

impl WidgetId {
    pub fn title(self) -> &'static str {
        match self {
            WidgetId::Cpu => "CPU",
            WidgetId::Memory => "Memory",
            WidgetId::Disk => "Disk",
            WidgetId::Network => "Network",
            WidgetId::Docker => "Docker",
            WidgetId::Process => "Processes",
            WidgetId::Services => "Services",
            WidgetId::Files => "Files",
            WidgetId::Logs => "Logs",
            WidgetId::SystemInfo => "System Info",
            WidgetId::Agent => "AI Agent",
        }
    }
}

/// 全部可用的模块（配置面板展示顺序）
pub const ALL_WIDGETS: [WidgetId; 11] = [
    WidgetId::Cpu,
    WidgetId::Memory,
    WidgetId::Disk,
    WidgetId::Network,
    WidgetId::Docker,
    WidgetId::Process,
    WidgetId::Services,
    WidgetId::Files,
    WidgetId::Logs,
    WidgetId::SystemInfo,
    WidgetId::Agent,
];

/// 布局切分方向
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LayoutDirection {
    Vertical,
    Horizontal,
}

/// 布局树节点：叶子为 Widget，容器为 Split
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum LayoutNode {
    Widget {
        id: WidgetId,
    },
    Split {
        direction: LayoutDirection,
        children: Vec<SplitChild>,
    },
}

/// Split 子节点（权重 + 子树）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplitChild {
    pub weight: u16,
    pub node: LayoutNode,
}

impl SplitChild {
    pub fn new(weight: u16, node: LayoutNode) -> Self {
        Self { weight, node }
    }
}

/// 单个 Profile 的 Dashboard 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileConfig {
    pub enabled: Vec<WidgetId>,
    pub layout: LayoutNode,
}

/// 全部 Dashboard 配置（多 Profile）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardConfig {
    pub active_profile: String,
    pub profiles: HashMap<String, ProfileConfig>,
}

/// 根据启用的模块列表生成默认布局
///
/// - 主列竖直切分所有非 Agent 模块（等权）
/// - Agent 固定在右侧窄栏（3:1）
pub fn default_layout_for(enabled: &[WidgetId]) -> LayoutNode {
    if enabled.is_empty() {
        return LayoutNode::Widget { id: WidgetId::Cpu };
    }

    let has_agent = enabled.contains(&WidgetId::Agent);
    let non_agent: Vec<WidgetId> = enabled
        .iter()
        .copied()
        .filter(|w| *w != WidgetId::Agent)
        .collect();

    let main_column = if non_agent.is_empty() {
        vec![SplitChild::new(
            100,
            LayoutNode::Widget {
                id: WidgetId::Agent,
            },
        )]
    } else {
        non_agent
            .into_iter()
            .map(|id| SplitChild::new(100, LayoutNode::Widget { id }))
            .collect()
    };

    let main = LayoutNode::Split {
        direction: LayoutDirection::Vertical,
        children: main_column,
    };

    if has_agent {
        LayoutNode::Split {
            direction: LayoutDirection::Horizontal,
            children: vec![
                SplitChild::new(3, main),
                SplitChild::new(
                    1,
                    LayoutNode::Widget {
                        id: WidgetId::Agent,
                    },
                ),
            ],
        }
    } else {
        main
    }
}

fn profile(enabled: &[WidgetId]) -> ProfileConfig {
    ProfileConfig {
        enabled: enabled.to_vec(),
        layout: default_layout_for(enabled),
    }
}

/// 默认配置（首次启动 / 重置时使用）
pub fn default_dashboard_config() -> DashboardConfig {
    let mut profiles = HashMap::new();
    profiles.insert(
        "Default".to_string(),
        profile(&[
            WidgetId::Cpu,
            WidgetId::Memory,
            WidgetId::Disk,
            WidgetId::Network,
            WidgetId::Docker,
            WidgetId::Process,
            WidgetId::Agent,
        ]),
    );
    profiles.insert(
        "Server Monitoring".to_string(),
        profile(&[
            WidgetId::Cpu,
            WidgetId::Memory,
            WidgetId::Disk,
            WidgetId::Network,
            WidgetId::Process,
            WidgetId::Services,
            WidgetId::SystemInfo,
        ]),
    );
    profiles.insert(
        "Docker".to_string(),
        profile(&[
            WidgetId::Docker,
            WidgetId::Logs,
            WidgetId::Cpu,
            WidgetId::Memory,
            WidgetId::Network,
        ]),
    );
    profiles.insert(
        "Minimal".to_string(),
        profile(&[WidgetId::Cpu, WidgetId::Memory, WidgetId::Network]),
    );
    profiles.insert(
        "AI".to_string(),
        profile(&[
            WidgetId::Agent,
            WidgetId::Cpu,
            WidgetId::Memory,
            WidgetId::Logs,
        ]),
    );

    DashboardConfig {
        active_profile: "Default".to_string(),
        profiles,
    }
}

/// Dashboard 配置文件路径
pub fn config_path() -> PathBuf {
    let base = dirs::config_dir().unwrap_or_else(|| {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".config")
    });
    base.join("quick-ssh").join("dashboard.json")
}

/// 加载配置（不存在或损坏时回退默认）
pub fn load_dashboard_config() -> DashboardConfig {
    let path = config_path();
    if let Ok(content) = std::fs::read_to_string(&path) {
        if let Ok(config) = serde_json::from_str(&content) {
            return config;
        }
    }
    default_dashboard_config()
}

/// 保存配置到磁盘
pub fn save_dashboard_config(config: &DashboardConfig) -> Result<()> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(config)?;
    std::fs::write(&path, content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn collect_widgets(node: &LayoutNode, out: &mut Vec<WidgetId>) {
        match node {
            LayoutNode::Widget { id } => out.push(*id),
            LayoutNode::Split { children, .. } => {
                for child in children {
                    collect_widgets(&child.node, out);
                }
            }
        }
    }

    #[test]
    fn default_layout_contains_all_enabled_widgets() {
        let config = default_dashboard_config();
        let profile = &config.profiles["Default"];
        let mut found = Vec::new();
        collect_widgets(&profile.layout, &mut found);
        for id in &profile.enabled {
            assert!(found.contains(id), "layout 缺少 {}", id.title());
        }
    }

    #[test]
    fn agent_profile_puts_agent_on_right() {
        let config = default_dashboard_config();
        let profile = &config.profiles["AI"];
        let LayoutNode::Split {
            direction: LayoutDirection::Horizontal,
            children,
        } = &profile.layout
        else {
            panic!("AI profile 应为水平 split");
        };
        assert_eq!(children.len(), 2);
        assert!(matches!(
            children[1].node,
            LayoutNode::Widget {
                id: WidgetId::Agent
            }
        ));
    }

    #[test]
    fn config_roundtrip() {
        let config = default_dashboard_config();
        let json = serde_json::to_string(&config).expect("序列化");
        let parsed: DashboardConfig = serde_json::from_str(&json).expect("反序列化");
        assert_eq!(parsed.active_profile, config.active_profile);
        assert_eq!(parsed.profiles.len(), config.profiles.len());
    }
}
