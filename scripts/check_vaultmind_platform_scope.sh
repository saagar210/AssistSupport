#!/usr/bin/env bash
set -euo pipefail

# Guardrail: VaultMind is treated as macOS-first in this monorepo today.
# If Linux distribution is introduced, re-evaluate the current Dependabot dismissal on `glib`
# (GHSA-wrw7-89jp-8q8g) and update the dependency chain accordingly.

OS_NAME="$(uname -s 2>/dev/null || echo unknown)"

if [[ "${VAULTMIND_ENABLE_LINUX_DIST:-}" == "1" ]]; then
  cat <<'EOF'
[FAIL] VAULTMIND_ENABLE_LINUX_DIST=1 is set, but VaultMind Linux distribution is not validated.

Next actions (required):
1) Re-open/resolve `glib` advisory GHSA-wrw7-89jp-8q8g for Linux builds (glib >= 0.20.0).
2) Update docs/monorepo/VAULTMIND_BOUNDARY.md platform scope and evidence.
3) Add Linux build/test gates to scripts/run_monorepo_readiness.sh (full).
EOF
  exit 1
fi

if [[ "$OS_NAME" == "Linux" ]]; then
  if [[ "${ASSISTSUPPORT_ALLOW_VAULTMIND_LINUX:-}" != "1" ]]; then
    cat <<'EOF'
[FAIL] VaultMind Linux distribution is not supported in this monorepo readiness gate.

Rationale:
- VaultMind is macOS-first today.
- A Dependabot alert on `glib` (GHSA-wrw7-89jp-8q8g) was dismissed as not_used based on macOS-only distribution.

If you are intentionally adding Linux support:
1) Resolve the `glib` advisory for Linux builds (glib >= 0.20.0).
2) Update docs/monorepo/VAULTMIND_BOUNDARY.md platform scope and evidence.
3) Re-run with ASSISTSUPPORT_ALLOW_VAULTMIND_LINUX=1 once the above is complete.
EOF
    exit 1
  fi
fi

echo "[ok] VaultMind platform scope check passed (os=${OS_NAME})."

