use crossterm::event::{KeyEvent, KeyModifiers};

use super::action::{Action, Mode};
use crate::monitor::docker::DockerAction;
use crate::monitor::files::FileAction;
use crate::monitor::services::ServiceAction;
use crate::tui::app::App;

/// 键盘事件 → Action 映射
pub fn map_key_to_action(key: KeyEvent, app: &App) -> Action {
    use crossterm::event::KeyCode;

    match app.mode {
        Mode::Normal => {
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                match key.code {
                    KeyCode::Char('p' | 'P') => return Action::MoveUp,
                    KeyCode::Char('n' | 'N') => return Action::MoveDown,
                    KeyCode::Char('k' | 'K') => return Action::OpenPalette,
                    _ => {}
                }
            }

            match key.code {
                KeyCode::Char('j') | KeyCode::Down => Action::MoveDown,
                KeyCode::Char('k') | KeyCode::Up => Action::MoveUp,
                KeyCode::Char('g') => Action::MoveTop,
                KeyCode::Char('G') => Action::MoveBottom,
                KeyCode::Enter => Action::Connect,
                KeyCode::Char(' ') => Action::ToggleSelect,
                KeyCode::Char('e') => Action::StartEdit,
                KeyCode::Char('a') => match app.view {
                    crate::tui::action::View::HostList => Action::StartAdd,
                    crate::tui::action::View::Dashboard => Action::OpenAgentOps,
                },
                KeyCode::Char('p') => Action::Ping,
                KeyCode::Char('P') => Action::PingAll,
                KeyCode::Char('/') => Action::StartSearch,
                KeyCode::Char('.') => Action::ToggleAddress,
                KeyCode::Char('b') => match app.view {
                    crate::tui::action::View::HostList => Action::ShowDashboard,
                    crate::tui::action::View::Dashboard => Action::ShowHostList,
                },
                KeyCode::Char('c') => match app.view {
                    crate::tui::action::View::Dashboard => Action::EditDashboard,
                    _ => Action::None,
                },
                KeyCode::Char('d') => match app.view {
                    crate::tui::action::View::Dashboard => Action::OpenDockerOps,
                    _ => Action::Delete,
                },
                KeyCode::Char('s') => match app.view {
                    crate::tui::action::View::Dashboard => Action::OpenServiceOps,
                    _ => Action::None,
                },
                KeyCode::Char('f') => match app.view {
                    crate::tui::action::View::Dashboard => Action::OpenFileOps,
                    _ => Action::None,
                },
                KeyCode::Char('l') => match app.view {
                    crate::tui::action::View::Dashboard => Action::OpenLogOps,
                    _ => Action::None,
                },
                KeyCode::Char('q') | KeyCode::Esc => Action::Quit,
                KeyCode::Char('?') => Action::ShowHelp,
                _ => Action::None,
            }
        }
        Mode::Palette => match key.code {
            KeyCode::Esc => Action::ClosePalette,
            KeyCode::Enter => Action::PaletteSelect,
            KeyCode::Up => Action::PaletteMove(-1),
            KeyCode::Down => Action::PaletteMove(1),
            KeyCode::Backspace => {
                let mut s = app.palette_query.clone();
                s.pop();
                Action::PaletteInput(s)
            }
            KeyCode::Char(c) => {
                let mut s = app.palette_query.clone();
                s.push(c);
                Action::PaletteInput(s)
            }
            _ => Action::None,
        },
        Mode::Search => match key.code {
            KeyCode::Esc => Action::CancelSearch,
            KeyCode::Enter => Action::SearchSubmit,
            KeyCode::Backspace => {
                let mut s = app.input_buffer.clone();
                s.pop();
                Action::SearchInput(s)
            }
            KeyCode::Char(c) => {
                let mut s = app.input_buffer.clone();
                s.push(c);
                Action::SearchInput(s)
            }
            _ => Action::None,
        },
        Mode::Add => Action::None,
        Mode::Edit => Action::None,
        Mode::Confirm => match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => Action::ConfirmDelete(true),
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => Action::ConfirmDelete(false),
            _ => Action::None,
        },
        Mode::Help => match key.code {
            KeyCode::Char('q') | KeyCode::Esc => Action::HideHelp,
            _ => Action::None,
        },
        Mode::DashboardConfig => match key.code {
            KeyCode::Char('q') | KeyCode::Esc => Action::CloseDashboardConfig,
            KeyCode::Char('s') | KeyCode::Char('S') | KeyCode::Enter => Action::SaveDashboardConfig,
            KeyCode::Char(c) => {
                if let Some(id) = module_key(c) {
                    Action::ToggleDashboardModule(id)
                } else {
                    Action::None
                }
            }
            _ => Action::None,
        },
        Mode::DockerOps => match key.code {
            KeyCode::Char('j') | KeyCode::Down => Action::DockerOpsMove(1),
            KeyCode::Char('k') | KeyCode::Up => Action::DockerOpsMove(-1),
            KeyCode::Char('r') => Action::DockerActionSelected(DockerAction::Restart),
            KeyCode::Char('s') => Action::DockerActionSelected(DockerAction::Stop),
            KeyCode::Char('x') => Action::DockerActionSelected(DockerAction::Delete),
            KeyCode::Char('q') | KeyCode::Esc => Action::CloseDockerOps,
            _ => Action::None,
        },
        Mode::DockerConfirm => match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => Action::ConfirmDocker(true),
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => Action::ConfirmDocker(false),
            _ => Action::None,
        },
        Mode::ServiceOps => match key.code {
            KeyCode::Char('j') | KeyCode::Down => Action::ServiceOpsMove(1),
            KeyCode::Char('k') | KeyCode::Up => Action::ServiceOpsMove(-1),
            KeyCode::Char('a') => Action::ServiceActionSelected(ServiceAction::Start),
            KeyCode::Char('s') => Action::ServiceActionSelected(ServiceAction::Stop),
            KeyCode::Char('r') => Action::ServiceActionSelected(ServiceAction::Restart),
            KeyCode::Char('q') | KeyCode::Esc => Action::CloseServiceOps,
            _ => Action::None,
        },
        Mode::ServiceConfirm => match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => Action::ConfirmService(true),
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => Action::ConfirmService(false),
            _ => Action::None,
        },
        Mode::FileOps => match key.code {
            KeyCode::Char('j') | KeyCode::Down => Action::FileOpsMove(1),
            KeyCode::Char('k') | KeyCode::Up => Action::FileOpsMove(-1),
            KeyCode::Enter => Action::FileOpsEnter,
            KeyCode::Char('h') | KeyCode::Left => Action::FileOpsUp,
            KeyCode::Char('l') | KeyCode::Right => Action::FileOpsEnter,
            KeyCode::Char('d') => Action::FileActionSelected(FileAction::Download),
            KeyCode::Char('q') | KeyCode::Esc => Action::CloseFileOps,
            _ => Action::None,
        },
        Mode::FileConfirm => match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => Action::ConfirmFile(true),
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => Action::ConfirmFile(false),
            _ => Action::None,
        },
        Mode::LogOps => match key.code {
            KeyCode::Char('j') | KeyCode::Down => Action::LogOpsMove(1),
            KeyCode::Char('k') | KeyCode::Up => Action::LogOpsMove(-1),
            KeyCode::Char('r') => Action::LogOpsRefresh,
            KeyCode::Char('f') => Action::LogUnitFilter(app.log_unit().to_string()),
            KeyCode::Char('q') | KeyCode::Esc => Action::CloseLogOps,
            _ => Action::None,
        },
        Mode::LogFilter => match key.code {
            KeyCode::Esc => Action::CloseLogOps,
            KeyCode::Enter => Action::LogOpsRefresh,
            KeyCode::Backspace => {
                let mut s = app.log_unit().to_string();
                s.pop();
                Action::LogUnitFilter(s)
            }
            KeyCode::Char(c) => {
                let mut s = app.log_unit().to_string();
                s.push(c);
                Action::LogUnitFilter(s)
            }
            _ => Action::None,
        },
        Mode::AgentOps => match key.code {
            KeyCode::Esc => Action::CloseAgentOps,
            KeyCode::Enter => Action::AgentSubmit,
            KeyCode::Backspace => {
                let mut s = app.agent_input().to_string();
                s.pop();
                Action::AgentInput(s)
            }
            KeyCode::Char(c) => {
                let mut s = app.agent_input().to_string();
                s.push(c);
                Action::AgentInput(s)
            }
            _ => Action::None,
        },
        Mode::AgentConfirm => match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => Action::AgentRespond(true),
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => Action::AgentRespond(false),
            _ => Action::None,
        },
        _ => Action::None,
    }
}

/// DashboardConfig 模式下 1-9/0 数字键 → 模块映射
fn module_key(c: char) -> Option<crate::tui::dashboard::WidgetId> {
    use crate::tui::dashboard::WidgetId;
    match c {
        '1' => Some(WidgetId::Cpu),
        '2' => Some(WidgetId::Memory),
        '3' => Some(WidgetId::Disk),
        '4' => Some(WidgetId::Network),
        '5' => Some(WidgetId::Docker),
        '6' => Some(WidgetId::Process),
        '7' => Some(WidgetId::Services),
        '8' => Some(WidgetId::Files),
        '9' => Some(WidgetId::Logs),
        '0' => Some(WidgetId::SystemInfo),
        'a' | 'A' => Some(WidgetId::Agent),
        _ => None,
    }
}

// ── Mode 的 UI 方法 ─────────────────────────────────────

impl Mode {
    /// 模式对应的状态栏标签
    pub fn label(&self) -> &str {
        match self {
            Mode::Normal => " NORMAL ",
            Mode::Search => " SEARCH ",
            Mode::Add => " ADD ",
            Mode::Edit => " EDIT ",
            Mode::Rename => " RENAME ",
            Mode::Export => " EXPORT ",
            Mode::Import => " IMPORT ",
            Mode::Confirm => " CONFIRM ",
            Mode::Help => " HELP ",
            Mode::Palette => " PALETTE ",
            Mode::DashboardConfig => " DASHBOARD_CFG ",
            Mode::DockerOps => " DOCKER_OPS ",
            Mode::DockerConfirm => " DOCKER_CONFIRM ",
            Mode::ServiceOps => " SERVICE_OPS ",
            Mode::ServiceConfirm => " SERVICE_CONFIRM ",
            Mode::FileOps => " FILE_OPS ",
            Mode::FileConfirm => " FILE_CONFIRM ",
            Mode::LogOps => " LOG_OPS ",
            Mode::LogFilter => " LOG_FILTER ",
            Mode::AgentOps => " AGENT_OPS ",
            Mode::AgentConfirm => " AGENT_CONFIRM ",
        }
    }

    /// 模式对应的提示信息
    pub fn hint(&self) -> &str {
        match self {
            Mode::Normal => {
                "j↓ k↑ Ctrl+N↓ Ctrl+P↑ gg↕ G↕ /搜索 a添加 e编辑 d删除 p检测 P全检 b监控 Ctrl+K面板 Enter连接 空格标记 .地址 q退出 ?帮助"
            }
            Mode::Search => "输入搜索关键词，Enter 确认，Esc 取消",
            Mode::Add => "字段添加弹窗: Tab 切换字段，Enter 下一项，Ctrl+S 保存，Esc 取消",
            Mode::Edit => "字段编辑弹窗: Tab 切换字段，Enter 下一项，Ctrl+S 保存，Esc 取消",
            Mode::Rename => "输入新别名，Enter 确认，Esc 取消",
            Mode::Export => "输入导出文件路径，Enter 确认，Esc 取消",
            Mode::Import => "输入导入文件路径，Enter 确认，Esc 取消",
            Mode::Confirm => "确认删除？y/Y 确认，n/N/Esc 取消",
            Mode::Help => "按 q/Esc 关闭帮助",
            Mode::Palette => "输入命令关键字，↑/↓ 选择，Enter 执行，Esc 关闭",
            Mode::DashboardConfig => {
                "1CPU 2内存 3磁盘 4网络 5Docker 6进程 7服务 8文件 9日志 0系统 aAgent | s保存 q/Esc关闭"
            }
            Mode::DockerOps => "j↓ k↑ 选择容器 | r重启 s停止 x删除 | q/Esc关闭",
            Mode::DockerConfirm => "确认危险操作？y/Y 执行，n/N/Esc 取消",
            Mode::ServiceOps => "j↓ k↑ 选择服务 | a启动 s停止 r重启 | q/Esc关闭",
            Mode::ServiceConfirm => "确认服务操作？y/Y 执行，n/N/Esc 取消",
            Mode::FileOps => "j↓ k↑ 选择 | Enter/l 进入目录 h/← 上级 d 下载 | q/Esc关闭",
            Mode::FileConfirm => "确认文件操作？y/Y 执行，n/N/Esc 取消",
            Mode::LogOps => "j↓ k↑ 浏览 | r 刷新 f 筛选 unit | q/Esc关闭",
            Mode::LogFilter => "输入 unit 名称（如 sshd）过滤日志，Enter 应用，Esc 取消",
            Mode::AgentOps => "输入指令回车发送 | Esc 关闭 | 执行步骤见下方时间线",
            Mode::AgentConfirm => "确认执行该危险操作？y/Y 执行，n/N/Esc 拒绝",
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    use super::map_key_to_action;
    use crate::config::types::SshConfig;
    use crate::tui::action::Action;
    use crate::tui::app::App;

    fn app() -> App {
        App::new(
            SshConfig {
                hosts: vec![],
                preamble: String::new(),
            },
            PathBuf::from("dummy"),
        )
    }

    #[test]
    fn ctrl_p_moves_up_instead_of_ping() {
        let action = map_key_to_action(
            KeyEvent::new(KeyCode::Char('p'), KeyModifiers::CONTROL),
            &app(),
        );

        assert!(matches!(action, Action::MoveUp));
    }

    #[test]
    fn ctrl_n_moves_down() {
        let action = map_key_to_action(
            KeyEvent::new(KeyCode::Char('n'), KeyModifiers::CONTROL),
            &app(),
        );

        assert!(matches!(action, Action::MoveDown));
    }

    #[test]
    fn plain_p_still_pings() {
        let action = map_key_to_action(
            KeyEvent::new(KeyCode::Char('p'), KeyModifiers::NONE),
            &app(),
        );

        assert!(matches!(action, Action::Ping));
    }
}
