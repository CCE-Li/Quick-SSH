//! 远程文件浏览模型 + `ls -l` 输出解析 + 文件操作
//!
//! Phase 6：Files 文件模块。
//! - 远程命令：`ls -lA --time-style=long-iso`（结构化输出）
//! - 解析为 [`RemoteFile`]，支持目录导航与下载

/// 远程文件/目录条目
#[derive(Debug, Clone, PartialEq)]
pub struct RemoteFile {
    /// 文件名
    pub name: String,
    /// 是否为目录
    pub is_dir: bool,
    /// 文件大小（字节）
    pub size: u64,
    /// 修改时间（`YYYY-MM-DD HH:MM`）
    pub mtime: String,
    /// 权限位（如 `-rw-r--r--`）
    pub perms: String,
}

/// 文件操作（目前仅下载；删除等危险操作留给后续 Phase）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileAction {
    /// 下载到本地
    Download,
}

impl FileAction {
    /// 操作显示名
    pub fn label(self) -> &'static str {
        match self {
            FileAction::Download => "下载",
        }
    }
}

/// 从 `ls -lA --time-style=long-iso` 输出解析文件列表
///
/// 行格式：`<perms> <links> <owner> <group> <size> <date> <time> <name>`
pub fn parse_ls_l(text: &str) -> Vec<RemoteFile> {
    text.lines()
        .filter_map(|line| {
            let line = line.trim_end();
            if line.is_empty() || line.starts_with("total") {
                return None;
            }
            let mut parts = line.splitn(8, ' ');
            let perms = parts.next()?.to_string();
            let _links = parts.next()?;
            let _owner = parts.next()?;
            let _group = parts.next()?;
            let size: u64 = parts.next()?.parse().ok()?;
            let date = parts.next()?;
            let time = parts.next()?;
            let name = parts.next()?.trim().to_string();
            if name.is_empty() {
                return None;
            }
            Some(RemoteFile {
                name,
                is_dir: perms.starts_with('d'),
                size,
                mtime: format!("{date} {time}"),
                perms,
            })
        })
        .collect()
}

/// 整理目录条目：过滤 `.`，目录排前（用于列表展示/导航）
pub fn dir_entries(mut files: Vec<RemoteFile>) -> Vec<RemoteFile> {
    files.retain(|f| f.name != ".");
    files.sort_by_key(|f| !f.is_dir);
    files
}

/// 上级目录（`~` 与 `/` 保持不变）
pub fn parent_dir(cwd: &str) -> String {
    if cwd == "~" || cwd == "/" {
        return "~".to_string();
    }
    let trimmed = cwd.trim_end_matches('/');
    match trimmed.rfind('/') {
        Some(0) => "/".to_string(),
        Some(i) => trimmed[..i].to_string(),
        None => "~".to_string(),
    }
}

/// 拼接子路径（`..` 特殊处理为上级）
pub fn join_dir(cwd: &str, name: &str) -> String {
    if name == ".." {
        return parent_dir(cwd);
    }
    if cwd == "~" {
        return format!("~/{name}");
    }
    if cwd == "/" {
        return format!("/{name}");
    }
    format!("{}/{}", cwd.trim_end_matches('/'), name)
}

/// shell 单引号转义
pub fn shell_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

/// 构建 `ls -l` 采集命令（`~` 不加引号，由 shell 展开）
pub fn ls_l_command(path: &str) -> String {
    let quoted = if path == "~" {
        "~".to_string()
    } else {
        shell_quote(path)
    };
    format!("LC_ALL=C ls -lA --time-style=long-iso {quoted}")
}

/// 人类可读文件大小
pub fn format_file_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit = 0;
    while size >= 1024.0 && unit < UNITS.len() - 1 {
        size /= 1024.0;
        unit += 1;
    }
    format!("{size:.1}{}", UNITS[unit])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ls_output() {
        let out = "total 8\ndrwxr-xr-x 2 user group 4096 2026-08-13 16:00 .\ndrwxr-xr-x 4 user group 4096 2026-08-12 10:00 ..\n-rw-r--r-- 1 user group 1234 2026-08-13 16:00 app.log\ndrwxr-xr-x 2 user group 4096 2026-08-13 15:00 logs\n-rw-r--r-- 1 user group 56 2026-08-13 14:30 config.yaml";
        let files = parse_ls_l(out);
        assert_eq!(files.len(), 5);
        assert!(files[0].is_dir);
        assert_eq!(files[0].name, ".");
        let app = files.iter().find(|f| f.name == "app.log").unwrap();
        assert!(!app.is_dir);
        assert_eq!(app.size, 1234);
        assert_eq!(app.mtime, "2026-08-13 16:00");
        assert_eq!(app.perms, "-rw-r--r--");
        let logs = files.iter().find(|f| f.name == "logs").unwrap();
        assert!(logs.is_dir);
    }

    #[test]
    fn empty_input_yields_empty_list() {
        assert!(parse_ls_l("").is_empty());
        assert!(parse_ls_l("total 0\n").is_empty());
    }

    #[test]
    fn dir_entries_filters_dot_and_sorts_dirs_first() {
        let out = "total 8\ndrwxr-xr-x 2 u g 4096 2026-08-13 16:00 .\ndrwxr-xr-x 2 u g 4096 2026-08-13 16:00 ..\n-rw-r--r-- 1 u g 100 2026-08-13 16:00 a.txt\ndrwxr-xr-x 2 u g 4096 2026-08-13 16:00 zdir";
        let files = dir_entries(parse_ls_l(out));
        assert_eq!(files.len(), 3);
        assert_eq!(files[0].name, "..");
        assert_eq!(files[1].name, "zdir");
        assert_eq!(files[2].name, "a.txt");
        assert!(files.iter().all(|f| f.name != "."));
    }

    #[test]
    fn parent_dir_variants() {
        assert_eq!(parent_dir("~"), "~");
        assert_eq!(parent_dir("/"), "~");
        assert_eq!(parent_dir("/var/log"), "/var");
        assert_eq!(parent_dir("/var"), "/");
        assert_eq!(parent_dir("~/logs"), "~");
    }

    #[test]
    fn join_dir_variants() {
        assert_eq!(join_dir("~", "logs"), "~/logs");
        assert_eq!(join_dir("/", "etc"), "/etc");
        assert_eq!(join_dir("/var", "log"), "/var/log");
        assert_eq!(join_dir("/var/log", ".."), "/var");
        assert_eq!(join_dir("~", ".."), "~");
    }

    #[test]
    fn shell_quote_handles_single_quote() {
        assert_eq!(shell_quote("plain"), "'plain'");
        assert_eq!(shell_quote("a'b"), "'a'\\''b'");
    }

    #[test]
    fn format_file_size_human() {
        assert_eq!(format_file_size(500), "500.0B");
        assert_eq!(format_file_size(2048), "2.0KB");
        assert_eq!(format_file_size(5 * 1024 * 1024), "5.0MB");
    }
}
