#!/usr/bin/env bash
set -euo pipefail

if git ls-files -- ':(glob)**/go.mod' | grep -q .; then
  echo "modules=true" >> "$GITHUB_OUTPUT"
fi
