#!/usr/bin/env bash
set -uo pipefail

marker="<!-- vigilante-report -->"
quiet_title="## Vigilante: this pull request adds no new powers"
limit=60000

warn() {
  echo "::warning::Vigilante cannot write the pull request comment: $1"
  exit 0
}

if [ ! -s "${REPORT_FILE:-}" ]; then
  exit 0
fi

query=".[] | select(.user.login == \"github-actions[bot]\" and (.body | startswith(\"$marker\"))) | .id"
existing="$(gh api "repos/$REPOSITORY/issues/$PR_NUMBER/comments" --paginate --jq "$query" 2>&1)" ||
  warn "$existing"
existing="$(printf '%s\n' "$existing" | head -n 1)"

if [ -z "$existing" ] && [ "$REVIEW_OUTCOME" = success ] &&
  [ "$(head -n 1 "$REPORT_FILE")" = "$quiet_title" ]; then
  exit 0
fi

body="$(mktemp)"
{
  echo "$marker"
  head -c "$limit" "$REPORT_FILE"
} > "$body"

if [ -n "$existing" ]; then
  output="$(gh api --method PATCH "repos/$REPOSITORY/issues/comments/$existing" -F "body=@$body" 2>&1)" ||
    warn "$output"
else
  output="$(gh api --method POST "repos/$REPOSITORY/issues/$PR_NUMBER/comments" -F "body=@$body" 2>&1)" ||
    warn "$output"
fi
