# Monorepo CI Gate Specification

## Purpose
Define the required CI gates for the unified AssistSupport + MemoryKernel monorepo execution model.

## Required Lanes

### 1) AssistSupport lanes
- `lint-frontend`
- `test-frontend`
- `memorykernel-contract`
- `test-e2e-smoke`
- `coverage-frontend`

### 2) MemoryKernel monorepo lane
- `memorykernel-monorepo`

Commands:
```bash
cd services/memorykernel
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
./scripts/verify_service_contract_alignment.sh --memorykernel-root "$(pwd)"
./scripts/verify_contract_parity.sh --canonical-root "$(pwd)"
./scripts/verify_trilogy_compatibility_artifacts.sh --memorykernel-root "$(pwd)"
./scripts/run_trilogy_smoke.sh --memorykernel-root "$(pwd)"
./scripts/run_trilogy_compliance_suite.sh --memorykernel-root "$(pwd)" --skip-baseline
./scripts/verify_producer_contract_manifest.sh --memorykernel-root "$(pwd)"
./scripts/verify_producer_handoff_payload.sh --memorykernel-root "$(pwd)"
```

### 3) Cross-governance lane
- `monorepo-governance`

Commands:
```bash
pnpm run check:memorykernel-pin
pnpm run check:memorykernel-governance
pnpm run check:memorykernel-handoff
pnpm run check:memorykernel-handoff:service-v3-candidate
pnpm run check:memorykernel-boundary
pnpm run check:memorykernel-cutover-policy
```

## Merge Policy
1. All lanes listed above are required for merges to `master`.
2. Any pin/matrix/manifest update must pass atomic-update policy in `memorykernel-contract` lane.
3. Runtime cutover remains blocked unless explicit bilateral decision records are updated and policy checks pass.
