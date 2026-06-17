/**
 * index.js - Quick-SSH npm 生命周期钩子
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
 */
function getPowerShellProfilePaths() {
    const userProfile = process.env.USERPROFILE;
    if (!userProfile) {
        console.error("[Quick-SSH] 错误：无法获取 %USERPROFILE% 环境变量。");
        return [];
    }
    return [
        path.join(userProfile, "Documents", "PowerShell", "Microsoft.PowerShell_profile.ps1"),
        path.join(userProfile, "Documents", "WindowsPowerShell", "Microsoft.PowerShell_profile.ps1"),
    ];
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
 */
function getModulePath() {
    // 在 npm 全局或本地安装时，index.js 位于包根目录
    return path.join(__dirname, "Quick-SSH.psm1");
}

// ============================================================
// postinstall - 安装后自动注册模块
// ============================================================

function runPostInstall() {
    console.log("[Quick-SSH] 正在配置 PowerShell 自动加载...");

    const profilePaths = getPowerShellProfilePaths();
    if (profilePaths.length === 0) {
        console.error("[Quick-SSH] ❌ 无法定位 PowerShell 配置文件。");
        process.exit(1);
    }

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

    console.log("[Quick-SSH] ✔ 安装完成！请重启 PowerShell 终端或执行:");
    console.log(`[Quick-SSH]    & (Get-Content "\${env:USERPROFILE}\\Documents\\PowerShell\\Microsoft.PowerShell_profile.ps1" -Raw) | Invoke-Expression`);
    console.log(`[Quick-SSH]    或 Windows PowerShell:`);
    console.log(`[Quick-SSH]    & (Get-Content "\${env:USERPROFILE}\\Documents\\WindowsPowerShell\\Microsoft.PowerShell_profile.ps1" -Raw) | Invoke-Expression`);
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
        console.log(`[Quick-SSH] index.js - Quick-SSH npm 生命周期脚本`);
        console.log(`  用法: node index.js <postinstall|preuninstall>`);
        break;
}
