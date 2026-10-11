#!/usr/bin/env bash
set -euo pipefail

version="${1#v}"
if curl --silent --fail https://index.crates.io/vi/gi/vigilante | grep -q "\"vers\":\"$version\""; then
  echo "exists=true"
else
  echo "exists=false"
fi
