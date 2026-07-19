# 打包总览

Quick-SSH 为 6 种主流包管理器提供开箱即用的配置，覆盖三大操作系统。

## 支持的包管理器

| 包管理器 | 平台 | 状态 | 配置位置 |
|---------|------|------|---------|
| **Scoop** | Windows | ✅ 就绪 | [`packaging/scoop/quick-ssh.json`](/packaging/scoop/quick-ssh.json) |
| **WinGet** | Windows | ✅ 就绪 | [`packaging/winget/`](/packaging/winget/) |
| **Homebrew** | macOS | ✅ 就绪 | [`packaging/homebrew/quick-ssh.rb`](/packaging/homebrew/quick-ssh.rb) |
| **AUR** | Arch Linux | ✅ 就绪 | [`packaging/pacman/PKGBUILD`](/packaging/pacman/PKGBUILD) |
| **APT** | Debian/Ubuntu | ✅ 就绪 | [`packaging/apt/`](/packaging/apt/) |

## 配置目录结构

```
packaging/
├── scoop/
│   └── quick-ssh.json        # Scoop manifest
├── winget/
│   ├── CCE-Li.Quick-SSH.installer.yaml
│   ├── CCE-Li.Quick-SSH.locale.en-US.yaml
│   └── CCE-Li.Quick-SSH.yaml
├── homebrew/
│   └── quick-ssh.rb           # Homebrew Formula
├── pacman/
│   └── PKGBUILD               # AUR PKGBUILD
└── apt/
    ├── Makefile               # .deb 构建脚本
    └── DEBIAN/
        └── control            # APT 控制文件
```

## 工作原理

所有包管理器配置都引用了 **GitHub Release 上的预编译二进制文件**。每次发布新版本后，运行 [`update-packaging.ps1`](/scripts/update-packaging.ps1) 脚本即可自动更新所有配置。

### 工作流程

```
发布新版本 → 运行 update-packaging.ps1 → 自动更新所有配置
    ↓
各包管理器配置文件中的版本号和 SHA256 校验和同步更新
    ↓
提交到各包管理器仓库 / 上传 .deb 到 Release Assets
```

### 自动化脚本

`scripts/update-packaging.ps1` 的功能：

1. **自动检测版本** — 从 `Cargo.toml` 读取当前版本号
2. **下载归档** — 从 GitHub Release 拉取各平台归档文件
3. **计算 SHA256** — 为每个文件生成校验和
4. **更新所有配置** — 自动修改 version 和 hash/sha256 字段
5. **保存 SHA256SUMS** — 生成校验和文件供验证

```powershell
# 常用命令
.\scripts\update-packaging.ps1                    # 自动模式
.\scripts\update-packaging.ps1 -Version "2.0.1"    # 指定版本
.\scripts\update-packaging.ps1 -Help               # 查看帮助

# 离线使用（事先下载好归档文件）
.\scripts\update-packaging.ps1 -Version "2.1.0" -LocalDir ".\downloads"
```
