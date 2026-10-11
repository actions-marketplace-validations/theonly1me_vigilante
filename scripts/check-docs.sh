#!/usr/bin/env bash
set -euo pipefail

checker=.agents/skills/write-plain-english/scripts/check-ste.mjs
files=()
while IFS= read -r -d '' file; do
  files+=("$file")
done < <(git ls-files -z '*.md' ':!:CHANGELOG.md' ':!:CODE_OF_CONDUCT.md' ':!:.agents/**' ':!:.claude/**')

report="$(node "$checker" "${files[@]}")" || status=$?
echo "$report"
if [ "${status:-0}" -ne 0 ] || grep -q ': warning ' <<<"$report"; then
  echo "Fix each error and warning above. The docs use ASD-STE100 at 90% strictness." >&2
  exit 1
fi
