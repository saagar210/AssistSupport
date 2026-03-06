#!/usr/bin/env bash
set -euo pipefail

echo "Running lock-await lint gate (clippy::await_holding_lock)..."
(
  cd src-tauri
  cargo clippy --tests -- -D clippy::await_holding_lock
)

echo "Validating lock-await exception annotations and generating report..."
node scripts/ci/report-lock-await-exceptions.mjs

echo "Lock policy gate passed."
