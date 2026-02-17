#!/usr/bin/env bash
set -euo pipefail

# codex-os-managed
max_files="${GIT_GUARD_MAX_FILES:-25}"
count="$(git diff --cached --name-only | sed '/^$/d' | wc -l | tr -d ' ')"

if (( count == 0 )); then
  echo "No staged files; skipping atomicity check."
  exit 0
fi

if (( count > max_files )); then
  echo "Too many staged files ($count > $max_files). Split into atomic commits."
  exit 1
fi
