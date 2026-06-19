"use strict";

const fs = require("fs");
const path = require("path");
const os = require("os");
const readline = require("readline");
const { Client } = require("ssh2");

function decodePayload(argv) {
    const encoded = argv[2];
    if (!encoded) {
        throw new Error("missing payload");
    }
    return JSON.parse(Buffer.from(encoded, "base64").toString("utf8"));
}

function getUploadConcurrency(payload) {
    const value = payload && payload.config ? payload.config.uploadConcurrency : 3;
    const parsed = Number.parseInt(String(value || ""), 10);
    return Number.isFinite(parsed) && parsed > 0 ? parsed : 3;
}

function waitForEnter() {
    return new Promise((resolve) => {
        const rl = readline.createInterface({
            input: process.stdin,
            output: process.stdout,
        });
        rl.question("按 Enter 关闭此窗口...", () => {
            rl.close();
            resolve();
        });
    });
}

function enableAltScreen() {
    if (!process.stdout.isTTY) {
        return;
    }
    process.stdout.write("\x1b[?1049h\x1b[?25l\x1b[2J");
}

function disableAltScreen() {
    if (!process.stdout.isTTY) {
        return;
    }
    process.stdout.write("\x1b[?25h\x1b[?1049l");
}

function moveCursor(row, col) {
    process.stdout.write(`\x1b[${row};${col}H`);
}

function clearLine() {
    process.stdout.write("\x1b[2K");
}

function formatBytes(bytes) {
    const units = ["B", "KB", "MB", "GB", "TB"];
    let value = bytes;
    let idx = 0;
    while (value >= 1024 && idx < units.length - 1) {
        value /= 1024;
        idx++;
    }
    return `${value >= 100 || idx === 0 ? value.toFixed(0) : value.toFixed(1)}${units[idx]}`;
}

function formatDuration(seconds) {
    const sec = Math.max(0, Math.round(seconds));
    const h = Math.floor(sec / 3600);
    const m = Math.floor((sec % 3600) / 60);
    const s = sec % 60;
    if (h > 0) {
        return `${h}h${String(m).padStart(2, "0")}m${String(s).padStart(2, "0")}s`;
    }
    if (m > 0) {
        return `${m}m${String(s).padStart(2, "0")}s`;
    }
    return `${s}s`;
}

function renderBar(ratio, width) {
    const normalized = Math.max(0, Math.min(1, Number.isFinite(ratio) ? ratio : 0));
    const filled = Math.round(normalized * width);
    return "=".repeat(filled) + " ".repeat(Math.max(0, width - filled));
}

function listLocalFiles(inputPath, rootName) {
    const stat = fs.statSync(inputPath);
    if (stat.isDirectory()) {
        let items = [];
        for (const entry of fs.readdirSync(inputPath)) {
            const abs = path.join(inputPath, entry);
            const rel = path.posix.join(rootName, entry).replace(/\\/g, "/");
            items = items.concat(listLocalFiles(abs, rel));
        }
        return items;
    }
    if (!stat.isFile()) {
        return [];
    }
    return [{ localPath: inputPath, remoteRelative: rootName.replace(/\\/g, "/"), size: stat.size }];
}

function flattenPayloadFiles(filePaths) {
    const files = [];
    for (const filePath of filePaths) {
        const baseName = path.basename(filePath);
        files.push(...listLocalFiles(filePath, baseName));
    }
    return files;
}

function mkdirSftp(sftp, remoteDir) {
    return new Promise((resolve, reject) => {
        sftp.mkdir(remoteDir, (error) => {
            if (!error || (error && /Failure|exists/i.test(String(error.message || error)))) {
                resolve();
                return;
            }
            reject(error);
        });
    });
}

async function ensureRemoteDir(sftp, remoteDir) {
    const parts = remoteDir.replace(/\\/g, "/").split("/").filter(Boolean);
    const isAbsolute = remoteDir.startsWith("/");
    let current = isAbsolute ? "/" : ".";

    for (const part of parts) {
        current = current === "/" ? `/${part}` : `${current}/${part}`;
        try {
            await mkdirSftp(sftp, current);
        } catch (error) {
            if (!/Failure|exists/i.test(String(error.message || error))) {
                throw error;
            }
        }
    }
}

function connectSftp(target) {
    return new Promise((resolve, reject) => {
        const client = new Client();
        client.on("ready", () => {
            client.sftp((error, sftp) => {
                if (error) {
                    client.end();
                    reject(error);
                    return;
                }
                resolve({ client, sftp });
            });
        });
        client.on("error", reject);
        client.connect({
            host: target.host,
            port: Number(target.port),
            username: target.user,
            privateKey: fs.readFileSync(target.key),
        });
    });
}

function uploadOneFile(sftp, file, remotePath, onProgress) {
    return new Promise((resolve, reject) => {
        const remoteFile = path.posix.join(remotePath.replace(/\\/g, "/"), file.remoteRelative);
        const remoteDir = path.posix.dirname(remoteFile);

        ensureRemoteDir(sftp, remoteDir)
            .then(() => {
                const readStream = fs.createReadStream(file.localPath);
                const writeStream = sftp.createWriteStream(remoteFile);
                let transferred = 0;

                readStream.on("data", (chunk) => {
                    transferred += chunk.length;
                    onProgress(transferred);
                });
                readStream.on("error", reject);
                writeStream.on("error", reject);
                writeStream.on("close", () => resolve());
                readStream.pipe(writeStream);
            })
            .catch(reject);
    });
}

function createRenderer(files, concurrency) {
    const start = Date.now();
    const totalBytes = files.reduce((sum, file) => sum + file.size, 0);
    const state = files.map((file, index) => ({
        id: index + 1,
        label: file.remoteRelative,
        size: file.size,
        transferred: 0,
        status: "waiting",
        startedAt: null,
        endedAt: null,
        error: "",
    }));

    function summarize() {
        const doneBytes = state.reduce((sum, item) => sum + item.transferred, 0);
        const elapsedSec = Math.max(1, (Date.now() - start) / 1000);
        const speed = doneBytes / elapsedSec;
        const remainingBytes = Math.max(0, totalBytes - doneBytes);
        const eta = speed > 0 ? remainingBytes / speed : 0;
        return { doneBytes, elapsedSec, speed, remainingBytes, eta };
    }

    let initialized = false;
    let lastLineCount = 0;

    function redraw() {
        const termWidth = process.stdout.columns || 120;
        const barWidth = Math.max(10, Math.min(40, termWidth - 70));
        const { doneBytes, speed, eta } = summarize();

        const lines = [];
        lines.push("Quick-SSH Upload");
        lines.push(`Concurrency: ${concurrency}    Files: ${state.length}`);
        lines.push("");

        for (const item of state) {
            const ratio = item.size > 0 ? item.transferred / item.size : 1;
            const pct = `${Math.floor(ratio * 100)}`.padStart(3, " ");
            const bar = renderBar(ratio, barWidth);
            const remain = item.status === "running" && item.startedAt
                ? formatDuration(((item.size - item.transferred) / Math.max(1, item.transferred / ((Date.now() - item.startedAt) / 1000 || 1))))
                : item.status === "done"
                    ? "done"
                    : item.status === "failed"
                        ? "fail"
                        : "wait";
            const name = item.label.length > 32 ? `...${item.label.slice(-29)}` : item.label;
            lines.push(`[${String(item.id).padStart(2, "0")}] ${name.padEnd(32)} [${bar}] ${pct}% ${formatBytes(item.transferred).padStart(8)}/${formatBytes(item.size).padStart(8)} ${remain}`);
        }

        lines.push("");
        const totalRatio = totalBytes > 0 ? doneBytes / totalBytes : 1;
        const totalBar = renderBar(totalRatio, barWidth);
        lines.push(`Total ${" ".repeat(29)} [${totalBar}] ${String(Math.floor(totalRatio * 100)).padStart(3, " ")}% ${formatBytes(doneBytes).padStart(8)}/${formatBytes(totalBytes).padStart(8)} ETA ${formatDuration(eta)}`);
        lines.push(`Speed ${formatBytes(speed)}/s`);

        if (!initialized) {
            enableAltScreen();
            initialized = true;
        }

        moveCursor(1, 1);
        const lineCount = Math.max(lastLineCount, lines.length);
        for (let i = 0; i < lineCount; i++) {
            clearLine();
            const text = lines[i] || "";
            process.stdout.write(text);
            if (i < lineCount - 1) {
                process.stdout.write("\n");
            }
        }
        lastLineCount = lines.length;
    }

    return {
        state,
        redraw,
        markRunning(index) {
            state[index].status = "running";
            state[index].startedAt = Date.now();
            redraw();
        },
        updateProgress(index, transferred) {
            state[index].transferred = transferred;
            redraw();
        },
        markDone(index) {
            state[index].status = "done";
            state[index].transferred = state[index].size;
            state[index].endedAt = Date.now();
            redraw();
        },
        markFailed(index, error) {
            state[index].status = "failed";
            state[index].endedAt = Date.now();
            state[index].error = error.message || String(error);
            redraw();
        },
        finish() {
            redraw();
            return state;
        },
        restoreTerminal() {
            disableAltScreen();
        },
    };
}

async function runUploadPool(payload, files) {
    const concurrency = Math.min(getUploadConcurrency(payload), files.length || 1);
    const renderer = createRenderer(files, concurrency);
    renderer.redraw();

    const queue = files.map((file, index) => ({ file, index }));
    const succeeded = [];
    const failed = [];

    async function worker() {
        const { client, sftp } = await connectSftp(payload.target);
        try {
            while (queue.length > 0) {
                const job = queue.shift();
                if (!job) {
                    return;
                }
                renderer.markRunning(job.index);
                try {
                    await uploadOneFile(sftp, job.file, payload.remotePath, (transferred) => {
                        renderer.updateProgress(job.index, transferred);
                    });
                    renderer.markDone(job.index);
                    succeeded.push(job.file.localPath);
                } catch (error) {
                    renderer.markFailed(job.index, error);
                    failed.push({ filePath: job.file.localPath, error: error.message || String(error) });
                }
            }
        } finally {
            client.end();
        }
    }

    await Promise.all(Array.from({ length: concurrency }, () => worker()));
    return { renderer, succeeded, failed };
}

async function main() {
    let payload;
    try {
        payload = decodePayload(process.argv);
    } catch (error) {
        process.stderr.write(`Quick-SSH upload runner payload error: ${error.message}\n`);
        process.exitCode = 1;
        return;
    }

    const files = flattenPayloadFiles(payload.files);
    if (files.length === 0) {
        process.stderr.write("没有可上传的文件。\n");
        process.exitCode = 1;
        return;
    }

    const { renderer, succeeded, failed } = await runUploadPool(payload, files);
    const finalState = renderer.finish();

    renderer.restoreTerminal();
    process.stdout.write("Summary\n");
    process.stdout.write(`Success: ${succeeded.length}\n`);
    process.stdout.write(`Failed : ${failed.length}\n`);

    if (failed.length > 0) {
        process.stdout.write("\nTransfer failed:\n");
        for (const item of failed) {
            process.stdout.write(`- ${item.filePath} (${item.error})\n`);
        }
        process.exitCode = 1;
    }

    const completed = finalState.filter((item) => item.status === "done").length;
    process.stdout.write(`Completed items: ${completed}/${finalState.length}\n\n`);
    await waitForEnter();
}

main().catch(async (error) => {
    disableAltScreen();
    process.stderr.write(`Quick-SSH upload runner fatal error: ${error.message}\n\n`);
    process.exitCode = 1;
    await waitForEnter();
});
