#!/usr/bin/env bash
set -euo pipefail

# codex-os-managed
if ! command -v gitleaks >/dev/null 2>&1; then
  echo "gitleaks not found. Install gitleaks to enforce secret scanning."
  exit 1
fi

gitleaks protect --staged --redact
