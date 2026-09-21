//! 嵌入式终端（Embedded Terminal）
//!
//! 让 SSH 交互式会话运行在 TUI 内部的面板中，而非接管整个终端。
//!
//! ## 架构
//!
//! - [`portable_pty`]：跨平台 PTY（Windows 走 ConPTY，Unix 走 openpty），
//!   为 `ssh` 子进程分配伪终端，使 `vim`/`top` 等全屏程序正常工作。
//! - [`vt100`]：纯 Rust ANSI 状态机，把 PTY 输出解析成屏幕网格（含颜色）。
//! - 后台读取线程持有 PTY 读端，把输出喂给共享的 `vt100::Parser`（`Arc<Mutex>`），
//!   每批数据后发 [`TermEvent::Output`]；主线程持有写端用于键盘转发，
//!   持有 master 用于面板 resize，持有 child 用于断开/kill。
//!
//! ## 与现有 `spawn.rs` 的关系
//!
//! [`spawn::start_interactive_session`] 保留给 CLI（`qssh connect`）全屏模式；
//! 本模块提供 TUI 内嵌模式。

use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use portable_pty::{native_pty_system, Child, MasterPty, PtySize};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};

use crate::ssh::session::SshTarget;

/// 终端会话事件（后台读取线程 → TUI 主循环）
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TermEvent {
    /// 新输出已写入解析器（屏幕内容已更新，需重绘）
    Output,
    /// 会话已结束
    Exited { code: Option<i32> },
}

/// 终端会话状态（供状态栏 / 面板标题显示）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TermStatus {
    /// 正在建立连接（SSH 握手 / 密码认证）
    Connecting,
    /// 会话运行中
    Running,
    /// 会话已结束
    Exited,
}

impl TermStatus {
    pub fn label(self) -> &'static str {
        match self {
            TermStatus::Connecting => "连接中",
            TermStatus::Running => "已连接",
            TermStatus::Exited => "已断开",
        }
    }
}

/// 嵌入式终端会话：PTY 进程 + vt100 解析器 + 事件通道
pub struct TermSession {
    /// 目标主机（面板标题栏展示）
    pub target: SshTarget,
    /// vt100 解析器（后台线程写入，渲染线程读取）
    parser: Arc<Mutex<vt100::Parser>>,
    /// PTY master（用于 resize）
    master: Option<Box<dyn MasterPty + Send>>,
    /// PTY 写端（键盘输入转发）
    writer: Option<Box<dyn Write + Send>>,
    /// SSH 子进程（用于 kill / 获取退出码）
    child: Option<Box<dyn Child + Send + Sync>>,
    /// 会话状态
    pub status: TermStatus,
    /// 退出码（None = 尚未退出）
    pub exit_code: Option<i32>,
    /// 会话是否已结束
    finished: bool,
    /// 是否已主动终止（用于通知读取线程退出）
    killed: Arc<AtomicBool>,
    /// 回看滚动偏移（0 = 最新，正值 = 向上回看的行数）
    pub scroll_offset: usize,
}

impl TermSession {
    /// 启动一个新的 SSH 嵌入式会话
    pub fn spawn(target: SshTarget, tx: Sender<TermEvent>, cols: u16, rows: u16) -> Result<Self> {
        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .context("无法创建伪终端 (PTY)")?;

        let slave = pair.slave;
        let master = pair.master;

        let cmd = build_ssh_command(&target);

        let child = slave
            .spawn_command(cmd)
            .context("无法启动 ssh 进程，请确保已安装 OpenSSH Client")?;

        // 关闭 slave，仅保留 master（读写端）
        drop(slave);

        // 共享解析器：后台线程喂数据，渲染线程读屏幕
        // scrollback=1000：保留最近 1000 行历史，供鼠标回看
        let parser = Arc::new(Mutex::new(vt100::Parser::new(rows, cols, 1000)));

        // 克隆读端给后台线程
        let mut reader = master.try_clone_reader().context("无法获取 PTY 读取端")?;
        let writer = master.take_writer().context("无法获取 PTY 写入端")?;

        let killed = Arc::new(AtomicBool::new(false));

        // 后台读取线程：PTY 输出 → vt100 → 事件
        {
            let parser_out = Arc::clone(&parser);
            let killed_out = Arc::clone(&killed);
            let tx_out = tx.clone();
            std::thread::Builder::new()
                .name(format!("term-read-{}", target.alias))
                .spawn(move || {
                    let mut buf = [0u8; 4096];
                    loop {
                        if killed_out.load(Ordering::Relaxed) {
                            break;
                        }
                        let n = match reader.read(&mut buf) {
                            Ok(0) => break,
                            Ok(n) => n,
                            Err(_) => break,
                        };
                        if let Ok(mut parser) = parser_out.lock() {
                            parser.process(&buf[..n]);
                        }
                        if tx_out.send(TermEvent::Output).is_err() {
                            break;
                        }
                    }
                    // 读取结束：通知主循环
                    let _ = tx_out.send(TermEvent::Exited { code: None });
                })
                .context("无法创建终端读取线程")?;
        }

        Ok(Self {
            target,
            parser,
            master: Some(master),
            writer: Some(writer),
            child: Some(child),
            status: TermStatus::Connecting,
            exit_code: None,
            finished: false,
            killed,
            scroll_offset: 0,
        })
    }

    /// 向 PTY 写入键盘输入（编码为 ANSI 字节流）
    pub fn write_key(&mut self, key: KeyEvent) {
        if let Some(bytes) = encode_key(key) {
            if let Some(writer) = self.writer.as_mut() {
                let _ = writer.write_all(&bytes);
                let _ = writer.flush();
            }
        }
    }

    /// 写入原始字节（预留：特殊控制序列）
    #[allow(dead_code)]
    pub fn write_raw(&mut self, bytes: &[u8]) {
        if let Some(writer) = self.writer.as_mut() {
            let _ = writer.write_all(bytes);
            let _ = writer.flush();
        }
    }

    /// 调整终端尺寸（面板 resize 时调用）
    pub fn resize(&mut self, cols: u16, rows: u16) {
        if let Some(master) = self.master.as_ref() {
            let _ = master.resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            });
        }
        if let Ok(mut parser) = self.parser.lock() {
            parser.set_size(rows, cols);
        }
    }

    /// 主动终止会话（用户按 Esc / q 断开）
    pub fn kill(&mut self) {
        self.killed.store(true, Ordering::Relaxed);
        if let Some(child) = self.child.as_mut() {
            let _ = child.kill();
        }
        // 丢弃写端，强制 PTY 关闭，使读取线程退出
        self.writer = None;
        self.finished = true;
        self.status = TermStatus::Exited;
        self.exit_code = Some(130);
    }

    /// 标记会话进入运行态（收到首个 PTY 输出时调用）
    pub fn mark_running(&mut self) {
        if self.status == TermStatus::Connecting {
            self.status = TermStatus::Running;
        }
    }

    /// 处理会话退出事件（由主循环在收到 `TermEvent::Exited` 时调用）
    pub fn handle_exit(&mut self, code: Option<i32>) {
        self.finished = true;
        // 若事件未携带退出码，尝试从子进程获取
        let code = code.or_else(|| {
            self.child
                .as_mut()
                .and_then(|c| c.try_wait().ok().flatten())
                .map(|s| s.exit_code() as i32)
        });
        self.exit_code = code;
        self.status = TermStatus::Exited;
        // 释放子进程句柄
        self.child = None;
    }

    /// 会话是否已结束（预留：供状态栏 / 面板退出判断使用）
    #[allow(dead_code)]
    pub fn is_finished(&self) -> bool {
        self.finished
    }

    /// 向上回看（delta > 0 向上，delta < 0 向下）
    pub fn scroll(&mut self, delta: isize) {
        if let Ok(parser) = self.parser.lock() {
            // scrollback() 返回当前已保存的历史行数（最大可回看量）
            let max_offset = parser.screen().scrollback();
            if delta > 0 {
                self.scroll_offset = (self.scroll_offset + delta as usize).min(max_offset);
            } else {
                self.scroll_offset = self.scroll_offset.saturating_sub((-delta) as usize);
            }
        }
    }

    /// 回到最新输出（取消回看）
    pub fn scroll_to_bottom(&mut self) {
        self.scroll_offset = 0;
    }

    /// 渲染到 ratatui buffer
    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        if let Ok(mut parser) = self.parser.lock() {
            // 临时设置 scrollback 偏移，渲染后立即复原为 0
            // vt100::set_scrollback 只影响 screen() 视口，不影响 process()
            parser.set_scrollback(self.scroll_offset);
            render_terminal(&parser, area, buf, self.scroll_offset > 0);
            parser.set_scrollback(0);
        }
    }

    /// 当前屏幕文本内容（测试用）
    #[allow(dead_code)]
    pub fn screen_text(&self) -> String {
        match self.parser.lock() {
            Ok(parser) => parser.screen().contents(),
            Err(_) => String::new(),
        }
    }
}

/// 构建 SSH 命令行（强制分配远程 PTY：`-tt`）
fn build_ssh_command(target: &SshTarget) -> portable_pty::CommandBuilder {
    let mut args = target.build_ssh_args();
    // 强制分配远程 PTY
    let has_tt = args.iter().any(|a| a == "-tt");
    if !has_tt {
        if let Some(pos) = args.iter().rposition(|a| !a.starts_with('-')) {
            args.insert(pos, "-tt".into());
        } else {
            args.insert(0, "-tt".into());
        }
    }

    let mut cmd = portable_pty::CommandBuilder::new("ssh");
    for arg in args {
        cmd.arg(arg);
    }

    // 已保存密码通过 SSH_ASKPASS 机制注入（复用 credentials 模块）
    if let Ok(true) = crate::config::credentials::has_password(&target.alias) {
        if let Ok(executable) = std::env::current_exe() {
            cmd.env("SSH_ASKPASS", &executable);
            cmd.env("SSH_ASKPASS_REQUIRE", "force");
            cmd.env(crate::config::credentials::ASKPASS_ACTIVE_ENV, "1");
            cmd.env(crate::config::credentials::ASKPASS_ALIAS_ENV, &target.alias);
            if std::env::var_os("DISPLAY").is_none() {
                cmd.env("DISPLAY", "qssh-askpass");
            }
        }
    }

    cmd
}

/// 将 vt100 颜色映射为 ratatui 颜色
fn map_color(c: vt100::Color) -> Color {
    match c {
        vt100::Color::Default => Color::Reset,
        vt100::Color::Idx(n) => Color::Indexed(n),
        vt100::Color::Rgb(r, g, b) => Color::Rgb(r, g, b),
    }
}

/// 将 vt100 屏幕网格渲染到 ratatui Buffer
///
/// 调用前应通过 `parser.set_scrollback(offset)` 设置视口；
/// `in_scrollback` 为 true 时隐藏光标并在右上角显示回看提示。
pub fn render_terminal(parser: &vt100::Parser, area: Rect, buf: &mut Buffer, in_scrollback: bool) {
    let screen = parser.screen();
    let (rows, cols) = screen.size();

    let render_rows = area.height.min(rows);
    let render_cols = area.width.min(cols);

    for y in 0..render_rows {
        for x in 0..render_cols {
            let target_x = area.x + x;
            let target_y = area.y + y;
            if target_x >= buf.area.width || target_y >= buf.area.height {
                continue;
            }

            let mut style = Style::default();
            let mut ch = ' ';
            if let Some(cell) = screen.cell(y, x) {
                ch = cell.contents().chars().next().unwrap_or(' ');
                style = style
                    .fg(map_color(cell.fgcolor()))
                    .bg(map_color(cell.bgcolor()));
                if cell.bold() {
                    style = style.add_modifier(Modifier::BOLD);
                }
                if cell.italic() {
                    style = style.add_modifier(Modifier::ITALIC);
                }
                if cell.underline() {
                    style = style.add_modifier(Modifier::UNDERLINED);
                }
                if cell.inverse() {
                    style = style.add_modifier(Modifier::REVERSED);
                }
            }
            if let Some(cell) = buf.cell_mut((target_x, target_y)) {
                cell.set_char(ch);
                cell.set_style(style);
            }
        }
    }

    // 回看模式：隐藏光标，右上角显示「↑ 回看中」提示
    if in_scrollback {
        let hint = " ↑ 回看中 ";
        let hint_x = area.x + area.width.saturating_sub(hint.chars().count() as u16);
        let hint_y = area.y;
        let indicator_style = Style::default()
            .fg(Color::Black)
            .bg(Color::Yellow)
            .add_modifier(Modifier::BOLD);
        for (i, ch) in hint.chars().enumerate() {
            let tx = hint_x + i as u16;
            if tx < buf.area.width && hint_y < buf.area.height {
                if let Some(cell) = buf.cell_mut((tx, hint_y)) {
                    cell.set_char(ch);
                    cell.set_style(indicator_style);
                }
            }
        }
        return;
    }

    // 正常模式：终端光标块状反色显示（尊重远程应用 DECTCEM 的隐藏状态）
    if !screen.hide_cursor() {
        let (row, col) = screen.cursor_position();
        if row < render_rows && col < render_cols {
            let target_x = area.x + col;
            let target_y = area.y + row;
            if let Some(cell) = buf.cell_mut((target_x, target_y)) {
                let style = cell.style();
                let fg = match style.fg {
                    Some(Color::Reset) | None => Color::White,
                    Some(c) => c,
                };
                let bg = match style.bg {
                    Some(Color::Reset) | None => Color::Black,
                    Some(c) => c,
                };
                let modifiers = cell.modifier;
                cell.set_style(Style::default().fg(bg).bg(fg).add_modifier(modifiers));
            }
        }
    }
}

/// 将 KeyEvent 编码为 ANSI 转义序列（写入 PTY）
///
/// 覆盖常见按键；特殊修饰组合（Shift+F 等）未覆盖时返回 `None`。
pub fn encode_key(key: KeyEvent) -> Option<Vec<u8>> {
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    let alt = key.modifiers.contains(KeyModifiers::ALT);
    let shift = key.modifiers.contains(KeyModifiers::SHIFT);

    let seq: Vec<u8> = match key.code {
        KeyCode::Enter => vec![b'\r'],
        KeyCode::Backspace => vec![0x7f],
        KeyCode::Tab => {
            if shift {
                vec![0x1b, b'[', b'Z']
            } else {
                vec![b'\t']
            }
        }
        KeyCode::Esc => vec![0x1b],
        KeyCode::Up => vec![0x1b, b'[', b'A'],
        KeyCode::Down => vec![0x1b, b'[', b'B'],
        KeyCode::Right => vec![0x1b, b'[', b'C'],
        KeyCode::Left => vec![0x1b, b'[', b'D'],
        KeyCode::Home => vec![0x1b, b'[', b'H'],
        KeyCode::End => vec![0x1b, b'[', b'F'],
        KeyCode::PageUp => vec![0x1b, b'[', b'5', b'~'],
        KeyCode::PageDown => vec![0x1b, b'[', b'6', b'~'],
        KeyCode::Delete => vec![0x1b, b'[', b'3', b'~'],
        KeyCode::Insert => vec![0x1b, b'[', b'2', b'~'],
        KeyCode::F(1) => vec![0x1b, b'O', b'P'],
        KeyCode::F(2) => vec![0x1b, b'O', b'Q'],
        KeyCode::F(3) => vec![0x1b, b'O', b'R'],
        KeyCode::F(4) => vec![0x1b, b'O', b'S'],
        KeyCode::F(n) if (5..=12).contains(&n) => vec![0x1b, b'[', (14 + n), b'~'],
        KeyCode::Char(c) => {
            // Ctrl 组合 → 控制字符
            if ctrl && c.is_ascii() {
                let code = match c.to_ascii_lowercase() {
                    'a' => 0x01,
                    'b' => 0x02,
                    'c' => 0x03,
                    'd' => 0x04,
                    'e' => 0x05,
                    'f' => 0x06,
                    'g' => 0x07,
                    'h' => 0x08,
                    'i' => 0x09,
                    'j' => 0x0a,
                    'k' => 0x0b,
                    'l' => 0x0c,
                    'm' => 0x0d,
                    'n' => 0x0e,
                    'o' => 0x0f,
                    'p' => 0x10,
                    'q' => 0x11,
                    'r' => 0x12,
                    's' => 0x13,
                    't' => 0x14,
                    'u' => 0x15,
                    'v' => 0x16,
                    'w' => 0x17,
                    'x' => 0x18,
                    'y' => 0x19,
                    'z' => 0x1a,
                    ' ' => 0x00,
                    '[' => 0x1b,
                    '\\' => 0x1c,
                    ']' => 0x1d,
                    '^' => 0x1e,
                    '_' => 0x1f,
                    _ => return None,
                };
                return Some(vec![code]);
            }
            // Alt 组合 → ESC + 字符
            if alt {
                let mut v = vec![0x1b];
                v.extend_from_slice(c.to_string().as_bytes());
                return Some(v);
            }
            return Some(c.to_string().as_bytes().to_vec());
        }
        _ => return None,
    };

    // 标记 shift 以避免 unused 警告（方向键 shift 变体暂不特殊处理）
    let _ = shift;
    Some(seq)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_simple_keys() {
        assert_eq!(
            encode_key(KeyEvent::from(KeyCode::Enter)),
            Some(vec![b'\r'])
        );
        assert_eq!(
            encode_key(KeyEvent::from(KeyCode::Char('a'))),
            Some(vec![b'a'])
        );
        assert_eq!(
            encode_key(KeyEvent::from(KeyCode::Up)),
            Some(vec![0x1b, b'[', b'A'])
        );
    }

    #[test]
    fn encode_ctrl_c() {
        let key = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert_eq!(encode_key(key), Some(vec![0x03]));
    }

    #[test]
    fn encode_alt_x() {
        let key = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::ALT);
        assert_eq!(encode_key(key), Some(vec![0x1b, b'x']));
    }

    #[test]
    fn encode_backspace_and_enter() {
        assert_eq!(
            encode_key(KeyEvent::from(KeyCode::Backspace)),
            Some(vec![0x7f])
        );
        assert_eq!(encode_key(KeyEvent::from(KeyCode::Tab)), Some(vec![b'\t']));
    }

    #[test]
    fn vt100_parses_ansi_output() {
        let mut parser = vt100::Parser::new(24, 80, 0);
        parser.process(b"hello \x1b[31mred\x1b[0m world");
        let text = parser.screen().contents();
        assert!(text.contains("hello"));
        assert!(text.contains("red"));
    }

    #[test]
    fn map_color_handles_ansi_red() {
        let mut parser = vt100::Parser::new(24, 80, 0);
        parser.process(b"\x1b[31mX\x1b[0m");
        let cell = parser.screen().cell(0, 0).unwrap();
        assert_eq!(map_color(cell.fgcolor()), Color::Indexed(1));
    }

    #[test]
    fn render_terminal_writes_cells() {
        use ratatui::buffer::Buffer;
        use ratatui::layout::Rect;

        let mut parser = vt100::Parser::new(24, 80, 0);
        parser.process(b"AB");
        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 24));
        render_terminal(&parser, Rect::new(0, 0, 80, 24), &mut buf, false);
        let row: String = buf.content[..2].iter().map(|c| c.symbol()).collect();
        assert_eq!(row, "AB");
    }

    #[test]
    fn render_terminal_shows_block_cursor() {
        use ratatui::buffer::Buffer;
        use ratatui::layout::Rect;
        use ratatui::style::Color;

        let mut parser = vt100::Parser::new(24, 80, 0);
        // 输入 "ab"，光标应停在 (0, 2)
        parser.process(b"ab");
        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 24));
        render_terminal(&parser, Rect::new(0, 0, 80, 24), &mut buf, false);

        let (row, col) = parser.screen().cursor_position();
        assert_eq!((row, col), (0, 2));
        let cell = &buf.content[(row * 80 + col) as usize];
        // 光标格被反色：fg 与 bg 互换
        assert_eq!(cell.style().fg, Some(Color::Black));
        assert_eq!(cell.style().bg, Some(Color::White));
    }

    #[test]
    fn render_terminal_hides_cursor_on_dectcem() {
        use ratatui::buffer::Buffer;
        use ratatui::layout::Rect;
        use ratatui::style::Color;

        let mut parser = vt100::Parser::new(24, 80, 0);
        parser.process(b"ab\x1b[?25l"); // 隐藏光标
        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 24));
        render_terminal(&parser, Rect::new(0, 0, 80, 24), &mut buf, false);

        let cell = &buf.content[(2) as usize];
        // 光标隐藏时该格保持默认样式（不反色）
        assert_eq!(cell.style().fg, Some(Color::Reset));
        assert_eq!(cell.style().bg, Some(Color::Reset));
    }
}
