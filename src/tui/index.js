#!/usr/bin/env node
/**
 * src/tui/index.js - Quick-SSH 终端用户界面 (TUI)
 *
 * 类似 yazi 的布局 + Vim 操作键位。
 * 依赖模块:
 *   - modes.js   : 模式常量/标签/提示
 *   - data.js    : 配置读写
 *   - network.js : SSH 连接/在线检测
 *
 * 布局:
 *   ┌─ 标题栏 ─────────────────────────────────┐
 *   │ 连接列表 (60%)        │ 详情面板 (40%)     │
 *   │  ● myserver           │ 别名: myserver     │
 *   │  ○ devbox             │ 主机: 192.168.1.100│
 *   │  ...                  │ ...                │
 *   ├─ 状态栏 ─────────────────────────────────┤
 *   │ NORMAL  j/k ↑↓  ↵连接  d删除  /搜索  q退出│
 *   └──────────────────────────────────────────┘
 */

const blessed   = require("blessed");
const path      = require("path");
const fs        = require("fs");
const os        = require("os");

const { MODE, MODE_LABELS, MODE_HINTS } = require("./modes");
const { SSH_CONFIG_PATH, loadHosts, saveHosts } = require("./data");
const { sshConnect, checkHost } = require("./network");

// ============================================================
// 阻止 blessed 切换备用屏幕缓冲区（保留 PowerShell 透明背景）
// ============================================================

if (blessed.Program) {
    blessed.Program.prototype.alternateBuffer = function (val, cb) {
        if (typeof val === "function") { cb = val; }
        if (typeof cb === "function") { cb(); }
        return this;
    };
    if (blessed.Program.prototype.smcup) blessed.Program.prototype.smcup = function () { return this; };
    // rmcup 仍然需要恢复终端状态：已经修复鼠标移到导致输出乱码的错误
    // 不能设为 no-op，否则 screen.destroy() 无法恢复终端
    if (blessed.Program.prototype.rmcup) {
        const origRmcup = blessed.Program.prototype.rmcup;
        blessed.Program.prototype.rmcup = function () {
            try { if (process.stdin.isTTY) process.stdin.setRawMode(false); } catch (e) {}
            process.stdout.write('\x1b[?1000l\x1b[?1002l\x1b[?1003l\x1b[?1006l');
            process.stdout.write('\x1b[?1049l\x1b[?25h\x1b[0m');
            return this;
        };
    }
}

/**
 * 手动恢复终端状态（兜底函数）
 *
 * 由于 smcup 被拦截（保留终端透明背景），screen.destroy() 不完全恢复终端。
 * 此函数确保退出时终端回到可交互的 cooked 模式，在 screen.destroy() 之前调用。
 */
function restoreTerminal() {
    try {
        if (process.stdin.isTTY) {
            process.stdin.setRawMode(false);
        }
    } catch (e) {
        // 忽略非 TTY 或已关闭的 stdin
    }
    // 禁用鼠标追踪（blessed 启用了 SGR 鼠标模式，退出必须关闭）
    process.stdout.write('\x1b[?1000l');  // 禁用鼠标按钮事件
    process.stdout.write('\x1b[?1002l');  // 禁用鼠标移动事件
    process.stdout.write('\x1b[?1003l');  // 禁用全部鼠标事件（兜底）
    process.stdout.write('\x1b[?1006l');  // 禁用 SGR 扩展鼠标模式
    // 退出备用屏幕
    process.stdout.write('\x1b[?1049l');
    // 显示光标
    process.stdout.write('\x1b[?25h');
    // 重置文本属性（前景色、背景色、加粗等）
    process.stdout.write('\x1b[0m');
    // 清屏并复位光标
    process.stdout.write('\x1b[2J\x1b[H');
}

// ============================================================
// TUI 组件引用
// ============================================================

let screen;
let headerBar;
let listBox;
let detailBox;
let statusBar;
let inputBox;
let helpBox;
let confirmBox;

// ============================================================
// 状态
// ============================================================

const hostStatus = {};        // { alias: "unknown" | "online" | "offline" }
let hosts        = [];
let filteredHosts = [];
let filterText   = "";
let currentMode  = MODE.NORMAL;
const selectedAliases = new Set();

// ============================================================
// UI 更新函数
// ============================================================

/**
 * 切换模式并更新界面
 */
function setMode(mode, inputValue) {
    currentMode = mode;
    statusBar.setContent(`${MODE_LABELS[mode] || ""} ${MODE_HINTS[mode] || ""}`);

    if (mode === MODE.NORMAL || mode === MODE.HELP) {
        inputBox.hide();
        confirmBox.hide();
        if (mode === MODE.NORMAL) listBox.focus();
    }

    if (mode === MODE.SEARCH || mode === MODE.ADD || mode === MODE.EXPORT ||
        mode === MODE.IMPORT || mode === MODE.RENAME) {
        inputBox.show();
        inputBox.setValue(inputValue || "");
        inputBox.focus();
        inputBox.readInput();
    }

    if (mode === MODE.CONFIRM) {
        confirmBox.show();
    }

    screen.render();
}

/**
 * 刷新连接列表
 */
function refreshList(keepSelection) {
    hosts = loadHosts();
    const validAliases = new Set(hosts.map(h => h.alias));
    for (const alias of [...selectedAliases]) {
        if (!validAliases.has(alias)) selectedAliases.delete(alias);
    }

    filteredHosts = filterText
        ? hosts.filter(h =>
            h.alias.toLowerCase().includes(filterText.toLowerCase()) ||
            h.host.toLowerCase().includes(filterText.toLowerCase())  ||
            h.user.toLowerCase().includes(filterText.toLowerCase())
          )
        : [...hosts];

    const prevIdx = listBox.selected;
    const indicators = {
        online:  "{green-fg}●{/green-fg}",
        offline: "{red-fg}○{/red-fg}",
        unknown: "{yellow-fg}◌{/yellow-fg}",
    };
    listBox.setItems(filteredHosts.map(h => {
        const sta = hostStatus[h.alias] || "unknown";
        const ind = indicators[sta] || indicators.unknown;
        const mark = selectedAliases.has(h.alias) ? "▌" : " ";
        return `${mark} ${ind} ${h.alias.padEnd(14)} ${h.user}@${h.host}:${h.port}`;
    }));

    if (keepSelection && prevIdx < filteredHosts.length) {
        listBox.select(prevIdx);
        listBox.scrollTo(prevIdx);
    } else if (filteredHosts.length > 0) {
        listBox.select(0);
        listBox.scrollTo(0);
    }

    updateDetail();
    headerBar.setContent(
        `{bold} Quick-SSH {/bold}(${filteredHosts.length}/${hosts.length})  {cyan-fg}已选:${selectedAliases.size}{/cyan-fg}`
    );
    screen.render();
}

/**
 * 更新详情面板
 */
function updateDetail() {
    const idx = listBox.selected;
    if (idx < 0 || idx >= filteredHosts.length) {
        detailBox.setContent("{center}{yellow-fg}--- 无选中连接 ---{/yellow-fg}{/center}");
        screen.render();
        return;
    }

    const h = filteredHosts[idx];
    const sta = hostStatus[h.alias] || "unknown";
    const statusMap = {
        online:  "{green-fg}● 在线{/green-fg}",
        offline: "{red-fg}○ 离线{/red-fg}",
        unknown: "{yellow-fg}◌ 未检测{/yellow-fg}",
    };
    detailBox.setContent(`
{bold}{cyan-fg}  连接详情{/cyan-fg}{/bold}

  {bold}别名:{/bold}     ${h.alias}
  {bold}主机:{/bold}     ${h.host}
  {bold}账号:{/bold}     ${h.user}
  {bold}端口:{/bold}     ${h.port}
  {bold}私钥:{/bold}     ${h.key || "(默认)"}
  {bold}状态:{/bold}     ${statusMap[sta]}
  {bold}已选:{/bold}     ${selectedAliases.has(h.alias) ? "{green-fg}是{/green-fg}" : "否"}

  {blue-fg}─────────────────────{/blue-fg}
  {green-fg} Enter → 连接{/green-fg}     {red-fg} d → 删除{/red-fg}
  {yellow-fg} Space → 选择{/yellow-fg}    {yellow-fg} P → 批量检测{/yellow-fg}
`);
    screen.render();
}

/**
 * 在状态栏显示短暂消息（3 秒后自动恢复）
 */
function flashMessage(msg, color) {
    statusBar.setContent(`{${color}-fg} ${msg}{/${color}-fg}`);
    screen.render();
    setTimeout(() => {
        if (currentMode === MODE.NORMAL) setMode(MODE.NORMAL);
    }, 3000);
}

// ============================================================
// 操作函数
// ============================================================

function connectSelected() {
    const idx = listBox.selected;
    if (idx < 0 || idx >= filteredHosts.length) return;
    // sshConnect 需要 screen 参数用于销毁
    sshConnect(filteredHosts[idx], screen, startTUI);
}

function getActionHosts() {
    const selected = filteredHosts.filter(h => selectedAliases.has(h.alias));
    if (selected.length > 0) return selected;

    const idx = listBox.selected;
    if (idx < 0 || idx >= filteredHosts.length) return [];
    return [filteredHosts[idx]];
}

function toggleCurrentSelection() {
    const idx = listBox.selected;
    if (idx < 0 || idx >= filteredHosts.length) return;

    const alias = filteredHosts[idx].alias;
    if (selectedAliases.has(alias)) {
        selectedAliases.delete(alias);
        refreshList(true);
        flashMessage(`已取消选择 '${alias}'`, "yellow");
    } else {
        selectedAliases.add(alias);
        refreshList(true);
        flashMessage(`已选择 '${alias}'`, "green");
    }
}

function deleteSelected() {
    const targets = getActionHosts();
    if (targets.length === 0) return;

    confirmBox.setContent(
        targets.length === 1
            ? `{center}{red-fg}确认删除连接 '${targets[0].alias}'？ (y/n){/red-fg}{/center}`
            : `{center}{red-fg}确认删除已选 ${targets.length} 个连接？ (y/n){/red-fg}{/center}`
    );
    setMode(MODE.CONFIRM);
}

function confirmDelete(confirmed) {
    confirmBox.hide();
    if (confirmed) {
        const targets = getActionHosts();
        const targetAliases = new Set(targets.map(h => h.alias));
        hosts = hosts.filter(item => !targetAliases.has(item.alias));
        saveHosts(hosts);
        for (const alias of targetAliases) selectedAliases.delete(alias);
        refreshList();
        flashMessage(
            targets.length === 1
                ? `已删除连接 '${targets[0].alias}'`
                : `已删除 ${targets.length} 个连接`,
            "green"
        );
    }
    setMode(MODE.NORMAL);
}

// ============================================================
// 在线检测操作
// ============================================================

function checkSelectedHost() {
    const idx = listBox.selected;
    if (idx < 0 || idx >= filteredHosts.length) return;
    const alias = filteredHosts[idx].alias;
    flashMessage(`正在检测 '${alias}' ...`, "yellow");
    checkHost(alias, hosts, hostStatus, () => refreshList(true)).then(ok => {
        flashMessage(`'${alias}'  ${ok ? "● 在线" : "○ 离线"}`, ok ? "green" : "red");
    });
}

function checkAllHosts() {
    const selected = filteredHosts.filter(h => selectedAliases.has(h.alias));
    const list = selected.length > 0 ? selected : (filteredHosts.length > 0 ? filteredHosts : hosts);
    if (list.length === 0) {
        flashMessage("没有可检测的服务器", "red");
        return;
    }
    flashMessage(
        `正在检测 ${list.length} 台服务器${selected.length > 0 ? "（已选）" : ""} ...`,
        "yellow"
    );
    let done = 0;
    for (const h of list) {
        checkHost(h.alias, hosts, hostStatus, () => refreshList(true)).then(() => {
            done++;
            if (done === list.length) {
                const online  = list.filter(e => hostStatus[e.alias] === "online").length;
                const offline = list.filter(e => hostStatus[e.alias] === "offline").length;
                flashMessage(`检测完成: ${online} 在线, ${offline} 离线`, online > 0 ? "green" : "red");
            }
        });
    }
}

// ============================================================
// 输入处理
// ============================================================

function handleInputSubmit(value) {
    const val = value.trim();

    switch (currentMode) {
        case MODE.SEARCH: {
            filterText = val;
            inputBox.hide();
            refreshList();
            setMode(MODE.NORMAL);
            break;
        }
        case MODE.ADD: {
            inputBox.hide();
            const parts = val.split(/\s+/);
            if (parts.length < 2) {
                flashMessage("格式错误: 需要 别名 用户@主机:端口", "red");
                setMode(MODE.NORMAL);
                break;
            }
            const alias = parts[0];
            const userAtHost = parts[1];
            let keyPath = "";
            const ki = parts.indexOf("--key");
            if (ki >= 0 && ki + 1 < parts.length) keyPath = parts[ki + 1];

            let user = "", hostname = "", port = 22;
            const m1 = userAtHost.match(/^(.+)@(.+):(\d+)$/);
            const m2 = userAtHost.match(/^(.+)@(.+)$/);
            if (m1) { user = m1[1]; hostname = m1[2]; port = parseInt(m1[3]); }
            else if (m2) { user = m2[1]; hostname = m2[2]; }
            else {
                flashMessage("格式无效: 请使用 用户@主机:端口", "red");
                setMode(MODE.NORMAL);
                break;
            }

            const homeDir = process.env.HOME || process.env.USERPROFILE || os.homedir();
            if (!keyPath) {
                keyPath = path.join(homeDir, ".ssh", "id_rsa");
                // 非 Windows 平台：确保路径使用正斜线
                if (process.platform !== "win32") {
                    keyPath = keyPath.replace(/\\/g, "/");
                }
            }

            if (hosts.find(h => h.alias === alias)) {
                flashMessage(`别名 '${alias}' 已存在`, "red");
            } else {
                hosts.push({ alias, host: hostname, user, port, key: keyPath });
                saveHosts(hosts);
                refreshList(true);
                flashMessage(`已添加 '${alias}' → ${user}@${hostname}:${port}`, "green");
            }
            setMode(MODE.NORMAL);
            break;
        }
        case MODE.EXPORT: {
            inputBox.hide();
            const fp = val || path.join(path.dirname(SSH_CONFIG_PATH), "config.quickssh.bak");
            try {
                fs.writeFileSync(fp, JSON.stringify(hosts, null, 4), "utf-8");
                flashMessage(`已导出 ${hosts.length} 个连接到 '${fp}'`, "green");
            } catch (e) {
                flashMessage(`导出失败: ${e.message}`, "red");
            }
            setMode(MODE.NORMAL);
            break;
        }
        case MODE.IMPORT: {
            inputBox.hide();
            if (!val || !fs.existsSync(val)) {
                flashMessage("文件不存在", "red");
                setMode(MODE.NORMAL);
                break;
            }
            try {
                const imported = JSON.parse(fs.readFileSync(val, "utf-8"));
                let added = 0, skipped = 0;
                for (const h of imported) {
                    if (!hosts.find(e => e.alias === h.alias)) { hosts.push(h); added++; }
                    else { skipped++; }
                }
                saveHosts(hosts);
                refreshList();
                flashMessage(`导入完成: 新增 ${added}, 跳过 ${skipped}`, "green");
            } catch (e) {
                flashMessage(`导入失败: ${e.message}`, "red");
            }
            setMode(MODE.NORMAL);
            break;
        }
        case MODE.RENAME: {
            inputBox.hide();
            const idx = listBox.selected;
            if (idx < 0 || idx >= filteredHosts.length) { setMode(MODE.NORMAL); break; }
            const oldAlias = filteredHosts[idx].alias;
            if (!val || val.trim() === oldAlias) { setMode(MODE.NORMAL); break; }
            const newAlias = val.trim();
            if (hosts.find(h => h.alias === newAlias)) {
                flashMessage(`别名 '${newAlias}' 已存在`, "red");
                setMode(MODE.NORMAL);
                break;
            }
            const target = hosts.find(h => h.alias === oldAlias);
            if (target) {
                target.alias = newAlias;
                saveHosts(hosts);
                refreshList(true);
                flashMessage(`已重命名 '${oldAlias}' → '${newAlias}'`, "green");
            }
            setMode(MODE.NORMAL);
            break;
        }
    }
}

function handleInputCancel() {
    inputBox.hide();
    switch (currentMode) {
        case MODE.SEARCH: setMode(MODE.NORMAL); break;
        case MODE.ADD:    setMode(MODE.NORMAL); break;
        case MODE.EXPORT: setMode(MODE.NORMAL); break;
        case MODE.IMPORT: setMode(MODE.NORMAL); break;
        case MODE.RENAME: setMode(MODE.NORMAL); break;
        default:          setMode(MODE.NORMAL); break;
    }
}

// ============================================================
// 启动 TUI
// ============================================================

function startTUI() {
    screen = blessed.screen({
        smartCSR: true,
        title: "Quick-SSH",
        cursor: { artificial: true, shape: "block" },
        dockBorders: true,
        fullUnicode: true,
    });

    // 标题栏
    headerBar = blessed.box({
        top: 0, left: 0, width: "100%", height: 1,
        content: "{bold} Quick-SSH {/bold}",
        tags: true,
        style: { fg: "white", bg: -1 },
    });

    // 连接列表
    listBox = blessed.list({
        top: 1, left: 0, width: "60%", height: "100%-2",
        label: " 连接列表 ",
        border: { type: "line", fg: "cyan" },
        tags: true, keys: false, vi: false, mouse: true,
        scrollbar: { ch: " ", bg: -1 },
        style: {
            fg: "white", bg: -1,
            selected: { fg: "white", bg: "blue", bold: true },
            item: { fg: "white", bg: -1 },
        },
    });

    // 详情面板
    detailBox = blessed.box({
        top: 1, right: 0, width: "40%", height: "100%-2",
        label: " 详情 ",
        border: { type: "line", fg: "cyan" },
        tags: true, style: { fg: "white", bg: -1 },
        scrollable: true, alwaysScroll: true,
    });

    // 状态栏
    statusBar = blessed.box({
        bottom: 0, left: 0, width: "100%", height: 1,
        tags: true, style: { fg: "white", bg: -1 },
    });

    // 输入框
    inputBox = blessed.textbox({
        top: "50%-3", left: "25%", width: "50%", height: 3,
        label: " 输入 ",
        border: { type: "line", fg: "cyan" },
        hidden: true, keys: true, vi: false,
        style: { fg: "white", bg: -1, border: { fg: "cyan" } },
    });

    // 确认框
    confirmBox = blessed.box({
        top: "50%-3", left: "30%", width: "40%", height: 3,
        border: { type: "line", fg: "red" },
        tags: true, hidden: true,
        style: { fg: "white", bg: -1 },
    });

    // 帮助弹窗
    helpBox = blessed.box({
        top: "5%", left: "10%", width: "80%", height: "90%",
        label: " 帮助 ",
        border: { type: "line", fg: "yellow" },
        tags: true, hidden: true, scrollable: true, alwaysScroll: true,
        keys: true, vi: false,
        style: { fg: "white", bg: -1 },
        content: `
{bold}{yellow-fg}Quick-SSH TUI — 快捷键帮助{/yellow-fg}{/bold}

{cyan-fg}━━━━━━━━━ 导航 ━━━━━━━━━{/cyan-fg}
  {green-fg}j{/green-fg} / {green-fg}↓{/green-fg}      向下移动
  {green-fg}k{/green-fg} / {green-fg}↑{/green-fg}      向上移动
  {green-fg}gg{/green-fg}           跳转到第一个
  {green-fg}G{/green-fg}            跳转到最后一个

{cyan-fg}━━━━━━━━━ 操作 ━━━━━━━━━{/cyan-fg}
  {green-fg}Enter{/green-fg}        连接选中的服务器
  {green-fg}Space{/green-fg}        选择/取消当前连接
  {green-fg}d{/green-fg}            删除选中的连接
  {green-fg}a{/green-fg}            添加新连接
  {green-fg}r{/green-fg}            重命名连接
  {green-fg}/ {/green-fg}           搜索 / 筛选
  {green-fg}Esc{/green-fg}          取消 / 返回普通模式

{cyan-fg}━━━━━━━━━ 在线检测 ━━━━━━━━━{/cyan-fg}
  {green-fg}p{/green-fg}            检测选中服务器是否在线
  {green-fg}P{/green-fg} / {green-fg}C-p{/green-fg}  检测已选服务器 / 全部服务器

{cyan-fg}━━━━━━━━━ 导入导出 ━━━━━━━━━{/cyan-fg}
  {green-fg}e{/green-fg}            导出全部配置
  {green-fg}i{/green-fg}            从 JSON 文件导入

{cyan-fg}━━━━━━━━━ 其他 ━━━━━━━━━{/cyan-fg}
  {green-fg}q{/green-fg}            退出 TUI
  {green-fg}?{/green-fg}            显示本帮助

{white-fg}按任意键关闭帮助{/white-fg}
`
    });

    screen.append(headerBar);
    screen.append(listBox);
    screen.append(detailBox);
    screen.append(statusBar);
    screen.append(inputBox);
    screen.append(confirmBox);
    screen.append(helpBox);

    // ============================================================
    // 键位绑定
    // ============================================================

    // ----- 全局键 -----
    screen.key(["C-c"], () => {
        restoreTerminal();
        screen.destroy();
        process.exit(0);
    });

    screen.key(["escape"], () => {
        if (currentMode === MODE.HELP) {
            helpBox.hide();
            setMode(MODE.NORMAL);
        } else if (currentMode === MODE.CONFIRM) {
            confirmDelete(false);
        } else if (currentMode !== MODE.NORMAL) {
            handleInputCancel();
        }
    });

    // ----- NORMAL 模式 -----
    screen.key(["q"], () => {
        if (currentMode === MODE.NORMAL) {
            restoreTerminal();
            screen.destroy();
            process.exit(0);
        }
    });

    screen.key(["/"], () => {
        if (currentMode === MODE.NORMAL) setMode(MODE.SEARCH, filterText);
    });

    screen.key(["a"], () => {
        if (currentMode === MODE.NORMAL) setMode(MODE.ADD);
    });

    screen.key(["d"], () => {
        if (currentMode === MODE.NORMAL) deleteSelected();
    });

    screen.key(["y"], () => {
        if (currentMode === MODE.CONFIRM) confirmDelete(true);
    });

    screen.key(["n"], () => {
        if (currentMode === MODE.CONFIRM) confirmDelete(false);
    });

    screen.key(["enter"], () => {
        if (currentMode === MODE.NORMAL) connectSelected();
    });

    screen.key(["space"], () => {
        if (currentMode === MODE.NORMAL) toggleCurrentSelection();
    });

    screen.key(["e"], () => {
        if (currentMode === MODE.NORMAL) setMode(MODE.EXPORT, path.join(path.dirname(SSH_CONFIG_PATH), "config.quickssh.bak"));
    });

    screen.key(["i"], () => {
        if (currentMode === MODE.NORMAL) setMode(MODE.IMPORT);
    });

    screen.key(["p"], () => {
        if (currentMode === MODE.NORMAL) checkSelectedHost();
    });

    // 全检: P(Shift) / S-p / C-p
    screen.key(["P", "S-p", "C-p"], () => {
        if (currentMode === MODE.NORMAL) checkAllHosts();
    });

    screen.key(["r"], () => {
        if (currentMode === MODE.NORMAL) {
            const idx = listBox.selected;
            if (idx >= 0 && idx < filteredHosts.length) {
                setMode(MODE.RENAME, filteredHosts[idx].alias);
            }
        }
    });

    screen.key(["?", "h"], () => {
        if (currentMode === MODE.NORMAL) { helpBox.show(); helpBox.focus(); setMode(MODE.HELP); }
    });

    // Vim 导航键
    screen.key(["j", "down"], () => {
        if (currentMode !== MODE.NORMAL) return;
        if (listBox.selected < filteredHosts.length - 1) {
            listBox.select(listBox.selected + 1);
            listBox.scrollTo(listBox.selected);
            updateDetail();
            screen.render();
        }
    });

    screen.key(["k", "up"], () => {
        if (currentMode !== MODE.NORMAL) return;
        if (listBox.selected > 0) {
            listBox.select(listBox.selected - 1);
            listBox.scrollTo(listBox.selected);
            updateDetail();
            screen.render();
        }
    });

    // gg = 跳转到第一个（500ms 内双击 g）
    let gPressTimer = null;
    screen.key(["g"], () => {
        if (currentMode !== MODE.NORMAL) return;
        if (gPressTimer) {
            clearTimeout(gPressTimer);
            gPressTimer = null;
            listBox.select(0);
            listBox.scrollTo(0);
            updateDetail();
            screen.render();
        } else {
            gPressTimer = setTimeout(() => {
                gPressTimer = null;
            }, 500);
        }
    });

    screen.key(["G", "S-g"], () => {
        if (currentMode !== MODE.NORMAL) return;
        if (filteredHosts.length > 0) {
            listBox.select(filteredHosts.length - 1);
            listBox.scrollTo(filteredHosts.length - 1);
            updateDetail();
            screen.render();
        }
    });

    // ----- 输入框事件 -----
    inputBox.on("submit", handleInputSubmit);
    inputBox.on("cancel", handleInputCancel);

    // ----- 列表事件 -----
    listBox.on("select", () => {
        if (currentMode === MODE.NORMAL) connectSelected();
    });

    listBox.on("select item", () => { updateDetail(); });

    // 鼠标滚轮
    listBox.on("wheeldown", () => {
        if (currentMode !== MODE.NORMAL) return;
        if (listBox.selected < filteredHosts.length - 1) {
            listBox.select(listBox.selected + 1);
            listBox.scrollTo(listBox.selected);
            updateDetail();
            screen.render();
        }
    });

    listBox.on("wheelup", () => {
        if (currentMode !== MODE.NORMAL) return;
        if (listBox.selected > 0) {
            listBox.select(listBox.selected - 1);
            listBox.scrollTo(listBox.selected);
            updateDetail();
            screen.render();
        }
    });

    // ----- 帮助弹窗关闭 -----
    helpBox.key(["escape", "q", "enter", "space"], () => {
        helpBox.hide();
        setMode(MODE.NORMAL);
    });

    // ============================================================
    // 初始化
    // ============================================================

    refreshList();
    listBox.focus();
    setMode(MODE.NORMAL);
    screen.render();
}

// ============================================================
// 入口（直接运行时自动启动）
// ============================================================

startTUI();
