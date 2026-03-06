#!/usr/bin/env bash
set -euo pipefail

if [[ "${GIT_GUARD_SKIP_BACKEND_GATES:-0}" == "1" ]]; then
  echo "Skipping backend gate guard (GIT_GUARD_SKIP_BACKEND_GATES=1)."
  exit 0
fi

pnpm run ci:backend:gates:all
