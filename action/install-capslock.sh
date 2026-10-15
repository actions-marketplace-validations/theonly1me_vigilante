#!/usr/bin/env bash
set -euo pipefail

folder="$RUNNER_TEMP/capslock"
GOBIN="$folder" go install "github.com/google/capslock/cmd/capslock@$CAPSLOCK_VERSION"
"$folder/capslock" -version
echo "$folder" >> "$GITHUB_PATH"
