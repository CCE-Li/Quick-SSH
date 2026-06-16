#!/usr/bin/env node
/**
 * qssh-tui.js - Quick-SSH 终端用户界面 (TUI)
 * 类似 yazi 的布局 + Vim 操作键位
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
 *
 * 依赖: blessed (npm install blessed)
 * 数据: %USERPROFILE%\.quickssh\hosts.json
 */

const blessed   = require("blessed");
const fs        = require("fs");
const path      = require("path");
const { spawn } = require("child_process");

// ============================================================
// 配置
// ============================================================

const CONFIG_DIR  = path.join(process.env.USERPROFILE || "~", ".quickssh");
const CONFIG_FILE = path.join(CONFIG_DIR, "hosts.json");

// ============================================================
// 数据层
// ============================================================

function ensureConfig() {
    if (!fs.existsSync(CONFIG_DIR))  fs.mkdirSync(CONFIG_DIR, { recursive: true });
    if (!fs.existsSync(CONFIG_FILE)) fs.writeFileSync(CONFIG_FILE, "[]", "utf-8");
}

function loadHosts() {
    ensureConfig();
    try {
        const raw = fs.readFileSync(CONFIG_FILE, "utf-8").trim();
        return raw ? JSON.parse(raw) : [];
    } catch { return []; }
}

function saveHosts(hosts) {
    ensureConfig();
    fs.writeFileSync(CONFIG_FILE, JSON.stringify(hosts, null, 4), "utf-8");
}

// ============================================================
// SSH 连接
// ============================================================

function sshConnect(host, cbReturn) {
    const sshExe = "ssh.exe";
    const args = [
        "-i", host.key,
        "-p", String(host.port),
        "-o", "HostKeyAlgorithms=+ssh-rsa",
        `${host.user}@${host.host}`,
    ];

    screen.destroy();
    process.stdin.removeAllListeners("data");

    process.stdout.write(`\n\x1b[32m正在连接到 '${host.alias}' (${host.user}@${host.host}:${host.port}) ...\x1b[0m\n\n`);

    const child = spawn(sshExe, args, {
        stdio: "inherit",
        shell: true,
    });

    child.on("exit", (code) => {
        process.stdout.write(`\n\x1b[33mSSH 会话已结束 (退出码: ${code})\x1b[0m\n`);
        process.stdout.write(`\x1b[32m按 Enter 键返回 Quick-SSH TUI...\x1b[0m`);
        process.stdin.setRawMode(false);
        process.stdin.once("data", () => cbReturn());
    });
}

// ============================================================
// TUI 组件
// ============================================================

let screen;
let headerBar;
let listBox;
let detailBox;
let statusBar;
let inputBox;
let helpBox;
let confirmBox;

let hosts        = [];
let filteredHosts = [];
let filterText   = "";

// 模式管理
const MODE = { NORMAL: 0, SEARCH: 1, ADD: 2, EXPORT: 3, IMPORT: 4, CONFIRM: 5, HELP: 6 };
let currentMode = MODE.NORMAL;

// ============================================================
// UI 更新函数
// ============================================================

const MODE_LABELS = {
    [MODE.NORMAL]:  "{green-fg}{bold} NORMAL {/bold}{/green-fg}",
    [MODE.SEARCH]:  "{cyan-fg}{bold} SEARCH {/bold}{/cyan-fg}",
    [MODE.ADD]:     "{yellow-fg}{bold} ADD {/bold}{/yellow-fg}",
    [MODE.EXPORT]:  "{yellow-fg}{bold} EXPORT {/bold}{/yellow-fg}",
    [MODE.IMPORT]:  "{yellow-fg}{bold} IMPORT {/bold}{/yellow-fg}",
    [MODE.CONFIRM]: "{red-fg}{bold} CONFIRM {/bold}{/red-fg}",
    [MODE.HELP]:    "{white-fg}{bold} HELP {/bold}{/white-fg}",
};

const MODE_HINTS = {
    [MODE.NORMAL]:  " j/k ↑↓  g首 G尾  ↵连接  d删除  a添加  /搜索  e导出  i导入  r重命名  ?帮助  q退出",
    [MODE.SEARCH]:  " 输入关键词过滤  Enter确认  Esc取消",
    [MODE.ADD]:     " 格式: 别名 用户@主机:端口 [--key 路径]  Enter确认  Esc取消",
    [MODE.EXPORT]:  " 输入导出文件路径 (默认: ~/.quickssh/export.json)  Enter确认  Esc取消",
    [MODE.IMPORT]:  " 输入导入文件路径  Enter确认  Esc取消",
    [MODE.CONFIRM]: " y确认  n取消",
    [MODE.HELP]:    " 按任意键返回",
};

function setMode(mode, inputValue) {
    currentMode = mode;
    statusBar.setContent(`${MODE_LABELS[mode] || ""} ${MODE_HINTS[mode] || ""}`);

    if (mode === MODE.NORMAL || mode === MODE.HELP) {
        inputBox.hide();
        confirmBox.hide();
        if (mode === MODE.NORMAL) listBox.focus();
    }

    if (mode === MODE.SEARCH || mode === MODE.ADD || mode === MODE.EXPORT || mode === MODE.IMPORT) {
        inputBox.show();
        inputBox.setValue(inputValue || "");
        inputBox.focus();
    }

    if (mode === MODE.CONFIRM) {
        confirmBox.show();
    }

    screen.render();
}

function refreshList(keepSelection) {
    hosts = loadHosts();
    filteredHosts = filterText
        ? hosts.filter(h =>
            h.alias.toLowerCase().includes(filterText.toLowerCase()) ||
            h.host.toLowerCase().includes(filterText.toLowerCase())  ||
            h.user.toLowerCase().includes(filterText.toLowerCase())
          )
        : [...hosts];

    const prevIdx = listBox.selected;
    listBox.setItems(filteredHosts.map(h =>
        `${h.alias.padEnd(16)} ${h.user}@${h.host}:${h.port}`
    ));

    if (keepSelection && prevIdx < filteredHosts.length) {
        listBox.select(prevIdx);
        listBox.scrollTo(prevIdx);
    } else if (filteredHosts.length > 0) {
        listBox.select(0);
        listBox.scrollTo(0);
    }

    updateDetail();
    headerBar.setContent(
        `{bold} Quick-SSH {/bold}(${filteredHosts.length}/${hosts.length})`
    );
    screen.render();
}

function updateDetail() {
    const idx = listBox.selected;
    if (idx < 0 || idx >= filteredHosts.length) {
        detailBox.setContent("{center}{yellow-fg}--- 无选中连接 ---{/yellow-fg}{/center}");
        screen.render();
        return;
    }

    const h = filteredHosts[idx];
    detailBox.setContent(`
{bold}{cyan-fg}  连接详情{/cyan-fg}{/bold}

  {bold}别名:{/bold}     ${h.alias}
  {bold}主机:{/bold}     ${h.host}
  {bold}账号:{/bold}     ${h.user}
  {bold}端口:{/bold}     ${h.port}
  {bold}私钥:{/bold}     ${h.key || "(默认)"}

  {blue-fg}─────────────────────{/blue-fg}
  {green-fg} Enter → 连接{/green-fg}
  {red-fg} d → 删除{/red-fg}
`);
    screen.render();
}

// ============================================================
// 操作函数
// ============================================================

function connectSelected() {
    const idx = listBox.selected;
    if (idx < 0 || idx >= filteredHosts.length) return;
    sshConnect(filteredHosts[idx], startTUI);
}

function deleteSelected() {
    const idx = listBox.selected;
    if (idx < 0 || idx >= filteredHosts.length) return;
    const h = filteredHosts[idx];
    confirmBox.setContent(
        `{center}{red-fg}确认删除连接 '{h.alias}'？ (y/n){/red-fg}{/center}`
    );
    setMode(MODE.CONFIRM);
}

function confirmDelete(confirmed) {
    confirmBox.hide();
    if (confirmed) {
        const idx = listBox.selected;
        const h = filteredHosts[idx];
        hosts = hosts.filter(item => item.alias !== h.alias);
        saveHosts(hosts);
        refreshList();
        flashMessage(`已删除连接 '${h.alias}'`, "green");
    }
    setMode(MODE.NORMAL);
}

function flashMessage(msg, color) {
    statusBar.setContent(`{${color}-fg} ${msg}{/${color}-fg}`);
    screen.render();
    setTimeout(() => {
        if (currentMode === MODE.NORMAL) setMode(MODE.NORMAL);
    }, 3000);
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

            if (!keyPath) keyPath = path.join(process.env.USERPROFILE || "~", ".ssh", "id_rsa");

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
            const fp = val || path.join(CONFIG_DIR, "export.json");
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
    }
}

function handleInputCancel() {
    inputBox.hide();
    switch (currentMode) {
        case MODE.SEARCH: setMode(MODE.NORMAL); break;
        case MODE.ADD:    setMode(MODE.NORMAL); break;
        case MODE.EXPORT: setMode(MODE.NORMAL); break;
        case MODE.IMPORT: setMode(MODE.NORMAL); break;
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
        cursor: { artificial: true, shape: "underline" },
        dockBorders: true,
        fullUnicode: true,
    });

    // 标题栏
    headerBar = blessed.box({
        top: 0, left: 0, width: "100%", height: 1,
        content: "{bold} Quick-SSH {/bold}",
        tags: true,
        style: { fg: "white", bg: "blue" },
    });

    // 连接列表
    listBox = blessed.list({
        top: 1, left: 0, width: "60%", height: "100%-2",
        label: " 连接列表 ",
        border: { type: "line", fg: "cyan" },
        tags: true, keys: false, vi: false, mouse: true,
        scrollbar: { ch: " ", bg: "cyan" },
        style: {
            fg: "white", bg: "black",
            selected: { fg: "white", bg: "blue", bold: true },
            item: { fg: "white", bg: "black" },
        },
    });

    // 详情面板
    detailBox = blessed.box({
        top: 1, right: 0, width: "40%", height: "100%-2",
        label: " 详情 ",
        border: { type: "line", fg: "cyan" },
        tags: true, style: { fg: "white", bg: "black" },
        scrollable: true, alwaysScroll: true,
    });

    // 状态栏
    statusBar = blessed.box({
        bottom: 0, left: 0, width: "100%", height: 1,
        tags: true, style: { fg: "white", bg: "black" },
    });

    // 输入框
    inputBox = blessed.textbox({
        top: "50%-3", left: "25%", width: "50%", height: 3,
        label: " 输入 ",
        border: { type: "line", fg: "cyan" },
        hidden: true, keys: true, vi: false,
        style: { fg: "white", bg: "black", border: { fg: "cyan" } },
    });

    // 确认框
    confirmBox = blessed.box({
        top: "50%-3", left: "30%", width: "40%", height: 3,
        border: { type: "line", fg: "red" },
        tags: true, hidden: true,
        style: { fg: "white", bg: "black" },
    });

    // 帮助弹窗
    helpBox = blessed.box({
        top: "5%", left: "10%", width: "80%", height: "90%",
        label: " 帮助 ",
        border: { type: "line", fg: "yellow" },
        tags: true, hidden: true, scrollable: true, alwaysScroll: true,
        keys: true, vi: false,
        style: { fg: "white", bg: "black" },
        content: `
{bold}{yellow-fg}Quick-SSH TUI — 快捷键帮助{/yellow-fg}{/bold}

{cyan-fg}━━━━━━━━━ 导航 ━━━━━━━━━{/cyan-fg}
  {green-fg}j{/green-fg} / {green-fg}↓{/green-fg}      向下移动
  {green-fg}k{/green-fg} / {green-fg}↑{/green-fg}      向上移动
  {green-fg}g{/green-fg}            跳转到第一个
  {green-fg}G{/green-fg}            跳转到最后一个

{cyan-fg}━━━━━━━━━ 操作 ━━━━━━━━━{/cyan-fg}
  {green-fg}Enter{/green-fg}        连接选中的服务器
  {green-fg}d{/green-fg}            删除选中的连接
  {green-fg}a{/green-fg}            添加新连接
  {green-fg}/ {/green-fg}           搜索 / 筛选
  {green-fg}Esc{/green-fg}          取消 / 返回普通模式

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
    screen.key(["C-c"], () => { screen.destroy(); process.exit(0); });

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
        if (currentMode === MODE.NORMAL) { screen.destroy(); process.exit(0); }
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

    screen.key(["e"], () => {
        if (currentMode === MODE.NORMAL) setMode(MODE.EXPORT, path.join(CONFIG_DIR, "export.json"));
    });

    screen.key(["i"], () => {
        if (currentMode === MODE.NORMAL) setMode(MODE.IMPORT);
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

    screen.key(["g"], () => {
        if (currentMode !== MODE.NORMAL) return;
        if (filteredHosts.length > 0) {
            listBox.select(0);
            listBox.scrollTo(0);
            updateDetail();
            screen.render();
        }
    });

    screen.key(["G"], () => {
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
// 入口
// ============================================================

startTUI();
