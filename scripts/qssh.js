#!/usr/bin/env node
/**
 * scripts/qssh.js - Quick-SSH npm bin 包装器
 *
 * npm 的 "bin" 字段指向此文件。
 * 检测当前操作系统平台，然后运行对应的原生二进制文件。
 *
 * 这样做的原因:
 *   原生二进制文件是平台相关的（qssh-win.exe / qssh-linux / qssh-darwin），
 *   而 npm 的 bin 字段只能指向单一入口。此包装器根据平台动态选择正确的二进制。
 */

const { spawnSync } = require("child_process");
const path = require("path");
const fs = require("fs");

const platform = process.platform;
let binName;

if (platform === "win32") {
    binName = "qssh-win.exe";
} else if (platform === "linux") {
    binName = "qssh-linux";
} else if (platform === "darwin") {
    binName = "qssh-darwin";
} else {
    console.error(`[Quick-SSH] 错误：不支持的平台 "${platform}"`);
    console.error(`[Quick-SSH] 支持的平台: win32, linux, darwin`);
    process.exit(1);
}

const binPath = path.join(__dirname, "..", "dist", "bin", binName);

if (!fs.existsSync(binPath)) {
    console.error(`[Quick-SSH] 错误：找不到当前平台的二进制文件`);
    console.error(`[Quick-SSH] 期望路径: ${binPath}`);
    console.error(`[Quick-SSH] 请尝试重新安装: npm i -g quick-ssh`);
    process.exit(1);
}

const result = spawnSync(binPath, process.argv.slice(2), {
    stdio: "inherit",
    windowsHide: true,
});

process.exit(result.status ?? 1);
