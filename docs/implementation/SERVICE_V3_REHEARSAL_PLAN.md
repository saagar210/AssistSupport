# Service.v3 Rehearsal Plan (Producer)

Updated: 2026-02-08  
Owner: MemoryKernel  
Mode: Planning/rehearsal only (no runtime cutover)

## Objective
Create a consumer-validatable `service.v3` rehearsal package while keeping live runtime pinned to:
- `release_tag`: `v0.3.2`
- `commit_sha`: `cf331449e1589581a5dcbb3adecd3e9ae4509277`
- `service_contract_version`: `service.v2`
- `api_contract_version`: `api.v1`
- `integration_baseline`: `integration/v1`

## Inputs and Constraints
1. AssistSupport must be able to validate rehearsal artifacts in CI immediately.
2. `service.v2` behavior remains stable during rehearsal.
3. No removal of `legacy_error` until explicit `service.v3` cutover gates pass.
4. Non-2xx envelope policy remains explicit:
   - `service.v2`: requires `service_contract_version`, `error.code`, `error.message`, `legacy_error`; forbids `api_contract_version`.
   - `service.v3` candidate: requires `service_contract_version`, `error.code`, `error.message`; optional `error.details`; forbids `legacy_error`, `api_contract_version`.

## Producer Deliverables (Phase 4 Rehearsal Block)
1. Rehearsal planning docs:
   - `docs/implementation/SERVICE_V3_REHEARSAL_PLAN.md`
   - `docs/implementation/SERVICE_V3_REHEARSAL_EXECUTION_TRACKER.md`
   - `docs/implementation/SERVICE_V3_RFC_DRAFT.md`
2. Producer cutover gates:
   - `docs/implementation/SERVICE_V3_CUTOVER_GATES.md`
3. Consumer-ready rehearsal handoff payload:
   - `docs/implementation/SERVICE_V3_REHEARSAL_HANDOFF_CANDIDATE.json`
   - `docs/implementation/PRODUCER_RELEASE_HANDOFF_LATEST.json` (generated in `service-v3-candidate` mode)
4. Deterministic payload generator:
   - `scripts/generate_service_v3_rehearsal_payload.sh`
   - `scripts/generate_producer_handoff_payload.sh --mode service-v3-candidate`

## Rehearsal Exit Criteria
All must pass:
1. Producer verification suite is green.
2. Rehearsal payload JSON matches current baseline manifest and cutover gate policy.
3. AssistSupport can run pin/contract CI checks against rehearsal payload (no runtime cutover required).
4. Joint cutover dependencies are explicit (producer prerequisites, consumer prerequisites, rollback triggers, evidence checklist).
5. Compatibility mapping is explicit:
   - `expected_service_contract_version` in candidate payload is `service.v3`.
   - runtime baseline fields still reflect pinned `v0.3.2` / `service.v2` state.

## Verification Commands
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

## Expected AssistSupport Validation Signals
1. `pnpm run check:memorykernel-pin` passes with rehearsal payload assumptions.
2. `pnpm run test:memorykernel-contract` passes for v2 stable + v3 rehearsal expectations.
3. `pnpm run test:ci` remains green with no runtime behavior change required.
