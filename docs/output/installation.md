# 安装指南

Quick-SSH 提供多种安装方式，覆盖主流操作系统和包管理器。

## 系统包管理器安装

<CardGroup cols={2}>
  <Card title="Windows — Scoop" icon="Terminal">
    <CodeGroup title="Scoop 安装">
      ```powershell title="添加 Bucket"
      scoop bucket add cceli https://github.com/CCE-Li/scoop-bucket
      ```
      ```powershell title="安装"
      scoop install quick-ssh
      ```
    </CodeGroup>
  </Card>
  <Card title="Arch Linux — AUR" icon="Terminal">
    ```bash
    yay -S quick-ssh
    ```
    <Note>
      也可使用 `paru -S quick-ssh`
    </Note>
  </Card>
</CardGroup>

<Callout title="渠道状态">
  WinGet、Homebrew 和 APT 配置目前尚未发布到可直接安装的渠道；v2.0.4 GitHub Release 也不包含 `.deb`。这些平台请先使用下方的预编译归档或从源码安装。
</Callout>

## 直接下载

从 [GitHub Releases](https://github.com/CCE-Li/Quick-SSH/releases) 下载对应平台的归档文件：

<CodeGroup title="平台归档">
  ```bash title="Linux x86_64"
  tar xzf qssh-x86_64-linux.tar.gz
  sudo cp qssh-x86_64-linux/qssh /usr/local/bin/
  sudo cp qssh-x86_64-linux/qssh-uploader /usr/local/bin/
  ```
  ```bash title="macOS x86_64"
  tar xzf qssh-x86_64-macos.tar.gz
  sudo cp qssh-x86_64-macos/qssh /usr/local/bin/
  sudo cp qssh-x86_64-macos/qssh-uploader /usr/local/bin/
  ```
  ```bash title="macOS ARM64"
  tar xzf qssh-aarch64-macos.tar.gz
  sudo cp qssh-aarch64-macos/qssh /usr/local/bin/
  sudo cp qssh-aarch64-macos/qssh-uploader /usr/local/bin/
  ```
  ```powershell title="Windows x86_64"
  # 解压 qssh-x86_64-windows.zip
  # 将 qssh-x86_64-windows 目录加入 PATH
  ```
</CodeGroup>

每个归档包含：
- `qssh` / `qssh.exe` — 主程序
- `qssh-uploader` / `qssh-uploader.exe` — 文件上传工具
- `LICENSE` — MIT 许可证
- `README.md` — 说明文档
- 各 Shell 补全脚本

## 从源码编译

需要 [Rust 工具链](https://rustup.rs)。

```bash
git clone https://github.com/CCE-Li/Quick-SSH.git
cd Quick-SSH
cargo build --release
# 二进制位于: target/release/qssh.exe (或 qssh)
```

## 快速验证安装

```bash
# 查看版本
qssh --version

# 启动 TUI
qssh

# 查看帮助
qssh help
```
