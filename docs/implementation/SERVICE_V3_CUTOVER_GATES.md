# Service.v3 Cutover Gates (Consumer)

Updated: 2026-02-08
Owner: AssistSupport (joint sign-off with MemoryKernel)

## Scope
Planning/rehearsal gates for service.v3 adoption. This document does **not** authorize runtime cutover by itself.

## Baseline Context
- Current production consumer baseline:
  - `release_tag`: `v0.3.2`
  - `commit_sha`: `cf331449e1589581a5dcbb3adecd3e9ae4509277`
  - `service_contract_version`: `service.v2`
  - `api_contract_version`: `api.v1`
  - `integration baseline`: `integration/v1`

## Required Producer Artifacts
1. Immutable service.v3 candidate handoff payload JSON at:
   - `/Users/d/Projects/MemoryKernel/docs/implementation/PRODUCER_RELEASE_HANDOFF_LATEST.json`
2. Updated service.v3 RFC and rehearsal docs:
   - `/Users/d/Projects/MemoryKernel/docs/implementation/SERVICE_V3_RFC_DRAFT.md`
   - `/Users/d/Projects/MemoryKernel/docs/implementation/SERVICE_V3_REHEARSAL_PLAN.md`
3. Producer verification evidence for candidate:
   - fmt/clippy/tests
   - alignment/parity/artifact/smoke/compliance checks
4. Explicit producer envelope policy statement for service.v3 non-2xx responses.

## Required Consumer Evidence
1. Handoff payload validation evidence:
   - `/Users/d/Projects/AssistSupport/artifacts/memorykernel-handoff-evidence.json`
2. Contract evidence artifact:
   - `/Users/d/Projects/AssistSupport/artifacts/memorykernel-contract-evidence.json`
3. Updated phase status evidence:
   - `/Users/d/Projects/AssistSupport/docs/implementation/PHASE_EXECUTION_STATUS_2026-02-08.md`
4. Rehearsal tracker updates:
   - `/Users/d/Projects/AssistSupport/docs/implementation/SERVICE_V3_REHEARSAL_EXECUTION_TRACKER.md`

## Exact Commands (Consumer)

### Current pinned baseline validation
```bash
pnpm run check:memorykernel-pin
pnpm run check:memorykernel-governance
pnpm run check:memorykernel-handoff
pnpm run test:memorykernel-contract
pnpm run test:ci
```

### service.v3 candidate rehearsal validation (no pin update, no runtime cutover)
```bash
pnpm run check:memorykernel-handoff:service-v3-candidate
pnpm run typecheck
pnpm run test
pnpm run test:memorykernel-contract
pnpm run test:ci
```

### Phase 3 dry-run regression path
```bash
pnpm run test:memorykernel-phase3-dry-run
```

## Fail-Fast Rollback Conditions
Rollback immediately to last approved baseline (`v0.3.2` / `cf331449...`) if any condition is true:
1. Any contract/gov check fails (`check:memorykernel-pin`, `check:memorykernel-governance`, `check:memorykernel-handoff`).
2. Deterministic fallback behavior regresses for offline/timeout/malformed/version-mismatch/non-2xx.
3. Draft flow is blocked or degraded due to MemoryKernel state transitions.
4. Producer handoff payload and consumer expected baseline cannot be reconciled by explicit policy.

## Sign-Off Checklist (AssistSupport + MemoryKernel)

### AssistSupport sign-off
- [ ] Consumer commands above are green.
- [ ] Fallback invariants confirmed unchanged.
- [ ] Evidence artifacts generated and linked.
- [ ] Cutover risk reviewed and accepted.

### MemoryKernel sign-off
- [ ] Producer service.v3 candidate artifacts published.
- [ ] Producer verification evidence attached.
- [ ] Envelope/error policy confirmed for service.v3.
- [ ] Rollback coordination path confirmed.

### Joint gate decision
- [ ] Joint GO/NO-GO documented for rehearsal entry.
- [ ] Joint GO/NO-GO documented for runtime cutover (separate decision, future step).

## Decision Rule
- Rehearsal entry: requires both sides green on rehearsal gates.
- Runtime cutover: requires explicit separate joint approval after rehearsal completion.
