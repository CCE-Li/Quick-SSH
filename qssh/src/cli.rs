use clap::{Parser, Subcommand};

/// Quick-SSH - 快速 SSH 连接管理工具
#[derive(Parser)]
#[command(
    name = "qssh",
    version,
    about = "Quick-SSH - SSH 连接管理工具",
    disable_help_subcommand = true,
    subcommand_required = false,
    allow_external_subcommands = true,
    long_about = "\
Quick-SSH 是一个 SSH 连接管理工具，提供 TUI 界面和命令行双模式操作。

支持的子命令：
  ps, ls      列出所有 SSH 主机
  add         添加新的 SSH 主机
  rm, remove  删除 SSH 主机
  connect, cn 连接 SSH 主机
  export      导出 SSH 配置
  import      导入 SSH 配置
  help        显示帮助信息

提示：不带任何参数时，自动启动 TUI 界面。
      直接传入主机别名时，自动连接到该主机。"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    /// 列出所有 SSH 主机
    #[command(alias = "ls")]
    Ps {
        /// 按关键词过滤主机列表
        keyword: Option<String>,
    },

    /// 添加新的 SSH 主机
    Add {
        /// 主机别名（用于快速连接）
        alias: String,
        /// user@host 格式的连接地址
        user_at_host: String,
        /// 指定密钥文件路径
        #[arg(long, short)]
        key: Option<String>,
        /// 指定 SSH 端口
        #[arg(long, short, default_value = "22")]
        port: u16,
    },

    /// 删除 SSH 主机
    #[command(alias = "remove")]
    Rm {
        /// 要删除的主机别名
        alias: String,
    },

    /// 连接 SSH 主机（默认操作）
    #[command(alias = "cn")]
    Connect {
        /// 要连接的主机别名或 user@host
        target: String,

        /// 额外的 SSH 参数（放在 -- 之后）
        #[arg(last = true, allow_hyphen_values = true)]
        ssh_args: Vec<String>,
    },

    /// 导出所有主机配置为 JSON
    Export {
        /// 导出文件路径（可选，默认输出到终端）
        file: Option<String>,
    },

    /// 从 JSON 文件导入主机配置
    Import {
        /// 导入文件路径
        file: String,
    },

    /// 打印帮助信息
    Help,

    /// 生成 Shell 补全脚本
    #[command(hide = true)]
    Completions {
        /// Shell 类型: bash | zsh | fish | powershell | elvish
        shell: String,
    },

    /// 直接连接主机（别名或 user@host）
    #[command(external_subcommand)]
    Bare(Vec<String>),
}
