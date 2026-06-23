#!/usr/bin/env node
/**
 * scripts/build.js - Quick-SSH 二进制构建脚本
 *
 * 使用 Node.js SEA (Single Executable Application) 将应用编译为原生可执行文件。
 * 需要 Node.js >= 20.11.0
 *
 * 原理:
 *   1. 用 esbuild 将应用及其纯 JS 依赖打包为单个 JS 文件
 *   2. 使用 node --experimental-sea-config 生成 SEA 数据 blob
 *   3. 复制 node.exe 并通过 postject 注入 blob
 *
 * 用法:
 *   node scripts/build.js                  # 构建当前平台
 *   node scripts/build.js --all            # 构建全部平台
 *   node scripts/build.js --platform win32 # 构建 Windows x64
 *   node scripts/build.js --platform linux # 构建 Linux x64
 *   node scripts/build.js --platform darwin# 构建 macOS x64
 *
 * 注意:
 *   - 仅构建当前平台时不需要特殊工具链
 *   - 交叉编译需要目标平台的 Node.js 二进制文件
 */

const { execSync } = require("child_process");
const fs   = require("fs");
const path = require("path");

// ============================================================
// 配置
// ============================================================

const ENTRY_POINT = "src/unix/cli.js";
const BUNDLE_OUT  = ".sea-bundle.js";
const SEA_CONFIG  = ".sea-config.json";
const SEA_BLOB    = ".sea-prep.blob";

const PLATFORM_CONFIG = {
    win32:  {
        output:  "qssh.exe",
        nodeExe: process.execPath,        // 使用当前 node.exe
    },
    linux:  {
        output:  "qssh",
        nodeExe: process.execPath,        // 交叉构建时需要替换
    },
    darwin: {
        output:  "qssh",
        nodeExe: process.execPath,
    },
};

// ============================================================
// 辅助函数
// ============================================================

function log(label, msg, color = "\x1b[36m") {
    const reset = "\x1b[0m";
    console.log(`${color}[${label}]${reset} ${msg}`);
}
function info(msg)  { log("INFO", msg); }
function ok(msg)    { log(" OK ", msg, "\x1b[32m"); }
function warn(msg)  { log("WARN", msg, "\x1b[33m"); }
function error(msg) { log("ERR",  msg, "\x1b[31m"); }

/**
 * 检查 Node.js 版本是否支持 SEA
 */
function checkSeaSupport() {
    const nodeMajor = parseInt(process.version.slice(1).split(".")[0], 10);
    if (nodeMajor < 20) {
        error(`Node.js >= 20.11.0 是 SEA 所必需的，当前版本: ${process.version}`);
        process.exit(1);
    }
    // Node.js 20.11.0+ 支持 SEA
    info(`Node.js ${process.version} - SEA 支持确认`);
}

/**
 * 检查必要工具
 */
function checkTools() {
    try {
        require("esbuild");
    } catch {
        warn("esbuild 未安装，尝试安装...");
        execSync("npm install -D esbuild", { cwd: __dirname, stdio: "inherit" });
    }

    try {
        require("postject");
    } catch {
        warn("postject 未安装，尝试安装...");
        execSync("npm install -D postject", { cwd: __dirname, stdio: "inherit" });
    }
}

/**
 * 步骤 1: 使用 esbuild 将应用打包为单个 JS 文件
 */
function bundleApp() {
    const distDir = path.join(__dirname, "..", "dist");
    if (!fs.existsSync(distDir)) {
        fs.mkdirSync(distDir, { recursive: true });
    }

    const entryPath = path.join(__dirname, "..", ENTRY_POINT);
    if (!fs.existsSync(entryPath)) {
        error(`入口文件不存在: ${entryPath}`);
        process.exit(1);
    }

    info("打包应用及依赖...");

    // 使用 esbuild 打包，将 blessed 和 ssh2 等依赖内联
    // ssh2 的 cpu-features 和 nan 是 optionalDependencies，不影响核心功能
    const esbuild = require("esbuild");
    esbuild.buildSync({
        entryPoints: [entryPath],
        bundle: true,
        platform: "node",
        target: `node${process.version.slice(1).split(".")[0]}`,
        outfile: path.join(distDir, BUNDLE_OUT),
        format: "cjs",
        // 注入编译时常量：标记为 SEA 二进制版本
        define: {
            "globalThis.__IS_BINARY__": "true",
        },
        // 排除无法打包的原生模块和 blessed 未使用的可选依赖
        external: [
            "cpu-features",  // ssh2 的可选原生依赖
            "nan",            // ssh2 的可选原生依赖
            "term.js",        // blessed 终端组件（未使用）
            "pty.js",         // blessed 终端组件（未使用）
        ],
        // 让 blessed 可以正确工作
        mainFields: ["main", "browser"],
        resolveExtensions: [".js", ".json", ".node"],
    });

    const stats = fs.statSync(path.join(distDir, BUNDLE_OUT));
    const sizeKB = (stats.size / 1024).toFixed(2);
    ok(`打包完成: ${BUNDLE_OUT} (${sizeKB} KB)`);
}

/**
 * 步骤 2: 创建 SEA 配置并生成 blob
 */
function createSeaBlob() {
    const distDir = path.join(__dirname, "..", "dist");

    // 创建 SEA 配置
    const seaConfig = {
        main: path.join(distDir, BUNDLE_OUT).replace(/\\/g, "/"),
        output: path.join(distDir, SEA_BLOB).replace(/\\/g, "/"),
        disableExperimentalSEAWarning: true,
    };

    const configPath = path.join(distDir, SEA_CONFIG);
    fs.writeFileSync(configPath, JSON.stringify(seaConfig, null, 2));
    info(`SEA 配置: ${SEA_CONFIG}`);

    // 生成 blob
    info("生成 SEA blob...");
    execSync(`node --experimental-sea-config "${configPath}"`, {
        cwd: path.join(__dirname, ".."),
        stdio: "inherit",
    });
    ok("SEA blob 生成完成");
}

/**
 * 步骤 3: 复制 node.exe 并注入 blob
 */
function injectBlob(platform) {
    const config = PLATFORM_CONFIG[platform];
    const distDir = path.join(__dirname, "..", "dist");
    const outPath = path.join(distDir, config.output);
    const blobPath = path.join(distDir, SEA_BLOB);
    const nodeExe = config.nodeExe;

    if (!fs.existsSync(blobPath)) {
        error(`SEA blob 不存在: ${blobPath}`);
        process.exit(1);
    }

    if (!fs.existsSync(nodeExe)) {
        error(`Node.js 二进制文件不存在: ${nodeExe}`);
        process.exit(1);
    }

    // 复制 Node.js 二进制文件
    info(`复制: ${nodeExe} → ${outPath}`);
    fs.copyFileSync(nodeExe, outPath);

    if (platform === "win32") {
        // Windows: 需要先移除签名
        info("移除 Windows 数字签名...");
        try {
            // 使用 signtool 或 powershell 移除签名
            execSync(
                `powershell -Command "& {Set-Content -Path '${outPath}' -Value (Get-Content '${outPath}' -ReadCount 0) -AsByteStream}"`,
                { stdio: "pipe", timeout: 30000 }
            );
        } catch {
            warn("签名移除失败（可能无签名），继续...");
        }
    }

    // 注入 blob
    info("注入 SEA blob...");
    try {
        execSync(
            `npx postject "${outPath}" NODE_SEA_BLOB "${blobPath}" --sentinel-fuse NODE_SEA_FUSE_fce680ab2cc467b6e072b8b5df1996b2`,
            {
                cwd: path.join(__dirname, ".."),
                stdio: "inherit",
                timeout: 60000,
            }
        );
    } catch (e) {
        error(`postject 注入失败: ${e.message}`);
        process.exit(1);
    }

    const finalStats = fs.statSync(outPath);
    const sizeMB = (finalStats.size / (1024 * 1024)).toFixed(2);
    ok(`构建成功: ${outPath} (${sizeMB} MB)`);
}

/**
 * 清理临时文件
 */
function cleanup() {
    const distDir = path.join(__dirname, "..", "dist");
    const files = [BUNDLE_OUT, SEA_CONFIG, SEA_BLOB];
    for (const f of files) {
        const fp = path.join(distDir, f);
        if (fs.existsSync(fp)) {
            try {
                fs.unlinkSync(fp);
            } catch { /* ignore */ }
        }
    }
}

// ============================================================
// 构建函数
// ============================================================

function buildTarget(platform) {
    const config = PLATFORM_CONFIG[platform];
    if (!config) {
        error(`不支持的平台: ${platform}`);
        const supported = Object.keys(PLATFORM_CONFIG).join(", ");
        error(`支持的平台: ${supported}`);
        process.exit(1);
    }

    info(`构建目标: ${platform} → ${config.output}`);

    // 步骤 1: 打包
    bundleApp();

    // 步骤 2: 生成 SEA blob
    createSeaBlob();

    // 步骤 3: 注入
    injectBlob(platform);

    // 清理
    cleanup();
}

// ============================================================
// 主入口
// ============================================================

function main() {
    const args = process.argv.slice(2);

    // 检查环境
    checkSeaSupport();
    checkTools();

    const buildAll    = args.includes("--all");
    const platformIdx = args.indexOf("--platform");
    let platform = null;

    if (platformIdx !== -1 && platformIdx + 1 < args.length) {
        platform = args[platformIdx + 1];
    }

    if (buildAll) {
        info("构建所有平台...");
        for (const p of Object.keys(PLATFORM_CONFIG)) {
            buildTarget(p);
        }
        ok("所有平台构建完成！");
        return;
    }

    if (platform) {
        buildTarget(platform);
        return;
    }

    // 默认：构建当前平台
    const currentPlatform = process.platform;
    info(`检测到当前平台: ${currentPlatform}`);
    buildTarget(currentPlatform);
}

main();
