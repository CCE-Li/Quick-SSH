// Quick-SSH 构建脚本
//
// 补全脚本通过 `qssh completions <shell>` 子命令生成，
// 构建时无需额外操作。
//
// Release 流程在 CI 中生成并打包补全文件。

fn main() {
    println!("cargo::rerun-if-changed=src/cli.rs");
}
