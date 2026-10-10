#!/usr/bin/env bash
set -euo pipefail

executable="$1"
version="$(sed -n 's/^version = "\(.*\)"$/\1/p' Cargo.toml | head -n 1)"

reported="$("$executable" --version)"
if [ "$reported" != "vigilante $version" ]; then
  echo "$executable: expected 'vigilante $version', got '$reported'" >&2
  exit 1
fi

status=0
"$executable" check --repository "$(mktemp -d)/missing" 2>/dev/null || status=$?
if [ "$status" -ne 2 ]; then
  echo "$executable: expected exit code 2 for a missing repository, got $status" >&2
  exit 1
fi

echo "$executable: the version and the error exit code are correct."
