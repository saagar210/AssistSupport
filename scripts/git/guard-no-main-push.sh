#!/usr/bin/env bash
set -euo pipefail

branch="$(git rev-parse --abbrev-ref HEAD)"

if [[ "$branch" == "main" || "$branch" == "master" ]]; then
  echo "Pushing from $branch is blocked."
  exit 1
fi

# pre-push hook payload: <local ref> <local sha> <remote ref> <remote sha>
while IFS=' ' read -r _local_ref _local_sha remote_ref _remote_sha; do
  if [[ "$remote_ref" =~ refs/heads/(main|master)$ ]]; then
    echo "Push to protected branch (${remote_ref#refs/heads/}) is blocked."
    exit 1
  fi
done

