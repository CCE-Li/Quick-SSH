#!/usr/bin/env node
/**
 * Verify that the npm package contains native binaries for every supported
 * platform. This prevents publishing a package that accidentally contains
 * Windows-built binaries for Linux/macOS.
 */

const fs = require("fs");
const path = require("path");

const ROOT_DIR = path.resolve(__dirname, "..");
const BIN_DIR = path.join(ROOT_DIR, "dist", "bin");

const PE_AMD64 = 0x8664;
const ELF_X86_64 = 0x3e;
const MACHO_X86_64 = 0x01000007;

function rel(filePath) {
    return path.relative(ROOT_DIR, filePath).replace(/\\/g, "/");
}

function readHeader(filePath) {
    const fd = fs.openSync(filePath, "r");
    try {
        const stat = fs.fstatSync(fd);
        const length = Math.min(stat.size, 8192);
        const buffer = Buffer.alloc(length);
        fs.readSync(fd, buffer, 0, length, 0);
        return buffer;
    } finally {
        fs.closeSync(fd);
    }
}

function hex(bytes) {
    return [...bytes].map((b) => `0x${b.toString(16).padStart(2, "0")}`).join(" ");
}

function describeFormat(buffer) {
    if (buffer.length < 4) return "too small";
    if (buffer[0] === 0x4d && buffer[1] === 0x5a) return "Windows PE/MZ";
    if (buffer[0] === 0x7f && buffer[1] === 0x45 && buffer[2] === 0x4c && buffer[3] === 0x46) return "Linux ELF";
    if (buffer[0] === 0xcf && buffer[1] === 0xfa && buffer[2] === 0xed && buffer[3] === 0xfe) return "Mach-O 64-bit little-endian";
    if (buffer[0] === 0xfe && buffer[1] === 0xed && buffer[2] === 0xfa && buffer[3] === 0xcf) return "Mach-O 64-bit big-endian";
    if (buffer[0] === 0xca && buffer[1] === 0xfe && buffer[2] === 0xba && buffer[3] === 0xbe) return "Mach-O universal/fat";
    if (buffer[0] === 0xbe && buffer[1] === 0xba && buffer[2] === 0xfe && buffer[3] === 0xca) return "Mach-O universal/fat reverse";
    return `unknown (${hex(buffer.subarray(0, 4))})`;
}

function fail(message) {
    return { ok: false, message };
}

function pass(message) {
    return { ok: true, message };
}

function validatePeX64(buffer) {
    if (buffer.length < 0x40 || buffer[0] !== 0x4d || buffer[1] !== 0x5a) {
        return fail(`expected Windows PE/MZ, got ${describeFormat(buffer)}`);
    }

    const peOffset = buffer.readUInt32LE(0x3c);
    if (buffer.length < peOffset + 6) {
        return fail("PE header is outside the readable file header");
    }

    const signature = buffer.toString("ascii", peOffset, peOffset + 4);
    if (signature !== "PE\u0000\u0000") {
        return fail("missing PE signature");
    }

    const machine = buffer.readUInt16LE(peOffset + 4);
    if (machine !== PE_AMD64) {
        return fail(`expected PE x64 machine 0x${PE_AMD64.toString(16)}, got 0x${machine.toString(16)}`);
    }

    return pass("Windows PE x64");
}

function validateElfX64(buffer) {
    if (buffer.length < 20 || buffer[0] !== 0x7f || buffer[1] !== 0x45 || buffer[2] !== 0x4c || buffer[3] !== 0x46) {
        return fail(`expected Linux ELF, got ${describeFormat(buffer)}`);
    }

    const elfClass = buffer[4];
    if (elfClass !== 2) {
        return fail(`expected 64-bit ELF class 2, got ${elfClass}`);
    }

    const endian = buffer[5];
    const machine = endian === 2 ? buffer.readUInt16BE(18) : buffer.readUInt16LE(18);
    if (machine !== ELF_X86_64) {
        return fail(`expected ELF x86_64 machine 0x${ELF_X86_64.toString(16)}, got 0x${machine.toString(16)}`);
    }

    return pass("Linux ELF x64");
}

function validateMachOX64(buffer) {
    if (buffer.length < 8) {
        return fail("file is too small for a Mach-O header");
    }

    const magic = buffer.subarray(0, 4);
    const magicHex = hex(magic);

    if (magicHex === "0xcf 0xfa 0xed 0xfe") {
        const cpuType = buffer.readUInt32LE(4);
        return cpuType === MACHO_X86_64
            ? pass("Mach-O x64")
            : fail(`expected Mach-O x64 cpu type 0x${MACHO_X86_64.toString(16)}, got 0x${cpuType.toString(16)}`);
    }

    if (magicHex === "0xfe 0xed 0xfa 0xcf") {
        const cpuType = buffer.readUInt32BE(4);
        return cpuType === MACHO_X86_64
            ? pass("Mach-O x64")
            : fail(`expected Mach-O x64 cpu type 0x${MACHO_X86_64.toString(16)}, got 0x${cpuType.toString(16)}`);
    }

    if (magicHex === "0xca 0xfe 0xba 0xbe" || magicHex === "0xbe 0xba 0xfe 0xca") {
        const littleEndian = magicHex === "0xbe 0xba 0xfe 0xca";
        const readUInt32 = littleEndian ? Buffer.prototype.readUInt32LE : Buffer.prototype.readUInt32BE;
        const archCount = readUInt32.call(buffer, 4);
        const maxArchCount = Math.min(archCount, Math.floor((buffer.length - 8) / 20));

        for (let i = 0; i < maxArchCount; i += 1) {
            const offset = 8 + i * 20;
            const cpuType = readUInt32.call(buffer, offset);
            if (cpuType === MACHO_X86_64) {
                return pass("Mach-O universal/fat with x64 slice");
            }
        }

        return fail("Mach-O universal/fat binary does not contain an x64 slice");
    }

    return fail(`expected macOS Mach-O, got ${describeFormat(buffer)}`);
}

const targets = [
    { name: "Windows x64", file: "qssh-win.exe", validate: validatePeX64 },
    { name: "Linux x64", file: "qssh-linux", validate: validateElfX64 },
    { name: "macOS x64", file: "qssh-darwin", validate: validateMachOX64 },
];

const runtimeFiles = [
    path.join(ROOT_DIR, "dist", "usr", "xterm"),
    path.join(ROOT_DIR, "dist", "usr", "xterm-256color"),
];

let hasError = false;

for (const target of targets) {
    const filePath = path.join(BIN_DIR, target.file);

    if (!fs.existsSync(filePath)) {
        console.error(`[verify] ${target.name}: missing ${rel(filePath)}`);
        hasError = true;
        continue;
    }

    const stat = fs.statSync(filePath);
    if (!stat.isFile()) {
        console.error(`[verify] ${target.name}: not a regular file: ${rel(filePath)}`);
        hasError = true;
        continue;
    }

    const result = target.validate(readHeader(filePath));
    if (!result.ok) {
        console.error(`[verify] ${target.name}: ${rel(filePath)} is invalid: ${result.message}`);
        hasError = true;
        continue;
    }

    console.log(`[verify] ${target.name}: ${rel(filePath)} (${result.message})`);
}

for (const filePath of runtimeFiles) {
    if (!fs.existsSync(filePath) || !fs.statSync(filePath).isFile()) {
        console.error(`[verify] runtime assets: missing ${rel(filePath)}`);
        hasError = true;
        continue;
    }

    console.log(`[verify] runtime assets: ${rel(filePath)}`);
}

if (hasError) {
    console.error("[verify] package binary verification failed");
    process.exit(1);
}

console.log("[verify] package binary verification passed");
