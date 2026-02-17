#!/usr/bin/env bash
set -euo pipefail

if [[ ! -f .git/CODEX_LAST_WIP ]]; then
  echo "No saved WIP tag found in .git/CODEX_LAST_WIP."
  exit 1
fi

tag="$(cat .git/CODEX_LAST_WIP)"

if ! git stash list | grep -F "$tag" >/dev/null; then
  echo "Saved WIP stash not found: $tag"
  exit 1
fi

git stash apply "stash^{/$tag}"
echo "Restored WIP: $tag"

