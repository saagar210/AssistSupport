# Service.v3 Rehearsal Execution Tracker (MemoryKernel)

Updated: 2026-02-08
Owner: MemoryKernel

## Baseline Guardrail
- Active runtime must remain pinned to `v0.3.2` / `cf331449e1589581a5dcbb3adecd3e9ae4509277`.
- This block is rehearsal-only and must not force consumer runtime cutover.

## Phase Execution Tasks

### Task P1: Rehearsal planning package
- Status: Completed
- Owner: MemoryKernel
- Definition of done:
  - rehearsal plan updated with explicit no-cutover policy
  - RFC draft and tracker aligned to the same baseline assumptions

### Task P2: Producer handoff candidate payload
- Status: Completed
- Owner: MemoryKernel
- Definition of done:
  - `SERVICE_V3_REHEARSAL_HANDOFF_CANDIDATE.json` generated from canonical manifest
  - payload includes error envelope policy, error-code expectations, migration overlap assumptions
  - payload can be consumed by AssistSupport CI checks

### Task P3: Producer verification suite
- Status: Completed
- Owner: MemoryKernel
- Definition of done:
  - all required producer commands pass
  - no regressions in parity/alignment/smoke/compliance gates

### Task P4: Joint rehearsal entry handshake
- Status: Ready
- Owner: Joint
- Definition of done:
  - AssistSupport confirms rehearsal payload acceptance
  - AssistSupport validates rehearsal assumptions with consumer contract checks
  - joint evidence references are recorded for cutover-governance

## Required Commands
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

## Evidence Artifacts
1. `docs/implementation/SERVICE_V3_REHEARSAL_HANDOFF_CANDIDATE.json`
2. `docs/implementation/SERVICE_V3_CUTOVER_GATES.md`
3. `docs/implementation/PRODUCER_RELEASE_HANDOFF_LATEST.json` (service.v3 candidate mode)
4. Producer verification outputs from required commands

## Exit Criteria
1. P1-P3 are completed and verified green.
2. P4 is acknowledged by AssistSupport with explicit acceptance criteria.
3. Service.v3 remains in rehearsal mode until cutover gates are explicitly passed.
