# Phase 5 UX Test Gates (Pre-Revamp Contract)

Status: Active (Phase 5 pre-revamp entry artifact)  
Scope: Required verification for Phase 5 UX changes  
Goal: Prevent regressions while redesigning.

This file defines the minimum test gates required for Phase 5 UX work. If a Phase 5 change
cannot keep these green, the change must be split, fixed, or reverted.

## Mandatory Gates (Every Phase 5 Merge)

1. `pnpm run typecheck`
2. `pnpm run test`
3. `pnpm run check:monorepo-readiness:full`

Rationale:
- Phase 5 is UX-focused, but the monorepo readiness suite is the safety rail that protects:
  - MemoryKernel pinned governance and cutover policy checks
  - Consumer/producer artifact isolation
  - VaultMind subtree lint/test integrity

## When To Add Additional Gates

Add or extend gates when:
1. A new revamp screen is introduced (add/extend component tests).
2. A keyboard shortcut mapping changes (update shortcut tests).
3. A degraded state UX changes (add regression tests for that state).

## Evidence Format (Per Slice)

For each Phase 5 slice, include:
- `docs/revamp/evidence/phase5/ux-<slice-id>/verification.txt`

Content requirements:
1. Commands run (exact commands)
2. PASS confirmation
3. Any environment flags required to reproduce (revamp flags, dev mode)

