# Service.v3 Rehearsal Execution Tracker (AssistSupport)

Updated: 2026-02-08
Owner: AssistSupport

## Working Branch Plan
- Primary branch: `codex/service-v3-rehearsal-consumer`
- Backup branch: `codex/service-v3-rehearsal-hotfix`

## Phase Execution Tasks

### Task C0: Cutover gates package
- Status: Complete
- Owner: AssistSupport
- Evidence:
  - `/Users/d/Projects/AssistSupport/docs/implementation/SERVICE_V3_CUTOVER_GATES.md`
- Definition of done:
  - explicit producer artifact requirements, consumer evidence, commands, rollback conditions, and joint sign-off checklist are published

### Task C1: Baseline rehearsal branch setup
- Status: Pending
- Owner: AssistSupport
- Definition of done:
  - branch created from current `master`
  - rehearsal checklist attached in PR description

### Task C2: Consumer contract assertions for service.v3 candidate
- Status: Pending
- Owner: AssistSupport
- Definition of done:
  - non-2xx assertions match service.v3 policy
  - fallback-path tests remain green

### Task C3: Governance gate validation
- Status: Pending
- Owner: AssistSupport
- Definition of done:
  - `pnpm run check:memorykernel-pin` green
  - `pnpm run check:memorykernel-governance` green
  - `pnpm run check:memorykernel-handoff:service-v3-candidate` green
  - hash-integrity validation green

### Task C4: Cutover rehearsal + rollback drill
- Status: Pending
- Owner: AssistSupport
- Definition of done:
  - cutover simulation checklist complete
  - rollback to prior baseline validated

## Required Commands
```bash
pnpm run typecheck
pnpm run test
pnpm run check:memorykernel-pin
pnpm run check:memorykernel-governance
pnpm run check:memorykernel-handoff:service-v3-candidate
pnpm run test:memorykernel-contract
pnpm run test:ci
```

## Exit Criteria
1. All tasks marked complete with evidence links.
2. Rehearsal branch green in CI.
3. Joint go/no-go call documented.
