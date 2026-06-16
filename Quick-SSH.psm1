# Quick-SSH.psm1 - PowerShell SSH Connection Manager
# 仿 Docker 命令行风格的 SSH 连接管理工具
# 配置文件路径: %USERPROFILE%\.quickssh\hosts.json

# ============================================================
# 内部函数 - 配置管理
# ============================================================

$Script:ConfigDir  = Join-Path $env:USERPROFILE ".quickssh"
$Script:ConfigFile = Join-Path $Script:ConfigDir "hosts.json"

# 初始化配置目录和空 JSON 文件
function Initialize-QuickSSHConfig {
    if (-not (Test-Path $Script:ConfigDir)) {
        New-Item -Path $Script:ConfigDir -ItemType Directory -Force | Out-Null
    }
    if (-not (Test-Path $Script:ConfigFile)) {
        '[]' | Set-Content -Path $Script:ConfigFile -Encoding UTF8 -NoNewline
    }
}

# 读取全部主机配置
function Get-QuickSSHHosts {
    Initialize-QuickSSHConfig
    try {
        $raw = Get-Content -Path $Script:ConfigFile -Raw -Encoding UTF8
        if ([string]::IsNullOrWhiteSpace($raw)) { return @() }
        return @($raw | ConvertFrom-Json)
    } catch {
        return @()
    }
}

# 保存全部主机配置
function Save-QuickSSHHosts($Hosts) {
    $Hosts | ConvertTo-Json -Depth 10 | Set-Content -Path $Script:ConfigFile -Encoding UTF8
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
        $KeyPath = Join-Path $env:USERPROFILE ".ssh" "id_rsa"
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
        Show-QuickSSHHelp
        return
    }

    $hosts  = Get-QuickSSHHosts
    $target = $hosts | Where-Object { $_.alias -eq $Alias }
    if (-not $target) {
        Write-QSError "错误：别名 '$Alias' 不存在。使用 'qssh ps' 查看可用连接。"
        return
    }

    $sshExe = if (Get-Command "ssh.exe" -ErrorAction SilentlyContinue) { "ssh.exe" } else { "ssh" }

    $cmd = "$sshExe -i `"$($target.key)`" -p $($target.port) -o HostKeyAlgorithms=+ssh-rsa $($target.user)@$($target.host)"

    Write-QSSuccess "正在连接到 '$Alias' ($($target.user)@$($target.host):$($target.port)) ..."
    Write-Host ""

    # 使用 cmd /c 避免 PowerShell 拦截 SSH 交互
    $encoded = [Convert]::ToBase64String([Text.Encoding]::Unicode.GetBytes($cmd))
    cmd /c "powershell -NoProfile -EncodedCommand $encoded"
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
# 帮助信息
# ============================================================

function Show-QuickSSHHelp {
    Write-Host ""
    Write-Host "Quick-SSH - PowerShell SSH 连接管理工具" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "用法:" -ForegroundColor Yellow
    Write-Host "  qssh ps [关键词]      列出所有已保存的 SSH 连接（对应 docker ps）"
    Write-Host "  qssh add <别名> <用户@主机:端口> [--key <私钥路径>]" -ForegroundColor Gray
    Write-Host "                        添加新 SSH 连接（端口默认 22，私钥默认 ~/.ssh/id_rsa）"
    Write-Host "  qssh rm <别名>        删除指定别名的 SSH 连接（对应 docker rm）"
    Write-Host "  qssh <别名>           一键连接 SSH 服务器"
    Write-Host "  qssh export <文件>    导出全部主机配置到 JSON 文件"
    Write-Host "  qssh import <文件>    从 JSON 文件批量导入连接"
    Write-Host "  qssh help             显示本帮助信息"
    Write-Host ""
    Write-Host "示例:" -ForegroundColor Yellow
    Write-Host "  qssh ps"
    Write-Host "  qssh ps 生产"
    Write-Host "  qssh add my-server root@192.168.1.100:22 --key D:\.ssh\id_rsa"
    Write-Host "  qssh add my-vm admin@10.0.0.5"
    Write-Host "  qssh my-server"
    Write-Host "  qssh rm my-vm"
    Write-Host "  qssh export D:\backup\hosts.json"
    Write-Host "  qssh import D:\backup\hosts.json"
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
        Show-QuickSSHHelp
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

    $subCommands = @("ps", "add", "rm", "export", "import", "help")

    # 获取已保存的主机别名
    $aliases = @()
    try {
        $cfg = Join-Path $env:USERPROFILE ".quickssh" "hosts.json"
        if (Test-Path $cfg) {
            $raw = Get-Content -Path $cfg -Raw -Encoding UTF8
            if ($raw) {
                $hosts = $raw | ConvertFrom-Json
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
