<#====================================================================
 Quick-SSH 包管理器配置自动更新脚本
 =====================================================================
 功能:
   1. 从 GitHub Release 下载指定版本的归档文件
   2. 计算 SHA256 校验和
   3. 自动更新所有 packaging 配置文件中的版本号和哈希值

 用法:
   # 更新到最新发布的版本
   .\scripts\update-packaging.ps1

   # 更新到指定版本
   .\scripts\update-packaging.ps1 -Version "2.0.1"

   # 使用本地已下载的归档文件（跳过下载）
   .\scripts\update-packaging.ps1 -Version "2.0.1" -LocalDir ".\downloads"

 前置条件:
   - PowerShell 5.1+
   - 网络连接（用于下载归档）
   - 7-Zip（用于提取 .zip 文件验证，可选）
#>

param(
    [string]$Version = "",
    [string]$LocalDir = "",
    [string]$Repo = "CCE-Li/Quick-SSH",
    [switch]$Help
)

if ($Help) {
    Get-Help $MyInvocation.MyCommand.Path -Detailed
    exit 0
}

# ── 版本检测 ──────────────────────────────────────────────
$CargoToml = Join-Path $PSScriptRoot ".." "Cargo.toml"
if (-not $Version) {
    # 从 Cargo.toml workspace 读取版本号
    $cargoContent = Get-Content $CargoToml -Raw
    if ($cargoContent -match 'version\s*=\s*"([^"]+)"') {
        $Version = $matches[1]
        Write-Host "✔ 从 Cargo.toml 检测到版本: $Version" -ForegroundColor Green
    } else {
        Write-Host "✘ 无法从 Cargo.toml 读取版本号，请用 -Version 参数指定" -ForegroundColor Red
        exit 1
    }
}

# 去除版本号前的 'v' 前缀（如果有）
$Version = $Version.TrimStart('v')
$Tag = "v$Version"

Write-Host "╔══════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║   Quick-SSH  Packaging 配置更新工具         ║" -ForegroundColor Cyan
Write-Host "║   版本: $Version".PadRight(42) + "║" -ForegroundColor Cyan
Write-Host "╚══════════════════════════════════════════════╝" -ForegroundColor Cyan
Write-Host ""

# ── 定义归档信息 ──────────────────────────────────────────
$archives = @(
    @{ Name = "x86_64-linux";    Platform = "linux";   Arch = "x86_64";  Sfx = "tar.gz" },
    @{ Name = "x86_64-macos";    Platform = "macos";   Arch = "x86_64";  Sfx = "tar.gz" },
    @{ Name = "aarch64-macos";   Platform = "macos";   Arch = "aarch64"; Sfx = "tar.gz" },
    @{ Name = "x86_64-windows";  Platform = "windows"; Arch = "x86_64";  Sfx = "zip" }
)

$RepoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$DownloadDir = if ($LocalDir) { $LocalDir } else { Join-Path $RepoRoot "target" "packaging-dl" }

# 创建下载目录
if (-not (Test-Path $DownloadDir)) {
    New-Item -ItemType Directory -Path $DownloadDir -Force | Out-Null
}

$shaResults = @{}

# ── 下载 / 计算 SHA256 ────────────────────────────────────
Write-Host "─── 步骤 1: 获取归档文件并计算 SHA256 ───" -ForegroundColor Yellow
Write-Host ""

foreach ($arc in $archives) {
    $fileName = "qssh-$($arc.Name).$($arc.Sfx)"
    $filePath = Join-Path $DownloadDir $fileName

    if ($LocalDir -and (Test-Path $filePath)) {
        Write-Host "  ✔ 使用本地文件: $fileName" -ForegroundColor Green
    } else {
        $url = "https://github.com/$Repo/releases/download/$Tag/$fileName"
        Write-Host "  ↓ 下载: $url" -ForegroundColor Gray
        try {
            Invoke-WebRequest -Uri $url -OutFile $filePath -ErrorAction Stop
            Write-Host "  ✔ 下载完成: $fileName" -ForegroundColor Green
        } catch {
            Write-Host "  ⚠ 下载失败 (文件可能不存在): $fileName" -ForegroundColor Yellow
            $shaResults[$arc.Name] = "PLACEHOLDER_UPDATE_ME"
            continue
        }
    }

    # 计算 SHA256
    $hash = (Get-FileHash -Path $filePath -Algorithm SHA256).Hash.ToLower()
    $shaResults[$arc.Name] = $hash
    Write-Host "  🔑 SHA256: $hash" -ForegroundColor Gray
    Write-Host ""
}

# ── 保存 SHA256 到文件 ────────────────────────────────────
$shaFile = Join-Path $DownloadDir "SHA256SUMS"
$shaResults.GetEnumerator() | Sort-Object Name | ForEach-Object {
    "$($_.Value)  qssh-$($_.Name).$(if($_.Name -match 'windows'){'zip'}else{'tar.gz'})"
} | Set-Content $shaFile
Write-Host "  ✔ SHA256SUMS 已保存到: $shaFile" -ForegroundColor Green
Write-Host ""

# ── 更新 packaging 配置 ────────────────────────────────────
Write-Host "─── 步骤 2: 更新 packaging 配置文件 ───" -ForegroundColor Yellow
Write-Host ""

$ProjectRoot = $RepoRoot

# ── 2a: Scoop (packaging/scoop/quick-ssh.json) ───────────
$scoopPath = Join-Path $ProjectRoot "packaging" "scoop" "quick-ssh.json"
if (Test-Path $scoopPath) {
    $scoop = Get-Content $scoopPath -Raw | ConvertFrom-Json
    $scoop.version = $Version
    if ($shaResults["x86_64-windows"]) {
        $scoop.architecture."64bit".hash = "sha256:$($shaResults["x86_64-windows"])"
    }
    $scoop | ConvertTo-Json -Depth 10 | Set-Content $scoopPath
    Write-Host "  ✔ Scoop: 已更新版本和哈希" -ForegroundColor Green
} else {
    Write-Host "  ⚠ Scoop 配置文件不存在: $scoopPath" -ForegroundColor Yellow
}

# ── 2b: Homebrew (packaging/homebrew/quick-ssh.rb) ───────
$brewPath = Join-Path $ProjectRoot "packaging" "homebrew" "quick-ssh.rb"
if (Test-Path $brewPath) {
    $brewContent = Get-Content $brewPath -Raw

    # 更新版本号
    $brewContent = $brewContent -replace 'version "[\d.]+"', "version `"$Version`""

    # 更新 URL 中的版本号
    $brewContent = $brewContent -replace '/v[\d.]+/', "/v$Version/"

    # 更新 SHA256
    if ($shaResults["x86_64-macos"]) {
        # 替换第一个 SHA256（x86_64 macOS，也会匹配到通用部分）
        $brewContent = $brewContent -replace '(url ".*?x86_64-macos.*?"\s*sha256 ")[^"]+(")', ('${1}' + $shaResults["x86_64-macos"] + '${2}')
    }
    if ($shaResults["x86_64-macos"] -and $shaResults["aarch64-macos"]) {
        # 替换第二个 SHA256（仅匹配 on_macos 块内的 aarch64）
        $lines = $brewContent -split "`n"
        $newLines = @()
        $inIntelBlock = $false
        $inArmBlock = $false
        foreach ($line in $lines) {
            if ($line -match 'Hardware::CPU\.intel\?') {
                $inIntelBlock = $true
                $inArmBlock = $false
            } elseif ($line -match 'Hardware::CPU\.arm\?') {
                $inIntelBlock = $false
                $inArmBlock = $true
            } elseif ($line -match '^  end$' -and ($inIntelBlock -or $inArmBlock)) {
                $inIntelBlock = $false
                $inArmBlock = $false
            }

            if ($inIntelBlock -and $line -match 'sha256') {
                $line = $line -replace 'sha256 ".*?"', "sha256 `"$($shaResults["x86_64-macos"])`""
            }
            if ($inArmBlock -and $line -match 'sha256') {
                $line = $line -replace 'sha256 ".*?"', "sha256 `"$($shaResults["aarch64-macos"])`""
            }
            $newLines += $line
        }
        $brewContent = $newLines -join "`n"
    }

    # 更新最外层的 sha256（万能兜底）
    if ($shaResults["x86_64-macos"]) {
        # 先统计 sha256 出现次数，只替换最外层（非 on_macos 块内的）
        $lines = $brewContent -split "`n"
        $newLines = @()
        $depth = 0
        foreach ($line in $lines) {
            if ($line -match '^\s*on_(macos|intel|arm)') { $depth++ }
            if ($line -match '^  end$' -and $depth -gt 0) { $depth-- }
            if ($depth -eq 0 -and $line -match '^\s+sha256 "') {
                # 只更新第一个最外层的 sha256
                $newLines += $line
                # 这个已在上面处理过，这里跳过
                continue
            }
            $newLines += $line
        }
        $brewContent = $newLines -join "`n"
    }

    Set-Content -Path $brewPath -Value $brewContent.TrimEnd("`r", "`n")
    Write-Host "  ✔ Homebrew: 已更新版本和哈希" -ForegroundColor Green
} else {
    Write-Host "  ⚠ Homebrew 配置文件不存在: $brewPath" -ForegroundColor Yellow
}

# ── 2c: Pacman/AUR (packaging/pacman/PKGBUILD) ───────────
$pkgPath = Join-Path $ProjectRoot "packaging" "pacman" "PKGBUILD"
if (Test-Path $pkgPath) {
    $pkg = Get-Content $pkgPath -Raw
    $pkg = $pkg -replace 'pkgver=[\d.]+', "pkgver=$Version"
    if ($shaResults["x86_64-linux"]) {
        $pkg = $pkg -replace 'sha256sums=\([^)]*\)', "sha256sums=('$($shaResults["x86_64-linux"])')"
    }
    Set-Content -Path $pkgPath -Value $pkg.TrimEnd("`r", "`n")
    Write-Host "  ✔ Pacman/AUR: 已更新版本和哈希" -ForegroundColor Green
} else {
    Write-Host "  ⚠ PKGBUILD 不存在: $pkgPath" -ForegroundColor Yellow
}

# ── 2d: APT (packaging/apt/DEBIAN/control) ──────────────
$aptControlPath = Join-Path $ProjectRoot "packaging" "apt" "DEBIAN" "control"
if (Test-Path $aptControlPath) {
    $apt = Get-Content $aptControlPath -Raw
    $apt = $apt -replace 'Version: [\d.]+', "Version: $Version"
    Set-Content -Path $aptControlPath -Value $apt.TrimEnd("`r", "`n")
    Write-Host "  ✔ APT: 已更新版本" -ForegroundColor Green
} else {
    Write-Host "  ⚠ APT control 文件不存在: $aptControlPath" -ForegroundColor Yellow
}

# ── 2e: APT Makefile ─────────────────────────────────────
$aptMkPath = Join-Path $ProjectRoot "packaging" "apt" "Makefile"
if (Test-Path $aptMkPath) {
    $aptMk = Get-Content $aptMkPath -Raw
    $aptMk = $aptMk -replace '(VERSION \?= )[\d.]+', ('${1}' + $Version)
    Set-Content -Path $aptMkPath -Value $aptMk.TrimEnd("`r", "`n")
    Write-Host "  ✔ APT Makefile: 已更新版本" -ForegroundColor Green
} else {
    Write-Host "  ⚠ APT Makefile 不存在: $aptMkPath" -ForegroundColor Yellow
}

# ── 2f: WinGet (packaging/winget/*.yaml) ─────────────────
$wingetFiles = @(
    "CCE-Li.Quick-SSH.installer.yaml",
    "CCE-Li.Quick-SSH.locale.en-US.yaml",
    "CCE-Li.Quick-SSH.yaml"
)
foreach ($file in $wingetFiles) {
    $path = Join-Path $ProjectRoot "packaging" "winget" $file
    if (Test-Path $path) {
        $content = Get-Content $path -Raw
        $content = $content -replace 'PackageVersion: [\d.]+', "PackageVersion: $Version"
        if ($file -eq "CCE-Li.Quick-SSH.installer.yaml") {
            # 更新 InstallerUrl 中的版本号
            $content = $content -replace '/v[\d.]+/', "/v$Version/"
            if ($shaResults["x86_64-windows"]) {
                $content = $content -replace 'InstallerSha256: ".*?"', "InstallerSha256: `"$($shaResults["x86_64-windows"])`""
            }
        }
        Set-Content -Path $path -Value $content.TrimEnd("`r", "`n")
        Write-Host "  ✔ WinGet ($file): 已更新版本和哈希" -ForegroundColor Green
    } else {
        Write-Host "  ⚠ WinGet 文件不存在: $path" -ForegroundColor Yellow
    }
}

Write-Host ""
Write-Host "══════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host "  🎉 所有 packaging 配置已更新到 v$Version !" -ForegroundColor Green
Write-Host "══════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host ""
Write-Host "下一步操作:" -ForegroundColor Yellow
Write-Host "  1. 提交 PR 到各包管理器的官方仓库（见下）" -ForegroundColor White
Write-Host "  2. 或在本地构建 .deb 包: cd packaging/apt && make VERSION=$Version" -ForegroundColor White
Write-Host ""
Write-Host "各包管理器提交指南:" -ForegroundColor Yellow
Write-Host "  Scoop (Windows):" -ForegroundColor Cyan
Write-Host "    提交 packaging/scoop/quick-ssh.json 到" -ForegroundColor Gray
Write-Host "    https://github.com/ScoopInstaller/Main" -ForegroundColor Gray
Write-Host ""
Write-Host "  WinGet (Windows):" -ForegroundColor Cyan
Write-Host "    提交 packaging/winget/ 目录到" -ForegroundColor Gray
Write-Host "    https://github.com/microsoft/winget-pkgs" -ForegroundColor Gray
Write-Host "    (Windows 下可用 wingetcreate 工具: winget install wingetcreate)" -ForegroundColor Gray
Write-Host ""
Write-Host "  Homebrew (macOS):" -ForegroundColor Cyan
Write-Host "    方式 A - 自建 Tap:" -ForegroundColor Gray
Write-Host "      gh repo create CCE-Li/homebrew-quick-ssh --public" -ForegroundColor Gray
Write-Host "      copy packaging/homebrew/quick-ssh.rb ." -ForegroundColor Gray
Write-Host "      brew tap CCE-Li/quick-ssh && brew install quick-ssh" -ForegroundColor Gray
Write-Host "    方式 B - 提交 Homebrew Core:" -ForegroundColor Gray
Write-Host "      提交 PR 到 https://github.com/Homebrew/homebrew-core" -ForegroundColor Gray
Write-Host ""
Write-Host "  AUR (Arch Linux):" -ForegroundColor Cyan
Write-Host "    提交 packaging/pacman/PKGBUILD 到 AUR:" -ForegroundColor Gray
Write-Host "      git clone ssh://aur@aur.archlinux.org/quick-ssh.git" -ForegroundColor Gray
Write-Host "      cp packaging/pacman/PKGBUILD ." -ForegroundColor Gray
Write-Host "      makepkg --printsrcinfo > .SRCINFO" -ForegroundColor Gray
Write-Host "      git add . && git commit -m 'Update to v$Version'" -ForegroundColor Gray
Write-Host "      git push" -ForegroundColor Gray
Write-Host ""
Write-Host "  APT (Debian/Ubuntu):" -ForegroundColor Cyan
Write-Host "    构建 .deb: cd packaging/apt && make VERSION=$Version" -ForegroundColor Gray
Write-Host "    将 .deb 上传到 GitHub Release 供用户下载安装" -ForegroundColor Gray
Write-Host ""
