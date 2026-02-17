#!/usr/bin/env bash
set -euo pipefail

# codex-os-managed
tag="codex-wip/$(git rev-parse --abbrev-ref HEAD)/$(date +%Y%m%d-%H%M%S)"
git stash push -u -m "$tag"
echo "$tag" > .git/CODEX_LAST_WIP
echo "Saved WIP: $tag"
