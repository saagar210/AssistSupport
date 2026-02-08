# Phase 6 Ops Hardening Playbook

Status: Active  
Date: 2026-02-08

## Purpose
Provide a deterministic, repeatable ops-hardening gate that validates integration governance, rollback readiness, and critical security regression coverage before moving to Phase 7/8 closure work.

## Scope
1. MemoryKernel governance + handoff candidate validation.
2. Local LLM gate freshness and threshold validation.
3. Rollback-readiness validation against baseline tag and runbook artifacts.
4. Security regression suite execution for high-risk classes.

## Canonical Command
```bash
pnpm run check:phase6-ops-hardening
```

## Command Breakdown
1. `pnpm run check:memorykernel-governance`
2. `pnpm run check:memorykernel-handoff`
3. `pnpm run check:memorykernel-handoff:service-v3-candidate`
4. `pnpm run check:llm-golden-set`
5. `pnpm run check:rollback-readiness`
6. `pnpm run test:security-regression`

## Evidence Artifacts
1. `docs/revamp/evidence/PHASE6_OPS_HARDENING_LATEST.json`
2. `artifacts/rollback-readiness-evidence.json`
3. `artifacts/memorykernel-handoff-evidence.json`

## Fail-Fast Rules
1. Any failed sub-command is a hard gate failure.
2. If baseline rollback tag is missing or not ancestor of HEAD, stop immediately.
3. If security regression suite fails, Phase 6 cannot close.
4. Runtime cutover posture remains NO-GO regardless of Phase 6 pass until bilateral cutover record is approved.

## Recovery Actions
1. Fix failed command scope only; do not broaden change-set during recovery.
2. Re-run `pnpm run check:phase6-ops-hardening` after remediation.
3. Log remediation in phase evidence summary before closure request.
