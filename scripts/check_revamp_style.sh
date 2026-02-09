#!/usr/bin/env bash
set -euo pipefail

# Guardrail for Phase 5: revamp UI should not introduce ad-hoc hardcoded colors.
# Tokens live in src/styles/revamp/tokens.css and revamp theme gradients live in
# src/styles/revamp/theme.css. Everything else should reference tokens.

cd "$(dirname "$0")/.."

# Note: this is intentionally conservative (it may flag false positives), but
# should never silently pass due to a regex error.
PATTERN='rgba\(|hsla?\(|#[0-9a-fA-F]{3,8}\b'

set +e
MATCHES="$(
  rg -n "$PATTERN" \
    -g'*.css' \
    src/features/revamp \
    src/styles/revamp \
    --glob '!src/styles/revamp/tokens.css' \
    --glob '!src/styles/revamp/theme.css' \
    --glob '!src/styles/revamp/motion.css'
)"
RG_STATUS=$?
set -e

if [[ $RG_STATUS -eq 0 ]]; then
  echo "Revamp style guard failed: hardcoded colors found in revamp CSS."
  echo
  echo "Use tokens from src/styles/revamp/tokens.css (or add a token) instead of raw rgba/hex."
  echo
  echo "$MATCHES"
  exit 1
fi

if [[ $RG_STATUS -ne 1 ]]; then
  echo "Revamp style guard error: rg exited with unexpected status $RG_STATUS"
  exit "$RG_STATUS"
fi

echo "Revamp style guard: PASS (no hardcoded colors detected)"
