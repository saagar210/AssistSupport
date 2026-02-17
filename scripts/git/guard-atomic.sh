#!/usr/bin/env bash
set -euo pipefail

if [[ "${GIT_GUARD_ALLOW_LARGE_COMMIT:-0}" == "1" ]]; then
  exit 0
fi

max_files="${GIT_GUARD_MAX_STAGED_FILES:-30}"
count="$(git diff --cached --name-only --diff-filter=ACMR | wc -l | tr -d ' ')"

if ((count > max_files)); then
  echo "Too many staged files for an atomic commit: $count > $max_files"
  echo "Split this into smaller logical commits, or set GIT_GUARD_ALLOW_LARGE_COMMIT=1 for an intentional exception."
  exit 1
fi

