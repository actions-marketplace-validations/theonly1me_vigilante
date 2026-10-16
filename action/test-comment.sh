#!/usr/bin/env bash
set -euo pipefail

action_folder="$(cd "$(dirname "$0")" && pwd)"
scratch="$(mktemp -d)"
failures=0

check() {
  local name="$1"
  shift
  if "$@"; then
    echo "ok: $name"
  else
    echo "FAIL: $name" >&2
    failures=$((failures + 1))
  fi
}

fake_bin="$scratch/bin"
mkdir -p "$fake_bin"
cat > "$fake_bin/gh" <<'GH'
#!/usr/bin/env bash
echo "$*" >> "$FAKE_GH_LOG"
for argument in "$@"; do
  case "$argument" in
    body=@*) cp "${argument#body=@}" "$FAKE_GH_BODY" ;;
  esac
done
case "$*" in
  "api repos/acme/app/issues/7/comments --paginate"*)
    printf '%s' "$FAKE_EXISTING" ;;
  *"--method POST"*|*"--method PATCH"*)
    if [ -n "$FAKE_WRITE_FAILS" ]; then
      echo "gh: Resource not accessible by integration (HTTP 403)" >&2
      exit 1
    fi ;;
esac
GH
chmod +x "$fake_bin/gh"

comment() {
  local report="$1" outcome="$2" existing="$3" write_fails="$4"
  report_file="$scratch/report.md"
  printf '%b' "$report" > "$report_file"
  gh_log="$(mktemp)"
  gh_body="$(mktemp)"
  comment_status=0
  comment_output="$(PATH="$fake_bin:$PATH" REPORT_FILE="$report_file" REVIEW_OUTCOME="$outcome" \
    PR_NUMBER=7 REPOSITORY=acme/app FAKE_GH_LOG="$gh_log" FAKE_GH_BODY="$gh_body" \
    FAKE_EXISTING="$existing" FAKE_WRITE_FAILS="$write_fails" \
    "$action_folder/comment.sh" 2>&1)" || comment_status=$?
}

new_power='## Vigilante: this pull request adds 1 new power\n\n**Result: fail.** 1 power is not in vigilante.lock.\n'
no_powers='## Vigilante: this pull request adds no new powers\n\n**Result: pass.**\n'

comment "$new_power" failure "" ""
check "a failed review posts a new comment" grep -q -- "--method POST repos/acme/app/issues/7/comments" "$gh_log"
check "the comment starts with the marker" grep -qx "<!-- vigilante-report -->" <(head -n 1 "$gh_body")
check "the comment has the report" grep -q "adds 1 new power" "$gh_body"

comment "$no_powers" success "" ""
check "a quiet report posts no comment" test "$(grep -c -- "--method" "$gh_log" || true)" -eq 0

comment "$no_powers" success "99" ""
check "a quiet report updates an earlier comment" grep -q -- "--method PATCH repos/acme/app/issues/comments/99" "$gh_log"
check "the update has the new report" grep -q "adds no new powers" "$gh_body"

comment "$new_power" failure "" "yes"
check "a comment that cannot be written does not fail the step" test "$comment_status" -eq 0
check "a comment that cannot be written is a warning" grep -q "^::warning::" <<< "$comment_output"

gh_log="$(mktemp)"
comment_status=0
PATH="$fake_bin:$PATH" REPORT_FILE="$scratch/missing.md" REVIEW_OUTCOME=failure PR_NUMBER=7 \
  REPOSITORY=acme/app FAKE_GH_LOG="$gh_log" FAKE_GH_BODY=/dev/null FAKE_EXISTING="" FAKE_WRITE_FAILS="" \
  "$action_folder/comment.sh" > /dev/null 2>&1 || comment_status=$?
check "no report means no comment" test "$comment_status" -eq 0 -a ! -s "$gh_log"

rm -rf "$scratch"
if [ "$failures" -gt 0 ]; then
  echo "$failures checks failed." >&2
  exit 1
fi
echo "All comment checks passed."
