#!/usr/bin/env bash
set -euo pipefail

checksums="$1"
version="$(sed -n 's/^version = "\(.*\)"$/\1/p' Cargo.toml | head -n 1)"
download_url="https://github.com/theonly1me/vigilante/releases/download/v$version"

archive() {
  local target="$1" file checksum
  file="vigilante-$target.tar.gz"
  checksum="$(awk -v file="$file" '$2 == file { print $1 }' "$checksums")"
  if [ -z "$checksum" ]; then
    echo "$checksums has no checksum for $file" >&2
    exit 1
  fi
  printf '      url "%s/%s"\n      sha256 "%s"\n' "$download_url" "$file" "$checksum"
}

macos_arm="$(archive aarch64-apple-darwin)"
macos_intel="$(archive x86_64-apple-darwin)"
linux_arm="$(archive aarch64-unknown-linux-musl)"
linux_intel="$(archive x86_64-unknown-linux-musl)"

cat <<FORMULA
class Vigilante < Formula
  desc "Show the new powers that a pull request gives to code, CI, and AI agents"
  homepage "https://github.com/theonly1me/vigilante"
  license any_of: ["MIT", "Apache-2.0"]

  on_macos do
    on_arm do
$macos_arm
    end
    on_intel do
$macos_intel
    end
  end

  on_linux do
    on_arm do
$linux_arm
    end
    on_intel do
$linux_intel
    end
  end

  def install
    bin.install "vigilante"
  end

  test do
    assert_match "vigilante #{version}", shell_output("#{bin}/vigilante --version")
  end
end
FORMULA
