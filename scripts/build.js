#!/usr/bin/env node
/**
 * scripts/build.js - Quick-SSH 二进制构建脚本
 *
 * 使用 @yao-pkg/pkg 将 Node.js 源码编译为原生可执行文件。
 *
 * 用法:
 *   node scripts/build.js                  # 构建当前平台
 *   node scripts/build.js --all            # 构建全部平台 (win/linux/macos x64)
 *   node scripts/build.js --platform win32 # 构建 Windows x64
 *   node scripts/build.js --platform linux # 构建 Linux x64
 *   node scripts/build.js --platform darwin# 构建 macOS x64
 *   node scripts/build.js --platform win32 --arch arm64 # 构建 Windows arm64
 *
 * 输出:
 *   dist/
 *     qssh-win-x64.exe     (Windows)
 *     qssh-linux-x64       (Linux)
 *     qssh-macos-x64       (macOS)
 */

const { execSync } = require("child_process");
const fs   = require("fs");
const path = require("path");

// ============================================================
// 配置
// ============================================================

const PKG_VERSION = "18";       // 内置 Node.js 版本
const ENTRY_POINT = "src/unix/cli.js";

const TARGETS = {
    win32:  { suffix: ".exe",  pkg: `node${PKG_VERSION}-win-x64`  },
    linux:  { suffix: "",      pkg: `node${PKG_VERSION}-linux-x64` },
    darwin: { suffix: "",      pkg: `node${PKG_VERSION}-macos-x64` },
};

const TARGETS_ARM = {
    win32:  { suffix: ".exe",  pkg: `node${PKG_VERSION}-win-arm64`  },
    linux:  { suffix: "",      pkg: `node${PKG_VERSION}-linux-arm64` },
    darwin: { suffix: "",      pkg: `node${PKG_VERSION}-macos-arm64` },
};

// ============================================================
// 辅助函数
// ============================================================

function getPkgBin() {
    // 优先使用本地安装的 @yao-pkg/pkg
    const localPkg = path.join(__dirname, "..", "node_modules", ".bin", "pkg");
    if (fs.existsSync(localPkg + ".cmd") || fs.existsSync(localPkg)) {
        return localPkg;
    }
    // 回退到全局
    return "pkg";
}

function log(label, msg, color = "\x1b[36m") {
    const reset = "\x1b[0m";
    console.log(`${color}[${label}]${reset} ${msg}`);
}

function info(msg)    { log("INFO", msg); }
function ok(msg)      { log(" OK ", msg, "\x1b[32m"); }
function warn(msg)    { log("WARN", msg, "\x1b[33m"); }
function error(msg)   { log("ERR",  msg, "\x1b[31m"); }

// ============================================================
// 构建函数
// ============================================================

function buildTarget(platform, arch = "x64") {
    const isArm = arch === "arm64";
    const targetMap = isArm ? TARGETS_ARM : TARGETS;
    const target = targetMap[platform];

    if (!target) {
        error(`不支持的平台: ${platform} (arch: ${arch})`);
        process.exit(1);
    }

    const distDir = path.join(__dirname, "..", "dist");
    const outName = `qssh-${platform}-${arch}${target.suffix}`;
    const outPath = path.join(distDir, outName);

    // 确保 dist 目录存在
    if (!fs.existsSync(distDir)) {
        fs.mkdirSync(distDir, { recursive: true });
    }

    const pkgBin = getPkgBin();
    const cmd = [
        `"${pkgBin}"`,
        ENTRY_POINT,
        `--target`, target.pkg,
        `--output`, `"${outPath}"`,
        `--compress`, `GZip`,
    ].join(" ");

    info(`构建: ${platform} (${arch}) → ${outName}`);
    info(`命令: ${cmd}`);

    try {
        execSync(cmd, {
            cwd: path.join(__dirname, ".."),
            stdio: "inherit",
            env: { ...process.env },
        });
        ok(`构建成功: ${outPath}`);

        // 输出文件大小
        const stats = fs.statSync(outPath);
        const sizeMB = (stats.size / (1024 * 1024)).toFixed(2);
        info(`文件大小: ${sizeMB} MB`);

        return outPath;
    } catch (e) {
        error(`构建失败: ${platform} (${arch})`);
        error(`错误信息: ${e.message}`);
        process.exit(1);
    }
}

// ============================================================
// 主入口
// ============================================================

function main() {
    const args = process.argv.slice(2);

    // 检查 pkg 是否可用
    try {
        execSync(`"${getPkgBin()}" --version`, {
            cwd: path.join(__dirname, ".."),
            stdio: "pipe",
        });
    } catch {
        warn("@yao-pkg/pkg 未安装，正在安装...");
        execSync("npm install -D @yao-pkg/pkg", {
            cwd: path.join(__dirname, ".."),
            stdio: "inherit",
        });
    }

    // 检查入口文件
    const entryPath = path.join(__dirname, "..", ENTRY_POINT);
    if (!fs.existsSync(entryPath)) {
        error(`入口文件不存在: ${entryPath}`);
        process.exit(1);
    }

    const buildAll = args.includes("--all");
    const platformIdx = args.indexOf("--platform");
    const archIdx = args.indexOf("--arch");

    let platform = null;
    let arch = "x64";

    if (platformIdx !== -1 && platformIdx + 1 < args.length) {
        platform = args[platformIdx + 1];
    }
    if (archIdx !== -1 && archIdx + 1 < args.length) {
        arch = args[archIdx + 1];
    }

    if (buildAll) {
        info("构建所有平台 (x64)...");
        buildTarget("win32", "x64");
        buildTarget("linux", "x64");
        buildTarget("darwin", "x64");
        ok("所有平台构建完成！");
        return;
    }

    if (platform) {
        buildTarget(platform, arch);
        return;
    }

    // 默认：构建当前平台
    const currentPlatform = process.platform;
    info(`检测到当前平台: ${currentPlatform}，构建中...`);
    buildTarget(currentPlatform, arch);
}

main();
