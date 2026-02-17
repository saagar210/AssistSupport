#!/usr/bin/env bash
set -euo pipefail

task="${1:-}"
kind="${2:-feat}"
base="${3:-origin/master}"

if [[ -z "$task" ]]; then
  echo "Usage: pnpm git:branch:create -- \"task summary\" [feat|fix|chore|refactor|docs|test|perf|ci|spike|hotfix] [base]"
  exit 1
fi

if ! [[ "$kind" =~ ^(feat|fix|chore|refactor|docs|test|perf|ci|spike|hotfix)$ ]]; then
  echo "Invalid branch type: $kind"
  exit 1
fi

slug="$(echo "$task" | tr '[:upper:]' '[:lower:]' | sed -E 's/[^a-z0-9]+/-/g; s/^-+//; s/-+$//; s/-+/-/g')"
slug="${slug:0:48}"
branch="codex/${kind}/${slug}"

git fetch origin --quiet || true

if git show-ref --verify --quiet "refs/heads/$branch"; then
  git checkout "$branch"
else
  git checkout -b "$branch" "$base"
fi

echo "Using branch: $branch"

