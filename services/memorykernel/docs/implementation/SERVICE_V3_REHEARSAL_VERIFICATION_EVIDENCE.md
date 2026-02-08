# Service.v3 Rehearsal Verification Evidence (Producer)

Updated: 2026-02-08  
Scope: planning/rehearsal evidence only (no runtime cutover)

## Runtime Baseline (unchanged)
- `release_tag`: `v0.3.2`
- `commit_sha`: `cf331449e1589581a5dcbb3adecd3e9ae4509277`
- `service_contract_version`: `service.v2`
- `api_contract_version`: `api.v1`
- `integration_baseline`: `integration/v1`

## Rehearsal Candidate Signals
- `docs/implementation/PRODUCER_RELEASE_HANDOFF_LATEST.json` generated in `service-v3-candidate` mode
- `docs/implementation/SERVICE_V3_REHEARSAL_HANDOFF_CANDIDATE.json` refreshed from canonical manifest
- Explicit non-2xx envelope policy is documented in:
  - `docs/implementation/SERVICE_V3_CUTOVER_GATES.md`
  - `docs/implementation/SERVICE_V3_RFC_DRAFT.md`

## Reproducibility Commands
```bash
./scripts/generate_producer_handoff_payload.sh \
  --mode service-v3-candidate \
  --memorykernel-root /Users/d/Projects/MemoryKernel \
  --out-json /Users/d/Projects/MemoryKernel/docs/implementation/PRODUCER_RELEASE_HANDOFF_LATEST.json

./scripts/generate_service_v3_rehearsal_payload.sh \
  --memorykernel-root /Users/d/Projects/MemoryKernel \
  --out-json /Users/d/Projects/MemoryKernel/docs/implementation/SERVICE_V3_REHEARSAL_HANDOFF_CANDIDATE.json
```

## Required Producer Verification Commands
```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
./scripts/verify_service_contract_alignment.sh --memorykernel-root /Users/d/Projects/MemoryKernel
./scripts/verify_contract_parity.sh --canonical-root /Users/d/Projects/MemoryKernel
./scripts/verify_trilogy_compatibility_artifacts.sh --memorykernel-root /Users/d/Projects/MemoryKernel
./scripts/verify_producer_handoff_payload.sh --memorykernel-root /Users/d/Projects/MemoryKernel
./scripts/run_trilogy_smoke.sh --memorykernel-root /Users/d/Projects/MemoryKernel
./scripts/run_trilogy_compliance_suite.sh --memorykernel-root /Users/d/Projects/MemoryKernel --skip-baseline
```

## Latest Producer Verification Run
- Run date (UTC): 2026-02-08
- Result: all required commands passed
- Summary:
  - `cargo fmt --all -- --check`: pass
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings`: pass
  - `cargo test --workspace --all-targets --all-features`: pass
  - `verify_service_contract_alignment.sh`: pass
  - `verify_contract_parity.sh`: pass
  - `verify_trilogy_compatibility_artifacts.sh`: pass
  - `verify_producer_handoff_payload.sh`: pass
  - `run_trilogy_smoke.sh`: pass
  - `run_trilogy_compliance_suite.sh --skip-baseline`: pass

## Expected Consumer Validation Commands
```bash
pnpm run check:memorykernel-handoff:service-v3-candidate
pnpm run check:memorykernel-pin
pnpm run test:memorykernel-contract
pnpm run test:ci
```
