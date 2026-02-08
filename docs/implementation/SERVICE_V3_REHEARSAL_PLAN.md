# Service.v3 Rehearsal Plan (Producer)

Updated: 2026-02-08
Owner: MemoryKernel

## Scope
Pre-cutover rehearsal for service.v3 artifacts and handoff discipline. No runtime production cutover in this phase.

## Rehearsal Window
- Duration: 14 calendar days (1 sprint)

## Producer Deliverables
1. Immutable rehearsal candidate tag/sha for service.v3 pre-release branch.
2. Updated canonical producer manifest:
   - `contracts/integration/v1/producer-contract-manifest.json`
3. OpenAPI/spec/docs updates for service.v3 envelope policy.
4. Handoff packet including consumer impact statement.

## Policy Locks
1. Non-2xx envelope keeps `api_contract_version` absent unless a future joint RFC changes policy.
2. `legacy_error` removal is blocked until all service.v3 cutover gates pass.
3. Consumer manifest-hash validation must remain green throughout rehearsal.

## Verification Commands
```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
./scripts/verify_producer_contract_manifest.sh --memorykernel-root /Users/d/Projects/MemoryKernel
./scripts/verify_service_contract_alignment.sh --memorykernel-root /Users/d/Projects/MemoryKernel
./scripts/verify_contract_parity.sh --canonical-root /Users/d/Projects/MemoryKernel
./scripts/verify_trilogy_compatibility_artifacts.sh --memorykernel-root /Users/d/Projects/MemoryKernel
./scripts/run_trilogy_smoke.sh --memorykernel-root /Users/d/Projects/MemoryKernel
./scripts/run_trilogy_compliance_suite.sh --memorykernel-root /Users/d/Projects/MemoryKernel --skip-baseline
```

## Exit Criteria
1. Producer verification suite is green.
2. Consumer rehearsal PR is green against service.v3 candidate.
3. Joint go/no-go record is captured before any runtime cutover.

