# Revamp Execution Checklist

Status: Active  
Date: 2026-02-08

## Per-Phase Required Actions
1. Confirm phase entry criteria.
2. Confirm in-scope/out-of-scope list.
3. Create/update required phase artifacts.
4. Execute required verification commands.
5. Record evidence under `docs/revamp/evidence/`.
6. Update `docs/revamp/PHASE_STATUS.md`.
7. Commit with phase closure note.

## Canonical Verification Commands
```bash
pnpm run typecheck
pnpm run test
pnpm run check:memorykernel-pin
pnpm run check:memorykernel-governance
pnpm run check:memorykernel-handoff
pnpm run check:memorykernel-handoff:service-v3-candidate
pnpm run check:llm-golden-set
pnpm run test:memorykernel-contract
pnpm run test:memorykernel-phase3-dry-run
pnpm run test:ci
```

Note: `pnpm run check:llm-golden-set` now regenerates `docs/revamp/evidence/LLM_GOLDEN_SET_LATEST.json` before validation; commit refreshed evidence with each gate transition.

## Gate Blocking Rules
1. Any failing command above blocks phase closure.
2. Missing mandatory artifact blocks phase closure.
3. Unresolved Critical/High risk blocks phase closure unless explicit exception logged.
