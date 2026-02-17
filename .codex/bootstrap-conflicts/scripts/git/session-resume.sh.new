#!/usr/bin/env bash
set -euo pipefail

# codex-os-managed
if [[ ! -f .git/CODEX_LAST_WIP ]]; then
  echo "No saved WIP tag found."
  exit 1
fi

tag="$(cat .git/CODEX_LAST_WIP)"
git stash list | grep -F "$tag" >/dev/null
git stash apply "stash^{/$tag}"
echo "Restored WIP: $tag"
