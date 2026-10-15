#!/usr/bin/env bash
set -euo pipefail

wheels="$(cd "$1" && pwd)"
environment="$(mktemp -d)/venv"
python3 -m venv "$environment"
"$environment/bin/pip" install --quiet --no-index --find-links "$wheels" vigilante-scan
packaging/smoke.sh "$environment/bin/vigilante"
