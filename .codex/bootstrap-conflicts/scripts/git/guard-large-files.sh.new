#!/usr/bin/env bash
set -euo pipefail

# codex-os-managed
max_bytes="${GIT_GUARD_MAX_BYTES:-2097152}"
fail=0
while IFS= read -r file; do
  [[ -f "$file" ]] || continue
  size=$(wc -c <"$file")
  if (( size > max_bytes )); then
    echo "Large file staged (>${max_bytes} bytes): $file"
    fail=1
  fi
done < <(git diff --cached --name-only --diff-filter=AM)
exit $fail
