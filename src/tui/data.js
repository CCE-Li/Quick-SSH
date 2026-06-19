/**
 * data.js - Quick-SSH TUI 数据层
 *
 * 使用 ~/.ssh/config (标准 OpenSSH 配置文件) 作为存储后端。
 * 兼容 Windows 和 Linux/macOS，路径统一为 ~/.ssh/config。
 *
 * SSH config 格式:
 *   Host alias
 *       HostName host
 *       User user
 *       Port port
 *       IdentityFile key
 *
 * 设计:
 *   - 读取时解析整个 ~/.ssh/config，提取所有 Host 块
 *   - 非 Host 内容（注释、全局选项、空行）保留为文本段
 *   - 保存时只替换别名匹配的 Host 块，其余内容原样保留
 */

const fs   = require("fs");
const path = require("path");
const os   = require("os");

// ============================================================
// 配置路径
// ============================================================

/** 跨平台获取用户 home 目录 */
function getHomeDir() {
    return process.env.HOME || process.env.USERPROFILE || os.homedir();
}

/** SSH 配置文件路径（Win/Linux/macOS 统一） */
const SSH_CONFIG_PATH = path.join(getHomeDir(), ".ssh", "config");

/** 平台是否为 Windows */
const IS_WIN = process.platform === "win32";

/**
 * 路径分隔符归一化：非 Windows 平台将反斜线替换为正斜线
 * 防止从 Windows 共享的 ~/.ssh/config 中遗留反斜线路径
 * @param {string} p
 * @returns {string}
 */
function normalizePath(p) {
    if (!p || IS_WIN) return p;
    return p.replace(/\\/g, "/");
}

// ============================================================
// SSH config 解析/生成
// ============================================================

/**
 * 从 SSH config 内容中解析出所有 Host 块
 *
 * @param {string} content - ~/.ssh/config 的完整内容
 * @returns {Array<{ alias, host, user, port, key, _start, _end }>}
 *   每个对象包含 host 字段和该块在原文中的行范围 (_start/_end)
 */
// TODO: 未省略#后的注释内容
function parseHostBlocks(content) {
    const hosts = [];
    const lines = content.split("\n");
    let i = 0;

    while (i < lines.length) {
        const trimmed = lines[i].trim();
        // 跳过空行和注释
        if (!trimmed || trimmed.startsWith("#")) {
            i++;
            continue;
        }

        const hostMatch = trimmed.match(/^Host\s+(.+)$/i);
        if (!hostMatch) {
            // 全局选项行，跳过
            i++;
            continue;
        }

        // === 发现 Host 块 ===
        const startLine = i;
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
                else if (k === "identityfile") entry.key = normalizePath(v);
            }
            i++;
        }

        entry._start = startLine;
        entry._end = i;  // 行范围 [start, end)
        hosts.push(entry);
    }

    return hosts;
}

/**
 * 将 host 对象渲染为 SSH config Host 块文本
 *
 * @param {Object} h - { alias, host, user, port, key }
 * @returns {string} 格式化的 SSH config 文本块
 */
function renderHostBlock(h) {
    const lines = [];
    lines.push(`Host ${h.alias}`);
    if (h.host) lines.push(`    HostName ${h.host}`);
    if (h.user) lines.push(`    User ${h.user}`);
    lines.push(`    Port ${h.port || 22}`);
    if (h.key)  lines.push(`    IdentityFile ${normalizePath(h.key)}`);
    return lines.join("\n");
}

// ============================================================
// 数据结构（对外保持兼容）
// ============================================================

/**
 * 确保 ~/.ssh/config 存在
 */
function ensureConfig() {
    const dir = path.dirname(SSH_CONFIG_PATH);
    if (!fs.existsSync(dir)) {
        fs.mkdirSync(dir, { recursive: true });
    }
    if (!fs.existsSync(SSH_CONFIG_PATH)) {
        fs.writeFileSync(SSH_CONFIG_PATH, "", "utf-8");
    }
}

/**
 * 从 ~/.ssh/config 读取所有 Host 配置
 *
 * @returns {Array<{ alias, host, user, port, key }>}
 *   返回与旧版 JSON 完全兼容的对象数组
 */
function loadHosts() {
    ensureConfig();
    try {
        const content = fs.readFileSync(SSH_CONFIG_PATH, "utf-8");
        const blocks = parseHostBlocks(content);
        // 去掉内部字段 _start/_end，对外暴露纯净对象
        return blocks.map(({ _start, _end, ...rest }) => rest);
    } catch {
        return [];
    }
}

/**
 * 保存全部 Host 配置到 ~/.ssh/config
 *
 * 策略：保留文件中所有非 Quick-SSH 管理的内容（注释、全局选项、非匹配的 Host 块），
 * 只替换别名匹配的 Host 块，新增不存在的 Host 块。
 *
 * @param {Array<{ alias, host, user, port, key }>} newHosts
 */
function saveHosts(newHosts) {
    ensureConfig();

    const oldContent = fs.readFileSync(SSH_CONFIG_PATH, "utf-8");
    const oldHosts = parseHostBlocks(oldContent);
    const lines = oldContent.split("\n");

    // 建立别名 → 新数据的快速查找
    const newMap = {};
    for (const h of newHosts) {
        newMap[h.alias] = h;
    }

    // 标记哪些旧 Host 块已被替换
    const replaced = new Set();

    // 构建结果行
    const result = [];

    // 遍历旧文件，按行重建
    let i = 0;
    while (i < lines.length) {
        const trimmed = lines[i].trim();
        const hostMatch = trimmed.match(/^Host\s+(.+)$/i);

        if (hostMatch) {
            const alias = hostMatch[1].trim();

            if (newMap[alias]) {
                // 这个 Host 块被 Quick-SSH 管理 → 用新数据替换
                // 跳过旧块的所有行
                const oldBlock = oldHosts.find(h => h.alias === alias);
                if (oldBlock) {
                    i = oldBlock._end;
                } else {
                    i++;
                    while (i < lines.length) {
                        const next = lines[i].trim();
                        if (!next || next.startsWith("#") || /^Host\s+/i.test(next)) break;
                        i++;
                    }
                }
                // 写入新块
                result.push(renderHostBlock(newMap[alias]));
                result.push("");
                replaced.add(alias);
            } else {
                // 非管理的 Host 块 → 原样保留
                result.push(lines[i]);
                i++;
                while (i < lines.length) {
                    const next = lines[i].trim();
                    if (!next || next.startsWith("#") || /^Host\s+/i.test(next)) break;
                    result.push(lines[i]);
                    i++;
                }
            }
        } else {
            // 非 Host 行 → 原样保留（注释、空行、全局选项）
            result.push(lines[i]);
            i++;
        }
    }

    // 追加全新的 Host 块（旧文件中不存在的别名）
    for (const h of newHosts) {
        if (!replaced.has(h.alias)) {
            result.push(renderHostBlock(h));
            result.push("");
        }
    }

    // 去除末尾多余换行
    let output = result.join("\n");
    // 确保文件以换行结尾
    if (output.length > 0 && !output.endsWith("\n")) {
        output += "\n";
    }

    fs.writeFileSync(SSH_CONFIG_PATH, output, "utf-8");
}

module.exports = {
    SSH_CONFIG_PATH,
    ensureConfig,
    loadHosts,
    saveHosts,
};
