#!/usr/bin/env bash
set -euo pipefail

packages="$(cd "$1" && pwd)"
version="$(sed -n 's/^version = "\(.*\)"$/\1/p' Cargo.toml | head -n 1)"
platform="$(node -p 'process.platform + "-" + process.arch')"
main_package="$packages/theonly1me-vigilante-$version.tgz"

project="$(mktemp -d)"
(
  cd "$project"
  npm init --yes >/dev/null
  npm install --no-audit --no-fund "$main_package" "$packages/theonly1me-vigilante-$platform-$version.tgz"
)
packaging/smoke.sh "$project/node_modules/.bin/vigilante"

project_without_binary="$(mktemp -d)"
(
  cd "$project_without_binary"
  npm init --yes >/dev/null
  npm install --no-audit --no-fund --omit=optional "$main_package"
)
status=0
message="$("$project_without_binary/node_modules/.bin/vigilante" --version 2>&1)" || status=$?
if [ "$status" -ne 2 ] || [[ "$message" != *"is not installed"* ]]; then
  echo "without a platform package: expected exit code 2 and 'is not installed', got $status: $message" >&2
  exit 1
fi
echo "without a platform package: the launcher exits 2 with its message."
