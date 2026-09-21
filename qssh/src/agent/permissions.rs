//! Agent 权限模型
//!
//! Phase 8：权限分级 + 危险操作前置确认。
//!
//! 四级权限（与迁移方案 §Phase 8 对应）：
//! - [`PermissionLevel::ReadOnly`]：只读，写操作一律拒绝
//! - [`PermissionLevel::AskBeforeExecute`]（默认）：所有写操作前置确认
//! - [`PermissionLevel::AutoSafe`]：安全写（如 service start）自动放行，危险写仍确认
//! - [`PermissionLevel::FullAccess`]：全部自动放行
//!
//! 决策由 [`PermissionLevel::decide`] 统一得出：`Allowed / NeedsApproval / Denied`。

/// 工具危险等级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DangerLevel {
    /// 只读：永远安全（status / processes / disk / logs ...）
    Read,
    /// 安全写：影响小、易恢复（如 `systemctl start`、`docker start`）
    SafeWrite,
    /// 危险写：影响大、不可逆（stop / restart / delete / rm）
    Dangerous,
}

impl DangerLevel {
    /// 是否为只读
    pub fn is_read(self) -> bool {
        matches!(self, DangerLevel::Read)
    }

    /// 中文标签
    pub fn label(self) -> &'static str {
        match self {
            DangerLevel::Read => "只读",
            DangerLevel::SafeWrite => "安全写",
            DangerLevel::Dangerous => "危险写",
        }
    }
}

/// Agent 权限级别（来自 `agent.json` 配置）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PermissionLevel {
    /// 只读：写操作直接拒绝
    ReadOnly,
    /// 默认：所有写操作需要用户确认
    #[default]
    AskBeforeExecute,
    /// 安全写自动执行，危险写确认
    AutoSafe,
    /// 全部自动执行
    FullAccess,
}

impl PermissionLevel {
    /// 中文标签（状态栏 / 配置面板展示）
    pub fn label(self) -> &'static str {
        match self {
            PermissionLevel::ReadOnly => "只读",
            PermissionLevel::AskBeforeExecute => "执行前确认",
            PermissionLevel::AutoSafe => "自动安全",
            PermissionLevel::FullAccess => "完全访问",
        }
    }

    /// 从配置字符串解析（大小写不敏感）
    pub fn from_str(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "read_only" | "readonly" | "ro" => Some(PermissionLevel::ReadOnly),
            "ask_before_execute" | "ask" | "confirm" => Some(PermissionLevel::AskBeforeExecute),
            "auto_safe" | "auto-safe" | "safe" => Some(PermissionLevel::AutoSafe),
            "full_access" | "full-access" | "full" | "all" => Some(PermissionLevel::FullAccess),
            _ => None,
        }
    }

    /// 配置字符串（与 [`PermissionLevel::from_str`] 互逆）
    pub fn as_str(self) -> &'static str {
        match self {
            PermissionLevel::ReadOnly => "read_only",
            PermissionLevel::AskBeforeExecute => "ask_before_execute",
            PermissionLevel::AutoSafe => "auto_safe",
            PermissionLevel::FullAccess => "full_access",
        }
    }

    /// 对一个危险等级的决策结果
    pub fn decide(self, danger: DangerLevel) -> Approval {
        match self {
            PermissionLevel::ReadOnly => match danger {
                DangerLevel::Read => Approval::Allowed,
                DangerLevel::SafeWrite | DangerLevel::Dangerous => Approval::Denied,
            },
            PermissionLevel::AskBeforeExecute => match danger {
                DangerLevel::Read => Approval::Allowed,
                DangerLevel::SafeWrite | DangerLevel::Dangerous => Approval::NeedsApproval,
            },
            PermissionLevel::AutoSafe => match danger {
                DangerLevel::Read | DangerLevel::SafeWrite => Approval::Allowed,
                DangerLevel::Dangerous => Approval::NeedsApproval,
            },
            PermissionLevel::FullAccess => Approval::Allowed,
        }
    }
}

/// 权限决策结果
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
pub enum Approval {
    /// 直接执行
    Allowed,
    /// 需要 UI 确认
    NeedsApproval,
    /// 被权限策略拒绝
    Denied,
}

impl Approval {
    /// 是否允许执行（允许或待确认后执行）
    #[allow(dead_code)]
    pub fn is_allowed_or_pending(self) -> bool {
        !matches!(self, Approval::Denied)
    }

    /// 中文提示
    /// 中文提示
    #[allow(dead_code)]
    pub fn label(self) -> &'static str {
        match self {
            Approval::Allowed => "允许",
            Approval::NeedsApproval => "待确认",
            Approval::Denied => "已拒绝",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_only_allows_reads_and_denies_writes() {
        assert_eq!(
            PermissionLevel::ReadOnly.decide(DangerLevel::Read),
            Approval::Allowed
        );
        assert_eq!(
            PermissionLevel::ReadOnly.decide(DangerLevel::SafeWrite),
            Approval::Denied
        );
        assert_eq!(
            PermissionLevel::ReadOnly.decide(DangerLevel::Dangerous),
            Approval::Denied
        );
    }

    #[test]
    fn default_asks_before_any_write() {
        assert_eq!(
            PermissionLevel::default().decide(DangerLevel::Read),
            Approval::Allowed
        );
        assert_eq!(
            PermissionLevel::default().decide(DangerLevel::SafeWrite),
            Approval::NeedsApproval
        );
        assert_eq!(
            PermissionLevel::default().decide(DangerLevel::Dangerous),
            Approval::NeedsApproval
        );
    }

    #[test]
    fn auto_safe_approves_safe_write_but_asks_dangerous() {
        assert_eq!(
            PermissionLevel::AutoSafe.decide(DangerLevel::SafeWrite),
            Approval::Allowed
        );
        assert_eq!(
            PermissionLevel::AutoSafe.decide(DangerLevel::Dangerous),
            Approval::NeedsApproval
        );
    }

    #[test]
    fn full_access_allows_everything() {
        for danger in [
            DangerLevel::Read,
            DangerLevel::SafeWrite,
            DangerLevel::Dangerous,
        ] {
            assert_eq!(
                PermissionLevel::FullAccess.decide(danger),
                Approval::Allowed
            );
        }
    }

    #[test]
    fn parse_and_serialize_roundtrip() {
        for level in [
            PermissionLevel::ReadOnly,
            PermissionLevel::AskBeforeExecute,
            PermissionLevel::AutoSafe,
            PermissionLevel::FullAccess,
        ] {
            assert_eq!(PermissionLevel::from_str(level.as_str()), Some(level));
        }
        assert_eq!(
            PermissionLevel::from_str("AUTO-SAFE"),
            Some(PermissionLevel::AutoSafe)
        );
        assert_eq!(PermissionLevel::from_str("unknown"), None);
    }
}
