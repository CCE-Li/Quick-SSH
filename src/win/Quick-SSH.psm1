# Quick-SSH.psm1 - PowerShell SSH Connection Manager (Cross-Platform)
# 仿 Docker 命令行风格的 SSH 连接管理工具
# 数据存储于标准 OpenSSH 配置文件 ~/.ssh/config

# ============================================================
# 内部函数 - 配置管理
# ============================================================

# 跨平台用户主目录检测
#   Windows: $env:USERPROFILE  (C:\Users\xxx)
#   Linux:   $env:HOME         (/home/xxx)
$Script:UserHome     = if ($env:USERPROFILE) { $env:USERPROFILE } else { $env:HOME }
$Script:SSHConfigDir = Join-Path $Script:UserHome ".ssh"
$Script:SSHConfigPath= Join-Path $Script:SSHConfigDir "config"
# ModuleRoot = src/ (psm1 在 src/win/ 下，$PSScriptRoot 指向 src/win/)
$Script:ModuleRoot   = Split-Path $PSScriptRoot -Parent
$Script:TUIScript    = [System.IO.Path]::Combine($Script:ModuleRoot, "tui", "index.js")

# 跨平台 SSH 可执行文件检测
$Script:SSHExe = if ($IsWindows -or $env:OS -match "Windows") {
    if (Get-Command "ssh.exe" -ErrorAction SilentlyContinue) { "ssh.exe" } else { "ssh" }
} else {
    if (Get-Command "ssh" -ErrorAction SilentlyContinue) { "ssh" } else { "ssh" }
}

# 跨平台判断：是否运行在 Windows 上
$Script:IsWindows = $IsWindows -or ($env:OS -match "Windows")

# 确保 ~/.ssh/config 存在
function Initialize-QuickSSHConfig {
    if (-not (Test-Path $Script:SSHConfigDir)) {
        New-Item -Path $Script:SSHConfigDir -ItemType Directory -Force | Out-Null
    }
    if (-not (Test-Path $Script:SSHConfigPath)) {
        "" | Set-Content -Path $Script:SSHConfigPath -Encoding UTF8 -NoNewline
    }
}

# 从 ~/.ssh/config 解析所有 Quick-SSH 管理的 Host 块
function Get-QuickSSHHosts {
    Initialize-QuickSSHConfig
    try {
        $raw = Get-Content -Path $Script:SSHConfigPath -Raw -Encoding UTF8
        if ([string]::IsNullOrWhiteSpace($raw)) { return @() }
        return Parse-SSHConfigHosts $raw
    } catch {
        return @()
    }
}

# SSH config 解析器：从文本中提取 Host 块
function Parse-SSHConfigHosts($content) {
    $hosts = @()
    $current = $null
    foreach ($line in ($content -split "`r`n|`n")) {
        $t = $line.Trim()
        if ([string]::IsNullOrWhiteSpace($t) -or $t.StartsWith("#")) { continue }
        if ($t -match "^Host\s+(.+)$") {
            if ($current) { $hosts += $current }
            $alias = $matches[1].Trim()
            $current = [PSCustomObject]@{ alias = $alias; host = ""; user = ""; port = 22; key = "" }
        } elseif ($current) {
            if ($t -match "^HostName\s+(.+)$")      { $current.host = $matches[1].Trim() }
            elseif ($t -match "^User\s+(.+)$")      { $current.user = $matches[1].Trim() }
            elseif ($t -match "^Port\s+(\d+)$")     { $current.port = [int]$matches[1] }
            elseif ($t -match "^IdentityFile\s+(.+)$") { $current.key = $matches[1].Trim() }
        }
    }
    if ($current) { $hosts += $current }
    # 只返回有效的（有 host 和 user 的）
    return @($hosts | Where-Object { $_.host -ne "" -and $_.user -ne "" })
}

# 将 host 对象渲染为 SSH config Host 块文本
function Render-SSHConfigBlock($h) {
    $lines = @()
    $lines += "Host $($h.alias)"
    if ($h.host) { $lines += "    HostName $($h.host)" }
    if ($h.user) { $lines += "    User $($h.user)" }
    $lines += "    Port $(if ($h.port) { $h.port } else { 22 })"
    if ($h.key)  { $lines += "    IdentityFile $($h.key)" }
    return $lines -join "`r`n"
}

# SSH config 块解析器：返回带行号范围的块，用于保存时做替换
function Parse-SSHConfigBlocks($content) {
    $blocks = @()
    $lines = $content -split "`r`n|`n"
    $i = 0
    while ($i -lt $lines.Count) {
        $t = $lines[$i].Trim()
        if ([string]::IsNullOrWhiteSpace($t) -or $t.StartsWith("#")) { $i++; continue }
        $m = [regex]::Match($t, "^Host\s+(.+)$")
        if (-not $m.Success) { $i++; continue }
        $start = $i
        $alias = $m.Groups[1].Value.Trim()
        $i++
        while ($i -lt $lines.Count) {
            $n = $lines[$i].Trim()
            if ([string]::IsNullOrWhiteSpace($n) -or $n.StartsWith("#")) { break }
            if ($n -match "^Host\s+") { break }
            $i++
        }
        $end = $i
        $blocks += [PSCustomObject]@{ alias = $alias; start = $start; end = $end }
    }
    return $blocks, $lines
}

# 保存全部主机配置到 ~/.ssh/config（保留非管理的 Host 块和注释）
function Save-QuickSSHHosts($Hosts) {
    Initialize-QuickSSHConfig
    $oldContent = Get-Content -Path $Script:SSHConfigPath -Raw -Encoding UTF8
    if ([string]::IsNullOrWhiteSpace($oldContent)) {
        # 空文件，直接写入
        $lines = @()
        foreach ($h in $Hosts) { $lines += Render-SSHConfigBlock $h; $lines += "" }
        $output = ($lines -join "`r`n").TrimEnd() + "`r`n"
        $output | Set-Content -Path $Script:SSHConfigPath -Encoding UTF8 -NoNewline
        return
    }

    $blocks, $lines = Parse-SSHConfigBlocks $oldContent
    $newMap = @{}
    foreach ($h in $Hosts) { $newMap[$h.alias] = $h }

    $replaced = @{}
    $result = @()
    $i = 0
    while ($i -lt $lines.Count) {
        $t = $lines[$i].Trim()
        $m = [regex]::Match($t, "^Host\s+(.+)$")
        if ($m.Success) {
            $alias = $m.Groups[1].Value.Trim()
            if ($newMap.ContainsKey($alias)) {
                # Quick-SSH 管理的 Host 块 → 替换
                $blk = $blocks | Where-Object { $_.alias -eq $alias } | Select-Object -First 1
                if ($blk) { $i = $blk.end }
                $result += (Render-SSHConfigBlock $newMap[$alias]) -split "`r`n|`n"
                $result += ""
                $replaced[$alias] = $true
            } else {
                # 非管理的 Host 块 → 原样保留
                $result += $lines[$i]
                $i++
                while ($i -lt $lines.Count) {
                    $n = $lines[$i].Trim()
                    if ([string]::IsNullOrWhiteSpace($n)) { $result += $lines[$i]; $i++; break }
                    if ($n.StartsWith("#")) { $result += $lines[$i]; $i++; continue }
                    if ($n -match "^Host\s+") { break }
                    $result += $lines[$i]; $i++
                }
            }
        } else {
            # 非 Host 行（注释、空行、全局选项）→ 原样保留
            $result += $lines[$i]; $i++
        }
    }

    # 追加全新的 Host 块
    foreach ($h in $Hosts) {
        if (-not $replaced.ContainsKey($h.alias)) {
            $result += (Render-SSHConfigBlock $h) -split "`r`n|`n"
            $result += ""
        }
    }

    $output = ($result -join "`r`n").TrimEnd() + "`r`n"
    $output | Set-Content -Path $Script:SSHConfigPath -Encoding UTF8 -NoNewline
}

# ============================================================
# 内部函数 - 颜色输出
# ============================================================

function Write-QSSuccess  { Write-Host $args[0] -ForegroundColor Green }
function Write-QSWarning { Write-Host $args[0] -ForegroundColor Yellow }
function Write-QSError   { Write-Host $args[0] -ForegroundColor Red }

# ============================================================
# 子命令实现
# ============================================================

# qssh ps [关键词] - 列出所有已保存的 SSH 连接
function Invoke-QuickSSHPs {
    param([string]$Keyword = "")

    $hosts = Get-QuickSSHHosts
    if ($hosts.Count -eq 0) {
        Write-QSWarning "当前没有已保存的 SSH 连接。使用 'qssh add' 添加一个。"
        return
    }

    if ($Keyword) {
        $kw = $Keyword.ToLower()
        $hosts = $hosts | Where-Object {
            $_.alias.ToLower().Contains($kw) -or
            $_.host.ToLower().Contains($kw)  -or
            $_.user.ToLower().Contains($kw)
        }
        if ($hosts.Count -eq 0) {
            Write-QSWarning "没有匹配 '$Keyword' 的 SSH 连接。"
            return
        }
    }

    $hosts | Format-Table -Property @(
        @{ Label = "别名";     Expression = { $_.alias } },
        @{ Label = "IP 地址";  Expression = { $_.host } },
        @{ Label = "账号";     Expression = { $_.user } },
        @{ Label = "端口";     Expression = { $_.port } },
        @{ Label = "私钥路径"; Expression = { $_.key } }
    ) -AutoSize
}

# qssh add [别名] [用户名@IP:端口] --key 私钥路径
function Invoke-QuickSSHAdd {
    param(
        [string]$Alias,
        [string]$UserAtHost,   # user@host 或 user@host:port
        [string]$KeyPath = ""
    )

    if ((-not $Alias) -or (-not $UserAtHost)) {
        Write-QSError "错误：用法 → qssh add <别名> <用户名@IP:端口> [--key <私钥路径>]"
        return
    }

    # 解析 user@host:port
    $user     = ""
    $hostname = ""
    $port     = 22

    if ($UserAtHost -match '^(.+)@(.+):(\d+)$') {
        $user     = $matches[1]
        $hostname = $matches[2]
        $port     = [int]$matches[3]
    } elseif ($UserAtHost -match '^(.+)@(.+)$') {
        $user     = $matches[1]
        $hostname = $matches[2]
    } else {
        Write-QSError "错误：格式无效，请使用「用户名@主机:端口」或「用户名@主机」格式。"
        return
    }

    if (-not $KeyPath) {
        $KeyPath = Join-Path $Script:UserHome ".ssh" "id_rsa"
    }

    # 检查别名重复
    $hosts    = Get-QuickSSHHosts
    $existing = $hosts | Where-Object { $_.alias -eq $Alias }
    if ($existing) {
        Write-QSError "错误：别名 '$Alias' 已存在，请使用其他名称。"
        return
    }

    $entry = @{
        alias = $Alias
        host  = $hostname
        user  = $user
        port  = $port
        key   = $KeyPath
    }
    $hosts += $entry
    Save-QuickSSHHosts $hosts

    Write-QSSuccess "✔ 已添加 SSH 连接 '$Alias' → $user@$hostname`:$port"
}

# qssh rm [别名] - 删除指定别名的 SSH 连接
function Invoke-QuickSSHRm {
    param([string]$Alias)

    if (-not $Alias) {
        Write-QSError "错误：用法 → qssh rm <别名>"
        return
    }

    $hosts  = Get-QuickSSHHosts
    $target = $hosts | Where-Object { $_.alias -eq $Alias }
    if (-not $target) {
        Write-QSError "错误：别名 '$Alias' 不存在。使用 'qssh ps' 查看可用连接。"
        return
    }

    $hosts = $hosts | Where-Object { $_.alias -ne $Alias }
    Save-QuickSSHHosts $hosts

    Write-QSSuccess "✔ 已删除 SSH 连接 '$Alias'。"
}

# qssh [别名] - 一键连接 SSH 服务器
function Invoke-QuickSSHConnect {
    param([string]$Alias)

    if (-not $Alias) {
        Invoke-QuickSSHTUI
        return
    }

    $hosts  = Get-QuickSSHHosts
    $target = $hosts | Where-Object { $_.alias -eq $Alias }
    if (-not $target) {
        Write-QSError "错误：别名 '$Alias' 不存在。使用 'qssh ps' 查看可用连接。"
        return
    }

    $sshExe = $Script:SSHExe

    Write-QSSuccess "正在连接到 '$Alias' ($($target.user)@$($target.host):$($target.port)) ..."
    Write-Host ""

    if ($Script:IsWindows) {
        # Windows: 使用 cmd /c 避免 PowerShell 拦截 SSH 交互
        # 将完整命令行编码后通过 cmd 启动，确保 SSH 获得原始 stdin 交互能力
        $cmd = "$sshExe -i `"$($target.key)`" -p $($target.port) -o HostKeyAlgorithms=+ssh-rsa $($target.user)@$($target.host)"
        $encoded = [Convert]::ToBase64String([Text.Encoding]::Unicode.GetBytes($cmd))
        cmd /c "powershell -NoProfile -EncodedCommand $encoded"
    } else {
        # Linux/macOS: 使用 & 调用运算符传参（避免字符串拼接的转义问题）
        # pwsh 原生支持交互式子进程，无需 cmd 中转
        & $sshExe -i $target.key -p $target.port -o HostKeyAlgorithms=+ssh-rsa "$($target.user)@$($target.host)"
    }
}

# qssh export [文件路径] - 导出全部主机配置到 JSON 文件
function Invoke-QuickSSHExport {
    param([string]$FilePath)

    if (-not $FilePath) {
        Write-QSError "错误：用法 → qssh export <文件路径>"
        return
    }

    $hosts = Get-QuickSSHHosts
    if ($hosts.Count -eq 0) {
        Write-QSWarning "没有可导出的 SSH 连接。"
        return
    }

    $hosts | ConvertTo-Json -Depth 10 | Set-Content -Path $FilePath -Encoding UTF8
    Write-QSSuccess "✔ 已导出 $($hosts.Count) 个连接到 '$FilePath'。"
}

# qssh import [文件路径] - 从 JSON 文件批量导入连接（自动去重）
function Invoke-QuickSSHImport {
    param([string]$FilePath)

    if (-not $FilePath) {
        Write-QSError "错误：用法 → qssh import <文件路径>"
        return
    }

    if (-not (Test-Path $FilePath)) {
        Write-QSError "错误：文件 '$FilePath' 不存在。"
        return
    }

    try {
        $imported = Get-Content -Path $FilePath -Raw -Encoding UTF8 | ConvertFrom-Json
    } catch {
        Write-QSError "错误：无法解析 JSON 文件，请检查格式。"
        return
    }

    if ($imported.Count -eq 0) {
        Write-QSWarning "文件中没有有效的连接配置。"
        return
    }

    $existing = Get-QuickSSHHosts
    $added   = 0
    $skipped = 0

    foreach ($h in $imported) {
        $dup = $existing | Where-Object { $_.alias -eq $h.alias }
        if (-not $dup) {
            $existing += $h
            $added++
        } else {
            $skipped++
        }
    }

    Save-QuickSSHHosts $existing
    Write-QSSuccess "✔ 导入完成：新增 $added 个，跳过 $skipped 个（别名重复）。"
}

# ============================================================
# TUI 终端界面
# ============================================================

function Invoke-QuickSSHTUI {
    # 检测 Node.js 是否可用
    $nodePath = (Get-Command "node" -ErrorAction SilentlyContinue).Source
    if (-not $nodePath) {
        Write-QSError "错误：启动 TUI 需要 Node.js，请先安装 Node.js (https://nodejs.org)"
        Write-QSError "或者使用命令行模式: qssh help"
        return
    }

    if (-not (Test-Path $Script:TUIScript)) {
        Write-QSError "错误：未找到 TUI 脚本: $Script:TUIScript"
        return
    }

    # 启动 TUI，等待退出后返回
    Write-Host "正在启动 Quick-SSH TUI ..." -ForegroundColor Cyan
    & "node" $Script:TUIScript
}

# ============================================================
# qssh init - 注册到 PowerShell 配置文件
# ============================================================

# 内部辅助函数：将 Import-Module 标记块写入单个配置文件
function Write-QuickSSHProfile {
    param([string]$ProfilePath)

    $modulePath = Join-Path $Script:ModuleRoot "win" "Quick-SSH.psm1"
    $importBlock = @"
# >>> Quick-SSH auto-generated (do not modify) >>>
# Quick-SSH PowerShell SSH 连接管理工具
# 安装路径: $modulePath
if (Test-Path "$modulePath") { Import-Module "$modulePath" -DisableNameChecking }
# <<< Quick-SSH auto-generated <<<
"@

    # 确保目录存在
    $profileDir = Split-Path $ProfilePath -Parent
    if (-not (Test-Path $profileDir)) {
        New-Item -Path $profileDir -ItemType Directory -Force | Out-Null
        Write-QSSuccess "✔ 已创建目录: $profileDir"
    }

    # 读取现有内容（文件可能不存在）
    $content = ""
    if (Test-Path $ProfilePath) {
        $content = Get-Content -Path $ProfilePath -Raw -Encoding UTF8
    }

    # 检查是否已注册，去重处理
    $markerStart = '# >>> Quick-SSH auto-generated (do not modify) >>>'
    if ($content -match [regex]::Escape($markerStart)) {
        $regex = [regex]::new(
            [regex]::Escape('# >>> Quick-SSH auto-generated (do not modify) >>>') +
            '[\s\S]*?' +
            [regex]::Escape('# <<< Quick-SSH auto-generated <<<')
        )
        $content = $regex.Replace($content, $importBlock)
        Write-QSSuccess "🔄 已更新: $ProfilePath"
    } else {
        $content = $content.TrimEnd() + "`r`n`r`n" + $importBlock + "`r`n"
        Write-QSSuccess "✔ 已写入: $ProfilePath"
    }

    $content | Set-Content -Path $ProfilePath -Encoding UTF8 -NoNewline
}

# qssh init - 注册到当前 PowerShell 版本的 $PROFILE
function Invoke-QuickSSHInit {

    $profilePath = $PROFILE
    if (-not $profilePath) {
        Write-QSError "错误：无法获取 `$PROFILE 路径。"
        return
    }

    Write-Host ""
    Write-QSSuccess "检测到当前 `$PROFILE 路径:"
    Write-Host "  $profilePath" -ForegroundColor Cyan
    Write-Host ""

    Write-QuickSSHProfile -ProfilePath $profilePath

    Write-Host ""
    Write-QSSuccess "✔ 配置完成！请重启 PowerShell 终端使其生效。"
    Write-Host ""
    Write-Host "   或执行以下命令立即加载：" -ForegroundColor Yellow
    Write-Host "   & (Get-Content '$profilePath' -Raw) | Invoke-Expression" -ForegroundColor Cyan
}

# ============================================================
# 帮助信息
# ============================================================

function Show-QuickSSHHelp {
    Write-Host ""
    Write-Host "Quick-SSH - PowerShell SSH 连接管理工具" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "用法:" -ForegroundColor Yellow
    Write-Host "  qssh                   启动 TUI 终端界面（推荐，类似 yazi 操作体验）"
    Write-Host "  qssh ps [关键词]      列出所有已保存的 SSH 连接（对应 docker ps）"
    Write-Host "  qssh add <别名> <用户@主机:端口> [--key <私钥路径>]" -ForegroundColor Gray
    Write-Host "                        添加新 SSH 连接（端口默认 22，私钥默认 ~/.ssh/id_rsa）"
    Write-Host "  qssh rm <别名>        删除指定别名的 SSH 连接（对应 docker rm）"
    Write-Host "  qssh <别名>           一键连接 SSH 服务器"
    Write-Host "  qssh init             将 Quick-SSH 注册到 `$PROFILE，重启终端自动加载"
    Write-Host "  qssh export <文件>    导出全部主机配置到 JSON 文件"
    Write-Host "  qssh import <文件>    从 JSON 文件批量导入连接"
    Write-Host "  qssh help             显示本帮助信息"
    Write-Host ""
    Write-Host "示例:" -ForegroundColor Yellow
    Write-Host "  qssh                         # 启动 TUI 界面"
    Write-Host "  qssh ps"
    Write-Host "  qssh ps 生产"
    Write-Host "  qssh add my-server root@192.168.1.100:22 --key ~/.ssh/id_rsa"
    Write-Host "  qssh add my-vm admin@10.0.0.5"
    Write-Host "  qssh my-server"
    Write-Host "  qssh rm my-vm"
    Write-Host "  qssh export ~/backup/hosts.json"
    Write-Host "  qssh import ~/backup/hosts.json"
    Write-Host ""
}

# ============================================================
# 主入口 - qssh 命令分发
# ============================================================

function global:qssh {
    param(
        [Parameter(Position = 0, Mandatory = $false)]
        [string]$Command,

        [Parameter(ValueFromRemainingArguments = $true)]
        [string[]]$ArgsList
    )

    # 首次调用自动初始化
    Initialize-QuickSSHConfig

    if (-not $Command) {
        Invoke-QuickSSHTUI
        return
    }

    switch -Wildcard ($Command) {
        "ps" {
            $keyword = if ($ArgsList.Count -gt 0) { $ArgsList[0] } else { "" }
            Invoke-QuickSSHPs -Keyword $keyword
        }
        "add" {
            $alias      = $null
            $userAtHost = $null
            $keyPath    = $null
            $i = 0
            while ($i -lt $ArgsList.Count) {
                if ($ArgsList[$i] -eq "--key" -or $ArgsList[$i] -eq "-k") {
                    $i++
                    if ($i -lt $ArgsList.Count) { $keyPath = $ArgsList[$i] }
                } elseif (-not $alias) {
                    $alias = $ArgsList[$i]
                } elseif (-not $userAtHost) {
                    $userAtHost = $ArgsList[$i]
                }
                $i++
            }
            Invoke-QuickSSHAdd -Alias $alias -UserAtHost $userAtHost -KeyPath $keyPath
        }
        "rm" {
            $alias = if ($ArgsList.Count -gt 0) { $ArgsList[0] } else { "" }
            Invoke-QuickSSHRm -Alias $alias
        }
        "export" {
            $file = if ($ArgsList.Count -gt 0) { $ArgsList[0] } else { "" }
            Invoke-QuickSSHExport -FilePath $file
        }
        "import" {
            $file = if ($ArgsList.Count -gt 0) { $ArgsList[0] } else { "" }
            Invoke-QuickSSHImport -FilePath $file
        }
        "init" {
            Invoke-QuickSSHInit
        }
        "help" {
            Show-QuickSSHHelp
        }
        default {
            # 未知命令当作别名尝试连接
            Invoke-QuickSSHConnect -Alias $Command
        }
    }
}

# ============================================================
# Tab 自动补全 - qssh 后按 Tab 可补全子命令和主机别名
# ============================================================

Register-ArgumentCompleter -CommandName "qssh" -ScriptBlock {
    param($commandName, $parameterName, $wordToComplete, $commandAst, $fakeBoundParameters)

    $subCommands = @("ps", "add", "rm", "init", "export", "import", "help")

    # 获取已保存的主机别名（从 ~/.ssh/config 解析）
    $aliases = @()
    try {
        $userHome = if ($env:USERPROFILE) { $env:USERPROFILE } else { $env:HOME }
        $cfg = Join-Path $userHome ".ssh" "config"
        if (Test-Path $cfg) {
            $raw = Get-Content -Path $cfg -Raw -Encoding UTF8
            if ($raw) {
                $hosts = Parse-SSHConfigHosts $raw
                $aliases = @($hosts | ForEach-Object { $_.alias })
            }
        }
    } catch {}

    $allCompletions = $subCommands + $aliases
    $allCompletions | Where-Object { $_ -like "$wordToComplete*" } | ForEach-Object {
        [System.Management.Automation.CompletionResult]::new($_, $_, "ParameterValue", $_)
    }
}

# ============================================================
# 模块加载时自动导出 qssh 函数
# ============================================================

Export-ModuleMember -Function "qssh"
