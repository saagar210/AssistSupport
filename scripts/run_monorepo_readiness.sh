#!/usr/bin/env bash
set -euo pipefail

MODE="${1:-quick}"
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

run_step() {
  echo
  echo "==> $*"
  "$@"
}

if [[ "$MODE" != "quick" && "$MODE" != "full" ]]; then
  echo "Usage: scripts/run_monorepo_readiness.sh [quick|full]"
  exit 1
fi

cd "$ROOT_DIR"

run_step pnpm run check:workstation-preflight
run_step pnpm run check:memorykernel-pin
run_step pnpm run check:memorykernel-governance
run_step pnpm run check:memorykernel-handoff
run_step pnpm run check:memorykernel-handoff:service-v3-candidate
run_step pnpm run check:memorykernel-boundary
run_step pnpm run check:memorykernel-cutover-policy
run_step pnpm run typecheck
run_step pnpm run test:memorykernel-contract

if [[ "$MODE" == "full" ]]; then
  run_step pnpm run test:ci

  pushd services/memorykernel >/dev/null
  run_step cargo fmt --all -- --check
  run_step cargo clippy --workspace --all-targets --all-features -- -D warnings
  run_step cargo test --workspace --all-targets --all-features
  run_step ./scripts/verify_service_contract_alignment.sh --memorykernel-root "$(pwd)"
  run_step ./scripts/verify_contract_parity.sh --canonical-root "$(pwd)"
  run_step ./scripts/verify_trilogy_compatibility_artifacts.sh --memorykernel-root "$(pwd)"
  run_step ./scripts/run_trilogy_smoke.sh --memorykernel-root "$(pwd)"
  run_step ./scripts/run_trilogy_compliance_suite.sh --memorykernel-root "$(pwd)" --skip-baseline
  run_step ./scripts/verify_producer_contract_manifest.sh --memorykernel-root "$(pwd)"
  run_step ./scripts/verify_producer_handoff_payload.sh --memorykernel-root "$(pwd)"
  popd >/dev/null
fi

echo
echo "Monorepo readiness check (${MODE}) passed."
