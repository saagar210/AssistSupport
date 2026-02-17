#!/usr/bin/env bash
set -euo pipefail

# codex-os-managed
forbidden='(^|/)(node_modules|dist|build|out|coverage|\.next|target)/'
if git diff --cached --name-only | grep -E "$forbidden" >/dev/null; then
  echo "Generated artifacts are staged. Unstage them before commit."
  exit 1
fi
