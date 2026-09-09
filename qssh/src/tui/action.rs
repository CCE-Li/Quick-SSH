use strum::{Display, EnumString};

use crate::monitor::docker::DockerAction;
use crate::monitor::files::FileAction;
use crate::monitor::services::ServiceAction;
use crate::tui::dashboard::WidgetId;

// ── View：主界面视图 ─────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    /// 主机列表（原有界面）
    HostList,
    /// Dashboard 监控面板
    Dashboard,
}

// ── Action：用户意图 ─────────────────

#[derive(Debug, Clone)]
pub enum Action {
    None,
    MoveUp,
    MoveDown,
    MoveTop,
    MoveBottom,
    Connect,
    ToggleSelect,
    /// 切换地址显示/隐藏
    ToggleAddress,
    /// 鼠标点击选中列表项（按当前模式分发到对应列表）
    #[allow(dead_code)]
    SelectListItem(usize),
    Delete,
    ConfirmDelete(bool),
    StartSearch,
    /// 搜索输入（追加/替换字符）
    SearchInput(String),
    /// 提交搜索
    SearchSubmit,
    /// 取消搜索
    CancelSearch,
    StartAdd,
    #[allow(dead_code)]
    DoAdd(String),
    StartEdit,
    #[allow(dead_code)]
    StartRename(String),
    #[allow(dead_code)]
    DoRename(String),
    #[allow(dead_code)]
    StartExport(String),
    #[allow(dead_code)]
    DoExport(String),
    #[allow(dead_code)]
    StartImport,
    #[allow(dead_code)]
    DoImport(String),
    Ping,
    PingAll,
    ShowHelp,
    HideHelp,
    /// 切换到 Dashboard 视图
    ShowDashboard,
    /// 切换回主机列表视图
    ShowHostList,
    /// 打开 Dashboard 配置弹窗（Phase 1.6 实现）
    EditDashboard,
    /// 打开命令面板（Ctrl+K）
    OpenPalette,
    /// 关闭命令面板
    ClosePalette,
    /// 命令面板输入
    PaletteInput(String),
    /// 命令面板移动选择（正数向下，负数向上）
    PaletteMove(isize),
    /// 命令面板执行当前选中项
    PaletteSelect,
    /// 关闭 Dashboard 配置弹窗
    CloseDashboardConfig,
    /// 切换 Dashboard 模块勾选状态
    ToggleDashboardModule(WidgetId),
    /// 保存 Dashboard 配置并关闭弹窗
    SaveDashboardConfig,
    /// 打开 Docker 容器操作面板
    OpenDockerOps,
    /// 关闭 Docker 容器操作面板
    CloseDockerOps,
    /// Docker 容器列表移动选择
    DockerOpsMove(isize),
    /// 选择容器操作类型（restart / stop / delete）
    DockerActionSelected(DockerAction),
    /// 确认/取消容器操作（y/n）
    ConfirmDocker(bool),
    /// 打开系统服务操作面板
    OpenServiceOps,
    /// 关闭系统服务操作面板
    CloseServiceOps,
    /// 服务列表移动选择
    ServiceOpsMove(isize),
    /// 选择服务操作类型（start / stop / restart）
    ServiceActionSelected(ServiceAction),
    /// 确认/取消服务操作（y/n）
    ConfirmService(bool),
    /// 打开文件浏览面板
    OpenFileOps,
    /// 关闭文件浏览面板
    CloseFileOps,
    /// 文件列表移动选择
    FileOpsMove(isize),
    /// 进入选中的目录
    FileOpsEnter,
    /// 返回上级目录
    FileOpsUp,
    /// 选择文件操作（下载）
    FileActionSelected(FileAction),
    /// 确认/取消文件操作（y/n）
    ConfirmFile(bool),
    /// 打开日志面板
    OpenLogOps,
    /// 关闭日志面板
    CloseLogOps,
    /// 日志列表移动选择
    LogOpsMove(isize),
    /// 刷新日志（重拉当前 unit）
    LogOpsRefresh,
    /// 设置日志筛选 unit
    LogUnitFilter(String),
    /// 打开 AI Agent 面板
    OpenAgentOps,
    /// 关闭 AI Agent 面板
    CloseAgentOps,
    /// 鼠标点击聚焦 AI Agent 面板（不重置输入，保留对话现场）
    FocusAgent,
    /// 鼠标点击聚焦嵌入式终端面板（SSH 会话仍连接时）
    FocusTerminal,
    /// 打开 Agent 设置弹窗（编辑 agent.json）
    OpenAgentConfig,
    /// 关闭 Agent 设置弹窗
    CloseAgentConfig,
    /// Agent 输入框追加/替换字符
    AgentInput(String),
    /// 提交 Agent 输入（发送给 LLM）
    AgentSubmit,
    /// 回应 Agent 危险操作批准请求（true=批准，false=拒绝）
    AgentRespond(bool),
    /// 打开 `:` 远程命令输入框
    OpenCommand,
    /// 关闭 `:` 远程命令输入框（取消）
    CloseCommand,
    /// 命令输入框追加/替换字符
    CommandInput(String),
    /// 提交远程命令（在选中主机上执行）
    CommandSubmit,
    /// 关闭远程命令执行结果弹窗
    CloseCommandResult,
    /// 关闭嵌入式终端会话（Esc / Ctrl+Shift+C）
    CloseTerminal,
    /// 开始拖动 Dashboard 分隔线（调整相邻窗口大小）
    StartSplitDrag {
        col: u16,
        row: u16,
    },
    /// 拖动中更新分隔线位置
    MoveSplitDrag {
        col: u16,
        row: u16,
    },
    /// 结束拖动并保存布局
    EndSplitDrag,
    Quit,
}

// ── Mode：TUI 交互模式 ──────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumString)]
pub enum Mode {
    /// 普通模式 - 浏览主机列表
    #[strum(to_string = "NORMAL")]
    Normal,
    /// 搜索模式
    #[strum(to_string = "SEARCH")]
    Search,
    /// 添加主机模式
    #[strum(to_string = "ADD")]
    Add,
    /// 编辑主机模式
    #[strum(to_string = "EDIT")]
    Edit,
    /// 重命名模式
    #[strum(to_string = "RENAME")]
    Rename,
    /// 导出确认
    #[strum(to_string = "EXPORT")]
    Export,
    /// 导入确认
    #[strum(to_string = "IMPORT")]
    Import,
    /// 删除确认
    #[strum(to_string = "CONFIRM")]
    Confirm,
    /// 帮助页面
    #[strum(to_string = "HELP")]
    Help,
    /// 命令面板（Ctrl+K 呼出）
    #[strum(to_string = "PALETTE")]
    Palette,
    /// Dashboard 模块配置弹窗
    #[strum(to_string = "DASHBOARD_CFG")]
    DashboardConfig,
    /// Docker 容器操作面板
    #[strum(to_string = "DOCKER_OPS")]
    DockerOps,
    /// Docker 危险操作确认
    #[strum(to_string = "DOCKER_CONFIRM")]
    DockerConfirm,
    /// 系统服务操作面板
    #[strum(to_string = "SERVICE_OPS")]
    ServiceOps,
    /// 服务危险操作确认
    #[strum(to_string = "SERVICE_CONFIRM")]
    ServiceConfirm,
    /// 文件浏览面板
    #[strum(to_string = "FILE_OPS")]
    FileOps,
    /// 文件操作确认
    #[strum(to_string = "FILE_CONFIRM")]
    FileConfirm,
    /// 日志面板
    #[strum(to_string = "LOG_OPS")]
    LogOps,
    /// 日志筛选输入
    #[strum(to_string = "LOG_FILTER")]
    LogFilter,
    /// AI Agent 面板
    #[strum(to_string = "AGENT_OPS")]
    AgentOps,
    /// Agent 危险操作确认
    #[strum(to_string = "AGENT_CONFIRM")]
    AgentConfirm,
    /// Agent 设置弹窗
    #[strum(to_string = "AGENT_CFG")]
    AgentConfig,
    /// `:` 远程命令输入框
    #[strum(to_string = "COMMAND")]
    Command,
    /// 远程命令执行结果弹窗
    #[strum(to_string = "CMD_RESULT")]
    CommandResult,
    /// 嵌入式终端（SSH 会话在 TUI 容器中）
    #[strum(to_string = "TERMINAL")]
    Terminal,
}
