# Release Gates (Revamp)

Status: Enforced  
Date: 2026-02-08

## Gate G0: Program Lock
### Entry
1. Program charter exists.
2. Non-negotiables approved.
3. Risk register created with owners.

### Exit
1. ADR index initialized.
2. Phase execution sequence agreed.

## Gate G1: Baseline and Safety
### Entry
1. Current master green.
2. Baseline commit identified.

### Exit
1. Baseline metrics captured.
2. Rollback runbook exists and dry-run documented.
3. Feature flag matrix complete.

## Gate G2: Architecture Freeze
### Entry
1. Baseline artifacts complete.

### Exit
1. Target architecture, contracts, state model, error taxonomy complete.
2. No unresolved boundary ownership conflicts.

## Gate G3: Foundation Refactor
### Exit Requirements
1. Feature-sliced structure in place.
2. Adapter boundaries enforced.
3. Canonical suite passes.

## Gate G4: UX Rebuild
### Exit Requirements
1. Inbox + Workspace critical workflows pass.
2. Keyboard-first acceptance tests pass.
3. Accessibility checks pass for new surfaces.

## Gate G5: LLM Runtime Governance
### Exit Requirements
1. Model profile registry active.
2. Prompt contracts validated.
3. Regression/eval thresholds green.
4. `pnpm run check:llm-golden-set` passes against generated latest evidence payload (case-driven refresh + freshness policy).

## Gate G6: Integration + Ops Hardening
### Exit Requirements
1. Candidate/stable handoff validations pass.
2. Failure-mode playbook tested.
3. Ops diagnostics and recovery flows documented.
4. `pnpm run check:phase6-ops-hardening` passes with fresh evidence.

## Gate G7: Security + Compliance Closure
### Exit Requirements
1. No unresolved high-severity security findings.
2. Control-to-evidence matrix complete.
3. Security signoff packet complete.
4. `pnpm run test:security-regression` passes on release candidate branch.

## Gate G8: Release Candidate + Handoff
### Exit Requirements
1. Full test/governance suite green.
2. Rollback drill successful and logged.
3. GO/NO-GO decision record complete.
4. Work-machine handoff runbook validated.

## Gate Closure Records (Current)
1. G4 closed with:
   - `docs/revamp/evidence/PHASE4_UX_CLOSURE_2026-02-08.md`
2. G5 closed with:
   - `docs/revamp/evidence/PHASE5_LLM_GOVERNANCE_CLOSURE_2026-02-08.md`
3. G6 closed with:
   - `docs/revamp/evidence/PHASE6_OPS_HARDENING_CLOSURE_2026-02-08.md`

## Mandatory Verification Suite (for gate transitions)
```bash
pnpm run typecheck
pnpm run test
pnpm run check:memorykernel-pin
pnpm run check:memorykernel-governance
pnpm run check:memorykernel-handoff
pnpm run check:memorykernel-handoff:service-v3-candidate
pnpm run check:llm-golden-set
pnpm run check:rollback-readiness
pnpm run check:phase6-ops-hardening
pnpm run test:security-regression
pnpm run test:memorykernel-contract
pnpm run test:memorykernel-phase3-dry-run
pnpm run test:ci
```
