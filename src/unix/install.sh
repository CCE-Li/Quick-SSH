#!/bin/bash
#
# src/unix/install.sh - 将 qssh 注册到 Shell 配置文件 (.bashrc / .zshrc)
#
# 用法:
#   source src/unix/install.sh              # 注册 qssh 到当前 shell
#   bash src/unix/install.sh                # 或直接执行
#
# 原理:
#   在 ~/.bashrc 或 ~/.zshrc 中追加一个 qssh() 函数，
#   通过 Node.js 调用 src/unix/cli.js 实现所有功能。

QSSH_CLI="$(cd "$(dirname "$0")/.." && pwd)/unix/cli.js"

if ! command -v node &>/dev/null; then
    echo "错误：需要 Node.js，请先安装 (https://nodejs.org)" >&2
    exit 1
fi

detect_shell_rc() {
    if [ -n "$ZSH_VERSION" ]; then
        echo "$HOME/.zshrc"
    elif [ -n "$BASH_VERSION" ]; then
        echo "$HOME/.bashrc"
    else
        # 默认探测
        [ -f "$HOME/.zshrc" ] && echo "$HOME/.zshrc" && return
        [ -f "$HOME/.bashrc" ] && echo "$HOME/.bashrc" && return
        echo "$HOME/.profile"
    fi
}

RC_FILE=$(detect_shell_rc)
IMPORT_LINE="# >>> Quick-SSH auto-generated (do not modify) >>>"

# 检查是否已注册
if grep -qF "$IMPORT_LINE" "$RC_FILE" 2>/dev/null; then
    echo "qssh 已注册到 $RC_FILE，无需重复操作。"
else
    cat >> "$RC_FILE" << 'EOF'

# >>> Quick-SSH auto-generated (do not modify) >>>
# Quick-SSH SSH 连接管理工具 (Node.js)
QSSH_CLI="__QSSH_CLI_PATH__"
if [ -f "$QSSH_CLI" ]; then
    qssh() { node "$QSSH_CLI" "$@"; }
fi
# <<< Quick-SSH auto-generated <<<
EOF
    # 替换占位符为真实路径
    sed -i "s|__QSSH_CLI_PATH__|$QSSH_CLI|g" "$RC_FILE"
    echo "✔ 已注册 qssh 到 $RC_FILE"
    echo "   请执行 'source $RC_FILE' 或重启终端生效。"
fi
