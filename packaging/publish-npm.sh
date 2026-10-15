#!/usr/bin/env bash
set -euo pipefail

folder="$1"

if [ -n "${NPM_TOKEN:-}" ]; then
  printf '//registry.npmjs.org/:_authToken=%s\n' "$NPM_TOKEN" > "$RUNNER_TEMP/.npmrc"
  export NPM_CONFIG_USERCONFIG="$RUNNER_TEMP/.npmrc"
elif ! node -e 'const [major, minor, patch] = process.argv[1].split(".").map(Number); process.exit(major > 11 || (major === 11 && (minor > 5 || (minor === 5 && patch >= 1))) ? 0 : 1)' "$(npm --version)"; then
  echo "Trusted publishing needs npm 11.5.1 or later. This runner has npm $(npm --version)." >&2
  exit 1
fi

publish() {
  local tarball="$1" package
  package="$(tar -xzOf "$tarball" package/package.json | node -p 'const manifest = JSON.parse(require("fs").readFileSync(0, "utf8")); manifest.name + "@" + manifest.version')"
  if npm view "$package" version > /dev/null 2>&1; then
    echo "$package is already on npm."
    return
  fi
  npm publish "$tarball" --access public --provenance
}

for tarball in "$folder"/theonly1me-vigilante-darwin-*.tgz "$folder"/theonly1me-vigilante-linux-*.tgz; do
  publish "$tarball"
done
publish "$(ls "$folder"/theonly1me-vigilante-[0-9]*.tgz)"
