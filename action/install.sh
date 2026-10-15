#!/usr/bin/env bash
set -euo pipefail

repository=theonly1me/vigilante

case "$RUNNER_OS $RUNNER_ARCH" in
  "Linux X64") target=x86_64-unknown-linux-musl ;;
  "Linux ARM64") target=aarch64-unknown-linux-musl ;;
  "macOS X64") target=x86_64-apple-darwin ;;
  "macOS ARM64") target=aarch64-apple-darwin ;;
  *)
    echo "vigilante: there is no binary for $RUNNER_OS $RUNNER_ARCH. Use a Linux or macOS runner." >&2
    exit 2
    ;;
esac

archive="vigilante-$target.tar.gz"
folder="$RUNNER_TEMP/vigilante-$VERSION"
mkdir -p "$folder"
gh release download "v$VERSION" --repo "$repository" --pattern "$archive" --dir "$folder" --clobber
gh attestation verify "$folder/$archive" \
  --repo "$repository" \
  --signer-workflow "$repository/.github/workflows/release.yml" \
  --source-ref "refs/tags/v$VERSION" \
  --deny-self-hosted-runners
tar -xzf "$folder/$archive" -C "$folder" --strip-components=1
"$folder/vigilante" --version
echo "$folder" >> "$GITHUB_PATH"
