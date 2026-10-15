#!/usr/bin/env bash
set -euo pipefail

report="$(mktemp)"
errors="$(mktemp)"
status=0

if [ -n "${BASE_SHA:-}" ]; then
  if [ "$(git rev-parse --is-shallow-repository)" = true ]; then
    echo "vigilante: the checkout is shallow, so the merge base is missing. Set fetch-depth: 0 on actions/checkout." > "$errors"
    status=2
  else
    vigilante diff --base "$BASE_SHA" --head "$HEAD_SHA" > "$report" 2> "$errors" || status=$?
  fi
else
  vigilante check > "$report" 2> "$errors" || status=$?
fi

cat "$report"
cat "$errors" >&2
{
  cat "$report"
  if [ -s "$errors" ]; then
    printf '## Vigilante: error\n\n'
    sed 's/^/    /' "$errors"
  fi
} >> "$GITHUB_STEP_SUMMARY"
exit "$status"
