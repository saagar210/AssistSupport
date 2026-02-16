#!/usr/bin/env bash
set -euo pipefail

# Codex artifact routing defaults (v6.0)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
if [ -f "$SCRIPT_DIR/_artifact_env.sh" ]; then
  # shellcheck source=/dev/null
  source "$SCRIPT_DIR/_artifact_env.sh"
fi
pnpm run build
