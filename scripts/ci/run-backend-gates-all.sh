#!/usr/bin/env bash
set -euo pipefail

artifacts_dir="${1:-artifacts/backend}"
summary_file="${artifacts_dir}/gates-summary.md"

mkdir -p "$artifacts_dir"
rm -f "$summary_file"

bash scripts/ci/run-backend-gate.sh \
  command-registry \
  "${artifacts_dir}/command-registry-gate.log" \
  "pnpm run ci:backend:gate:command-registry" \
  "$summary_file"

bash scripts/ci/run-backend-gate.sh \
  command-lifecycle \
  "${artifacts_dir}/command-lifecycle-gate.log" \
  "pnpm run ci:backend:gate:command-lifecycle" \
  "$summary_file"

bash scripts/ci/run-backend-gate.sh \
  lock-policy \
  "${artifacts_dir}/lock-policy-gate.log" \
  "pnpm run ci:backend:gate:lock-policy" \
  "$summary_file"

bash scripts/ci/run-backend-gate.sh \
  contracts-fixtures \
  "${artifacts_dir}/contracts-gate.log" \
  "pnpm run ci:backend:gate:contracts" \
  "$summary_file"

bash scripts/ci/run-backend-gate.sh \
  backend-tests \
  "${artifacts_dir}/backend-tests.log" \
  "pnpm run ci:backend:gate:backend-tests" \
  "$summary_file"

bash scripts/ci/run-backend-gate.sh \
  security-regression \
  "${artifacts_dir}/security-regression.log" \
  "pnpm run ci:backend:gate:security" \
  "$summary_file"

echo "Backend gates complete. Summary: ${summary_file}"
