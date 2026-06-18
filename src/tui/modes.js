/**
 * modes.js - Quick-SSH TUI 模式定义
 *
 * 集中管理所有模式常量、标签和提示文本。
 * 添加新模式只需在此文件增加一个条目，无需修改核心逻辑。
 */

// ============================================================
// 模式枚举
// ============================================================

const MODE = {
    NORMAL:  0,
    SEARCH:  1,
    ADD:     2,
    EXPORT:  3,
    IMPORT:  4,
    CONFIRM: 5,
    HELP:    6,
    RENAME:  7,
};

// ============================================================
// 模式标签（状态栏左侧显示）
// ============================================================

const MODE_LABELS = {
    [MODE.NORMAL]:  "{green-fg}{bold} NORMAL {/bold}{/green-fg}",
    [MODE.SEARCH]:  "{cyan-fg}{bold} SEARCH {/bold}{/cyan-fg}",
    [MODE.ADD]:     "{yellow-fg}{bold} ADD {/bold}{/yellow-fg}",
    [MODE.EXPORT]:  "{yellow-fg}{bold} EXPORT {/bold}{/yellow-fg}",
    [MODE.IMPORT]:  "{yellow-fg}{bold} IMPORT {/bold}{/yellow-fg}",
    [MODE.CONFIRM]: "{red-fg}{bold} CONFIRM {/bold}{/red-fg}",
    [MODE.HELP]:    "{white-fg}{bold} HELP {/bold}{/white-fg}",
    [MODE.RENAME]:  "{cyan-fg}{bold} RENAME {/bold}{/cyan-fg}",
};

// ============================================================
// 模式提示（状态栏右侧显示）
// ============================================================

const MODE_HINTS = {
    [MODE.NORMAL]:  " j/k ↑↓  gg首 G尾  ↵连接  d删除  a添加  /搜索  e导出  i导入  r重命名  p检测  P全检  ?帮助  q退出",
    [MODE.SEARCH]:  " 输入关键词过滤  Enter确认  Esc取消",
    [MODE.ADD]:     " 格式: 别名 用户@主机:端口 [--key 路径]  Enter确认  Esc取消",
    [MODE.EXPORT]:  " 输入导出文件路径 (默认: ~/.ssh/config.quickssh.bak)  Enter确认  Esc取消",
    [MODE.IMPORT]:  " 输入导入文件路径  Enter确认  Esc取消",
    [MODE.CONFIRM]: " y确认  n取消",
    [MODE.HELP]:    " 按任意键返回",
    [MODE.RENAME]:  " 输入新别名  Enter确认  Esc取消",
};

module.exports = { MODE, MODE_LABELS, MODE_HINTS };
