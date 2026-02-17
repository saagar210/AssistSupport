#!/usr/bin/env bash
set -euo pipefail

BASE="${1:-origin/master}"

echo "## What"
git log --no-merges --format="- %s" "${BASE}..HEAD"
echo
echo "## Added"
git diff --name-only --diff-filter=A "${BASE}...HEAD" | sed 's/^/- /' || true
echo
echo "## Modified"
git diff --name-only --diff-filter=M "${BASE}...HEAD" | sed 's/^/- /' || true
echo
echo "## Removed"
git diff --name-only --diff-filter=D "${BASE}...HEAD" | sed 's/^/- /' || true
echo
echo "## Files changed"
git diff --name-status "${BASE}...HEAD" | awk '{print "- "$0}'
echo
echo "## Testing"
echo "- Commands run:"
echo "- Results:"
echo
echo "## Risks / Notes"
echo "- Add known tradeoffs and follow-ups."
