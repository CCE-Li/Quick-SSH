# Shell 补全

Quick-SSH 内置了 Shell 补全脚本生成功能，可通过 `qssh completions` 子命令为各种 Shell 生成补全脚本。

## 支持的类型

| Shell | 命令参数 | 文件后缀 |
|-------|---------|---------|
| Bash | `bash` | `.bash` |
| Zsh | `zsh` | `.zsh` |
| Fish | `fish` | `.fish` |
| PowerShell | `powershell` / `ps1` | `.ps1` |
| Elvish | `elvish` | — |

## 安装补全脚本

### Bash

```bash
qssh completions bash > /etc/bash_completion.d/qssh
# 或用户级安装
qssh completions bash > ~/.local/share/bash-completion/completions/qssh
```

### Zsh

```bash
# 创建补全目录（如果不存在）
mkdir -p ~/.zsh/completions

# 生成补全脚本
qssh completions zsh > ~/.zsh/completions/_qssh

# 在 ~/.zshrc 中添加
echo 'fpath=(~/.zsh/completions $fpath)' >> ~/.zshrc
echo 'autoload -Uz compinit && compinit' >> ~/.zshrc
```

### Fish

```bash
qssh completions fish > ~/.config/fish/completions/qssh.fish
```

### PowerShell

```powershell
# 生成补全脚本
qssh completions powershell > qssh.ps1

# 在 PowerShell 配置文件中添加
# 将 qssh.ps1 的内容添加到 $PROFILE
```

### Elvish

```bash
qssh completions elvish > ~/.elvish/completions/qssh.elv
```

## 构建时自动生成

在 CI/CD 发布流程中，`release.yml` 会自动为所有支持的 Shell 生成补全脚本，并打包到发布归档中：

| 文件 | Shell |
|------|-------|
| `qssh.bash` | Bash |
| `qssh.zsh` | Zsh |
| `qssh.fish` | Fish |
| `qssh.ps1` | PowerShell |

这些文件会随 Release 归档一起提供下载。

## 实现原理

补全生成使用 `clap_complete` crate，通过 `clap::CommandFactory` 获取 CLI 定义，然后使用 `generate()` 函数输出补全脚本。该功能在构建时无需额外操作，通过 `qssh completions <shell>` 子命令在运行时生成。

```rust
// completions.rs 核心实现
let mut cmd = Cli::command();
generate(shell, &mut cmd, "qssh", &mut io::stdout());
```
