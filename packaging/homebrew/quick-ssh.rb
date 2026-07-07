# typed: false
# frozen_string_literal: true

# Homebrew Formula for Quick-SSH (Rust native binary)
#
# 使用方法:
#   brew tap CCE-Li/quick-ssh
#   brew install quick-ssh
#
# 或直接:
#   brew install CCE-Li/quick-ssh/quick-ssh
#
# 更新步骤:
#   1. 更新 version 到新版本号
#   2. 从 GitHub Release 获取 .tar.gz 的 SHA256
#   3. 更新 sha256 值

class QuickSsh < Formula
  desc "🚀 Quick-SSH - Docker-style SSH connection manager with TUI (Rust native binary)"
  homepage "https://github.com/CCE-Li/Quick-SSH"
  $151a197681a345125783aec636829646a1f8a55d1e84103c186fab33d68f9fbfb"
  license "MIT"
  version "2.0.1"

  depends_on :macos

  on_macos do
    if Hardware::CPU.intel?
      $151a197681a345125783aec636829646a1f8a55d1e84103c186fab33d68f9fbfb"
    elsif Hardware::CPU.arm?
      url "https://github.com/CCE-Li/Quick-SSH/releases/download/v2.0.1/qssh-aarch64-macos.tar.gz"
      sha256 "26374f7623faa2e547e5b9c2fd3dec6fa157fc63f25264bd7c98d4aacff288c1"
    end
  end

  def install
    bin.install "qssh"
    bin.install "qssh-uploader"
  end

  test do
    assert_match "qssh", shell_output("#{bin}/qssh --version")
  end
end

