/**
 * src/lib/index.js - Quick-SSH npm 生命周期钩子（二进制版本）
 *
 * 职责:
 *   安装后将 Quick-SSH 二进制文件所在目录写入环境变量 PATH，
 *   而不是以前那样注入脚本包装函数（Import-Module / qssh() shell 函数）。
 *
 * 工作原理:
 *   - 检测二进制文件位置（dist/ 或全局 node_modules/.bin/）
 *   - 将该目录添加到用户 PATH 环境变量
 *   - Windows: 通过 PowerShell $PROFILE 添加（setx 方案做备选）
 *   - Linux/macOS: 通过 ~/.bashrc / ~/.zshrc 添加
 *
 * 为什么改:
 *   以前用脚本包装（shell function / Import-Module），需要 Node.js 依赖。
 *   现在使用 pkg 编译为原生二进制 (qssh.exe / qssh)，无需 Node.js 即可运行。
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
 * 仅在 Linux/macOS 下有效
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
// 二进制路径检测
// ============================================================

/**
 * 获取 Quick-SSH 二进制文件的预期安装路径
 *
 * 查找顺序:
 *   1. dist/bin/ 下当前平台的 qssh-* 二进制（发布包）
 *   2. 全局 node_modules 下 quick-ssh/dist/bin/qssh-* 二进制
 *   3. 全局 node_modules/.bin/qssh (npm 链接)
 *
 * @returns {{ binDir: string, binName: string } | null}
 */
function getBinaryInfo() {
    const osType = detectOS();

    // 平台区分二进制名称（与 build.js PLATFORM_CONFIG 一致）
    const platformMap = {
        windows: "qssh-win.exe",
        linux:   "qssh-linux",
        macos:   "qssh-darwin",
    };
    const binName = platformMap[osType] || "qssh";

    // 候选路径列表
    const candidates = [];

    // 1) dist/bin/（发布包的二进制目录，优先）
    candidates.push(path.join(__dirname, "..", "..", "dist", "bin"));

    // 2) 本地源码 dist/ 目录（旧版兼容）
    candidates.push(path.join(__dirname, "..", "..", "dist"));

    // 3) 全局 node_modules 下
    const npmPrefix = process.env.NPM_CONFIG_PREFIX
        || (osType === "windows"
            ? path.join(process.env.APPDATA || "", "npm")
            : "/usr/local");
    candidates.push(path.join(npmPrefix, "lib", "node_modules", "quick-ssh", "dist"));
    candidates.push(path.join(npmPrefix, "node_modules", "quick-ssh", "dist"));

    // 3) node_modules/.bin
    const binCandidates = [
        path.join(__dirname, "..", "..", "node_modules", ".bin"),
        path.join(npmPrefix, "node_modules", ".bin"),
    ];

    // 先检查 bin 目录（npm 链接）
    for (const dir of binCandidates) {
        const fullPath = path.join(dir, binName);
        if (fs.existsSync(fullPath)) {
            return { binDir: dir, binName };
        }
    }

    // 再检查 dist 目录
    for (const dir of candidates) {
        const fullPath = path.join(dir, binName);
        if (fs.existsSync(fullPath)) {
            return { binDir: dir, binName };
        }
    }

    // 都找不到，用第一个候选路径（可能在 CI 构建中）
    return { binDir: candidates[0], binName };
}

// ============================================================
// PATH 注入块构建
// ============================================================

/**
 * 构建 PowerShell PATH 注入块（Windows）
 *
 * 将 binDir 添加到用户 PATH 的两种方式：
 *   A) $PROFILE 中永久添加（推荐）
 *   B) [Environment]::SetEnvironmentVariable 添加到 User PATH
 */
function buildPowerShellPathBlock(binDir) {
    const lines = [
        IMPORT_MARKER_START,
        "# Quick-SSH SSH 连接管理工具 - 二进制路径",
        `# 安装路径: ${binDir}`,
        `$quickSshBinDir = "${binDir.replace(/\\/g, "\\\\")}"`,
        `if (Test-Path $quickSshBinDir) {`,
        `    $currentPath = [Environment]::GetEnvironmentVariable("PATH", "User")`,
        `    if ($currentPath -notlike "*$quickSshBinDir*") {`,
        `        [Environment]::SetEnvironmentVariable("PATH", "$currentPath;$quickSshBinDir", "User")`,
        `        Write-Host "✔ Quick-SSH 已添加到用户 PATH" -ForegroundColor Green`,
        `    }`,
        `}`,
        IMPORT_MARKER_END,
    ];
    return lines.join("\r\n") + "\r\n";
}

/**
 * 构建 Shell PATH 注入块（Linux/macOS）
 *
 * 将 binDir 添加到 PATH 环境变量
 */
function buildShellPathBlock(binDir) {
    const escapedPath = binDir.replace(/'/g, "'\\''");
    const lines = [
        IMPORT_MARKER_START,
        "# Quick-SSH SSH 连接管理工具 - 二进制路径",
        `# 安装路径: ${escapedPath}`,
        `if [ -d '${escapedPath}' ]; then`,
        `    case ":$PATH:" in`,
        `        *:${escapedPath}:*) ;;`,
        `        *) export PATH="${escapedPath}:$PATH" ;;`,
        `    esac`,
        `fi`,
        IMPORT_MARKER_END,
    ];
    return lines.join("\n") + "\n";
}

// ============================================================
// 配置文件路径
// ============================================================

/**
 * 获取 PowerShell 配置文件路径列表
 */
function getPowerShellProfilePaths() {
    const userProfile = process.env.USERPROFILE;
    if (!userProfile) return [];

    const pathSet = new Set();
    pathSet.add(path.join(userProfile, "Documents", "PowerShell", "Microsoft.PowerShell_profile.ps1"));
    pathSet.add(path.join(userProfile, "Documents", "WindowsPowerShell", "Microsoft.PowerShell_profile.ps1"));

    const oneDriveVars = ["OneDrive", "OneDriveConsumer"];
    for (const envVar of oneDriveVars) {
        const root = process.env[envVar];
        if (root && typeof root === "string" && root.length > 0) {
            const docs = path.join(root, "Documents");
            if (docs !== path.join(userProfile, "Documents")) {
                pathSet.add(path.join(docs, "PowerShell", "Microsoft.PowerShell_profile.ps1"));
                pathSet.add(path.join(docs, "WindowsPowerShell", "Microsoft.PowerShell_profile.ps1"));
            }
        }
    }

    return Array.from(pathSet);
}

function resolveProfilePaths(candidates) {
    const ps7   = [];
    const winPs = [];
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

    const sorter = (a, b) => (fs.existsSync(b) ? 1 : 0) - (fs.existsSync(a) ? 1 : 0);
    ps7.sort(sorter);
    winPs.sort(sorter);

    return [ps7, winPs];
}

/**
 * 根据 Shell 类型获取对应的 rc 文件路径
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
// 配置文件读写工具
// ============================================================

function escapeRegExp(str) {
    return str.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function writeMarkerBlock(profilePath, markerBlock, platform) {
    const eol = platform === "windows" ? "\r\n" : "\n";

    const profileDir = path.dirname(profilePath);
    if (!fs.existsSync(profileDir)) {
        fs.mkdirSync(profileDir, { recursive: true });
        console.log(`[Quick-SSH] ✔ 已创建目录: ${profileDir}`);
    }

    let content = "";
    if (fs.existsSync(profilePath)) {
        content = fs.readFileSync(profilePath, "utf-8");
    }

    if (content.includes(IMPORT_MARKER_START)) {
        const regex = new RegExp(
            `${escapeRegExp(IMPORT_MARKER_START)}[\\s\\S]*?${escapeRegExp(IMPORT_MARKER_END)}`,
            "g"
        );
        content = content.replace(regex, markerBlock.trimEnd());
        console.log(`[Quick-SSH] 🔄 已更新: ${profilePath}`);
    } else {
        content = content.trimEnd() + eol + eol + markerBlock.trimEnd() + eol;
        console.log(`[Quick-SSH] ➕ 已写入: ${profilePath}`);
    }

    fs.writeFileSync(profilePath, content, "utf-8");
}

function removeMarkerBlock(profilePath) {
    if (!fs.existsSync(profilePath)) {
        return false;
    }

    const regex = new RegExp(
        `\\s*${escapeRegExp(IMPORT_MARKER_START)}[\\s\\S]*?${escapeRegExp(IMPORT_MARKER_END)}\\s*`,
        "g"
    );

    let content = fs.readFileSync(profilePath, "utf-8");

    if (regex.test(content)) {
        content = content.replace(regex, "");
        content = content.replace(/\n{3,}/g, "\n\n").trimEnd() + "\n";
        fs.writeFileSync(profilePath, content, "utf-8");
        console.log(`[Quick-SSH] ✔ 已清理 PATH 配置: ${profilePath}`);
        return true;
    }

    return false;
}

// ============================================================
// .qsshrc 默认配置生成
// ============================================================

function ensureQsshrc() {
    const configPath = path.join(getHomeDir(), ".qsshrc");
    if (fs.existsSync(configPath)) {
        console.log(`[Quick-SSH] - 已存在，跳过: ${configPath}`);
        return;
    }

    const defaultContent = [
        "# Quick-SSH 配置文件",
        "#",
        "# UploadConcurrency: 拖拽上传的并发数（默认: 3，必须为大于 0 的整数）",
        "# UploadInNewWindow:  是否在新窗口打开上传窗口（默认: true）",
        "#",
        "UploadConcurrency=3",
        "UploadInNewWindow=true",
        "",
    ].join("\n");

    try {
        fs.writeFileSync(configPath, defaultContent, "utf-8");
        console.log(`[Quick-SSH] ✔ 已创建默认配置文件: ${configPath}`);
    } catch (err) {
        console.error(`[Quick-SSH] ⚠ 创建配置文件失败: ${configPath}`, err.message);
    }
}

// ============================================================
// postinstall - 安装后将二进制路径添加到 PATH
// ============================================================

function runPostInstall() {
    const osType = detectOS();
    console.log(`[Quick-SSH] 检测到操作系统: ${osType}`);

    // 确保 ~/.qsshrc 存在
    ensureQsshrc();

    // 获取二进制路径信息
    const binInfo = getBinaryInfo();
    if (!binInfo) {
        console.error("[Quick-SSH] ❌ 无法定位 Quick-SSH 二进制文件。");
        console.error("[Quick-SSH]    如果是从源码安装，请先运行: npm run build");
        process.exit(1);
    }

    const binDir = binInfo.binDir;
    const binName = binInfo.binName;
    const binPath = path.join(binDir, binName);

    if (!fs.existsSync(binPath)) {
        console.error(`[Quick-SSH] ❌ 未找到二进制文件: ${binPath}`);
        console.error(`[Quick-SSH]    请先运行: npm run build`);
        process.exit(1);
    }

    console.log(`[Quick-SSH] ✔ 检测到二进制: ${binPath}`);

    if (osType === "windows") {
        // ─── Windows: 添加到用户 PATH 环境变量 ───
        console.log("[Quick-SSH] 正在将 Quick-SSH 添加到用户 PATH...");

        // 方式 A: 通过 PowerShell $PROFILE 添加
        const candidates = getPowerShellProfilePaths();
        if (candidates.length > 0) {
            const [ps7Paths, winPsPaths] = resolveProfilePaths(candidates);
            const profilePaths = [];
            if (ps7Paths.length > 0)   profilePaths.push(ps7Paths[0]);
            if (winPsPaths.length > 0) profilePaths.push(winPsPaths[0]);

            const pathBlock = buildPowerShellPathBlock(binDir);
            for (const profilePath of profilePaths) {
                writeMarkerBlock(profilePath, pathBlock, "windows");
            }
        }

        // 方式 B: 同时通过 setx 设置（全局生效，无需重启终端）
        try {
            const execSync = require("child_process").execSync;
            const currentPath = execSync(
                `echo %PATH%`, { shell: "cmd.exe", encoding: "utf-8" }
            ).trim();
            if (!currentPath.includes(binDir)) {
                execSync(
                    `setx PATH "${binDir};%PATH%"`,
                    { shell: "cmd.exe", stdio: "pipe" }
                );
                console.log(`[Quick-SSH] ✔ 已通过 setx 将 ${binDir} 添加到系统 PATH`);
            } else {
                console.log(`[Quick-SSH] - ${binDir} 已在 PATH 中`);
            }
        } catch (e) {
            console.log(`[Quick-SSH] ⚠ setx 设置失败（$PROFILE 方式已生效）: ${e.message}`);
        }

        console.log(`[Quick-SSH] ✔ 安装完成！`);
        console.log(`[Quick-SSH]    ${binName} 已添加到 PATH。`);
        console.log(`[Quick-SSH]    Windows 用户请重启终端或执行: $PROFILE`);

    } else {
        // ─── Linux / macOS: 添加到 Shell rc 文件 ───
        console.log("[Quick-SSH] 正在将 Quick-SSH 添加到 Shell 配置文件...");

        const shell = detectShell();
        console.log(`[Quick-SSH] 检测到 Shell: ${shell}`);

        const profilePath = getShellProfilePath(shell);
        if (!profilePath) {
            console.error(`[Quick-SSH] ❌ 不支持的 Shell 类型: "${shell}"。`);
            console.error(`[Quick-SSH]    请手动将以下内容添加到您的 Shell 配置文件中:`);
            console.error(`[Quick-SSH]    export PATH="${binDir}:$PATH"`);
            process.exit(1);
        }

        console.log(`[Quick-SSH] 配置文件路径: ${profilePath}`);

        const pathBlock = buildShellPathBlock(binDir);
        writeMarkerBlock(profilePath, pathBlock, "unix");

        console.log(`[Quick-SSH] ✔ 安装完成！请重启终端或执行:`);
        console.log(`[Quick-SSH]    export PATH="${binDir}:$PATH"`);
    }
}

// ============================================================
// preuninstall - 卸载前从 PATH 中移除
// ============================================================

function runPreUninstall() {
    const osType = detectOS();
    console.log(`[Quick-SSH] 检测到操作系统: ${osType}`);
    console.log("[Quick-SSH] 正在清理 PATH 配置...");

    if (osType === "windows") {
        // ─── Windows: 清理 PowerShell $PROFILE ───
        const profilePaths = getPowerShellProfilePaths();
        for (const profilePath of profilePaths) {
            removeMarkerBlock(profilePath);
        }

        // 尝试通过 setx 从 PATH 中移除
        try {
            const execSync = require("child_process").execSync;

            // 找到我们之前添加的 binDir
            const binInfo = getBinaryInfo();
            if (binInfo) {
                const binDir = binInfo.binDir;
                const currentPath = execSync(
                    `echo %PATH%`, { shell: "cmd.exe", encoding: "utf-8" }
                ).trim();
                const newPath = currentPath
                    .split(";")
                    .filter(p => p.trim().toLowerCase() !== binDir.trim().toLowerCase())
                    .join(";");
                if (newPath !== currentPath) {
                    execSync(
                        `setx PATH "${newPath}"`,
                        { shell: "cmd.exe", stdio: "pipe" }
                    );
                    console.log(`[Quick-SSH] ✔ 已从 PATH 中移除: ${binDir}`);
                }
            }
        } catch (e) {
            console.log(`[Quick-SSH] ⚠ setx 清理失败: ${e.message}`);
        }
    } else {
        // ─── Linux / macOS: 清理 Shell rc 文件 ───
        const shell = detectShell();
        const profilePath = getShellProfilePath(shell);
        if (profilePath) {
            removeMarkerBlock(profilePath);
        }

        // 清扫其他常见 rc 文件
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
                removeMarkerBlock(extraPath);
            }
        }
    }

    console.log("[Quick-SSH] ✔ 清理完成！用户配置数据已保留 (~/.ssh/config)。");
    console.log("[Quick-SSH]   如果仍需要手动清理 PATH，请编辑您的 Shell 配置文件。");
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
