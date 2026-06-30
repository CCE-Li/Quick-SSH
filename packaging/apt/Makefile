# Quick-SSH .deb 构建脚本 (Rust 版本)
# 用法:
#   make VERSION=2.0.0    # 构建 .deb 包
#
# 从 GitHub Release 下载 tar.gz 归档，提取二进制并打包

VERSION ?= 2.0.0
ARCH    ?= amd64
TARGET  ?= x86_64-unknown-linux-gnu
PKGNAME  = quick-ssh_$(VERSION)_$(ARCH).deb

.PHONY: all clean

all: $(PKGNAME)

build/usr/bin:
	mkdir -p build/usr/bin

build/DEBIAN:
	mkdir -p build/DEBIAN
	cp DEBIAN/control build/DEBIAN/control
	sed -i 's/Version: .*/Version: $(VERSION)/' build/DEBIAN/control

qssh-x86_64-unknown-linux-gnu.tar.gz:
	curl -L -o $@ \
		"https://github.com/CCE-Li/Quick-SSH/releases/download/v$(VERSION)/qssh-x86_64-linux.tar.gz"

build/usr/bin/qssh: qssh-x86_64-unknown-linux-gnu.tar.gz | build/usr/bin
	tar xzf $< -C build/usr/bin/ --strip-components=1 qssh/qssh
	chmod 755 build/usr/bin/qssh

$(PKGNAME): build/usr/bin/qssh build/DEBIAN
	dpkg-deb --build build $@
	@echo "=== 构建完成: $(PKGNAME) ==="

clean:
	rm -rf build *.tar.gz *.deb
