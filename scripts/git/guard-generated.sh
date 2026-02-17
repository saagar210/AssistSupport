#!/usr/bin/env bash
set -euo pipefail

forbidden='(^|/)(node_modules|dist|build|out|coverage|\.next|target|playwright-report|test-results|\.lighthouseci)/'

if git diff --cached --name-only | grep -E "$forbidden" >/dev/null; then
  echo "Generated artifacts are staged. Unstage them before commit."
  exit 1
fi

