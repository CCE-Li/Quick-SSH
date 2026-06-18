/**
 * network.js - Quick-SSH TUI 网络层
 *
 * 提供 SSH 连接和服务器在线检测功能。
 * 通过回调函数与 UI 层解耦，便于测试和复用。
 */

const net       = require("net");
const { spawn } = require("child_process");
const os       = require("os");

// ============================================================
// 跨平台 SSH 可执行文件检测
// ============================================================

/**
 * 获取当前平台的 SSH 可执行文件名
 *  - Windows: ssh.exe
 *  - Linux/macOS: ssh
 */
function getSSHExe() {
    const platform = process.platform;
    return platform === "win32" ? "ssh.exe" : "ssh";
}

// ============================================================
// SSH 连接
// ============================================================

/**
 * 发起 SSH 连接会话
 * @param {Object} host        - 主机配置对象
 * @param {Object} screen      - blessed screen 实例（用于销毁）
 * @param {Function} cbReturn  - 会话结束后回调，重新进入 TUI
 */
function sshConnect(host, screen, cbReturn) {
    const sshExe = getSSHExe();
    const args = [
        "-i", host.key,
        "-p", String(host.port),
        "-o", "HostKeyAlgorithms=+ssh-rsa",
        `${host.user}@${host.host}`,
    ];

    screen.destroy();

    // 手动恢复终端状态（rmcup 被拦截，screen.destroy() 无法完全恢复）
    try {
        if (process.stdin.isTTY) process.stdin.setRawMode(false);
    } catch (e) {}
    // 禁用鼠标追踪（blessed 启用了 SGR 鼠标模式，必须关闭）
    process.stdout.write('\x1b[?1000l\x1b[?1002l\x1b[?1003l\x1b[?1006l');
    process.stdout.write('\x1b[?1049l');
    process.stdout.write('\x1b[?25h');
    process.stdout.write('\x1b[0m');

    process.stdin.removeAllListeners("data");

    process.stdout.write(`\n\x1b[32m正在连接到 '${host.alias}' (${host.user}@${host.host}:${host.port}) ...\x1b[0m\n\n`);

    const child = spawn(sshExe, args, {
        stdio: "inherit",
        shell: true,
    });

    child.on("exit", (code) => {
        process.stdout.write(`\n\x1b[33mSSH 会话已结束 (退出码: ${code})\x1b[0m\n`);
        process.stdout.write(`\x1b[32m按 Enter 键返回 Quick-SSH TUI...\x1b[0m`);
        // SSH 退出后确保终端处于 cooked 模式
        try {
            if (process.stdin.isTTY) process.stdin.setRawMode(false);
        } catch (e) {}
        process.stdin.once("data", () => cbReturn());
    });
}

// ============================================================
// 在线检测
// ============================================================

/**
 * 检测单台服务器是否在线（TCP 连接 SSH 端口）
 * @param {string} alias       - 主机别名
 * @param {Array} hosts        - 全部主机列表
 * @param {Object} hostStatus  - 在线状态缓存对象（会被修改）
 * @param {Function} onRefresh - 状态变化时回调，用于刷新 UI
 * @returns {Promise<boolean>} - true=在线, false=离线
 */
function checkHost(alias, hosts, hostStatus, onRefresh) {
    return new Promise((resolve) => {
        const h = hosts.find(e => e.alias === alias);
        if (!h) { resolve(false); return; }

        hostStatus[alias] = "checking";
        if (onRefresh) onRefresh();

        const sock = new net.Socket();
        const port = h.port || 22;
        const timeout = 3000;

        sock.setTimeout(timeout);
        sock.on("connect", () => {
            sock.destroy();
            hostStatus[alias] = "online";
            if (onRefresh) onRefresh();
            resolve(true);
        });
        sock.on("error", () => {
            sock.destroy();
            hostStatus[alias] = "offline";
            if (onRefresh) onRefresh();
            resolve(false);
        });
        sock.on("timeout", () => {
            sock.destroy();
            hostStatus[alias] = "offline";
            if (onRefresh) onRefresh();
            resolve(false);
        });
        sock.connect(port, h.host);
    });
}

module.exports = { sshConnect, checkHost };
