use strum::{Display, EnumString};

// ── TUI 模式定义 ─────────────────────────────────────────

/// TUI 交互模式
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

impl Mode {
    /// 模式对应的状态栏标签
    pub fn label(&self) -> &str {
        match self {
            Mode::Normal => " NORMAL ",
            Mode::Search => " SEARCH ",
            Mode::Add => " ADD ",
            Mode::Rename => " RENAME ",
            Mode::Export => " EXPORT ",
            Mode::Import => " IMPORT ",
            Mode::Confirm => " CONFIRM ",
            Mode::Help => " HELP ",
        }
    }

    /// 模式对应的提示信息
    pub fn hint(&self) -> &str {
        match self {
            Mode::Normal => "j↓ k↑ gg↕ G↕ /搜索 a添加 d删除 p检测 P全检 Enter连接 q退出 ?帮助",
            Mode::Search => "输入搜索关键词，Enter 确认，Esc 取消",
            Mode::Add => "格式: user@hostname[:port] 可选 --key <path>",
            Mode::Rename => "输入新别名，Enter 确认，Esc 取消",
            Mode::Export => "输入导出文件路径，Enter 确认，Esc 取消",
            Mode::Import => "输入导入文件路径，Enter 确认，Esc 取消",
            Mode::Confirm => "确认删除？y/Y 确认，n/N/Esc 取消",
            Mode::Help => "按 q/Esc 关闭帮助",
        }
    }
}
