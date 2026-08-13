use strum::{Display, EnumString};

use crate::monitor::docker::DockerAction;
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
}
