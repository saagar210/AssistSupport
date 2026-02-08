# Service.v3 Cutover-Day Dry-Run Session Plan

Updated: 2026-02-08  
Owners: AssistSupport + MemoryKernel

## Session Goal
Execute a joint rehearsal-grade dry run that validates cutover governance, fallback safety, and rollback readiness while keeping runtime cutover disabled.

## Preconditions
1. Producer candidate handoff payload is published and validated.
2. Consumer baseline remains pinned to `service.v2` / `api.v1` / `integration/v1`.
3. Both repos are green on their latest required verification suites.

## Joint Agenda (60 minutes)
1. Baseline confirmation (10 min)
   - verify pinned release tag + commit SHA
   - confirm no runtime-cutover approval is being granted in this session
2. Consumer dry-run command execution (20 min)
   - run `pnpm run test:memorykernel-cutover-dry-run`
   - capture command output and artifacts
3. Producer artifact review (15 min)
   - verify handoff payload fields and policy sections
   - verify producer-side cutover-day and rollback protocol docs
4. Rollback readiness walkthrough (10 min)
   - step through rollback checklist and evidence requirements
5. Joint verdict recording (5 min)
   - record rehearsal continuation GO/NO-GO
   - record runtime cutover GO/NO-GO (expected NO-GO)

## Consumer Commands
```bash
pnpm run test:memorykernel-cutover-dry-run
pnpm run check:memorykernel-governance
pnpm run test:memorykernel-phase3-dry-run
```

## Evidence to Capture
1. `/Users/d/Projects/AssistSupport/artifacts/memorykernel-handoff-evidence.json`
2. `/Users/d/Projects/AssistSupport/artifacts/memorykernel-contract-evidence.json`
3. command output transcript attached to meeting notes
4. updated checkpoint verdict in:
   - `/Users/d/Projects/AssistSupport/docs/implementation/JOINT_CHECKPOINT_STATUS_2026-02-08.md`

## Guardrails
1. No change to `config/memorykernel-integration-pin.json`.
2. No runtime endpoint or contract switch to `service.v3`.
3. Any fallback regression ends session with immediate NO-GO verdict.

## Session Output Template
- Rehearsal continuation verdict: `GO` / `NO-GO`
- Runtime cutover verdict: `NO-GO` (unless explicit joint gate override, out of scope here)
- Risks found:
- Required follow-up owners and dates:
