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
    /// 嵌入式终端（SSH 会话面板）
    Terminal,
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
            WidgetId::Terminal => "Terminal",
        }
    }
}

/// 全部可用的模块（配置面板展示顺序）
pub const ALL_WIDGETS: [WidgetId; 12] = [
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
    WidgetId::Terminal,
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
/// 布局规则：
/// - 若启用 [`WidgetId::Terminal`]，它作为中央主区域（权重 3），
///   其余监控 Widget 竖直切分为左侧栏（权重 1），Agent 固定右侧（权重 1）。
/// - 未启用 Terminal 时回退到旧布局：主列 + Agent 右栏。
pub fn default_layout_for(enabled: &[WidgetId]) -> LayoutNode {
    if enabled.is_empty() {
        return LayoutNode::Widget { id: WidgetId::Cpu };
    }

    let has_agent = enabled.contains(&WidgetId::Agent);
    let has_terminal = enabled.contains(&WidgetId::Terminal);
    // 仅监控 Widget（排除 Agent / Terminal）
    let monitors: Vec<WidgetId> = enabled
        .iter()
        .copied()
        .filter(|w| *w != WidgetId::Agent && *w != WidgetId::Terminal)
        .collect();

    // ── 中央 Terminal 布局 ──────────────────────────────
    if has_terminal {
        // 左栏：监控 Widget（等权竖直切分）；无监控 Widget 时不再重复放 Terminal
        let left_column = if monitors.is_empty() {
            None
        } else {
            Some(LayoutNode::Split {
                direction: LayoutDirection::Vertical,
                children: monitors
                    .into_iter()
                    .map(|id| SplitChild::new(100, LayoutNode::Widget { id }))
                    .collect(),
            })
        };
        // 中央：Terminal 主区域
        let main = LayoutNode::Widget {
            id: WidgetId::Terminal,
        };

        if has_agent {
            let mut children = Vec::with_capacity(3);
            if let Some(left) = left_column {
                children.push(SplitChild::new(1, left));
            }
            children.push(SplitChild::new(3, main));
            children.push(SplitChild::new(
                1,
                LayoutNode::Widget {
                    id: WidgetId::Agent,
                },
            ));
            LayoutNode::Split {
                direction: LayoutDirection::Horizontal,
                children,
            }
        } else if let Some(left) = left_column {
            LayoutNode::Split {
                direction: LayoutDirection::Horizontal,
                children: vec![SplitChild::new(1, left), SplitChild::new(3, main)],
            }
        } else {
            main
        }
    } else if has_agent && monitors.is_empty() {
        // ── 仅 Agent ────────────────────────────────────
        LayoutNode::Widget {
            id: WidgetId::Agent,
        }
    } else {
        // ── 旧布局：监控主列 + Agent 右栏 ────────────────
        let main_column: Vec<SplitChild> = monitors
            .into_iter()
            .map(|id| SplitChild::new(100, LayoutNode::Widget { id }))
            .collect();
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
            WidgetId::Terminal,
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
        "Terminal".to_string(),
        profile(&[
            WidgetId::Terminal,
            WidgetId::Cpu,
            WidgetId::Memory,
            WidgetId::Network,
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
///
/// 布局始终按 `enabled` 重新生成：历史配置中可能残留旧版 `default_layout_for`
/// 产生的重复 Widget（如两个 Terminal），按模块列表重建可保证布局一致。
pub fn load_dashboard_config() -> DashboardConfig {
    let path = config_path();
    if let Ok(content) = std::fs::read_to_string(&path) {
        if let Ok(mut config) = serde_json::from_str::<DashboardConfig>(&content) {
            for profile in config.profiles.values_mut() {
                profile.layout = default_layout_for(&profile.enabled);
            }
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

    /// 只启用 Terminal 时布局中只出现一个 Terminal
    #[test]
    fn terminal_only_layout_has_single_terminal() {
        let layout = default_layout_for(&[WidgetId::Terminal]);
        let mut found = Vec::new();
        collect_widgets(&layout, &mut found);
        assert_eq!(found, vec![WidgetId::Terminal]);
    }

    /// Terminal + Agent（无监控 Widget）时不重复 Terminal
    #[test]
    fn terminal_and_agent_layout_has_no_duplicate_terminal() {
        let layout = default_layout_for(&[WidgetId::Terminal, WidgetId::Agent]);
        let mut found = Vec::new();
        collect_widgets(&layout, &mut found);
        assert_eq!(
            found.iter().filter(|id| **id == WidgetId::Terminal).count(),
            1
        );
        assert_eq!(found.iter().filter(|id| **id == WidgetId::Agent).count(), 1);
    }

    /// 仅 Agent 时布局中只出现一个 Agent
    #[test]
    fn agent_only_layout_has_single_agent() {
        let layout = default_layout_for(&[WidgetId::Agent]);
        let mut found = Vec::new();
        collect_widgets(&layout, &mut found);
        assert_eq!(found, vec![WidgetId::Agent]);
    }
}
