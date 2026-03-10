#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root/search-api"

if [[ -x "./venv/bin/python3" ]]; then
  exec ./venv/bin/python3 "$@"
fi

exec python3 "$@"
