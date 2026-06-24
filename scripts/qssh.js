#!/usr/bin/env node
/**
 * scripts/qssh.js - Quick-SSH npm bin 包装器
 *
 * npm 的 "bin" 字段指向此文件。
 * 检测当前操作系统平台，然后运行对应的原生二进制文件。
 *
 * 这样做的原因:
 *   原生二进制文件是平台相关的（qssh-win.exe / qssh / qssh-darwin），
 *   而 npm 的 bin 字段只能指向单一入口。此包装器根据平台动态选择正确的二进制。
 *
 * 额外检测:
 *   验证二进制文件确实是当前平台的有效可执行文件（通过读取魔数），
 *   避免因跨平台构建问题导致无提示失败。
 */

const { spawnSync } = require("child_process");
const path = require("path");
const fs = require("fs");

const platform = process.platform;

// ============================================================
// 平台配置
// ============================================================

const PLATFORM_CONFIG = {
    win32: {
        binNames: ["qssh-win.exe"],
        magic: null,             // Windows 不检测魔数
    },
    linux: {
        binNames: ["qssh-linux", "qssh"],
        magic: [0x7f, 0x45, 0x4c, 0x46],  // \x7fELF
        magicLabel: "ELF (Linux 可执行文件)",
    },
    darwin: {
        binNames: ["qssh-darwin", "qssh"],
        magic: [0xfe, 0xed, 0xfa, 0xce],  // Mach-O (x86_64)
        magicLabel: "Mach-O (macOS 可执行文件)",
    },
};

// ============================================================
// 辅助函数
// ============================================================

/**
 * 读取文件的前 4 个字节（魔数），用于判断可执行文件格式
 */
function readMagicBytes(filePath) {
    try {
        const fd = fs.openSync(filePath, "r");
        const buf = Buffer.alloc(4);
        fs.readSync(fd, buf, 0, 4, 0);
        fs.closeSync(fd);
        return [buf[0], buf[1], buf[2], buf[3]];
    } catch {
        return null;
    }
}

/**
 * 检查二进制文件是否有效（可执行+正确平台格式）
 * 返回错误字符串，或 null 表示有效
 */
function validateBinary(binPath, config) {
    if (!fs.existsSync(binPath)) {
        return `文件不存在: ${binPath}`;
    }

    // 检查是否为普通文件
    const stat = fs.statSync(binPath);
    if (!stat.isFile()) {
        return `不是普通文件: ${binPath}`;
    }

    // 在非 Windows 平台上检查可执行权限
    if (platform !== "win32") {
        try {
            fs.accessSync(binPath, fs.constants.X_OK);
        } catch {
            // 尝试修复权限
            try {
                fs.chmodSync(binPath, 0o755);
                // 再次验证
                fs.accessSync(binPath, fs.constants.X_OK);
            } catch {
                return `文件缺少执行权限，且无法自动修复（请手动执行: sudo chmod +x "${binPath}"）`;
            }
        }
    }

    // 在 Linux/macOS 上检查魔数
    if (config.magic) {
        const magic = readMagicBytes(binPath);
        if (!magic) {
            return `无法读取文件魔数，文件可能已损坏`;
        }

        const expected = config.magic;
        const isMatch = magic.length >= expected.length &&
            expected.every((b, i) => magic[i] === b);

        if (!isMatch) {
            const actualHex = magic.map(b => "0x" + b.toString(16).padStart(2, "0")).join(" ");
            const expectedHex = expected.map(b => "0x" + b.toString(16).padStart(2, "0")).join(" ");

            // 通过魔数前几个字节识别已知格式
            function identifyFormat(bytes) {
                const hex2 = bytes.slice(0, 2).map(b => "0x" + b.toString(16).padStart(2, "0")).join(" ");
                if (bytes[0] === 0x7f && bytes[1] === 0x45 && bytes[2] === 0x4c && bytes[3] === 0x46) return "ELF (Linux)";
                if (hex2 === "0x4d 0x5a") return "MZ (Windows PE)";
                if (bytes[0] === 0xfe && bytes[1] === 0xed && bytes[2] === 0xfa && bytes[3] === 0xce) return "Mach-O (macOS x86_64)";
                if (bytes[0] === 0xce && bytes[1] === 0xfa && bytes[2] === 0xed && bytes[3] === 0xfe) return "Mach-O (macOS x86_64 reverse)";
                if (bytes[0] === 0xfe && bytes[1] === 0xed && bytes[2] === 0xfa && bytes[3] === 0xcf) return "Mach-O (macOS arm64)";
                if (bytes[0] === 0xcf && bytes[1] === 0xfa && bytes[2] === 0xed && bytes[3] === 0xfe) return "Mach-O (macOS arm64 reverse)";
                return `未知格式 (${actualHex})`;
            }

            const actualName = identifyFormat(magic);
            return `文件格式不正确: 期望 ${config.magicLabel} (${expectedHex})，实际为 ${actualName}` +
                `\n   Quick-SSH 当前提供的 Linux 二进制实际是 Windows PE 格式（MZ）。` +
                `\n   这是因为构建工具链在 Windows 上运行，未正确交叉编译。` +
                `\n   ` +
                `\n   解决方案：` +
                `\n   1. 从源码构建（推荐）：在 Linux 上执行 npm run build` +
                `\n   2. 或使用包管理器安装: apt install quick-ssh / pacman -S quick-ssh`;
        }
    }

    return null; // 验证通过
}

// ============================================================
// 主流程
// ============================================================

const config = PLATFORM_CONFIG[platform];

if (!config) {
    console.error(`[Quick-SSH] 错误：不支持的平台 "${platform}"`);
    console.error(`[Quick-SSH] 支持的平台: win32, linux, darwin`);
    process.exit(1);
}

// 尝试多个二进制文件名（按优先级）
const pkgDir = path.resolve(__dirname, "..");
let binPath = null;
let binNameUsed = null;

for (const candidate of config.binNames) {
    const candidatePath = path.join(pkgDir, "dist", "bin", candidate);
    if (fs.existsSync(candidatePath)) {
        binPath = candidatePath;
        binNameUsed = candidate;
        break;
    }
}

if (!binPath) {
    console.error(`[Quick-SSH] 错误：找不到当前平台的二进制文件`);
    console.error(`[Quick-SSH] 已尝试以下文件名:`);
    for (const name of config.binNames) {
        console.error(`[Quick-SSH]   - ${path.join(pkgDir, "dist", "bin", name)}`);
    }
    console.error(`[Quick-SSH] 请尝试重新安装: npm i -g quick-ssh`);
    console.error(`[Quick-SSH] 或从源码构建: git clone 仓库后执行 npm run build`);
    process.exit(1);
}

// 验证二进制文件有效性
const validationError = validateBinary(binPath, config);
if (validationError) {
    console.error(`[Quick-SSH] ⚠ 二进制文件验证失败:`);
    console.error(`[Quick-SSH]   路径: ${binPath}`);
    console.error(`[Quick-SSH]   原因: ${validationError}`);
    console.error(``);
    console.error(`[Quick-SSH] 正在尝试以源码模式降级运行...`);

    // 降级方案：尝试直接用 Node.js 运行源码
    // 注意：源码模式需要依赖已安装
    const sourceEntry = path.join(pkgDir, "src", "unix", "cli.js");
    if (fs.existsSync(sourceEntry)) {
        console.error(`[Quick-SSH] 以源码模式启动: node ${sourceEntry}`);
        const result = spawnSync(process.execPath, [sourceEntry, ...process.argv.slice(2)], {
            stdio: "inherit",
        });
        process.exit(result.status ?? 1);
    } else {
        console.error(`[Quick-SSH] 源码文件不存在，无法降级运行。`);
        console.error(`[Quick-SSH] 请尝试以下解决方案:`);
        console.error(`[Quick-SSH]   1. 从源码构建: npm run build`);
        console.error(`[Quick-SSH]   2. 或使用包管理器安装: apt install quick-ssh / pacman -S quick-ssh`);
        process.exit(1);
    }
}

// 二进制有效，启动
const result = spawnSync(binPath, process.argv.slice(2), {
    stdio: "inherit",
    windowsHide: true,
});

process.exit(result.status ?? 1);
