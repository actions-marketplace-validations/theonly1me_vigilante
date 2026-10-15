#!/usr/bin/env bash
set -euo pipefail

formula="$1"
tag="$2"
folder="$RUNNER_TEMP/homebrew-tap"
key="$RUNNER_TEMP/tap-key"
known_hosts="$RUNNER_TEMP/known_hosts"

printf '%s\n' "$TAP_DEPLOY_KEY" > "$key"
chmod 600 "$key"
trap 'rm -f "$key"' EXIT
gh api meta --jq '.ssh_keys[] | "github.com " + .' > "$known_hosts"
export GIT_SSH_COMMAND="ssh -i $key -o IdentitiesOnly=yes -o UserKnownHostsFile=$known_hosts"

git clone --quiet --depth 1 "git@github.com:$GITHUB_REPOSITORY_OWNER/homebrew-tap.git" "$folder"
mkdir -p "$folder/Formula"
cp "$formula" "$folder/Formula/vigilante.rb"
git -C "$folder" add Formula/vigilante.rb
if git -C "$folder" diff --cached --quiet; then
  echo "The tap already has vigilante $tag."
  exit 0
fi
git -C "$folder" -c user.name="github-actions[bot]" \
  -c user.email="41898282+github-actions[bot]@users.noreply.github.com" \
  commit --quiet --message "vigilante ${tag#v}"
git -C "$folder" push --quiet origin HEAD
echo "The tap now has vigilante $tag."
