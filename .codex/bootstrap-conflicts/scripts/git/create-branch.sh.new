#!/usr/bin/env bash
set -euo pipefail

# codex-os-managed
if [[ $# -lt 2 ]]; then
  echo "usage: $0 <type> <task description>"
  exit 2
fi

branch_type="$1"
shift

task="$*"
slug="$(echo "$task" | tr '[:upper:]' '[:lower:]' | sed -E 's/[^a-z0-9]+/-/g; s/^-+|-+$//g; s/-+/-/g')"
branch="codex/${branch_type}/${slug}"

git switch -c "$branch"
echo "$branch"
