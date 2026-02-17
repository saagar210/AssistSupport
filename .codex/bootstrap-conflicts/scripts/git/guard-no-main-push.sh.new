#!/usr/bin/env bash
set -euo pipefail

# codex-os-managed
branch="$(git rev-parse --abbrev-ref HEAD)"
if [[ "$branch" == "main" || "$branch" == "master" ]]; then
  echo "Pushing from $branch is blocked."
  exit 1
fi
