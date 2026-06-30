use anyhow::Result;

/// 打印自定义帮助信息
pub fn run() -> Result<()> {
    println!("🚀 Quick-SSH - SSH 连接管理工具");
    println!();
    println!("使用方式:");
    println!("  qssh                    启动 TUI 界面");
    println!("  qssh ps [keyword]      列出所有主机（可按关键词过滤）");
    println!("  qssh ls [keyword]      同上 (ps 的别名)");
    println!("  qssh add <alias> <user@host> 添加主机");
    println!("  qssh rm <alias>        删除主机");
    println!("  qssh remove <alias>    同上 (rm 的别名)");
    println!("  qssh connect <target>  连接主机 (别名 或 user@host)");
    println!("  qssh cn <target>       同上 (connect 的别名)");
    println!("  qssh export [file]     导出配置为 JSON");
    println!("  qssh import <file>     从 JSON 导入配置");
    println!("  qssh help              显示此帮助");
    println!();
    println!("TUI 快捷键:");
    println!("  j/↓  下移    k/↑  上移    gg  顶部    G  底部");
    println!("  a    添加    d    删除    /   搜索    Space  选择");
    println!("  p    检测    P    全检    Enter  连接");
    println!("  q    退出    ?    帮助");
    println!();
    println!("项目地址: https://github.com/CCE-Li/Quick-SSH");

    Ok(())
}
