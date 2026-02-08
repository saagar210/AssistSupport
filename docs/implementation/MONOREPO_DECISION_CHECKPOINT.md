# Monorepo Decision Checkpoint

Owner: AssistSupport + MemoryKernel  
Last updated: 2026-02-08

## Decision Scope
Final gate for adopting the unified monorepo as the primary operating model.

## Decision Inputs
1. Import provenance and structure evidence:
   - `/Users/d/Projects/AssistSupport/docs/implementation/evidence/IMPORT_PROVENANCE_MAP.json`
   - `/Users/d/Projects/AssistSupport/docs/implementation/MONOREPO_IMPORT_MANIFEST.json`
   - `/Users/d/Projects/AssistSupport/docs/implementation/MONOREPO_STRUCTURE_SPEC.md`
2. Consumer integration baseline and closure:
   - `/Users/d/Projects/AssistSupport/docs/implementation/JOINT_PROGRAM_DELIVERY_CLOSEOUT_2026-02-08.md`
3. Producer closure references (embedded source of truth):
   - `/Users/d/Projects/AssistSupport/services/memorykernel/docs/implementation/JOINT_PROGRAM_DELIVERY_CLOSEOUT_PRODUCER_2026-02-08.md`
4. Gate evidence bundles:
   - `GATE_G5_ASSISTSUPPORT_MONOREPO_VALIDATION.log`
   - `GATE_G6_IMPORTED_MEMORYKERNEL_VALIDATION.log`

## Hard Gates
1. Consumer commands pass:
   - `pnpm run typecheck`
   - `pnpm run test`
   - `pnpm run test:memorykernel-contract`
   - `pnpm run test:ci`
2. Governance commands pass:
   - `pnpm run check:memorykernel-pin`
   - `pnpm run check:memorykernel-governance`
   - `pnpm run check:memorykernel-handoff`
   - `pnpm run check:memorykernel-handoff:service-v3-candidate`
   - `pnpm run check:memorykernel-boundary`
   - `pnpm run check:memorykernel-cutover-policy`
3. Embedded MemoryKernel full suite passes from `services/memorykernel`.

## Automatic NO-GO Criteria
1. Pin/matrix/manifest drift.
2. Boundary violation (direct MemoryKernel access outside adapter).
3. Deterministic fallback regression.
4. Missing import provenance linkage.

## Decision Record
1. Monorepo operating model: `GO`.
2. Runtime baseline: `service.v3/api.v1/integration/v1` remains unchanged.
3. Migration risk posture: `ACCEPTABLE` with gates enforced in CI.
4. Rollback posture: `AVAILABLE` via branch/worktree recovery plus stable master baseline.
