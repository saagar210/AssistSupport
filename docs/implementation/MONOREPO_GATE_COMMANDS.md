# Monorepo Gate Commands

## AssistSupport Baseline and Validation Commands
```bash
pnpm run check:workstation-preflight
pnpm run check:monorepo-readiness
pnpm run check:monorepo-readiness:full
pnpm run typecheck
pnpm run test
pnpm run check:memorykernel-pin
pnpm run check:memorykernel-governance
pnpm run check:memorykernel-handoff
pnpm run check:memorykernel-handoff:service-v3-candidate
pnpm run test:memorykernel-contract
pnpm run test:memorykernel-phase3-dry-run
pnpm run test:memorykernel-cutover-dry-run
pnpm run test:ci
```

## MemoryKernel Baseline and Validation Commands
```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
cd services/memorykernel
./scripts/verify_service_contract_alignment.sh --memorykernel-root "$(pwd)"
./scripts/verify_contract_parity.sh --canonical-root "$(pwd)"
./scripts/verify_trilogy_compatibility_artifacts.sh --memorykernel-root "$(pwd)"
./scripts/run_trilogy_smoke.sh --memorykernel-root "$(pwd)"
./scripts/run_trilogy_compliance_suite.sh --memorykernel-root "$(pwd)" --skip-baseline
./scripts/verify_producer_contract_manifest.sh --memorykernel-root "$(pwd)"
./scripts/verify_producer_handoff_payload.sh --memorykernel-root "$(pwd)"
```

## Required Evidence Artifacts Per Gate
For every gate execution, write:
1. Human-readable evidence markdown under `docs/implementation/evidence/`
2. Machine-readable summary JSON under `docs/implementation/evidence/`

## Final Migration Gates
1. `G5`: AssistSupport monorepo validation (`artifacts/monorepo-gates/GATE_G5_ASSISTSUPPORT_MONOREPO_VALIDATION.log`)
2. `G6`: Imported MemoryKernel validation (`artifacts/monorepo-gates/GATE_G6_IMPORTED_MEMORYKERNEL_VALIDATION.log`)
3. `G7`: Monorepo decision checkpoint (`docs/implementation/MONOREPO_DECISION_CHECKPOINT.md`)
