# MemoryKernel Release Handoff Template (Producer -> AssistSupport Consumer)

Use this template for every MemoryKernel baseline promotion request.

## Producer Release Identity
- release_tag:
- commit_sha:
- service_contract_version:
- api_contract_version:
- integration_baseline:

## Contract Guarantees
- Non-2xx envelope policy statement:
- `legacy_error` lifecycle statement:
- `error.code` enum changes (if any):
  - Added:
  - Removed:
  - Renamed:

## Required Producer Artifacts
- [ ] Updated canonical producer manifest (`contracts/integration/v1/producer-contract-manifest.json`)
- [ ] OpenAPI updates for changed behavior
- [ ] Contract/spec documentation updates
- [ ] Verification evidence links attached

## Required Producer Verification
```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
./scripts/verify_service_contract_alignment.sh --memorykernel-root /Users/d/Projects/MemoryKernel
./scripts/verify_contract_parity.sh --canonical-root /Users/d/Projects/MemoryKernel
./scripts/verify_trilogy_compatibility_artifacts.sh --memorykernel-root /Users/d/Projects/MemoryKernel
./scripts/run_trilogy_smoke.sh --memorykernel-root /Users/d/Projects/MemoryKernel
./scripts/run_trilogy_compliance_suite.sh --memorykernel-root /Users/d/Projects/MemoryKernel --skip-baseline
```

## Consumer Update Requirements (Atomic in one PR)
- [ ] Update `config/memorykernel-integration-pin.json`
- [ ] Update `config/memorykernel-producer-manifest.json`
- [ ] Update `docs/MEMORYKERNEL_COMPATIBILITY_MATRIX.md`
- [ ] Run `pnpm run check:memorykernel-pin`
- [ ] Run `pnpm run check:memorykernel-governance`
- [ ] Run `pnpm run test:memorykernel-contract`
- [ ] Run `pnpm run test:ci`

## Change Risk Assessment
- Runtime behavior changes expected in consumer: Yes/No
- Rollback required if failure: Yes/No
- Rollback target baseline:

## Handoff Notes
- Additional migration notes:
- Known caveats:
- Earliest safe adoption date:
