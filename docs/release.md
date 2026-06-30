# 发布流程

## 使用 cargo-dist（推荐）

```bash
# 安装 cargo-dist
cargo install cargo-dist

# 初始化（生成 GitHub Actions 工作流）
cargo dist init

# 根据提示选择：
#   - Host: github
#   - Target platforms: x86_64-unknown-linux-gnu, x86_64-pc-windows-msvc, aarch64-apple-darwin, x86_64-apple-darwin
#   - CI: github-actions
```

生成的 `.github/workflows/release.yml` 会自动：
- 编译各平台二进制
- 创建 `.tar.gz`/`.zip` 归档（含 LICENSE + README）
- 上传到 GitHub Release

## 手动发布

```bash
# 1. 更新版本
# 修改 Cargo.toml 中的 version

# 2. 编译
cargo build --release

# 3. 创建归档
mkdir -p dist/Quick-SSH-v2.0.0-x86_64-pc-windows-msvc
cp target/release/qssh.exe dist/Quick-SSH-v2.0.0-x86_64-pc-windows-msvc/
cp target/release/qssh-uploader.exe dist/Quick-SSH-v2.0.0-x86_64-pc-windows-msvc/
cp LICENSE README.md dist/Quick-SSH-v2.0.0-x86_64-pc-windows-msvc/
cd dist && zip -r Quick-SSH-v2.0.0-x86_64-pc-windows-msvc.zip Quick-SSH-v2.0.0-x86_64-pc-windows-msvc/

# 4. 创建 GitHub Release
gh release create v2.0.0 dist/*.zip dist/*.tar.gz --generate-notes
```

## 包管理器更新

发布新版本后，更新各包管理器配置中的版本号和哈希：

### Winget
- 更新 [`packaging/winget/CCE-Li.Quick-SSH.installer.yaml`](../packaging/winget/CCE-Li.Quick-SSH.installer.yaml)
- 提交 PR 到 microsoft/winget-pkgs

### Scoop
- 更新 [`packaging/scoop/quick-ssh.json`](../packaging/scoop/quick-ssh.json) 中的 url 和 hash

### Homebrew
- 更新 Formula 中的 url 和 sha256
- 提交 PR 到 homebrew-core 或自建 tap

### AUR
- 更新 PKGBUILD 中的 pkgver 和 sha256sums

### APT
- 更新 packaging/apt/DEBIAN/control
- 构建 .deb 包
