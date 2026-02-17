#!/usr/bin/env bash
set -euo pipefail

# codex-os-managed
base="${1:-origin/main}"
if ! git rev-parse --verify "$base" >/dev/null 2>&1; then
  base="origin/master"
fi

echo "## What"
git log --no-merges --format='- %s' "${base}..HEAD"
echo
echo "## Files changed"
git diff --name-status "${base}...HEAD" | awk '{print "- "$0}'
echo
echo "## Testing"
echo "- Add commands run and outcomes."
echo
echo "## Performance impact"
echo "- Bundle delta:"
echo "- Build time delta:"
echo "- Lighthouse/API/DB:"
echo
echo "## Risk / Notes"
echo "- Add tradeoffs and follow-ups."
