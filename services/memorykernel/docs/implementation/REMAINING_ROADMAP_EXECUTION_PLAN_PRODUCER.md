# Remaining Roadmap Execution Plan (Producer)

> Historical Snapshot: This document captures pre-cutover planning/rehearsal state and is superseded by the runtime closure records in both repos.


Updated: 2026-02-08  
Scope: MemoryKernel producer execution for AssistSupport integration  
Constraint: Runtime baseline remains `v0.3.2` / `service.v2` / `api.v1` / `integration/v1` until explicit joint cutover approval.

## Objective
Close the remaining producer roadmap with clear, testable governance gates across:
1. Phase 4: service.v3 rehearsal package hardening
2. Phase 5: producer cutover-prep controls (no runtime cutover)
3. Phase 6: producer cutover governance + rollback evidence scaffolding

## Dependency Map
1. Canonical source of truth remains:
   - `contracts/integration/v1/producer-contract-manifest.json`
2. Producer handoff artifacts must stay deterministic and reproducible:
   - `docs/implementation/PRODUCER_RELEASE_HANDOFF_LATEST.json`
   - `docs/implementation/SERVICE_V3_REHEARSAL_HANDOFF_CANDIDATE.json`
3. CI/release gates must enforce policy drift detection.
4. AssistSupport consumes payloads in candidate mode without runtime cutover.

## Phase 4: Rehearsal Package Hardening

### Objective
Lock rehearsal artifacts so consumer CI can validate service.v3 candidate expectations without ambiguity.

### Producer deliverables
1. Stable rehearsal package docs:
   - `docs/implementation/SERVICE_V3_REHEARSAL_PLAN.md`
   - `docs/implementation/SERVICE_V3_REHEARSAL_EXECUTION_TRACKER.md`
   - `docs/implementation/SERVICE_V3_RFC_DRAFT.md`
2. Explicit envelope policy:
   - `docs/implementation/SERVICE_V3_CUTOVER_GATES.md`
3. Reproducible verification evidence:
   - `docs/implementation/SERVICE_V3_REHEARSAL_VERIFICATION_EVIDENCE.md`
4. Candidate handoff payloads generated from canonical manifest:
   - `docs/implementation/PRODUCER_RELEASE_HANDOFF_LATEST.json` (candidate mode)
   - `docs/implementation/SERVICE_V3_REHEARSAL_HANDOFF_CANDIDATE.json`

### Entry criteria
1. Current baseline is green in full producer verification.
2. Candidate mode generation scripts exist and are executable.

### Exit criteria
1. Candidate payload fields and policy semantics are validated by producer checks.
2. AssistSupport can run candidate handoff check without runtime cutover.

## Phase 5: Producer Cutover-Prep Controls (No Runtime Cutover)

### Objective
Convert producer handoff correctness into mandatory automated checks across CI and release.

### Producer deliverables
1. Handoff payload governance checker:
   - `scripts/verify_producer_handoff_payload.sh`
2. CI/release wiring:
   - `.github/workflows/ci.yml`
   - `.github/workflows/release.yml`
3. Governance test enforcement in integration suite:
   - `crates/memory-kernel-cli/tests/cli_integration.rs`

### Entry criteria
1. Phase 4 artifacts present.
2. Producer manifest alignment checks already green.

### Exit criteria
1. CI and release both fail on handoff payload contract/policy drift.
2. Published handoff payload is validated against canonical manifest and documented envelope policy.

## Phase 6: Cutover Governance + Rollback Evidence Scaffold

### Objective
Prepare cutover-day governance and incident reversal controls before any runtime cutover.

### Producer deliverables
1. Cutover-day execution checklist:
   - `docs/implementation/SERVICE_V3_CUTOVER_DAY_CHECKLIST.md`
2. Rollback communication protocol:
   - `docs/implementation/SERVICE_V3_ROLLBACK_COMMUNICATION_PROTOCOL.md`
3. Joint decision points and evidence references:
   - go/no-go ownership split
   - rollback triggers and mandatory evidence bundle

### Entry criteria
1. Phase 5 controls are active and green.
2. Candidate rehearsal validation remains green.

### Exit criteria
1. Cutover and rollback paths are actionable with no ambiguity.
2. Runtime cutover remains blocked until explicit joint approval gate.

## Joint Decision Points
1. Rehearsal continuation gate:
   - Producer candidate payload checks green
   - Consumer candidate handoff check green
2. Cutover readiness gate:
   - producer full suite green
   - consumer full suite green
   - joint sign-off recorded
3. Rollback trigger gate:
   - any non-2xx envelope shape drift
   - consumer deterministic fallback regression
   - unresolved critical incident in rehearsal/cutover window

## Risk Register
| Risk | Mitigation | Early signal |
|---|---|---|
| Handoff payload drift from manifest | verify payload in CI/release against canonical manifest | payload check failure |
| Policy drift between docs and machine artifacts | enforce non-2xx policy checks in governance script | policy assertion failure |
| Inconsistent candidate vs runtime baseline semantics | require explicit `active_runtime_baseline` in candidate payload | consumer handoff check fails |
| Premature runtime cutover | maintain explicit no-cutover guardrails in docs and checklists | release proposal missing joint approval evidence |
| Rollback ambiguity under incident pressure | cutover-day checklist + rollback protocol docs | delayed incident comms or missing rollback evidence |

## Verification Commands
```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
./scripts/verify_service_contract_alignment.sh --memorykernel-root /Users/d/Projects/MemoryKernel
./scripts/verify_contract_parity.sh --canonical-root /Users/d/Projects/MemoryKernel
./scripts/verify_trilogy_compatibility_artifacts.sh --memorykernel-root /Users/d/Projects/MemoryKernel
./scripts/verify_producer_contract_manifest.sh --memorykernel-root /Users/d/Projects/MemoryKernel
./scripts/verify_producer_handoff_payload.sh --memorykernel-root /Users/d/Projects/MemoryKernel
./scripts/run_trilogy_smoke.sh --memorykernel-root /Users/d/Projects/MemoryKernel
./scripts/run_trilogy_compliance_suite.sh --memorykernel-root /Users/d/Projects/MemoryKernel --skip-baseline
```

## Runtime Cutover Guardrail
- Rehearsal continuation can be `GO`.
- Runtime cutover remains `NO-GO` unless all joint cutover gates are explicitly met and recorded.
