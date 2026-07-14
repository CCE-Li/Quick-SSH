use strum::{Display, EnumString};

// ── Action：用户意图（由事件转换而来） ─────────────────

#[derive(Debug, Clone)]
pub enum Action {
    None,
    MoveUp,
    MoveDown,
    MoveTop,
    MoveBottom,
    Connect,
    ToggleSelect,
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
}
