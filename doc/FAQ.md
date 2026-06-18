# 常见问题 (FAQ)

> 本文档收录 Quick-SSH 使用过程中可能遇到的常见问题及解决方案。

---

## 目录

1. [SSH 私钥权限错误](#1-ssh-私钥权限错误)
2. [WSL/Linux 上密钥路径包含反斜线](#2-wsllinux-上密钥路径包含反斜线)
3. [TUI 退出后终端键盘输入异常](#3-tui-退出后终端键盘输入异常)
4. [PowerShell 5 中文乱码 / 语法错误](#4-powershell-5-中文乱码--语法错误)
5. [PowerShell 执行策略限制](#5-powershell-执行策略限制)
6. [Linux/macOS 上 `qssh` 命令找不到](#6-linuxmacos-上-qssh-命令找不到)
7. [npm 全局安装后 `qssh` 不是可执行命令](#7-npm-全局安装后-qssh-不是可执行命令)

---

## 1. SSH 私钥权限错误

### 错误信息

```text
@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@
@         WARNING: UNPROTECTED PRIVATE KEY FILE!          @
@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@
Permissions 0755 for 'id_rsa' are too open.
It is required that your private key files are NOT accessible by others.
This private key will be ignored.
Bad permissions. Try removing permissions for other users.
```

或者 SSH 连接时被拒绝密钥认证，只能走密码登录。

### 原因

OpenSSH 出于安全考虑，要求私钥文件**只能被文件所有者读取**。如果其他用户或组也有读取权限，SSH 会直接拒绝使用该密钥。

> Windows WSL / Linux 下容易出现此问题，特别是复制或移动密钥文件后，权限可能被重置为 `0644` 或 `0755`。

### 解决方案 (Linux / WSL)

```bash
# 进入 .ssh 目录
cd ~/.ssh

# 私钥：仅所有者可读写（SSH 强制要求）
chmod 600 id_rsa

# 公钥：所有人可读（无写入风险）
chmod 644 id_rsa.pub

# .ssh 目录：仅所有者可访问
chmod 700 ~/.ssh
```

执行完成后重新连接即可：

```bash
qssh my-server
```

### 权限对照表

| 文件/目录 | 推荐权限 | 说明 |
|-----------|----------|------|
| `~/.ssh/` | `700` (drwx------) | 仅所有者能读写执行 |
| `id_rsa` (私钥) | `600` (-rw-------) | 仅所有者读写，其他人完全无权限 |
| `id_rsa.pub` (公钥) | `644` (-rw-r--r--) | 所有人可读，无写入风险 |
| `authorized_keys` | `600` (-rw-------) | 仅所有者读写 |
| `config` | `644` (-rw-r--r--) | 配置文件所有人可读即可 |

### 解决方案 (Windows 原生 SSH)

Windows 下权限错误通常是因为 `id_rsa` 文件权限中包含了 `Users` 组。

**修复步骤：**

1. 右键 `id_rsa` → **属性**
2. 选择 **安全** 选项卡
3. 点击 **高级**
4. 点击 **禁用继承** → 选择 **"将已继承的权限转换为此对象的显式权限"**
5. 删除所有**非当前用户**的权限条目（如 `Users`、`Authenticated Users` 等）
6. 只保留当前用户，权限设置为：**读取** + **写入**
7. 点击 **确定** 保存

---

## 2. WSL/Linux 上密钥路径包含反斜线

### 错误信息

```text
tilde_expand: No such user \.ssh\id_rsa
```

或者：

```text
Warning: Identity file C:\Users\Lenovo\.ssh\id_rsa not accessible: No such file or directory.
```

### 原因

`~/.ssh/config` 配置文件中的 `IdentityFile` 路径使用了 Windows 反斜线分隔符（如 `~\.ssh\id_rsa` 或 `C:\Users\Lenovo\.ssh\id_rsa`）。在 WSL/Linux 上，OpenSSH 将反斜线解释为文件名的一部分，而不是目录分隔符。

产生反斜线的原因：

- 使用 Windows 版 `qssh` 添加连接，默认密钥路径由 `path.join()` 生成，在 Windows 上使用 `\`
- 手动编辑 `~/.ssh/config` 时使用了反斜线
- 从 Windows 复制 `~/.ssh/config` 到 WSL

### 解决方案

**方案一：使用 `qssh` 重新添加（推荐）**

```bash
# 删除旧的错误配置
qssh rm my-server

# 重新添加，显式指定密钥路径（使用正斜线）
qssh add my-server root@192.168.1.100 --key ~/.ssh/id_rsa
```

`qssh` 在非 Windows 平台上会自动将路径中的 `\` 转换为 `/`。

**方案二：手动编辑 `~/.ssh/config`**

```bash
vim ~/.ssh/config
```

找到对应的 `Host` 块，将 `IdentityFile` 中的反斜线改为正斜线：

```diff
  Host my-server
      HostName 192.168.1.100
      User root
      Port 22
-     IdentityFile C:\Users\Lenovo\.ssh\id_rsa
+     IdentityFile ~/.ssh/id_rsa
```

> **提示**：在 WSL 中，Windows 路径 `C:\Users\Lenovo\.ssh\id_rsa` 对应的正确路径是 `/mnt/c/Users/Lenovo/.ssh/id_rsa`。但更推荐的做法是将密钥复制到 WSL 的 `~/.ssh/` 目录下。

---

## 3. TUI 退出后终端键盘输入异常

### 错误信息

退出 TUI 界面后，键盘输入的内容显示为奇怪的字符或乱码，而不是正常的命令行输入。

### 原因

TUI 使用 [blessed](https://github.com/chjj/blessed) 库，它切换到了终端的**备用屏幕缓冲区 (Alternate Screen Buffer)** 并启用了**鼠标追踪 (SGR Mouse Mode)**。如果退出时没有正确恢复终端状态，会导致：

- 备用屏幕未切换回主屏幕
- 鼠标追踪未关闭，鼠标移动和点击输出转义序列
- 终端处于 raw 模式而非 cooked 模式

### 解决方案

**方法一：执行 `reset` 命令（通用）**

```bash
reset
```

> 这会完全重置终端状态，包括屏幕缓冲区、鼠标模式、回显模式等。在大多数终端模拟器中都有效。

**方法二：手动恢复终端（临时应急）**

```bash
# 恢复光标
printf '\x1b[?25h'

# 退出备用屏幕
printf '\x1b[?1049l'

# 关闭鼠标追踪
printf '\x1b[?1000l\x1b[?1002l\x1b[?1003l\x1b[?1006l'
```

**方法三：关闭终端窗口重新打开**

如果不方便执行命令，直接关闭当前终端标签页重新打开也是最快的解决办法。

> **注意**：自 v1.0.4 起，Quick-SSH 已内置 `restoreTerminal()` 修复，正常情况下退出 TUI 不会出现此问题。如果你仍遇到此问题，请提 [Issue](https://github.com/CCE-Li/Quick-SSH/issues)。

---

## 4. PowerShell 5 中文乱码 / 语法错误

### 错误信息

PowerShell 5 (Windows PowerShell) 中加载模块时报错，错误信息包含乱码字符，例如：

```text
# 错误信息中原本的中文注释或字符串显示为乱码
# 如：╬╓╓╓╓├╫...
```

### 原因

PowerShell 5 默认使用系统 ANSI 编码读取 `.psm1` 文件（简体中文 Windows 下为 GBK/Windows-1252）。如果文件保存为 **UTF-8 无 BOM (Byte Order Mark)** 格式，PowerShell 5 无法正确识别编码，导致中文字符乱码。

PowerShell 7 (pwsh) 默认使用 UTF-8，因此无此问题。

### 解决方案

确保 `Quick-SSH.psm1` 文件以 **UTF-8 with BOM** 格式保存。

**使用 VS Code：**

1. 打开 `src/win/Quick-SSH.psm1`
2. 点击右下角状态栏的编码名称（如 `UTF-8`）
3. 选择 **"Save with Encoding"** → **"UTF-8 with BOM"**
4. 保存文件

**使用 Node.js 添加 BOM：**

```javascript
const fs = require("fs");
const path = "src/win/Quick-SSH.psm1";
const content = fs.readFileSync(path);
const BOM = Buffer.from([0xEF, 0xBB, 0xBF]);
fs.writeFileSync(path, Buffer.concat([BOM, content]));
```

**使用 PowerShell 添加 BOM：**

```powershell
$path = "src/win/Quick-SSH.psm1"
$content = Get-Content $path -Raw
[System.IO.File]::WriteAllText($path, $content, [System.Text.Encoding]::UTF8)
```

> 注意：`.NET` 的 `UTF8` 编码**不带 BOM**。如需带 BOM 的 UTF-8，应使用 `New-Object System.Text.UTF8Encoding $true`。

### 验证方法

用十六进制编辑器查看文件开头，确认前三个字节为 `EF BB BF`：

```powershell
# 检查文件前三个字节是否为 BOM
$bytes = New-Object byte[] 3
[System.IO.File]::OpenRead("src/win/Quick-SSH.psm1").Read($bytes, 0, 3)
if ($bytes[0] -eq 0xEF -and $bytes[1] -eq 0xBB -and $bytes[2] -eq 0xBF) {
    "✅ 文件包含 UTF-8 BOM"
} else {
    "❌ 文件缺少 UTF-8 BOM"
}
```

---

## 5. PowerShell 执行策略限制

### 错误信息

```text
无法加载文件 ...\Quick-SSH.psm1，因为在此系统上禁止运行脚本。
```

### 原因

PowerShell 的 **执行策略 (Execution Policy)** 限制了脚本的运行。

### 解决方案

```powershell
# 查看当前执行策略
Get-ExecutionPolicy

# 设置为 RemoteSigned（推荐）
Set-ExecutionPolicy RemoteSigned -Scope CurrentUser
```

详见 README 中的 [安装前注意 ⚠️](README.md#安装前注意-) 章节。

---

## 6. Linux/macOS 上 `qssh` 命令找不到

### 错误信息

```bash
$ qssh
bash: qssh: command not found
```

### 原因

npm 全局安装后，`qssh` 命令需要通过 shell 配置文件（`~/.bashrc` 或 `~/.zshrc`）中的函数包装才能调用。

### 解决方案

**方法一：重新运行安装脚本**

```bash
npm install -g quick-ssh
```

安装脚本会自动检测当前 shell 并写入配置。

**方法二：手动添加 qssh 函数**

```bash
# 获取 cli.js 的路径
CLI_PATH=$(npm root -g)/quick-ssh/src/unix/cli.js

# 根据当前 shell 写入配置
if [ -n "$($SHELL -c 'echo $ZSH_VERSION')" ]; then
    echo "qssh() { node \"$CLI_PATH\" \"\$@\"; }" >> ~/.zshrc
    echo "已写入 ~/.zshrc，请执行 source ~/.zshrc 生效"
elif [ -n "$($SHELL -c 'echo $BASH_VERSION')" ]; then
    echo "qssh() { node \"$CLI_PATH\" \"\$@\"; }" >> ~/.bashrc
    echo "已写入 ~/.bashrc，请执行 source ~/.bashrc 生效"
fi
```

**方法三：立即生效**

```bash
source ~/.bashrc   # 如果使用 bash
source ~/.zshrc    # 如果使用 zsh
```

---

## 7. npm 全局安装后 `qssh` 不是可执行命令

### 错误信息

```text
'qssh' 不是内部或外部命令，也不是可运行的程序
或批处理文件。
```

### 原因

Quick-SSH 并非通过 npm `bin` 字段注册为系统命令，而是通过注入 shell 配置文件来实现命令注册。安装脚本需要在安装完成后手动触发。

### 解决方案

```powershell
# 检查 npm 全局安装根目录
npm root -g

# 手动执行 postinstall 脚本
node %APPDATA%\npm\node_modules\quick-ssh\src\lib\index.js postinstall

# 然后重启终端
```

或者在安装后执行：

```powershell
# 导入模块（临时生效）
Import-Module (Join-Path (npm root -g) "quick-ssh\src\win\Quick-SSH.psm1") -DisableNameChecking

# 然后执行注册
qssh init
```

---

> **找不到你要的问题？** 请提交 [GitHub Issue](https://github.com/CCE-Li/Quick-SSH/issues) 或直接在终端执行 `qssh help` 查看帮助。
