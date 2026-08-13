//! Agent 时间线：执行步骤逐步回报 + 状态机
//!
//! Phase 8：TUI Timeline 面板展示每一步状态：
//! `●Ready → ◇Thinking → ●Executing 3/7 → !Approval → ▲Error → ✓Done`

/// 步骤状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepStatus {
    /// 等待执行
    Pending,
    /// 正在执行
    Running,
    /// 执行成功
    Done,
    /// 被权限拒绝或用户取消
    Skipped,
    /// 执行失败
    Failed,
}

impl StepStatus {
    /// 状态符号（Timeline 面板渲染）
    pub fn symbol(self) -> &'static str {
        match self {
            StepStatus::Pending => "○",
            StepStatus::Running => "●",
            StepStatus::Done => "✓",
            StepStatus::Skipped => "⊘",
            StepStatus::Failed => "✗",
        }
    }

    /// 中文标签
    pub fn label(self) -> &'static str {
        match self {
            StepStatus::Pending => "等待",
            StepStatus::Running => "执行中",
            StepStatus::Done => "成功",
            StepStatus::Skipped => "跳过",
            StepStatus::Failed => "失败",
        }
    }
}

/// 时间线中的一步
#[derive(Debug, Clone)]
pub struct TimelineStep {
    /// 显示名称（如 "server.status"、"docker.list"）
    pub name: String,
    /// 工具描述
    pub description: String,
    /// 是否只读
    pub read_only: bool,
    /// 状态
    pub status: StepStatus,
    /// 结果摘要（成功输出 / 错误信息）
    pub detail: String,
}

impl TimelineStep {
    /// 创建新的待执行步骤
    pub fn new(name: impl Into<String>, description: impl Into<String>, read_only: bool) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            read_only,
            status: StepStatus::Pending,
            detail: String::new(),
        }
    }
}

/// Agent 整体运行状态（状态栏 AI 指示）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AgentStatus {
    /// 就绪（●Ready）
    #[default]
    Ready,
    /// 思考中（◇Thinking）
    Thinking,
    /// 执行中（●Executing）
    Executing,
    /// 等待批准（!Approval）
    Approval,
    /// 出错（▲Error）
    Error,
    /// 完成（✓Done）
    Done,
}

impl AgentStatus {
    /// 状态栏符号 + 标签
    pub fn label(self) -> &'static str {
        match self {
            AgentStatus::Ready => "● Ready",
            AgentStatus::Thinking => "◇ Thinking",
            AgentStatus::Executing => "● Executing",
            AgentStatus::Approval => "! Approval",
            AgentStatus::Error => "▲ Error",
            AgentStatus::Done => "✓ Done",
        }
    }
}

/// Agent 会话时间线
#[derive(Debug, Clone, Default)]
pub struct Timeline {
    steps: Vec<TimelineStep>,
}

impl Timeline {
    /// 新建空时间线
    pub fn new() -> Self {
        Self::default()
    }

    /// 追加一步
    pub fn push(&mut self, step: TimelineStep) {
        self.steps.push(step);
    }

    /// 所有步骤
    pub fn steps(&self) -> &[TimelineStep] {
        &self.steps
    }

    /// 步骤总数
    pub fn len(&self) -> usize {
        self.steps.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    /// 修改某一步状态（按索引）
    pub fn set_status(&mut self, index: usize, status: StepStatus) -> bool {
        match self.steps.get_mut(index) {
            Some(step) => {
                step.status = status;
                true
            }
            None => false,
        }
    }

    /// 记录步骤执行结果
    pub fn set_result(
        &mut self,
        index: usize,
        status: StepStatus,
        detail: impl Into<String>,
    ) -> bool {
        match self.steps.get_mut(index) {
            Some(step) => {
                step.status = status;
                step.detail = detail.into();
                true
            }
            None => false,
        }
    }

    /// 已完成 + 运行中 + 失败的总数（用于 "Executing 3/7" 计数）
    pub fn started_count(&self) -> usize {
        self.steps
            .iter()
            .filter(|step| !matches!(step.status, StepStatus::Pending))
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timeline_tracks_steps_and_counts() {
        let mut timeline = Timeline::new();
        timeline.push(TimelineStep::new("server.status", "查询状态", true));
        timeline.push(TimelineStep::new("docker.list", "容器列表", true));
        assert_eq!(timeline.len(), 2);
        assert_eq!(timeline.started_count(), 0);

        assert!(timeline.set_result(0, StepStatus::Done, "ok"));
        assert_eq!(timeline.started_count(), 1);
        assert_eq!(timeline.steps()[0].status, StepStatus::Done);
        assert_eq!(timeline.steps()[0].detail, "ok");

        assert!(!timeline.set_status(99, StepStatus::Done));
    }

    #[test]
    fn status_symbols_and_labels() {
        assert_eq!(StepStatus::Done.symbol(), "✓");
        assert_eq!(StepStatus::Failed.label(), "失败");
        assert_eq!(AgentStatus::default().label(), "● Ready");
        assert_eq!(AgentStatus::Approval.label(), "! Approval");
    }

    #[test]
    fn step_new_defaults_to_pending() {
        let step = TimelineStep::new("files.list", "文件列表", true);
        assert_eq!(step.status, StepStatus::Pending);
        assert!(step.read_only);
        assert!(step.detail.is_empty());
    }
}
