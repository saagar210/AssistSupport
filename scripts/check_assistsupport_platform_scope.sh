#!/usr/bin/env bash
set -euo pipefail

# Guardrail: AssistSupport is treated as macOS-only in this repo today.
#
# Rationale:
# - README advertises macOS as the supported platform.
# - Linux builds pull in GTK/WebKitGTK dependency chains (via wry) which can surface
#   RustSec advisories/warnings that are not relevant to macOS distribution, but
#   become real obligations if Linux distribution is introduced.

OS_NAME="$(uname -s 2>/dev/null || echo unknown)"

if [[ "${ASSISTSUPPORT_ENABLE_LINUX_DIST:-}" == "1" ]]; then
  cat <<'EOF'
[FAIL] ASSISTSUPPORT_ENABLE_LINUX_DIST=1 is set, but AssistSupport Linux distribution is not validated.

Next actions (required):
1) Validate and remediate Linux-only dependency chains (GTK/WebKitGTK via wry/tauri), including any RustSec advisories.
2) Update docs and CI/build gates to reflect Linux platform scope and verification.
3) Add Linux build/test coverage and re-run full readiness gates.
EOF
  exit 1
fi

if [[ "$OS_NAME" == "Linux" ]]; then
  if [[ "${ASSISTSUPPORT_ALLOW_LINUX:-}" != "1" ]]; then
    cat <<'EOF'
[FAIL] AssistSupport Linux builds are out of scope for this monorepo readiness gate.

Rationale:
- AssistSupport is macOS-only today.
- Linux distribution changes the security/compliance posture due to additional native dependencies.

If you are intentionally adding Linux support:
1) Set ASSISTSUPPORT_ALLOW_LINUX=1 to bypass this guardrail temporarily.
2) Implement Linux-specific remediation and add Linux CI gates before shipping.
EOF
    exit 1
  fi
fi

echo "[ok] AssistSupport platform scope check passed (os=${OS_NAME})."

