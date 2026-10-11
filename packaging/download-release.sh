#!/usr/bin/env bash
set -euo pipefail

tag="$1"
pattern="$2"
folder="$3"
repository="$GITHUB_REPOSITORY"
commit="$(gh api "repos/$repository/commits/$tag" --jq .sha)"

mkdir -p "$folder"
gh release download "$tag" --repo "$repository" --pattern "$pattern" --dir "$folder" --clobber
for file in "$folder"/*; do
  gh attestation verify "$file" \
    --repo "$repository" \
    --signer-workflow "$repository/.github/workflows/release.yml" \
    --source-ref refs/heads/main \
    --source-digest "$commit" \
    --deny-self-hosted-runners > /dev/null
  echo "Verified $(basename "$file")."
done
