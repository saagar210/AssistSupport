#!/usr/bin/env bash
set -euo pipefail

tag="${1:-}"
shift || true

if [[ -z "$tag" ]]; then
  echo "Usage: run-playwright-tag.sh <@tag> [extra args]"
  exit 1
fi

if [[ ! -d tests/ui ]]; then
  echo "No tests/ui directory found; cannot run Playwright tests for $tag."
  exit 1
fi

if ! find tests/ui -type f -name "*.spec.ts" -print -quit | grep -q .; then
  echo "No Playwright specs found in tests/ui; cannot run tests for $tag."
  exit 1
fi

pnpm exec playwright test tests/ui --grep "$tag" "$@"
