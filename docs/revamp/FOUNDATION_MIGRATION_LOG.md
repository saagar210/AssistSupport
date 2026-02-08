# Foundation Migration Log (Phase 3)

Status: In Progress  
Date: 2026-02-08

## Scope
Track low-risk structural extraction tasks that prepare the queue-first UX rebuild without changing runtime behavior.

## Completed in Current Block
1. Introduced app-shell feature module under `src/features/app-shell/`.
2. Extracted tab model and shortcut mapping into dedicated contracts.
3. Extracted command palette command construction from `App.tsx` into `buildAppShellCommands`.
4. Extracted active tab rendering switch into `renderActiveTab`.
5. Kept existing UI behavior and navigation semantics unchanged.
6. Added unit tests for app-shell command and shortcut contracts.
7. Re-ran MemoryKernel contract and full CI suites with green results.
8. Introduced `workspace` and `inbox` feature wrappers and routed tab rendering through these domain modules.
9. Introduced revamp runtime flag parser with explicit precedence (storage > env > default false).
10. Added queue-first inbox wrapper behind `ASSISTSUPPORT_REVAMP_INBOX` (default-off).
11. Extracted app-shell orchestration into dedicated hooks (`useAppShellState`, `useDraftActions`, `useAppShellCommands`).
12. Added feature-domain compatibility wrappers for all remaining tabs (`sources`, `ingest`, `knowledge`, `analytics`, `pilot`, `search`, `ops`, `settings`).

## Rationale
1. Reduce `App.tsx` orchestration complexity.
2. Create a stable seam for replacing legacy tabs with new Inbox/Workspace surfaces.
3. Enforce clearer boundaries for future module migration.
4. Complete domain-wrapper seams before UX rebuild to avoid mixed ownership migration risk.

## Pending Phase 3 Work
1. Add focused tests for app-shell orchestration hooks and tab rendering contracts.
2. Define cutover criteria for default-on of first revamp flag.

## Verification Requirements
Run canonical revamp verification suite before Phase 3 closure.

## Evidence
1. `docs/revamp/evidence/PHASE3_FOUNDATION_SLICE_VERIFICATION_2026-02-08.md`
2. `docs/revamp/evidence/PHASE3_FOUNDATION_SLICE_2_SUMMARY_2026-02-08.md`
3. `docs/revamp/evidence/PHASE3_FOUNDATION_SLICE_3_SUMMARY_2026-02-08.md`
4. `docs/revamp/evidence/PHASE3_FOUNDATION_SLICE_4_SUMMARY_2026-02-08.md`
5. `docs/revamp/evidence/PHASE3_FOUNDATION_SLICE_5_SUMMARY_2026-02-08.md`
