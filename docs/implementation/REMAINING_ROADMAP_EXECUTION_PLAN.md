# Remaining Roadmap Execution Plan (Consumer)

Updated: 2026-02-08  
Owner: AssistSupport  
Scope: phases 4/5/6 for AssistSupport consumer track

## Baseline
- MemoryKernel release tag: `v0.3.2`
- MemoryKernel commit SHA: `cf331449e1589581a5dcbb3adecd3e9ae4509277`
- Service/API/baseline: `service.v3` / `api.v1` / `integration/v1`
- Candidate handoff mode available: `service-v3-candidate`
- Runtime cutover state: `GO` (executed)

## Phase 4: Rehearsal Closure

### Objective
Close consumer rehearsal-entry evidence using producer candidate handoff artifacts, while keeping runtime behavior unchanged.

### Owner
AssistSupport (joint sign-off with MemoryKernel)

### Dependencies
1. Producer candidate handoff payload published.
2. Producer verification evidence published.
3. Consumer handoff validator supports candidate-mode schema checks.

### Entry Criteria
1. `pnpm run check:memorykernel-handoff:service-v3-candidate` passes.
2. `pnpm run check:memorykernel-pin` passes with runtime baseline still `service.v2`.
3. `pnpm run test:memorykernel-contract` and `pnpm run test:ci` pass.

### Exit Criteria
1. Phase 4 closeout artifact is published with explicit GO/NO-GO verdicts.
2. Rehearsal tracker updated with evidence links.
3. Joint checkpoint notes reflect rehearsal continuation GO and runtime cutover NO-GO.

### Deliverables
1. `PHASE4_CONSUMER_REHEARSAL_CLOSEOUT_2026-02-08.md`
2. Updated `SERVICE_V3_REHEARSAL_EXECUTION_TRACKER.md`
3. Updated `PHASE_EXECUTION_STATUS_2026-02-08.md`

### Verification Commands
```bash
pnpm run check:memorykernel-handoff:service-v3-candidate
pnpm run check:memorykernel-pin
pnpm run test:memorykernel-contract
pnpm run test:ci
```

## Phase 5: Consumer Cutover-Prep Controls

### Objective
Strengthen consumer-side policy and CI controls that can be enforced unilaterally before any runtime cutover.

### Owner
AssistSupport

### Dependencies
1. Phase 4 closure evidence complete.
2. Existing pin/matrix/manifest governance checks green.

### Entry Criteria
1. Consumer boundary and policy checks are available in `package.json`.
2. Governance bundle check validates phase artifacts and policy docs.

### Exit Criteria
1. Adapter boundary check is enforced (`check:memorykernel-boundary`).
2. Cutover policy check is enforced (`check:memorykernel-cutover-policy`).
3. MemoryKernel contract suite includes both checks and remains green.

### Deliverables
1. `scripts/validate_memorykernel_boundary.mjs`
2. `scripts/validate_memorykernel_cutover_policy.mjs`
3. Updated `package.json` command wiring
4. Updated `validate_memorykernel_governance_bundle.mjs`
5. `PHASE5_CONSUMER_CUTOVER_PREP_2026-02-08.md`

### Verification Commands
```bash
pnpm run check:memorykernel-boundary
pnpm run check:memorykernel-cutover-policy
pnpm run test:memorykernel-contract
```

## Phase 6: Cutover Governance + Rollback Readiness

### Objective
Publish cutover-day governance scaffolding and rollback/incident procedures while runtime cutover completed.

### Owner
AssistSupport (joint decision point with MemoryKernel)

### Dependencies
1. Phase 5 controls complete and green.
2. Joint cutover gates document published.

### Entry Criteria
1. Runtime baseline remains pinned to `service.v2`.
2. Candidate handoff validation is passing consistently.

### Exit Criteria
1. Explicit cutover command checklist published and marked policy-disabled.
2. Rollback success checklist published with measurable criteria.
3. Incident communication/escalation template published.

### Deliverables
1. `PHASE6_CONSUMER_CUTOVER_GOVERNANCE_2026-02-08.md`
2. `SERVICE_V3_CUTOVER_DAY_DRY_RUN_SESSION_PLAN.md`
3. Operations runbook references to phase-6 procedures
4. Governance checks confirm runtime cutover completed

### Verification Commands
```bash
pnpm run check:memorykernel-cutover-policy
pnpm run check:memorykernel-governance
pnpm run test:ci
```

## Risk Register (Remaining)

| Risk | Severity | Mitigation | Owner |
|---|---|---|---|
| Producer candidate payload schema drift | High | Candidate-mode handoff validation with required field checks | AssistSupport |
| Adapter boundary erosion | High | Automated boundary scanner in CI | AssistSupport |
| Premature runtime cutover | Critical | Cutover-policy guard requiring explicit joint approval | Joint |
| Fallback regression during rehearsal | Critical | Keep contract/fallback tests in required suites | AssistSupport |

## Joint Decision Gates
1. Gate C (rehearsal continuation): requires Phase 4 exit criteria complete.
2. Gate D (runtime cutover): blocked until:
   - joint sign-off recorded,
   - cutover checklist complete,
   - rollback drill evidence updated.

Current gate state:
- Rehearsal continuation: `GO`
- Runtime cutover: `NO-GO`

## Command Set (Canonical for this roadmap block)
```bash
pnpm run typecheck
pnpm run test
pnpm run check:memorykernel-pin
pnpm run check:memorykernel-governance
pnpm run check:memorykernel-handoff
pnpm run check:memorykernel-handoff:service-v3-candidate
pnpm run check:memorykernel-boundary
pnpm run check:memorykernel-cutover-policy
pnpm run test:memorykernel-cutover-dry-run
pnpm run test:memorykernel-contract
pnpm run test:memorykernel-phase3-dry-run
pnpm run test:ci
```

## Policy Statement
For this roadmap block, runtime cutover completed. Any runtime contract pin change to `service.v3` is out of scope until explicit joint approval and cutover gate completion.
