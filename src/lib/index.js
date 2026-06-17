/**
 * src/lib/index.js - Quick-SSH npm 生命周期钩子
 *
 * postinstall : 将 Import-Module 语句写入 PowerShell 配置文件，重启终端自动加载
 * preuninstall: 从 PowerShell 配置文件中移除 Import-Module 语句，保留用户数据
 */

const fs   = require("fs");
const path = require("path");

// ============================================================
// 常量定义
// ============================================================

const IMPORT_MARKER_START = "# >>> Quick-SSH auto-generated (do not modify) >>>";
const IMPORT_MARKER_END   = "# <<< Quick-SSH auto-generated <<<";

/**
 * 获取 PowerShell 配置文件路径列表
 *  同时兼容 PowerShell 7+ 和 Windows PowerShell 5.1-
 *
 *  注意：部分用户的 Documents 被重定向到 OneDrive，必须同时检测
 *  OneDrive 目录下的配置文件路径，否则 postinstall 可能写到错误位置。
 */
function getPowerShellProfilePaths() {
    const userProfile = process.env.USERPROFILE;
    if (!userProfile) {
        console.error("[Quick-SSH] 错误：无法获取 %USERPROFILE% 环境变量。");
        return [];
    }

    /** 生成 PS7 和 Windows PowerShell 的配置文件路径 */
    function makePaths(docsRoot) {
        return [
            path.join(docsRoot, "PowerShell", "Microsoft.PowerShell_profile.ps1"),
            path.join(docsRoot, "WindowsPowerShell", "Microsoft.PowerShell_profile.ps1"),
        ];
    }

    const pathSet = new Set();

    // 1) 标准路径：%USERPROFILE%\Documents\...
    pathSet.add(path.join(userProfile, "Documents", "PowerShell", "Microsoft.PowerShell_profile.ps1"));
    pathSet.add(path.join(userProfile, "Documents", "WindowsPowerShell", "Microsoft.PowerShell_profile.ps1"));

    // 2) OneDrive 路径（如果已配置）
    const oneDriveVars = ["OneDrive", "OneDriveConsumer"];
    for (const envVar of oneDriveVars) {
        const root = process.env[envVar];
        if (root && typeof root === "string" && root.length > 0) {
            const docs = path.join(root, "Documents");
            if (docs !== path.join(userProfile, "Documents")) {
                for (const p of makePaths(docs)) pathSet.add(p);
            }
        }
    }

    return Array.from(pathSet);
}

/**
 * 从候选路径列表中筛选出实际存在的配置文件（按 PS7 / Windows PowerShell 分组）
 *  返回 [ps7Paths, winPsPaths]，每组按 "已存在优先" 排序
 */
function resolveProfilePaths(candidates) {
    const ps7     = [];  // PowerShell 7
    const winPs   = [];  // Windows PowerShell 5.1-
    const ps7Key  = "PowerShell\\Microsoft.PowerShell_profile.ps1";
    const winKey  = "WindowsPowerShell\\Microsoft.PowerShell_profile.ps1";

    for (const p of candidates) {
        const norm = p.replace(/\\/g, "/");
        if (norm.endsWith(ps7Key.replace(/\\/g, "/"))) {
            ps7.push(p);
        } else if (norm.endsWith(winKey.replace(/\\/g, "/"))) {
            winPs.push(p);
        }
    }

    // 按 "已存在 → 不存在" 排序
    const sorter = (a, b) => (fs.existsSync(b) ? 1 : 0) - (fs.existsSync(a) ? 1 : 0);
    ps7.sort(sorter);
    winPs.sort(sorter);

    return [ps7, winPs];
}

/**
 * 构建 Import-Module 语句块
 */
function buildImportBlock(modulePath) {
    const lines = [
        IMPORT_MARKER_START,
        `# Quick-SSH PowerShell SSH 连接管理工具`,
        `# 安装路径: ${modulePath}`,
        `if (Test-Path "${modulePath}") { Import-Module "${modulePath}" -DisableNameChecking }`,
        IMPORT_MARKER_END,
    ];
    return lines.join("\n");
}

/**
 * 获取当前包中 Quick-SSH.psm1 的路径
 * 本文件位于 src/lib/ 目录，因此需要上两级到包根目录，再到 src/Quick-SSH.psm1
 */
function getModulePath() {
    return path.join(__dirname, "..", "..", "src", "Quick-SSH.psm1");
}

// ============================================================
// postinstall - 安装后自动注册模块
// ============================================================

function runPostInstall() {
    console.log("[Quick-SSH] 正在配置 PowerShell 自动加载...");

    // 收集所有候选路径并按 "已存在优先" 排序
    const candidates  = getPowerShellProfilePaths();
    if (candidates.length === 0) {
        console.error("[Quick-SSH] ❌ 无法定位 PowerShell 配置文件。");
        process.exit(1);
    }

    const [ps7Paths, winPsPaths] = resolveProfilePaths(candidates);

    // 各取第一个（即已存在的路径优先，不存在则退回到标准路径）
    const profilePaths = [];
    if (ps7Paths.length > 0)   profilePaths.push(ps7Paths[0]);
    if (winPsPaths.length > 0) profilePaths.push(winPsPaths[0]);

    const modulePath  = getModulePath();
    if (!fs.existsSync(modulePath)) {
        console.error(`[Quick-SSH] ❌ 未找到模块文件: ${modulePath}`);
        process.exit(1);
    }

    const importBlock = buildImportBlock(modulePath);

    for (const profilePath of profilePaths) {
        let content = "";

        // 读取现有配置
        if (fs.existsSync(profilePath)) {
            content = fs.readFileSync(profilePath, "utf-8");
        }

        // 检查是否已注册，避免重复插入
        if (content.includes(IMPORT_MARKER_START)) {
            // 已存在则替换旧块
            const regex = new RegExp(
                `${escapeRegExp(IMPORT_MARKER_START)}[\\s\\S]*?${escapeRegExp(IMPORT_MARKER_END)}`,
                "g"
            );
            if (regex.test(content)) {
                content = content.replace(regex, importBlock);
                console.log(`[Quick-SSH] 🔄 已更新: ${profilePath}`);
            }
        } else {
            // 追加到文件末尾
            content = content.trimEnd() + "\n\n" + importBlock + "\n";
            console.log(`[Quick-SSH] ➕ 已写入: ${profilePath}`);
        }

        // 确保目录存在
        const dir = path.dirname(profilePath);
        if (!fs.existsSync(dir)) {
            fs.mkdirSync(dir, { recursive: true });
        }

        fs.writeFileSync(profilePath, content, "utf-8");
    }

    // 打印友好的提示信息
    const relPath = (idx) => {
        const p = profilePaths[idx];
        if (!p) return "";
        const parts = p.split(path.sep);
        const docsIdx = parts.findIndex(s => /^documents$/i.test(s));
        if (docsIdx !== -1 && docsIdx + 2 < parts.length) {
            return "..." + path.sep + parts.slice(docsIdx).join(path.sep);
        }
        return p;
    };

    console.log("[Quick-SSH] ✔ 安装完成！请重启 PowerShell 终端或执行:");
    if (ps7Paths.length > 0) {
        console.log(`[Quick-SSH]    PowerShell 7:   & (Get-Content "${relPath(0)}" -Raw) | Invoke-Expression`);
    }
    if (winPsPaths.length > 0) {
        const idx = ps7Paths.length > 0 ? 1 : 0;
        console.log(`[Quick-SSH]    Windows PowerShell: & (Get-Content "${relPath(idx)}" -Raw) | Invoke-Expression`);
    }
}

// ============================================================
// preuninstall - 卸载前清理注册（保留用户数据）
// ============================================================

function runPreUninstall() {
    console.log("[Quick-SSH] 正在清理 PowerShell 配置文件...");

    const profilePaths = getPowerShellProfilePaths();
    if (profilePaths.length === 0) {
        console.log("[Quick-SSH] ✔ 未找到 PowerShell 配置文件，无需清理。");
        return;
    }

    const regex = new RegExp(
        `\\s*${escapeRegExp(IMPORT_MARKER_START)}[\\s\\S]*?${escapeRegExp(IMPORT_MARKER_END)}\\s*`,
        "g"
    );

    for (const profilePath of profilePaths) {
        if (!fs.existsSync(profilePath)) {
            console.log(`[Quick-SSH] - 跳过(不存在): ${profilePath}`);
            continue;
        }

        let content = fs.readFileSync(profilePath, "utf-8");

        if (regex.test(content)) {
            content = content.replace(regex, "");
            // 清理多余空行
            content = content.replace(/\n{3,}/g, "\n\n").trimEnd() + "\n";
            fs.writeFileSync(profilePath, content, "utf-8");
            console.log(`[Quick-SSH] ✔ 已清理: ${profilePath}`);
        } else {
            console.log(`[Quick-SSH] - 无配置: ${profilePath}`);
        }
    }

    console.log("[Quick-SSH] ✔ 清理完成！用户配置数据已保留 (%USERPROFILE%\\.quickssh\\hosts.json)。");
}

// ============================================================
// 工具函数
// ============================================================

function escapeRegExp(str) {
    return str.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

// ============================================================
// 入口
// ============================================================

const command = process.argv[2];

switch (command) {
    case "postinstall":
        runPostInstall();
        break;
    case "preuninstall":
        runPreUninstall();
        break;
    default:
        console.log(`[Quick-SSH] src/lib/index.js - Quick-SSH npm 生命周期脚本`);
        console.log(`  用法: node src/lib/index.js <postinstall|preuninstall>`);
        break;
}
