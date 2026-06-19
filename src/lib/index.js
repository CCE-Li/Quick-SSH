/**
 * src/lib/index.js - Quick-SSH npm 生命周期钩子
 *
 * 跨平台 Shell 配置文件注入器：
 *   - Windows  → 写入 PowerShell $PROFILE (PowerShell 7 / Windows PowerShell 5.1)
 *   - Linux    → 写入 ~/.bashrc / ~/.zshrc 等 (自动检测 SHELL 环境变量)
 *   - macOS    → 写入 ~/.zshrc (macOS Catalina+ 默认 shell 为 zsh) / ~/.bashrc
 *
 * 工作原理 (按平台):
 *   ┌──────────┬──────────────────────┬──────────────────────────┐
 *   │ 平台     │ 注入方式             │ 运行时后端               │
 *   ├──────────┼──────────────────────┼──────────────────────────────────┤
 *   │ Windows  │ PowerShell $PROFILE  │ src/win/Quick-SSH.psm1 (原生)    │
 *   │ Linux    │ ~/.bashrc / ~/.zshrc │ src/unix/cli.js (Node.js)        │
 *   │ macOS    │ ~/.zshrc / ~/.bashrc │ src/unix/cli.js (Node.js)        │
 *   └──────────┴──────────────────────┴──────────────────────────────────┘
 *
 * 生命周期:
 *   postinstall  : 安装后将 shell 包装函数注入配置文件
 *   preuninstall : 卸载前从配置文件中移除注入的代码块，保留用户数据
 */

const fs   = require("fs");
const path = require("path");
const os   = require("os");

// ============================================================
// 常量定义
// ============================================================

const IMPORT_MARKER_START = "# >>> Quick-SSH auto-generated (do not modify) >>>";
const IMPORT_MARKER_END   = "# <<< Quick-SSH auto-generated <<<";

// ============================================================
// 平台 / Shell 检测
// ============================================================

/**
 * 检测当前操作系统
 * @returns {"windows" | "linux" | "macos" | "unknown"}
 */
function detectOS() {
    const p = process.platform;
    if (p === "win32") return "windows";
    if (p === "linux") return "linux";
    if (p === "darwin") return "macos";
    return "unknown";
}

/**
 * 检测当前用户的 Shell 类型
 * 仅在 Linux/macOS 下有效，Windows 返回 "powershell"
 * @returns {"bash" | "zsh" | "fish" | "sh" | "powershell" | "unknown"}
 */
function detectShell() {
    if (detectOS() === "windows") return "powershell";

    const shell = (process.env.SHELL || "").toLowerCase().split("/").pop();
    if (shell.includes("zsh"))      return "zsh";
    if (shell.includes("bash"))     return "bash";
    if (shell.includes("fish"))     return "fish";
    if (shell === "sh")            return "sh";
    return "unknown";
}

/**
 * 获取用户 home 目录（跨平台兼容）
 */
function getHomeDir() {
    return process.env.HOME || process.env.USERPROFILE || os.homedir();
}

// ============================================================
// 配置文件路径解析
// ============================================================

/**
 * 获取 PowerShell 配置文件路径列表（仅 Windows）
 * 同时兼容 PowerShell 7+ 和 Windows PowerShell 5.1-
 * 注意：部分用户的 Documents 被重定向到 OneDrive，必须同时检测
 */
// TODO: OneDrive用户暂未测试成功
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
 * 返回 [ps7Paths, winPsPaths]，每组按 "已存在优先" 排序
 */
function resolveProfilePaths(candidates) {
    const ps7     = [];  // PowerShell 7
    const winPs   = [];  // Windows PowerShell 5.1-
    // 注意：前面必须带路径分隔符，避免 "WindowsPowerShell" 误匹配 "PowerShell"
    const ps7Key  = "\\PowerShell\\Microsoft.PowerShell_profile.ps1";
    const winKey  = "\\WindowsPowerShell\\Microsoft.PowerShell_profile.ps1";

    for (const p of candidates) {
        const norm = p.replace(/\\/g, "/");
        const ps7Match  = ps7Key.replace(/\\/g, "/");
        const winMatch  = winKey.replace(/\\/g, "/");
        if (norm.endsWith(ps7Match)) {
            ps7.push(p);
        } else if (norm.endsWith(winMatch)) {
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
 * 根据 Shell 类型获取对应的 rc 文件路径（Linux/macOS）
 * @param {"bash"|"zsh"|"fish"|"sh"|"unknown"} shell
 * @returns {string} 配置文件路径
 * @returns {null} 不支持的 shell 类型
 */
function getShellProfilePath(shell) {
    const home = getHomeDir();
    switch (shell) {
        case "zsh":  return path.join(home, ".zshrc");
        case "bash": return path.join(home, ".bashrc");
        case "fish": return path.join(home, ".config", "fish", "config.fish");
        case "sh":   return path.join(home, ".profile");
        default:     return null;
    }
}

// ============================================================
// 注入代码块构建
// ============================================================
// TODO: 目前只能依靠注入实现，先办法写到环境变量中？


/**
 * 构建 PowerShell Import-Module 语句块（Windows）
 */
function buildPowerShellImportBlock(modulePath) {
    const lines = [
        IMPORT_MARKER_START,
        "# Quick-SSH PowerShell SSH 连接管理工具",
        `# 安装路径: ${modulePath}`,
        `if (Test-Path "${modulePath}") { Import-Module "${modulePath}" -DisableNameChecking }`,
        IMPORT_MARKER_END,
    ];
    return lines.join("\n");
}

/**
 * 构建 Shell 包装函数块（Linux/macOS）
 *
 * 原理：
 *   在 bash/zsh 中定义一个 qssh() 函数，通过 Node.js 直接调用
 *   src/unix/cli.js 实现所有功能，完全不需要 PowerShell。
 *
 *   如果 Node.js 不可用，则跳过定义，不污染 shell 环境。
 */
function buildShellImportBlock(cliPath) {
    const escapedPath = cliPath.replace(/'/g, "'\\''");
    const lines = [
        IMPORT_MARKER_START,
        "# Quick-SSH SSH 连接管理工具 (Node.js CLI)",
        `# 安装路径: ${escapedPath}`,
        `if command -v node &> /dev/null && [ -f '${escapedPath}' ]; then`,
        `    qssh() {`,
        `        node "${escapedPath}" "$@"`,
        `    }`,
        `fi`,
        IMPORT_MARKER_END,
    ];
    return lines.join("\n");
}

/**
 * 获取当前包中 Quick-SSH.psm1 的路径（仅 Windows 使用）
 * 本文件位于 src/lib/ 目录，因此需要上两级到包根目录，再到 src/win/Quick-SSH.psm1
 */
function getModulePath() {
    return path.join(__dirname, "..", "..", "src", "win", "Quick-SSH.psm1");
}

/**
 * 获取当前包中 src/unix/cli.js 的路径（Linux/macOS 使用）
 * 本文件位于 src/lib/ 目录，因此需要上两级到包根目录，再到 src/unix/cli.js
 */
function getCLIPath() {
    return path.join(__dirname, "..", "..", "src", "unix", "cli.js");
}

// ============================================================
// 配置文件读写工具
// ============================================================

/**
 * 通用：将注入块写入单个配置文件
 * @param {string} profilePath - 配置文件路径
 * @param {string} importBlock - 要写入的代码块
 * @param {string} platform    - "windows" | "unix" (影响换行符)
 */
function writeImportBlock(profilePath, importBlock, platform) {
    const eol = platform === "windows" ? "\r\n" : "\n";

    // 确保目录存在
    const profileDir = path.dirname(profilePath);
    if (!fs.existsSync(profileDir)) {
        fs.mkdirSync(profileDir, { recursive: true });
        console.log(`[Quick-SSH] ✔ 已创建目录: ${profileDir}`);
    }

    // 读取现有内容（文件可能不存在）
    let content = "";
    if (fs.existsSync(profilePath)) {
        content = fs.readFileSync(profilePath, "utf-8");
    }

    const markerStart = IMPORT_MARKER_START;

    // 检查是否已注册，去重处理
    if (content.includes(markerStart)) {
        // 已存在则替换旧块
        const regex = new RegExp(
            `${escapeRegExp(markerStart)}[\\s\\S]*?${escapeRegExp(IMPORT_MARKER_END)}`,
            "g"
        );
        content = content.replace(regex, importBlock);
        console.log(`[Quick-SSH] 🔄 已更新: ${profilePath}`);
    } else {
        // 追加到文件末尾
        content = content.trimEnd() + eol + eol + importBlock + eol;
        console.log(`[Quick-SSH] ➕ 已写入: ${profilePath}`);
    }

    fs.writeFileSync(profilePath, content, "utf-8");
}

/**
 * 通用：从配置文件中移除 Quick-SSH 注入块
 * @param {string} profilePath
 * @returns {boolean} true=有改动, false=无改动
 */
function removeImportBlock(profilePath) {
    if (!fs.existsSync(profilePath)) {
        console.log(`[Quick-SSH] - 跳过(不存在): ${profilePath}`);
        return false;
    }

    const regex = new RegExp(
        `\\s*${escapeRegExp(IMPORT_MARKER_START)}[\\s\\S]*?${escapeRegExp(IMPORT_MARKER_END)}\\s*`,
        "g"
    );

    let content = fs.readFileSync(profilePath, "utf-8");

    if (regex.test(content)) {
        content = content.replace(regex, "");
        // 清理多余空行
        content = content.replace(/\n{3,}/g, "\n\n").trimEnd() + "\n";
        fs.writeFileSync(profilePath, content, "utf-8");
        console.log(`[Quick-SSH] ✔ 已清理: ${profilePath}`);
        return true;
    }

    console.log(`[Quick-SSH] - 无配置: ${profilePath}`);
    return false;
}

// ============================================================
// postinstall - 安装后自动注册模块
// ============================================================

function runPostInstall() {
    const osType = detectOS();
    console.log(`[Quick-SSH] 检测到操作系统: ${osType}`);

    if (osType === "windows") {
        // ─── Windows: 注入 PowerShell $PROFILE ───
        const modulePath = getModulePath();
        if (!fs.existsSync(modulePath)) {
            console.error(`[Quick-SSH] ❌ 未找到模块文件: ${modulePath}`);
            process.exit(1);
        }
        console.log(`[Quick-SSH] 模块路径: ${modulePath}`);
        console.log("[Quick-SSH] 正在配置 PowerShell 自动加载...");

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

        const importBlock = buildPowerShellImportBlock(modulePath);

        for (const profilePath of profilePaths) {
            writeImportBlock(profilePath, importBlock, "windows");
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
    } else {
        // ─── Linux / macOS: 注入 Shell rc 文件 ───
        // 使用 Node.js CLI (src/unix/cli.js)，不依赖 PowerShell
        const cliPath = getCLIPath();
        if (!fs.existsSync(cliPath)) {
            console.error(`[Quick-SSH] ❌ 未找到 CLI 脚本: ${cliPath}`);
            process.exit(1);
        }
        console.log(`[Quick-SSH] CLI 路径: ${cliPath}`);

        const shell = detectShell();
        console.log(`[Quick-SSH] 检测到 Shell: ${shell}`);

        const profilePath = getShellProfilePath(shell);
        if (!profilePath) {
            console.error(`[Quick-SSH] ❌ 不支持的 Shell 类型: "${shell}"。`);
            console.error(`[Quick-SSH]    当前 SHELL=${process.env.SHELL || "(未设置)"}`);
            console.error(`[Quick-SSH]    请手动将以下内容添加到您的 Shell 配置文件中:`);
            console.error("");
            console.error(buildShellImportBlock(cliPath));
            process.exit(1);
        }

        console.log(`[Quick-SSH] 配置文件路径: ${profilePath}`);

        const importBlock = buildShellImportBlock(cliPath);
        writeImportBlock(profilePath, importBlock, "unix");

        console.log("[Quick-SSH] ✔ 安装完成！请重启终端或执行:");
        console.log(`[Quick-SSH]    source ${profilePath}`);
    }
}

// ============================================================
// preuninstall - 卸载前清理注册（保留用户数据）
// ============================================================
// TODO: 卸载时没有完全删除$PROFILE、$Shell中的Quick-SSH相关配置
function runPreUninstall() {
    const osType = detectOS();
    console.log(`[Quick-SSH] 检测到操作系统: ${osType}`);
    console.log("[Quick-SSH] 正在清理配置文件...");

    if (osType === "windows") {
        // ─── Windows: 清理 PowerShell $PROFILE ───
        const profilePaths = getPowerShellProfilePaths();
        if (profilePaths.length === 0) {
            console.log("[Quick-SSH] ✔ 未找到 PowerShell 配置文件，无需清理。");
            return;
        }

        for (const profilePath of profilePaths) {
            removeImportBlock(profilePath);
        }
    } else {
        // ─── Linux / macOS: 清理 Shell rc 文件 ───
        const shell = detectShell();
        const profilePath = getShellProfilePath(shell);

        if (profilePath) {
            removeImportBlock(profilePath);
        } else {
            console.log(`[Quick-SSH] - 无法确定 Shell 配置文件路径 (SHELL=${process.env.SHELL || "未设置"})`);
        }

        // 同时也清扫一下其他常见的 rc 文件，防止用户切换 Shell 后残留
        const home = getHomeDir();
        const extraPaths = [
            path.join(home, ".bashrc"),
            path.join(home, ".zshrc"),
            path.join(home, ".profile"),
            path.join(home, ".bash_profile"),
            path.join(home, ".zprofile"),
            path.join(home, ".config", "fish", "config.fish"),
        ];
        for (const extraPath of extraPaths) {
            if (extraPath !== profilePath) {
                removeImportBlock(extraPath);
            }
        }
    }

    console.log("[Quick-SSH] ✔ 清理完成！用户配置数据已保留 (~/.ssh/config)。");
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
