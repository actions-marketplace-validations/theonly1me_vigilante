#!/usr/bin/env bash
set -euo pipefail

version=0.20.2
archive="cargo-deny-$version-x86_64-unknown-linux-musl.tar.gz"
checksum=9f12ed4c49936e09b48bf862b595cde2fe64fcbd9d74dfacac6131ca824c8d5f
folder="$RUNNER_TEMP/cargo-deny"

mkdir -p "$folder"
curl --silent --show-error --fail --location --output "$folder/$archive" \
  "https://github.com/EmbarkStudios/cargo-deny/releases/download/$version/$archive"
echo "$checksum  $folder/$archive" | sha256sum --check --strict
tar -xzf "$folder/$archive" -C "$folder" --strip-components=1
"$folder/cargo-deny" --version
echo "$folder" >> "$GITHUB_PATH"
