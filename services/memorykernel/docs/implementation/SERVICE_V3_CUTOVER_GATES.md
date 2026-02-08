# Service.v3 Cutover Gates (Producer + Consumer)

Updated: 2026-02-08  
Scope: planning/rehearsal only (no runtime cutover in this phase)

## Purpose
Define explicit, testable cutover gates for migrating from `service.v2` to `service.v3` with clear rollback triggers and evidence requirements.

## Baseline Stability Statement
Until all cutover gates pass, runtime compatibility baseline remains:
- `release_tag`: `v0.3.2`
- `commit_sha`: `cf331449e1589581a5dcbb3adecd3e9ae4509277`
- `service_contract_version`: `service.v2`
- `api_contract_version`: `api.v1`
- `integration_baseline`: `integration/v1`

## Explicit Non-2xx Envelope Policy
Current stable (`service.v2`) policy:
1. Required fields: `service_contract_version`, `error.code`, `error.message`, `legacy_error`
2. Forbidden field: `api_contract_version`

Service.v3 rehearsal candidate policy:
1. Required fields: `service_contract_version`, `error.code`, `error.message`
2. Optional field: `error.details`
3. Forbidden fields: `legacy_error`, `api_contract_version`

## Producer prerequisites
1. Immutable release candidate prepared with:
   - release tag
   - commit SHA
   - updated OpenAPI
   - updated `producer-contract-manifest.json`
2. Contract docs and changelog updated with migration notes.
3. Full producer verification suite green.
4. Producer handoff packet published with consumer impact statement.

## Consumer prerequisites
1. Atomic PR updates:
   - pin
   - compatibility matrix
   - mirrored producer manifest
2. Contract verification green:
   - `pnpm run check:memorykernel-pin`
   - `pnpm run test:memorykernel-contract`
   - `pnpm run test:ci`
3. Deterministic fallback remains non-blocking for Draft flow.
4. Consumer replay/rollback rehearsal documented.

## Cutover go/no-go gates
All gates must pass:
1. Producer suite is green.
2. Consumer suite is green.
3. Non-2xx envelope contract assertions pass for candidate shape.
4. Joint sign-off record captured (producer + consumer).

## Gate Evidence Mapping
1. Producer suite is green:
   - `cargo fmt --all -- --check`
   - `cargo clippy --workspace --all-targets --all-features -- -D warnings`
   - `cargo test --workspace --all-targets --all-features`
   - `./scripts/verify_service_contract_alignment.sh --memorykernel-root /Users/d/Projects/MemoryKernel`
   - `./scripts/verify_contract_parity.sh --canonical-root /Users/d/Projects/MemoryKernel`
   - `./scripts/verify_trilogy_compatibility_artifacts.sh --memorykernel-root /Users/d/Projects/MemoryKernel`
   - `./scripts/run_trilogy_smoke.sh --memorykernel-root /Users/d/Projects/MemoryKernel`
   - `./scripts/run_trilogy_compliance_suite.sh --memorykernel-root /Users/d/Projects/MemoryKernel --skip-baseline`
2. Consumer suite is green:
   - `pnpm run check:memorykernel-pin`
   - `pnpm run test:memorykernel-contract`
   - `pnpm run test:ci`
3. Candidate shape assertions:
   - Non-2xx envelope assertions for service.v3 candidate pass.
4. Joint sign-off:
   - Handoff + acceptance evidence captured in both repos' implementation docs.

## Rollback triggers
Any of these triggers immediate rollback to last approved `service.v2` baseline:
1. Consumer contract suite fails after candidate repin.
2. Non-2xx envelope shape mismatch detected in integration tests.
3. Deterministic fallback regression in consumer flow.
4. Critical production-like rehearsal incident with unresolved root cause.

## Rollback action
1. Consumer repins to last approved `service.v2` release tag + SHA.
2. Producer publishes rollback advisory with impact and mitigation.
3. Overlap window reopens until blocking issue is fixed and re-verified.

## Evidence checklist
- Producer evidence:
  - release handoff payload JSON
  - command results for required producer verification suite
  - migration notes in changelog/spec docs
- Consumer evidence:
  - command results for required consumer suite
  - contract evidence artifact
  - rollback rehearsal note
- Joint evidence:
  - go/no-go decision log
  - known-risk acknowledgment list
