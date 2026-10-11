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
expect_version action.yml "$(sed -n 's/^    default: "\([^"]*\)".*$/\1/p' action.yml)"

npm_manifest=npm/vigilante/package.json
expect_version "$npm_manifest" "$(node -p "require('./$npm_manifest').version")"
while read -r name version; do
  expect_version "$npm_manifest optionalDependencies $name" "$version"
done < <(node -p "Object.entries(require('./$npm_manifest').optionalDependencies).map((entry) => entry.join(' ')).join('\\n')")

if [ "$mismatches" -gt 0 ]; then
  exit 1
fi
echo "All versions are $expected."
