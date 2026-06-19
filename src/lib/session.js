"use strict";

const fs = require("fs");
const os = require("os");
const path = require("path");
const { spawn } = require("child_process");

const REMOTE_PWD_PREFIX = "__QSSH_PWD__";
const REMOTE_PWD_SUFFIX = "__";
const REMOTE_PWD_COMMAND =
    "printf \"$(printf '\\137\\137QSSH\\137PWD\\137\\137')%s$(printf '\\137\\137')\\n\" \"$PWD\"\n";

function getSSHExe() {
    return process.platform === "win32" ? "ssh.exe" : "ssh";
}

function getSCPExe() {
    return process.platform === "win32" ? "scp.exe" : "scp";
}

function buildSSHArgs(target) {
    return [
        "-tt",
        "-i", target.key,
        "-p", String(target.port),
        "-o", "HostKeyAlgorithms=+ssh-rsa",
        `${target.user}@${target.host}`,
    ];
}

function buildSCPArgs(target, files, remotePath) {
    const args = [
        "-i", target.key,
        "-P", String(target.port),
        "-o", "BatchMode=yes",
        "-o", "HostKeyAlgorithms=+ssh-rsa",
    ];
    if (files.some((file) => {
        try {
            return fs.statSync(file).isDirectory();
        } catch {
            return false;
        }
    })) {
        args.push("-r");
    }
    args.push(...files, `${target.user}@${target.host}:${remotePath}`);
    return args;
}

function parsePositiveInt(value, fallback) {
    const num = Number.parseInt(String(value || ""), 10);
    return Number.isFinite(num) && num > 0 ? num : fallback;
}

function readQsshConfig() {
    const configPath = path.join(os.homedir(), ".qsshrc");
    const config = { uploadConcurrency: 3 };

    try {
        const content = fs.readFileSync(configPath, "utf8");
        for (const rawLine of content.split(/\r?\n/)) {
            const line = rawLine.replace(/#.*$/, "").trim();
            if (!line) {
                continue;
            }
            const idx = line.indexOf("=");
            if (idx < 0) {
                continue;
            }
            const key = line.slice(0, idx).trim().toLowerCase();
            const value = line.slice(idx + 1).trim();
            if (key === "uploadconcurrency") {
                config.uploadConcurrency = parsePositiveInt(value, 3);
            }
        }
    } catch {}

    return config;
}

function encodeUploadPayload(target, files, remotePath, config) {
    return Buffer.from(JSON.stringify({ target, files, remotePath, config }), "utf8").toString("base64");
}

function hasDisplayServer() {
    return !!(process.env.DISPLAY || process.env.WAYLAND_DISPLAY);
}

function isWSL() {
    return process.platform === "linux" && (
        !!process.env.WSL_DISTRO_NAME ||
        /microsoft/i.test(os.release()) ||
        /microsoft/i.test(process.env.OSTYPE || "")
    );
}

function spawnDetached(command, args, options = {}) {
    return new Promise((resolve) => {
        let settled = false;
        try {
            const child = spawn(command, args, {
                detached: options.detached !== false,
                stdio: options.stdio || "ignore",
                windowsVerbatimArguments: !!options.windowsVerbatimArguments,
            });
            child.once("error", () => {
                if (!settled) {
                    settled = true;
                    resolve(false);
                }
            });
            child.once("spawn", () => {
                child.unref();
                if (!settled) {
                    settled = true;
                    resolve(true);
                }
            });
        } catch {
            resolve(false);
        }
    });
}

async function openUploadTerminal(target, files, remotePath) {
    const runnerScript = path.join(__dirname, "upload_runner.js");
    const payload = encodeUploadPayload(target, files, remotePath, readQsshConfig());
    const nodeExe = process.execPath;

    if (process.platform === "win32") {
        const escapedRunner = runnerScript.replace(/'/g, "''");
        const escapedPayload = payload.replace(/'/g, "''");
        const startProcessCommand = [
            "$runner = '" + escapedRunner + "'",
            "$payload = '" + escapedPayload + "'",
            "$cmd = 'node ' + [char]34 + $runner + [char]34 + ' ' + [char]34 + $payload + [char]34",
            "Start-Process powershell -ArgumentList @('-Command',$cmd)",
        ].join("; ");
        if (await spawnDetached("powershell.exe", [
            "-NoProfile",
            "-Command",
            startProcessCommand,
        ], { detached: false })) {
            return true;
        }
        const title = "Quick-SSH Upload";
        return spawnDetached("cmd.exe", [
            "/c",
            "start",
            `"${title}"`,
            "powershell.exe",
            "-Command",
            `"${nodeExe}" "${runnerScript}" "${payload}"`,
        ], { detached: false, windowsVerbatimArguments: true });
    }

    if (isWSL()) {
        const distro = process.env.WSL_DISTRO_NAME || "";
        const escapedRunner = runnerScript.replace(/\\/g, "\\\\").replace(/"/g, '\\"');
        const escapedPayload = payload.replace(/"/g, '\\"');
        const escapedDistro = distro.replace(/"/g, '\\"');
        const wslCommand = `wsl.exe${distro ? ` -d "${escapedDistro}"` : ""} node "${escapedRunner}" "${escapedPayload}"`;

        if (await spawnDetached("wt.exe", [
            "new-tab",
            "powershell.exe",
            "-NoExit",
            "-Command",
            wslCommand,
        ], { detached: false })) {
            return true;
        }

        return spawnDetached("powershell.exe", [
            "-NoProfile",
            "-Command",
            `Start-Process powershell -ArgumentList @('-Command','${wslCommand.replace(/'/g, "''")}')`,
        ], { detached: false });
    }

    if (process.platform === "darwin") {
        const escapedRunner = runnerScript.replace(/"/g, '\\"');
        const escapedPayload = payload.replace(/"/g, '\\"');
        const script = `tell application "Terminal" to do script "${nodeExe} \\"${escapedRunner}\\" \\"${escapedPayload}\\""`;
        return spawnDetached("osascript", ["-e", script]);
    }

    if (process.platform === "linux") {
        if (hasDisplayServer()) {
            const candidates = [
                { cmd: "x-terminal-emulator", args: ["-e", nodeExe, runnerScript, payload] },
                { cmd: "gnome-terminal", args: ["--", nodeExe, runnerScript, payload] },
                { cmd: "konsole", args: ["-e", nodeExe, runnerScript, payload] },
                { cmd: "xfce4-terminal", args: ["--command", `${nodeExe} "${runnerScript}" "${payload}"`] },
                { cmd: "xterm", args: ["-e", nodeExe, runnerScript, payload] },
            ];

            for (const candidate of candidates) {
                if (await spawnDetached(candidate.cmd, candidate.args)) {
                    return true;
                }
            }
        }

        if (process.env.TMUX) {
            return spawnDetached("tmux", [
                "split-window",
                "-v",
                `${nodeExe} "${runnerScript}" "${payload}"`,
            ], { detached: false });
        }

        if (process.env.STY) {
            return spawnDetached("screen", [
                "-X",
                "screen",
                `${nodeExe} "${runnerScript}" "${payload}"`,
            ], { detached: false });
        }
    }
    return false;
}

function isRegularOrDirectory(filePath) {
    try {
        const stat = fs.statSync(filePath);
        return stat.isFile() || stat.isDirectory();
    } catch {
        return false;
    }
}

function stripPasteMarkers(text) {
    return text.replace(/\x1b\[200~/g, "").replace(/\x1b\[201~/g, "");
}

function splitShellWords(text) {
    const tokens = [];
    let current = "";
    let quote = null;

    for (let i = 0; i < text.length; i++) {
        const ch = text[i];

        if (quote) {
            if (ch === quote) {
                quote = null;
                continue;
            }
            if (ch === "\\" && quote === '"' && i + 1 < text.length) {
                current += text[++i];
                continue;
            }
            current += ch;
            continue;
        }

        if (ch === "'" || ch === '"') {
            quote = ch;
            continue;
        }
        if (ch === "\\") {
            if (i + 1 < text.length) {
                current += text[++i];
            } else {
                current += ch;
            }
            continue;
        }
        if (/\s/.test(ch)) {
            if (current) {
                tokens.push(current);
                current = "";
            }
            continue;
        }
        current += ch;
    }

    if (quote) {
        return null;
    }
    if (current) {
        tokens.push(current);
    }
    return tokens;
}

function parseWindowsDrag(text) {
    const files = [];
    let index = 0;

    while (index < text.length) {
        while (index < text.length && /\s/.test(text[index])) {
            index++;
        }
        if (index >= text.length) {
            break;
        }

        let token = "";
        if (text[index] === '"') {
            const end = text.indexOf('"', index + 1);
            if (end < 0) {
                return { prefix: true, files: null };
            }
            token = text.slice(index + 1, end);
            index = end + 1;
        } else {
            const match = text.slice(index).match(/^[A-Za-z]:\\[^\r\n\t ]*/);
            if (!match) {
                return { prefix: false, files: null };
            }
            token = match[0];
            index += token.length;
        }

        if (!/^[A-Za-z]:\\/.test(token) || !isRegularOrDirectory(token)) {
            return { prefix: false, files: null };
        }
        files.push(token);
    }

    return files.length > 0 ? { prefix: true, files } : { prefix: false, files: null };
}

function parseUnixDrag(text) {
    const tokens = splitShellWords(text);
    if (!tokens || tokens.length === 0) {
        return { prefix: false, files: null };
    }

    const files = [];
    for (const token of tokens) {
        let normalized = token;
        if (isWSL() && /^[A-Za-z]:\\/.test(token)) {
            const drive = token[0].toLowerCase();
            normalized = `/mnt/${drive}/${token.slice(3).replace(/\\/g, "/")}`;
        }
        if (!normalized.startsWith("/") || !isRegularOrDirectory(normalized)) {
            return { prefix: false, files: null };
        }
        files.push(normalized);
    }
    return { prefix: true, files };
}

function detectDragFiles(text) {
    const cleaned = stripPasteMarkers(text).replace(/\u0010/g, "");
    if (cleaned.length < 2) {
        return { prefix: false, files: null };
    }
    return process.platform === "win32" ? parseWindowsDrag(cleaned) : parseUnixDrag(cleaned);
}

function createDragDetector(onDrag, onFallback) {
    let timer = null;
    let buffer = "";

    function flush() {
        const input = buffer;
        buffer = "";
        timer = null;

        const drag = detectDragFiles(input);
        if (drag.files && drag.files.length > 0) {
            onDrag(drag.files);
            return;
        }
        onFallback(input);
    }

    return {
        handle(chunk) {
            const text = chunk.toString("utf8");
            if (buffer) {
                buffer += text;
                clearTimeout(timer);
                timer = setTimeout(flush, 250);
                return true;
            }

            const drag = detectDragFiles(text);
            if (!drag.prefix) {
                return false;
            }

            buffer = text;
            timer = setTimeout(flush, 250);
            return true;
        },
        reset() {
            if (timer) {
                clearTimeout(timer);
            }
            timer = null;
            buffer = "";
        },
    };
}

function startInteractiveSession(target, options = {}) {
    const ssh = spawn(getSSHExe(), buildSSHArgs(target), {
        stdio: ["pipe", "pipe", "pipe"],
    });

    let rawMode = false;
    let closed = false;
    let uploading = false;
    let probeBuffer = "";
    let probeResolver = null;
    let probeRejector = null;
    let probeTimer = null;

    const writeStatus = options.writeStatus || ((text) => process.stdout.write(text));
    const onExit = options.onExit || (() => {});

    function restoreInput() {
        try {
            if (rawMode && process.stdin.isTTY) {
                process.stdin.setRawMode(false);
            }
        } catch {}
        rawMode = false;
        process.stdin.pause();
        process.stdin.removeListener("data", onStdin);
    }

    function finish(code) {
        if (closed) {
            return;
        }
        closed = true;
        detector.reset();
        restoreInput();
        onExit(code);
    }

    function forwardInput(data) {
        if (!ssh.killed && ssh.stdin.writable) {
            ssh.stdin.write(data);
        }
    }

    function clearProbeState() {
        probeBuffer = "";
        if (probeTimer) {
            clearTimeout(probeTimer);
            probeTimer = null;
        }
        probeResolver = null;
        probeRejector = null;
    }

    function resolveRemotePwd() {
        return new Promise((resolve, reject) => {
            clearProbeState();
            probeResolver = resolve;
            probeRejector = reject;
            probeTimer = setTimeout(() => {
                const rejectFn = probeRejector;
                clearProbeState();
                if (rejectFn) {
                    rejectFn(new Error("timeout"));
                }
            }, 1500);
            forwardInput(Buffer.from(REMOTE_PWD_COMMAND, "utf8"));
        });
    }

    async function uploadFiles(files, remotePath) {
        if (uploading) {
            writeStatus("\n\x1b[33m已有拖拽上传任务正在执行，请稍后重试。\x1b[0m\n");
            return;
        }

        uploading = true;
        const opened = await openUploadTerminal(target, files, remotePath);
        uploading = false;

        if (opened) {
            writeStatus(`\n\x1b[36m[Quick-SSH] 已在新终端中启动上传任务，目标目录: ${remotePath}\x1b[0m\n`);
            forwardInput(Buffer.from("\n"));
            return;
        }

        writeStatus(`\n\x1b[31m[Quick-SSH] 无法打开新的终端窗口，请检查本机终端程序配置。\x1b[0m\n`);
    }

    const detector = createDragDetector(triggerUpload, (text) => forwardInput(Buffer.from(text, "utf8")));

    function onStdin(data) {
        if (uploading) {
            return;
        }
        if (!detector.handle(data)) {
            forwardInput(data);
        }
    }

    function handleSSHStdout(data) {
        if (!probeResolver) {
            process.stdout.write(data);
            return;
        }

        probeBuffer += data.toString("utf8");
        const escapedPrefix = REMOTE_PWD_PREFIX.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
        const escapedSuffix = REMOTE_PWD_SUFFIX.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
        const regexp = new RegExp(`${escapedPrefix}([^\r\n]+)${escapedSuffix}`);
        const match = probeBuffer.match(regexp);
        if (!match) {
            return;
        }

        const remotePath = match[1].trim() || ".";
        const resolveFn = probeResolver;
        clearProbeState();
        if (resolveFn) {
            resolveFn(remotePath);
        }
    }

    try {
        if (process.stdin.isTTY) {
            process.stdin.setRawMode(true);
            rawMode = true;
        }
    } catch {}

    process.stdin.resume();
    process.stdin.on("data", onStdin);

    ssh.stdout.on("data", (data) => handleSSHStdout(data));
    ssh.stderr.on("data", (data) => process.stderr.write(data));

    ssh.on("error", (error) => {
        writeStatus(`\n\x1b[31mSSH 启动失败: ${error.message}\x1b[0m\n`);
        finish(1);
    });

    ssh.on("exit", (code) => finish(code != null ? code : 0));

    detector.reset = ((originalReset) => () => {
        originalReset.call(detector);
        clearProbeState();
    })(detector.reset);

    function triggerUpload(files) {
        if (uploading) {
            writeStatus("\n\x1b[33m已有拖拽上传任务正在执行，请稍后重试。\x1b[0m\n");
            return;
        }
        forwardInput(Buffer.from([0x03]));
        setTimeout(() => {
            resolveRemotePwd()
                .then((remotePath) => uploadFiles(files, remotePath))
                .catch(() => uploadFiles(files, "."));
        }, 200);
    }

    return ssh;
}

module.exports = {
    getSSHExe,
    startInteractiveSession,
};
