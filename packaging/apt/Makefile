# Makefile for building Quick-SSH .deb package (Binary Version)
# Usage:
#   make deb     - Build .deb package
#   make clean   - Clean build artifacts

PKG_NAME    := quick-ssh
VERSION     := 1.1.0
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
	# 下载预编译二进制（如果不存在）
	if [ ! -f "qssh-linux-x64" ]; then \
		echo "下载 Quick-SSH Linux 二进制..."; \
		wget -q "https://github.com/CCE-Li/Quick-SSH/releases/download/v$(VERSION)/qssh-linux-x64" -O qssh-linux-x64 || \
		curl -sL "https://github.com/CCE-Li/Quick-SSH/releases/download/v$(VERSION)/qssh-linux-x64" -o qssh-linux-x64; \
	fi
	install -m 755 qssh-linux-x64 $(DEB_DIR)/usr/bin/qssh

clean:
	rm -rf $(BUILD_DIR) qssh-linux-x64
