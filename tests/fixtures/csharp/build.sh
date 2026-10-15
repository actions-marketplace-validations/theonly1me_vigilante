#!/bin/bash
set -eu
folder="$(cd "$(dirname "$0")" && pwd)"
docker run --rm -v "$folder:/work" mcr.microsoft.com/dotnet/sdk:8.0 bash /work/compile.sh
