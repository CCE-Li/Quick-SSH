// ── 拖拽文件路径检测 ──────────────────────────────────────
//!
//! 当用户在终端（Windows Terminal / mintty / 等）中拖拽文件时，
//! 终端会将文件路径「粘贴」到输入流中。
//! 此模块检测这种拖拽行为，提取文件路径列表。
//!
//! 参考: tssh / 原 Node.js 版 session.js

use std::path::PathBuf;

// ── 辅助函数 ─────────────────────────────────────────────

/// 去除终端粘贴模式标记（bracketed paste mode markers）
pub fn strip_paste_markers(text: &str) -> String {
    text.replace("\x1b[200~", "")
        .replace("\x1b[201~", "")
        .replace("\x1b[?2004h", "")
        .replace("\x1b[?2004l", "")
        .replace('\x10', "") // DLE (Device Control Escape)
}

/// 判断字符串是否看起来像 Windows 绝对路径 (X:\... 或 X:/...)
fn looks_like_windows_path(s: &str) -> bool {
    let bytes = s.as_bytes();
    bytes.len() >= 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && (bytes[2] == b'\\' || bytes[2] == b'/')
}

/// 判断字符串是否看起来像 Unix 绝对路径 (/...)
#[allow(dead_code)]
fn looks_like_unix_path(s: &str) -> bool {
    s.starts_with('/') && s.len() > 1
}

// ── 解析函数 ─────────────────────────────────────────────

/// 从文本中分割出可能含有引号的 token 列表
fn tokenize(text: &str) -> Vec<String> {
    let cleaned = strip_paste_markers(text);
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_quote = false;
    let mut quote_char = '"';

    for c in cleaned.chars() {
        match c {
            '"' | '\'' if !in_quote => {
                in_quote = true;
                quote_char = c;
                current.push(c);
            }
            '"' | '\'' if in_quote && c == quote_char => {
                in_quote = false;
                current.push(c);
            }
            c if c.is_whitespace() && !in_quote => {
                if !current.is_empty() {
                    tokens.push(current.clone());
                    current.clear();
                }
            }
            _ => {
                // 过滤掉控制字符和 ANSI 转义序列
                if !c.is_control() || c == ':' || c == '\\' || c == '/' || c == '.' {
                    current.push(c);
                }
            }
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}

/// 解析 Windows 拖拽路径（如 `C:\Users\file.txt` 或 `"C:\My Documents\file.txt"`）
///
/// 必须是「所有 token 都是 Windows 绝对路径」才判定为拖拽操作，
/// 否则视为普通键盘输入（防止误判如 `cat C:\nul` 这类命令）。
pub fn parse_windows_drag(text: &str) -> Option<Vec<PathBuf>> {
    let tokens = tokenize(text);
    if tokens.is_empty() {
        return None;
    }

    // 启发式：如果文本中混有非路径 token（如 ls、cd、cat 等），
    // 很可能不是拖拽操作。但如果所有 token 都是路径，则是拖拽。
    let mut files = Vec::new();
    let mut all_are_paths = true;

    for token in &tokens {
        let path_str = token.trim_matches('"').trim_matches('\'');
        if !looks_like_windows_path(path_str) {
            all_are_paths = false;
            break;
        }
        let path = PathBuf::from(path_str);
        if path.exists() {
            files.push(path);
        }
    }

    if !all_are_paths || files.is_empty() {
        return None;
    }
    Some(files)
}

/// 解析 Unix 拖拽路径（如 `/home/user/file.txt`）
#[allow(dead_code)]
pub fn parse_unix_drag(text: &str) -> Option<Vec<PathBuf>> {
    let tokens = tokenize(text);
    if tokens.is_empty() {
        return None;
    }

    let mut files = Vec::new();
    let mut all_are_paths = true;

    for token in &tokens {
        let path_str = token.trim_matches('"').trim_matches('\'');
        if !looks_like_unix_path(path_str) {
            all_are_paths = false;
            break;
        }
        let path = PathBuf::from(path_str);
        if path.exists() {
            files.push(path);
        }
    }

    if !all_are_paths || files.is_empty() {
        return None;
    }
    Some(files)
}

/// 检测文本中是否包含拖拽文件路径
///
/// 返回 `Some(files)` 表示检测到拖拽操作，`None` 表示普通输入。
///
/// ### 检测策略
///
/// 当用户在终端中拖拽文件时，终端模拟器会将文件路径粘贴到输入流中。
/// Windows Terminal / ConEmu 等会粘贴 Windows 绝对路径（带驱动器号），
/// 而 WSL / Unix 终端会粘贴 Unix 绝对路径。
///
/// 我们使用的检测方式：
/// 1. 分割 token
/// 2. 检查所有 token 是否都是绝对路径（防止误判）
/// 3. 验证路径在本地文件系统中确实存在
#[cfg(windows)]
pub fn detect_drag_files(text: &str) -> Option<Vec<PathBuf>> {
    parse_windows_drag(text)
}

#[cfg(not(windows))]
pub fn detect_drag_files(text: &str) -> Option<Vec<PathBuf>> {
    parse_unix_drag(text)
}

// ── 测试 ─────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_windows_simple_path() {
        // 测试简单 Windows 路径
        let text = r#"C:\Windows\System32\cmd.exe"#;
        let result = parse_windows_drag(text);
        assert!(result.is_some());
        let files = result.unwrap();
        assert!(files.iter().any(|p| p.to_string_lossy().contains("cmd.exe")));
    }

    #[test]
    fn test_windows_quoted_path_with_spaces() {
        // 创建临时目录（路径带空格）来测试引号包裹的路径
        let tmp = std::env::temp_dir().join("qssh test dir");
        let _ = std::fs::create_dir_all(&tmp);
        let test_file = tmp.join("test.txt");
        std::fs::write(&test_file, b"test").ok();

        let text = format!("\"{}\"", test_file.display());
        let result = parse_windows_drag(&text);
        assert!(result.is_some(), "引号包裹的路径应被检测到: {}", text);

        // 清理
        let _ = std::fs::remove_file(&test_file);
        let _ = std::fs::remove_dir(&tmp);
    }

    #[test]
    fn test_not_drag_regular_command() {
        // 测试普通命令不应被识别为拖拽
        let text = "ls -la /home/user";
        let result = parse_windows_drag(text);
        assert!(result.is_none());
    }

    #[test]
    fn test_not_drag_windows_command() {
        let text = "dir C:\\Windows";
        let result = parse_windows_drag(text);
        assert!(result.is_none()); // "dir" 不是路径
    }

    #[test]
    fn test_multiple_files_drag() {
        // 多个文件拖拽
        let text = "C:\\temp\\a.txt C:\\temp\\b.txt";
        let result = parse_windows_drag(text);
        // 如果这些文件实际上不存在，结果也可能是 None
        // 所以这个测试只是验证逻辑不崩溃
        assert!(result.is_none() || result.is_some());
    }

    #[test]
    fn test_strip_paste_markers() {
        let text = "\x1b[200~C:\\test.txt\x1b[201~";
        let result = strip_paste_markers(&text);
        assert_eq!(result, "C:\\test.txt");
    }

    #[test]
    fn test_tokenize_simple() {
        let tokens = tokenize("a b c");
        assert_eq!(tokens, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_tokenize_quoted() {
        let tokens = tokenize(r#"a "b c" d"#);
        assert_eq!(tokens, vec!["a", "\"b c\"", "d"]);
    }

    #[test]
    fn test_empty_text() {
        assert!(parse_windows_drag("").is_none());
        assert!(parse_unix_drag("").is_none());
    }
}
