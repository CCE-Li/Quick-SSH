//! 命令面板（Command Palette）
//!
//! Ctrl+K 呼出，输入关键字过滤可执行动作。
//! Phase 1.4 将 [`PaletteAction`] 映射到 [`crate::tui::action::Action`]。

/// 面板动作（独立于 UI 动作，避免依赖方向混乱）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaletteAction {
    ShowDashboard,
    ShowHostList,
    EditDashboard,
    EditAgentConfig,
    OpenAgent,
    RefreshAll,
    AddHost,
    SearchHost,
    Quit,
}

/// 面板条目
#[derive(Debug, Clone, Copy)]
pub struct PaletteItem {
    pub action: PaletteAction,
    pub title: &'static str,
    pub description: &'static str,
}

/// 全部可执行动作（面板展示顺序）
pub const PALETTE_ACTIONS: &[PaletteItem] = &[
    PaletteItem {
        action: PaletteAction::ShowDashboard,
        title: "Show Dashboard",
        description: "切换到服务器监控视图",
    },
    PaletteItem {
        action: PaletteAction::ShowHostList,
        title: "Show Host List",
        description: "回到主机列表视图",
    },
    PaletteItem {
        action: PaletteAction::EditDashboard,
        title: "Edit Dashboard",
        description: "配置 Dashboard 模块与布局",
    },
    PaletteItem {
        action: PaletteAction::EditAgentConfig,
        title: "Agent Settings",
        description: "配置 AI Agent（provider / model / 权限 / 超时）",
    },
    PaletteItem {
        action: PaletteAction::OpenAgent,
        title: "Open AI Agent",
        description: "打开 AI Agent 对话面板（输入指令）",
    },
    PaletteItem {
        action: PaletteAction::RefreshAll,
        title: "Refresh All",
        description: "重新检测全部主机的连通性",
    },
    PaletteItem {
        action: PaletteAction::AddHost,
        title: "Add Host",
        description: "添加新的 SSH 主机",
    },
    PaletteItem {
        action: PaletteAction::SearchHost,
        title: "Search Host",
        description: "在主机列表中搜索",
    },
    PaletteItem {
        action: PaletteAction::Quit,
        title: "Quit",
        description: "退出 Quick-SSH",
    },
];

/// 按关键字过滤面板条目（大小写不敏感，匹配标题或描述）
pub fn filter_palette<'a>(items: &'a [PaletteItem], query: &str) -> Vec<&'a PaletteItem> {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return items.iter().collect();
    }
    items
        .iter()
        .filter(|item| {
            item.title.to_lowercase().contains(&q) || item.description.to_lowercase().contains(&q)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_query_returns_all_items() {
        let filtered = filter_palette(PALETTE_ACTIONS, "");
        assert_eq!(filtered.len(), PALETTE_ACTIONS.len());
    }

    #[test]
    fn query_matches_title_case_insensitive() {
        let filtered = filter_palette(PALETTE_ACTIONS, "dashboard");
        assert!(filtered
            .iter()
            .any(|item| item.action == PaletteAction::ShowDashboard));
    }

    #[test]
    fn query_matches_description() {
        let filtered = filter_palette(PALETTE_ACTIONS, "主机");
        assert!(filtered
            .iter()
            .any(|item| item.action == PaletteAction::AddHost));
    }
}
