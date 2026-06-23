#!/usr/bin/env node
/**
 * scripts/build.js - Quick-SSH 二进制构建脚本
 *
 * 使用 Node.js SEA (Single Executable Application) + @vercel/ncc 打包。
 * 需要 Node.js >= 20.11.0
 *
 * 原理:
 *   1. 用 @vercel/ncc 将应用及其依赖打包为单个 JS 文件（支持动态 require）
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
    info(`Node.js ${process.version} - SEA 支持确认`);
}

/**
 * 步骤 1: 使用 @vercel/ncc 将应用打包为单个 JS 文件
 * ncc 基于 webpack，可以处理 blessed 的动态 require('./widgets/' + name)
 */
async function bundleApp() {
    const distDir = path.join(__dirname, "..", "dist");
    if (!fs.existsSync(distDir)) {
        fs.mkdirSync(distDir, { recursive: true });
    }

    const entryPath = path.join(__dirname, "..", ENTRY_POINT);
    if (!fs.existsSync(entryPath)) {
        error(`入口文件不存在: ${entryPath}`);
        process.exit(1);
    }

    info("使用 @vercel/ncc 打包应用及依赖...");

    // 创建包装入口文件，注入编译时常量
    const wrapPath = path.join(distDir, ".sea-entry-wrap.js");
    const wrapContent = `
// Quick-SSH SEA 包装入口
// 编译时常量注入
globalThis.__IS_BINARY__ = true;

// 加载主入口
require(${JSON.stringify(entryPath.replace(/\\/g, "/"))});
`;
    fs.writeFileSync(wrapPath, wrapContent, "utf-8");

    const ncc = require("@vercel/ncc");
    const result = await ncc(wrapPath, {
        minify: false,
        sourceMap: false,
        sourceMapRegister: false,
        assetBuilds: false,
    });

    const outPath = path.join(distDir, BUNDLE_OUT);
    fs.writeFileSync(outPath, result.code, "utf-8");

    const stats = fs.statSync(outPath);
    const sizeKB = (stats.size / 1024).toFixed(2);
    ok(`打包完成: ${BUNDLE_OUT} (${sizeKB} KB)`);

    // 复制 blessed 的 terminfo 文件到 dist/usr/
    // blessed 需要这些文件来检测终端能力
    const blessedUsr = path.join(__dirname, "..", "node_modules", "blessed", "usr");
    const distUsr = path.join(distDir, "usr");
    if (fs.existsSync(blessedUsr)) {
        info("复制 terminfo 文件...");
        if (!fs.existsSync(distUsr)) {
            fs.mkdirSync(distUsr, { recursive: true });
        }
        // 递归复制
        function copyDir(src, dest) {
            const entries = fs.readdirSync(src, { withFileTypes: true });
            for (const entry of entries) {
                const srcPath = path.join(src, entry.name);
                const destPath = path.join(dest, entry.name);
                if (entry.isDirectory()) {
                    if (!fs.existsSync(destPath)) {
                        fs.mkdirSync(destPath, { recursive: true });
                    }
                    copyDir(srcPath, destPath);
                } else {
                    fs.copyFileSync(srcPath, destPath);
                }
            }
        }
        copyDir(blessedUsr, distUsr);
        ok("terminfo 文件复制完成");
    }
}

/**
 * 步骤 2: 创建 SEA 配置并生成 blob
 */
function createSeaBlob() {
    const distDir = path.join(__dirname, "..", "dist");

    // 创建 SEA 配置（必须使用绝对路径，正斜杠）
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
 * 步骤 4: 使用 UPX 压缩二进制文件
 * UPX 可大幅减小文件体积（约 90 MB → 25-30 MB），对功能无影响。
 * 首次启动时略有解压开销。
 */
function ensureUpx() {
    const toolsDir = path.join(__dirname, "tools");
    const upxPath = path.join(toolsDir, "upx.exe");

    if (fs.existsSync(upxPath)) {
        return upxPath;
    }

    info("UPX 未找到，正在自动下载...");
    if (!fs.existsSync(toolsDir)) {
        fs.mkdirSync(toolsDir, { recursive: true });
    }

    const zipUrl = "https://github.com/upx/upx/releases/download/v5.0.0/upx-5.0.0-win64.zip";
    const zipPath = path.join(toolsDir, "upx.zip");

    try {
        execSync(
            `powershell -Command "& {Invoke-WebRequest -Uri '${zipUrl}' -OutFile '${zipPath}'}"`,
            { stdio: "pipe", timeout: 60000 }
        );
        execSync(
            `powershell -Command "& {Expand-Archive -Path '${zipPath}' -DestinationPath '${toolsDir}' -Force}"`,
            { stdio: "pipe", timeout: 30000 }
        );
        // upx.exe is inside a subfolder after extraction
        const extractedDirs = fs.readdirSync(toolsDir).filter(d =>
            d.startsWith("upx-") && fs.statSync(path.join(toolsDir, d)).isDirectory()
        );
        if (extractedDirs.length > 0) {
            const exeSrc = path.join(toolsDir, extractedDirs[0], "upx.exe");
            if (fs.existsSync(exeSrc)) {
                fs.copyFileSync(exeSrc, upxPath);
            }
        }
        // cleanup
        if (fs.existsSync(zipPath)) fs.unlinkSync(zipPath);
        extractedDirs.forEach(d => {
            const dirPath = path.join(toolsDir, d);
            if (fs.existsSync(dirPath)) {
                fs.rmSync(dirPath, { recursive: true, force: true });
            }
        });

        if (fs.existsSync(upxPath)) {
            ok("UPX 下载完成");
            return upxPath;
        }
    } catch (e) {
        warn(`UPX 自动下载失败: ${e.message}`);
    }

    return null;
}

function compressWithUpx(platform) {
    const config = PLATFORM_CONFIG[platform];
    const distDir = path.join(__dirname, "..", "dist");
    const outPath = path.join(distDir, config.output);

    if (!fs.existsSync(outPath)) {
        warn(`二进制文件不存在，跳过 UPX 压缩: ${outPath}`);
        return;
    }

    const upxPath = ensureUpx();
    if (!upxPath) {
        warn("UPX 不可用，跳过压缩");
        return;
    }

    const beforeSize = fs.statSync(outPath).size;
    info(`UPX 压缩: ${config.output} (${(beforeSize / (1024 * 1024)).toFixed(2)} MB)...`);

    try {
        execSync(`"${upxPath}" -7 --no-color "${outPath}"`, {
            cwd: distDir,
            stdio: "pipe",
            timeout: 180000,
        });
    } catch (e) {
        // UPX 对某些文件返回非零退出码但仍成功压缩，检查结果
        warn(`UPX 压缩警告: ${e.message}`);
    }

    const afterSize = fs.statSync(outPath).size;
    const savedMB = ((beforeSize - afterSize) / (1024 * 1024)).toFixed(2);
    const ratio = ((afterSize / beforeSize) * 100).toFixed(1);
    ok(`UPX 压缩完成: ${(afterSize / (1024 * 1024)).toFixed(2)} MB (节省 ${savedMB} MB, ${ratio}%)`);
}

/**
 * 清理临时文件
 */
function cleanup() {
    const distDir = path.join(__dirname, "..", "dist");
    const files = [BUNDLE_OUT, SEA_CONFIG, SEA_BLOB, ".sea-entry-wrap.js"];
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

async function buildTarget(platform) {
    const config = PLATFORM_CONFIG[platform];
    if (!config) {
        error(`不支持的平台: ${platform}`);
        const supported = Object.keys(PLATFORM_CONFIG).join(", ");
        error(`支持的平台: ${supported}`);
        process.exit(1);
    }

    info(`构建目标: ${platform} → ${config.output}`);

    // 步骤 1: 打包
    await bundleApp();

    // 步骤 2: 生成 SEA blob
    createSeaBlob();

    // 步骤 3: 注入
    injectBlob(platform);

    // 步骤 4: UPX 压缩（缩小二进制体积）
    compressWithUpx(platform);

    // 清理
    cleanup();
}

// ============================================================
// 主入口
// ============================================================

async function main() {
    const args = process.argv.slice(2);

    // 检查环境
    checkSeaSupport();

    const buildAll    = args.includes("--all");
    const platformIdx = args.indexOf("--platform");
    let platform = null;

    if (platformIdx !== -1 && platformIdx + 1 < args.length) {
        platform = args[platformIdx + 1];
    }

    if (buildAll) {
        info("构建所有平台...");
        for (const p of Object.keys(PLATFORM_CONFIG)) {
            await buildTarget(p);
        }
        ok("所有平台构建完成！");
        return;
    }

    if (platform) {
        await buildTarget(platform);
        return;
    }

    // 默认：构建当前平台
    const currentPlatform = process.platform;
    info(`检测到当前平台: ${currentPlatform}`);
    await buildTarget(currentPlatform);
}

main().catch((e) => {
    error(`构建失败: ${e.message}`);
    process.exit(1);
});
