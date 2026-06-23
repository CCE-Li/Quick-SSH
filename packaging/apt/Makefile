# Makefile for building Quick-SSH .deb package
# Usage:
#   make deb     - Build .deb package
#   make clean   - Clean build artifacts

PKG_NAME    := quick-ssh
VERSION     := 1.1.0
ARCH        := all

BUILD_DIR   := build
DEB_DIR     := $(BUILD_DIR)/$(PKG_NAME)_$(VERSION)_$(ARCH)
ROOT_DIR    := $(abspath ../..)

.PHONY: deb clean

deb: $(BUILD_DIR)/$(PKG_NAME)_$(VERSION)_$(ARCH).deb

$(BUILD_DIR)/$(PKG_NAME)_$(VERSION)_$(ARCH).deb: $(DEB_DIR)
	dpkg-deb --build $(DEB_DIR) $@

$(DEB_DIR): $(DEB_DIR)/DEBIAN $(DEB_DIR)/usr
	@echo "Building package structure..."

$(DEB_DIR)/DEBIAN:
	mkdir -p $@
	cp DEBIAN/control $@/
	# Generate md5sums
	cd $(DEB_DIR) && find usr -type f -exec md5sum {} \; > DEBIAN/md5sums

$(DEB_DIR)/usr:
	# Create directory structure
	mkdir -p $@/share/quick-ssh
	mkdir -p $@/local/bin
	mkdir -p $@/local/lib/node_modules/quick-ssh

	# Copy source files
	cp -r $(ROOT_DIR)/src $@/share/quick-ssh/
	cp $(ROOT_DIR)/package.json $@/share/quick-ssh/
	cp $(ROOT_DIR)/README.md $@/share/quick-ssh/
	cp $(ROOT_DIR)/LICENSE $@/share/quick-ssh/

	# Create symlinks
	ln -sf /usr/share/quick-ssh/src/unix/cli.js $@/local/bin/qssh

clean:
	rm -rf $(BUILD_DIR)
