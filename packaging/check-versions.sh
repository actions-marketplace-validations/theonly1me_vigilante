#!/usr/bin/env bash
set -euo pipefail

crate_version="$(sed -n 's/^version = "\(.*\)"$/\1/p' Cargo.toml | head -n 1)"
expected="${1:-$crate_version}"
expected="${expected#v}"
mismatches=0

expect_version() {
  local source="$1" found="$2"
  if [ "$found" != "$expected" ]; then
    echo "$source has version '$found', expected '$expected'" >&2
    mismatches=$((mismatches + 1))
  fi
}

expect_version Cargo.toml "$crate_version"

if [ "$mismatches" -gt 0 ]; then
  exit 1
fi
echo "All versions are $expected."
