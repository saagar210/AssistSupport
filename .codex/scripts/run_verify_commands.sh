#!/usr/bin/env bash
set -euo pipefail

VERIFY_FILE="${1:-.codex/verify.commands}"
if [[ ! -f "$VERIFY_FILE" ]]; then
  echo "missing verify commands file: $VERIFY_FILE" >&2
  exit 1
fi

failed=0
skip_regex="${VERIFY_SKIP_REGEX:-}"
while IFS= read -r cmd || [[ -n "$cmd" ]]; do
  [[ -z "$cmd" ]] && continue
  [[ "$cmd" =~ ^# ]] && continue
  if [[ -n "$skip_regex" && "$cmd" =~ $skip_regex ]]; then
    echo ">>> (skipped by VERIFY_SKIP_REGEX) $cmd"
    continue
  fi
  echo ">>> $cmd"
  if ! bash -lc "$cmd"; then
    failed=1
    break
  fi
done < "$VERIFY_FILE"

exit "$failed"
