# Makefile for building Quick-SSH .deb package (Binary Version)
# Quick-SSH uses Node.js SEA (Single Executable Application) to build
# native binaries — no Node.js runtime needed at user end.
#
# Usage:
#   make deb     - Build .deb package
#   make clean   - Clean build artifacts
#
# CI: GitHub Actions provides the binary as artifact named "qssh-linux".
#     The CI renames it to "qssh-linux-x64" before calling make deb.
# Local: This Makefile falls back to downloading from GitHub Releases.

PKG_NAME    := quick-ssh
VERSION     := 1.1.8
ARCH        := amd64

BUILD_DIR   := build
DEB_DIR     := $(BUILD_DIR)/$(PKG_NAME)_$(VERSION)_$(ARCH)
DEB_FILE    := $(BUILD_DIR)/$(PKG_NAME)_$(VERSION)_$(ARCH).deb

.PHONY: deb clean

deb: $(DEB_DIR)/DEBIAN/control $(DEB_DIR)/usr/bin/qssh
	dpkg-deb --build $(DEB_DIR) $(DEB_FILE)
	@echo ""
	@echo "✔ 构建完成: $(DEB_FILE)"

$(DEB_DIR)/DEBIAN/control:
	mkdir -p $(DEB_DIR)/DEBIAN
	cp DEBIAN/control $(DEB_DIR)/DEBIAN/
	# 生成 md5sums
	cd $(DEB_DIR) && find usr -type f -exec md5sum {} \; > DEBIAN/md5sums 2>/dev/null || true

$(DEB_DIR)/usr/bin/qssh:
	mkdir -p $(DEB_DIR)/usr/bin
	# 优先使用本地已存在的二进制（CI 构件或手动放置）
	# CI 提供 qssh-linux，本地构建产生 qssh-linux-x64
	if [ -f "qssh-linux-x64" ]; then \
		echo "使用本地 qssh-linux-x64"; \
	elif [ -f "qssh-linux" ]; then \
		echo "重命名 CI 构件 qssh-linux → qssh-linux-x64"; \
		mv qssh-linux qssh-linux-x64; \
	else \
		echo "下载 Quick-SSH Linux 二进制 (v$(VERSION))..."; \
		wget -q "https://github.com/CCE-Li/Quick-SSH/releases/download/v$(VERSION)/qssh-linux-x64" -O qssh-linux-x64 || \
		curl -sL "https://github.com/CCE-Li/Quick-SSH/releases/download/v$(VERSION)/qssh-linux-x64" -o qssh-linux-x64; \
	fi
	chmod +x qssh-linux-x64
	install -m 755 qssh-linux-x64 $(DEB_DIR)/usr/bin/qssh

clean:
	rm -rf $(BUILD_DIR) qssh-linux-x64 qssh-linux
