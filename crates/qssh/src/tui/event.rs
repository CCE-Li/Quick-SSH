/// 用户意图（由事件转换而来）
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
    DoAdd(String),
    StartRename(String),
    DoRename(String),
    StartExport(String),
    DoExport(String),
    StartImport,
    DoImport(String),
    Ping,
    PingAll,
    ShowHelp,
    HideHelp,
    Quit,
}
