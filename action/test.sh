#!/usr/bin/env bash
set -euo pipefail

action_folder="$(cd "$(dirname "$0")" && pwd)"
scratch="$(mktemp -d)"
failures=0

export GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_NOSYSTEM=1
export GIT_AUTHOR_NAME=Test GIT_AUTHOR_EMAIL=test@example.com
export GIT_COMMITTER_NAME=Test GIT_COMMITTER_EMAIL=test@example.com

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

commit_all() {
  git -C "$repository" add --all
  git -C "$repository" commit --quiet --message "$1"
  git -C "$repository" rev-parse HEAD
}

review() {
  local folder="$1" base="$2" head="$3"
  summary="$(mktemp)"
  review_status=0
  (cd "$folder" && BASE_SHA="$base" HEAD_SHA="$head" GITHUB_STEP_SUMMARY="$summary" \
    "$action_folder/review.sh" > /dev/null 2>&1) || review_status=$?
}

repository="$scratch/repository"
git init --quiet --initial-branch=main "$repository"
echo "fixture" > "$repository/README.md"
base="$(commit_all base)"
mkdir -p "$repository/.github/workflows"
printf 'on: push\npermissions:\n  contents: write\njobs: {}\n' > "$repository/.github/workflows/release.yml"
head="$(commit_all "add a workflow that can push")"

review "$repository" "$base" "$head"
check "diff mode fails on a power that is not in the lock" test "$review_status" -eq 1
check "diff mode writes the pull request report" grep -q "this pull request adds 1 new power" "$summary"

vigilante lock --repository "$repository" > /dev/null
locked_head="$(commit_all "lock the new power")"
review "$repository" "$base" "$locked_head"
check "diff mode passes when the lock has the power" test "$review_status" -eq 0
check "diff mode still reports a locked power" grep -q "in lock" "$summary"

review "$repository" "" ""
check "check mode runs without a base" grep -q "vigilante.lock is current" "$summary"
check "check mode passes on a current lock" test "$review_status" -eq 0

git clone --quiet --depth 1 "file://$repository" "$scratch/shallow"
review "$scratch/shallow" "$base" "$locked_head"
check "a shallow checkout is an error" test "$review_status" -eq 2
check "a shallow checkout names the fix" grep -q "fetch-depth: 0" "$summary"

fake_bin="$scratch/fake-bin"
mkdir -p "$fake_bin"
cat > "$fake_bin/gh" <<'GH'
#!/usr/bin/env bash
echo "$*" >> "$FAKE_GH_LOG"
if [ "$1 $2" = "attestation verify" ]; then
  exit "$FAKE_ATTESTATION_STATUS"
fi
while [ $# -gt 0 ]; do
  case "$1" in
    --pattern) pattern="$2" ;;
    --dir) folder="$2" ;;
  esac
  shift
done
target="${pattern#vigilante-}"
target="${target%.tar.gz}"
mkdir -p "$FAKE_STAGING/vigilante-$target"
printf '#!/bin/sh\necho "vigilante 0.1.0"\n' > "$FAKE_STAGING/vigilante-$target/vigilante"
chmod +x "$FAKE_STAGING/vigilante-$target/vigilante"
tar -czf "$folder/$pattern" -C "$FAKE_STAGING" "vigilante-$target"
GH
chmod +x "$fake_bin/gh"

install() {
  local runner_os="$1" runner_arch="$2" attestation_status="$3"
  runner_temp="$(mktemp -d)"
  github_path="$runner_temp/path"
  gh_log="$runner_temp/gh.log"
  : > "$github_path"
  install_status=0
  PATH="$fake_bin:$PATH" RUNNER_OS="$runner_os" RUNNER_ARCH="$runner_arch" RUNNER_TEMP="$runner_temp" \
    GITHUB_PATH="$github_path" VERSION=0.1.0 FAKE_GH_LOG="$gh_log" FAKE_STAGING="$runner_temp/staging" \
    FAKE_ATTESTATION_STATUS="$attestation_status" "$action_folder/install.sh" > /dev/null 2>&1 || install_status=$?
}

for platform in "Linux X64 x86_64-unknown-linux-musl" "Linux ARM64 aarch64-unknown-linux-musl" \
  "macOS X64 x86_64-apple-darwin" "macOS ARM64 aarch64-apple-darwin"; do
  read -r runner_os runner_arch target <<< "$platform"
  install "$runner_os" "$runner_arch" 0
  check "$runner_os $runner_arch downloads $target" grep -q -- "--pattern vigilante-$target.tar.gz" "$gh_log"
  check "$runner_os $runner_arch adds the verified binary to PATH" test -x "$(cat "$github_path")/vigilante"
done

install Windows X64 0
check "Windows is an error" test "$install_status" -eq 2

install Linux X64 1
check "a failed attestation stops the install" test "$install_status" -ne 0
check "a failed attestation adds nothing to PATH" test ! -s "$github_path"
check "the attestation check pins the release workflow and tag" \
  grep -q -- "--signer-workflow theonly1me/vigilante/.github/workflows/release.yml --source-ref refs/tags/v0.1.0" "$gh_log"

rm -rf "$scratch"
if [ "$failures" -gt 0 ]; then
  echo "$failures checks failed." >&2
  exit 1
fi
echo "All action checks passed."
