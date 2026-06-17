/**
 * data.js - Quick-SSH TUI 数据层
 *
 * 负责配置文件的读取、保存和初始化。
 * 纯数据操作，不依赖任何 UI 组件。
 */

const fs   = require("fs");
const path = require("path");

// ============================================================
// 配置路径
// ============================================================

const CONFIG_DIR  = path.join(process.env.USERPROFILE || "~", ".quickssh");
const CONFIG_FILE = path.join(CONFIG_DIR, "hosts.json");

// ============================================================
// 数据操作
// ============================================================

/**
 * 确保配置目录和文件存在
 */
function ensureConfig() {
    if (!fs.existsSync(CONFIG_DIR))  fs.mkdirSync(CONFIG_DIR, { recursive: true });
    if (!fs.existsSync(CONFIG_FILE)) fs.writeFileSync(CONFIG_FILE, "[]", "utf-8");
}

/**
 * 读取全部主机配置
 * 兼容单个对象 { ... } 和数组 [{ ... }, { ... }] 两种格式
 */
function loadHosts() {
    ensureConfig();
    try {
        const raw = fs.readFileSync(CONFIG_FILE, "utf-8").trim();
        if (!raw) return [];
        const data = JSON.parse(raw);
        return Array.isArray(data) ? data : [data];
    } catch {
        return [];
    }
}

/**
 * 保存全部主机配置（始终保存为数组格式）
 */
function saveHosts(hosts) {
    ensureConfig();
    const data = Array.isArray(hosts) ? hosts : [];
    fs.writeFileSync(CONFIG_FILE, JSON.stringify(data, null, 4), "utf-8");
}

module.exports = { CONFIG_DIR, CONFIG_FILE, ensureConfig, loadHosts, saveHosts };
