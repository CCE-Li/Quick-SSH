#!/usr/bin/env node
/**
 * src/unix/cli.js - Quick-SSH 命令行接口 (Node.js)
 *
 * 职责:
 *   在 Linux/macOS 上替代 PowerShell 模块，提供完整的 qssh 命令支持。
 *   在 Windows 上也可作为 PowerShell 模块的备选方案。
 *
 * 设计原则:
 *   - 复用 src/tui/data.js 的数据操作（同一份 ~/.ssh/config）
 *   - 自身不依赖 PowerShell，只依赖 Node.js + ssh 命令
 *   - 无参数时启动 TUI (blessed)
 *
 * 用法:
 *   node src/unix/cli.js           # 启动 TUI
 *   node src/unix/cli.js ps [...]  # 列出连接
 */

const path    = require("path");
const fs      = require("fs");
const { spawn } = require("child_process");

// 复用 TUI 数据层（同一份 ~/.ssh/config）
const { SSH_CONFIG_PATH, loadHosts, saveHosts } = require("../tui/data");

// ============================================================
// 颜色工具
// ============================================================

const COLOR = {
    green:  (s) => `\x1b[32m${s}\x1b[0m`,
    yellow: (s) => `\x1b[33m${s}\x1b[0m`,
    red:    (s) => `\x1b[31m${s}\x1b[0m`,
    cyan:   (s) => `\x1b[36m${s}\x1b[0m`,
    bold:   (s) => `\x1b[1m${s}\x1b[0m`,
};

// ============================================================
// 跨平台 SSH 可执行文件
// ============================================================

function getSSHExe() {
    return process.platform === "win32" ? "ssh.exe" : "ssh";
}

// ============================================================
// 命令实现
// ============================================================

/**
 * qssh ps [keyword] - 列出所有已保存的连接
 */
function cmdPs(keyword) {
    const hosts = loadHosts();
    if (hosts.length === 0) {
        console.log(COLOR.yellow("当前没有已保存的 SSH 连接。使用 'qssh add' 添加一个。"));
        return;
    }

    let filtered = hosts;
    if (keyword) {
        const kw = keyword.toLowerCase();
        filtered = hosts.filter(h =>
            h.alias.toLowerCase().includes(kw) ||
            (h.host || "").toLowerCase().includes(kw) ||
            (h.user || "").toLowerCase().includes(kw)
        );
        if (filtered.length === 0) {
            console.log(COLOR.yellow(`没有匹配 '${keyword}' 的 SSH 连接。`));
            return;
        }
    }

    // 计算列宽
    const aliasLen  = Math.max(6, ...filtered.map(h => (h.alias || "").length));
    const hostLen   = Math.max(10, ...filtered.map(h => (h.host || "").length));
    const userLen   = Math.max(6, ...filtered.map(h => (h.user || "").length));

    const sep = "  ";
    const header =
        COLOR.bold("别名".padEnd(aliasLen)) + sep +
        COLOR.bold("IP 地址".padEnd(hostLen)) + sep +
        COLOR.bold("账号".padEnd(userLen)) + sep +
        COLOR.bold("端口") + sep +
        COLOR.bold("私钥路径");

    console.log("");
    console.log(header);
    console.log("-".repeat(header.length));

    for (const h of filtered) {
        console.log(
            h.alias.padEnd(aliasLen) + sep +
            (h.host || "").padEnd(hostLen) + sep +
            (h.user || "").padEnd(userLen) + sep +
            String(h.port || 22) + sep +
            (h.key || "(默认)")
        );
    }
    console.log("");
}

/**
 * qssh add <alias> <user@host:port> [--key <path>] - 添加新连接
 */
function cmdAdd(alias, userAtHost, keyPath) {
    if (!alias || !userAtHost) {
        console.error(COLOR.red("错误：用法 → qssh add <别名> <用户名@IP:端口> [--key <私钥路径>]"));
        process.exit(1);
    }

    // 解析 user@host:port
    let user = "", hostname = "", port = 22;
    const m1 = userAtHost.match(/^(.+)@(.+):(\d+)$/);
    const m2 = userAtHost.match(/^(.+)@(.+)$/);
    if (m1) {
        user = m1[1];
        hostname = m1[2];
        port = parseInt(m1[3], 10);
    } else if (m2) {
        user = m2[1];
        hostname = m2[2];
    } else {
        console.error(COLOR.red("错误：格式无效，请使用「用户名@主机:端口」或「用户名@主机」格式。"));
        process.exit(1);
    }

    if (!keyPath) {
        keyPath = path.join(path.dirname(SSH_CONFIG_PATH), "id_rsa");
    }

    const hosts = loadHosts();
    if (hosts.find(h => h.alias === alias)) {
        console.error(COLOR.red(`错误：别名 '${alias}' 已存在，请使用其他名称。`));
        process.exit(1);
    }

    hosts.push({ alias, host: hostname, user, port, key: keyPath });
    saveHosts(hosts);

    console.log(COLOR.green(`✔ 已添加 SSH 连接 '${alias}' → ${user}@${hostname}:${port}`));
}

/**
 * qssh rm <alias> - 删除连接
 */
function cmdRm(alias) {
    if (!alias) {
        console.error(COLOR.red("错误：用法 → qssh rm <别名>"));
        process.exit(1);
    }

    const hosts = loadHosts();
    const idx = hosts.findIndex(h => h.alias === alias);
    if (idx === -1) {
        console.error(COLOR.red(`错误：别名 '${alias}' 不存在。使用 'qssh ps' 查看可用连接。`));
        process.exit(1);
    }

    hosts.splice(idx, 1);
    saveHosts(hosts);
    console.log(COLOR.green(`✔ 已删除 SSH 连接 '${alias}'.`));
}

/**
 * qssh <alias> - 一键 SSH 连接
 */
function cmdConnect(alias) {
    if (!alias) {
        // 无参数 → 启动 TUI
        launchTUI();
        return;
    }

    const hosts = loadHosts();
    const target = hosts.find(h => h.alias === alias);
    if (!target) {
        console.error(COLOR.red(`错误：别名 '${alias}' 不存在。使用 'qssh ps' 查看可用连接。`));
        process.exit(1);
    }

    const sshExe = getSSHExe();
    const args = [
        "-i", target.key,
        "-p", String(target.port),
        "-o", "HostKeyAlgorithms=+ssh-rsa",
        `${target.user}@${target.host}`,
    ];

    console.log(COLOR.green(`正在连接到 '${alias}' (${target.user}@${target.host}:${target.port}) ...`));
    console.log("");

    const child = spawn(sshExe, args, { stdio: "inherit" });
    child.on("exit", (code) => {
        process.exit(code != null ? code : 0);
    });
}

/**
 * qssh export <file> - 导出配置为标准 SSH config 格式
 */
function cmdExport(filePath) {
    if (!filePath) {
        console.error(COLOR.red("错误：用法 → qssh export <文件路径>"));
        process.exit(1);
    }

    const hosts = loadHosts();
    if (hosts.length === 0) {
        console.log(COLOR.yellow("没有可导出的 SSH 连接。"));
        return;
    }

    // 以 SSH config 格式导出
    const lines = [];
    for (const h of hosts) {
        lines.push(`Host ${h.alias}`);
        if (h.host) lines.push(`    HostName ${h.host}`);
        if (h.user) lines.push(`    User ${h.user}`);
        lines.push(`    Port ${h.port || 22}`);
        if (h.key)  lines.push(`    IdentityFile ${h.key}`);
        lines.push("");
    }
    fs.writeFileSync(filePath, lines.join("\n"), "utf-8");
    console.log(COLOR.green(`✔ 已导出 ${hosts.length} 个连接到 '${filePath}' (SSH config 格式)。`));
}

/**
 * qssh import <file> - 导入 SSH config 格式文件
 */
function cmdImport(filePath) {
    if (!filePath) {
        console.error(COLOR.red("错误：用法 → qssh import <文件路径>"));
        process.exit(1);
    }

    if (!fs.existsSync(filePath)) {
        console.error(COLOR.red(`错误：文件 '${filePath}' 不存在。`));
        process.exit(1);
    }

    // 复用 data.js 的解析逻辑（内联避免循环依赖）
    function parseConfig(content) {
        const hosts = [];
        const lines = content.split("\n");
        let i = 0;
        while (i < lines.length) {
            const trimmed = lines[i].trim();
            if (!trimmed || trimmed.startsWith("#")) { i++; continue; }
            const hostMatch = trimmed.match(/^Host\s+(.+)$/i);
            if (!hostMatch) { i++; continue; }
            const alias = hostMatch[1].trim();
            const entry = { alias, host: "", user: "", port: 22, key: "" };
            i++;
            while (i < lines.length) {
                const prop = lines[i].trim();
                if (!prop || prop.startsWith("#") || /^Host\s+/i.test(prop)) break;
                const kv = prop.match(/^(\w+)\s+(.+)$/);
                if (kv) {
                    const k = kv[1].toLowerCase();
                    const v = kv[2].trim();
                    if (k === "hostname")      entry.host = v;
                    else if (k === "user")     entry.user = v;
                    else if (k === "port")     entry.port = parseInt(v, 10) || 22;
                    else if (k === "identityfile") entry.key = v;
                }
                i++;
            }
            hosts.push(entry);
        }
        return hosts;
    }

    let imported;
    try {
        imported = parseConfig(fs.readFileSync(filePath, "utf-8"));
    } catch {
        console.error(COLOR.red("错误：无法解析文件，请检查 SSH config 格式。"));
        process.exit(1);
    }

    if (imported.length === 0) {
        console.log(COLOR.yellow("文件中没有有效的 Host 配置。"));
        return;
    }

    const hosts = loadHosts();
    let added = 0, skipped = 0;
    for (const h of imported) {
        if (!hosts.find(e => e.alias === h.alias)) {
            hosts.push(h);
            added++;
        } else {
            skipped++;
        }
    }
    saveHosts(hosts);
    console.log(COLOR.green(`✔ 导入完成：新增 ${added} 个，跳过 ${skipped} 个（别名重复）。`));
}

/**
 * 启动 TUI 界面
 */
function launchTUI() {
    const tuiScript = path.join(__dirname, "..", "tui", "index.js");
    if (!fs.existsSync(tuiScript)) {
        console.error(COLOR.red(`错误：未找到 TUI 脚本: ${tuiScript}`));
        process.exit(1);
    }

    const child = spawn(process.execPath, [tuiScript], {
        stdio: "inherit",
        env: { ...process.env },
    });
    child.on("exit", (code) => {
        process.exit(code != null ? code : 0);
    });
}

/**
 * qssh help - 显示帮助
 */
function cmdHelp() {
    console.log("");
    console.log(COLOR.cyan("Quick-SSH - SSH 连接管理工具 (Node.js CLI)"));
    console.log("");
    console.log(COLOR.yellow("用法:"));
    console.log("  qssh                   启动 TUI 终端界面（推荐）");
    console.log("  qssh ps [关键词]      列出所有已保存的 SSH 连接");
    console.log("  qssh add <别名> <用户@主机:端口> [--key <私钥路径>]");
    console.log("                        添加新 SSH 连接（端口默认 22，私钥默认 ~/.ssh/id_rsa）");
    console.log("  qssh rm <别名>        删除指定别名的 SSH 连接");
    console.log("  qssh <别名>           一键连接 SSH 服务器");
    console.log("  qssh export <文件>    导出全部主机配置为 SSH config 格式");
    console.log("  qssh import <文件>    从 SSH config 文件批量导入连接");
    console.log("  qssh init             重新注册 Quick-SSH 到 Shell 配置文件");
    console.log("  qssh help             显示本帮助信息");
    console.log("");
    console.log(COLOR.yellow("示例:"));
    console.log("  qssh");
    console.log("  qssh ps");
    console.log("  qssh ps 生产");
    console.log("  qssh add my-server root@192.168.1.100:22 --key ~/.ssh/id_rsa");
    console.log("  qssh add my-vm admin@10.0.0.5");
    console.log("  qssh my-server");
    console.log("  qssh rm my-vm");
    console.log("");
}

// ============================================================
// CLI 派发入口
// ============================================================

function main() {
    const args = process.argv.slice(2);

    if (args.length === 0) {
        // qssh (无参数) → 启动 TUI
        launchTUI();
        return;
    }

    const cmd = args[0];
    const rest = args.slice(1);

    switch (cmd) {
        case "ps": {
            const keyword = rest.length > 0 ? rest[0] : "";
            cmdPs(keyword);
            break;
        }

        case "add": {
            let alias = null, userAtHost = null, keyPath = null;
            let i = 0;
            while (i < rest.length) {
                if (rest[i] === "--key" || rest[i] === "-k") {
                    i++;
                    if (i < rest.length) keyPath = rest[i];
                } else if (!alias) {
                    alias = rest[i];
                } else if (!userAtHost) {
                    userAtHost = rest[i];
                }
                i++;
            }
            cmdAdd(alias, userAtHost, keyPath);
            break;
        }

        case "rm": {
            const alias = rest.length > 0 ? rest[0] : "";
            cmdRm(alias);
            break;
        }

        case "export": {
            const filePath = rest.length > 0 ? rest[0] : "";
            cmdExport(filePath);
            break;
        }

        case "import": {
            const filePath = rest.length > 0 ? rest[0] : "";
            cmdImport(filePath);
            break;
        }

        case "init": {
            // 重新运行 postinstall 逻辑，重新注入 shell 配置文件
            console.log(COLOR.cyan("[Quick-SSH] 正在重新注册到 Shell 配置文件..."));
            const postinstall = path.join(__dirname, "lib", "index.js");
            if (fs.existsSync(postinstall)) {
                const child = spawn(process.execPath, [postinstall, "postinstall"], {
                    stdio: "inherit",
                });
                child.on("exit", (code) => {
                    process.exit(code != null ? code : 0);
                });
            } else {
                console.error(COLOR.red(`错误：未找到 postinstall 脚本: ${postinstall}`));
                process.exit(1);
            }
            break;
        }

        case "help":
        case "--help":
        case "-h": {
            cmdHelp();
            break;
        }

        default: {
            // 未知命令当作别名尝试连接
            cmdConnect(cmd);
            break;
        }
    }
}

main();
