# Delta Improvement Plan

## A) Executive Summary

### Current state (repo-grounded)
- Frontend is React + TypeScript with Vite entrypoint in `src/main.tsx` and shell composition in `src/App.tsx`.
- Queue operations logic lives in `src/features/inbox/queueModel.ts` with localStorage-backed metadata and handoff snapshots.
- Queue model already has focused unit coverage in `src/features/inbox/queueModel.test.ts`.
- CI frontend gates run TypeScript checks and Vitest (`.github/workflows/ci.yml`).
- Baseline local verification is currently green (`pnpm run lint`, `pnpm run test`).

### Key risks
- Persisted metadata is parsed but not structurally validated prior to use.
- Invalid `priority` values can create undefined math paths in priority summarization.
- Invalid `state/owner/updatedAt` fields can produce inconsistent queue filters and SLA calculations.

### Improvement themes (prioritized)
1. Harden persisted queue metadata parsing/normalization.
2. Expand queue model test coverage for malformed storage payloads.
3. Preserve current UX semantics for valid metadata while improving fault tolerance.

## B) Constraints & Invariants (Repo-derived)

### Explicit invariants
- Keep localStorage keys unchanged (`assistsupport.queue.meta.v1`, handoff snapshot key).
- Preserve `QueueMeta` domain unions (`state`, `priority`) as existing public internal contract.
- Keep current queue sort/filter behavior for valid inputs.

### Implicit inferences from tests/contracts
- Queue items should still render and default safely when metadata is missing/broken.
- Snapshot loading should fail closed (`null`) for malformed payloads.

### Non-goals
- No UI redesign or queue workflow feature changes.
- No Rust/Tauri backend changes.
- No migration of existing localStorage key versions.

## C) Proposed Changes by Theme (Prioritized)

### Theme 1: Metadata normalization
- **Current approach:** `safeParseQueueMeta` returns parsed object directly if JSON parses.
- **Proposed change:** Add narrow runtime normalization for each metadata entry (`state`, `priority`, `owner`, `updatedAt`) with defaults and trimming.
- **Why:** Corrupted/legacy values should not poison queue metrics.
- **Tradeoffs:** Slight extra code path on load; improves determinism.
- **Scope boundary:** only queue metadata parsing and downstream summaries.
- **Migration approach:** read old payloads with best-effort normalization, no schema/key change.

### Theme 2: Regression tests
- **Current approach:** tests cover broken JSON but not structurally invalid JSON.
- **Proposed change:** add tests for malformed-but-parseable metadata objects and ensure safe defaults.
- **Why:** locks behavior and prevents regressions.
- **Scope boundary:** `queueModel.test.ts` only.

## D) File/Module Delta (Exact)
- **ADD:** none.
- **MODIFY:**
  - `src/features/inbox/queueModel.ts` — metadata normalization helpers + safe parse update.
  - `src/features/inbox/queueModel.test.ts` — malformed metadata tests.
  - `codex/*.md` — session artifacts.
- **REMOVE/DEPRECATE:** none.

## E) Data Models & API Contracts (Delta)
- **Current:** `QueueMetaMap` in `queueModel.ts` loaded from localStorage.
- **Proposed:** no type/interface changes; runtime acceptance tightened.
- **Compatibility:** backward-compatible read path with defaults.
- **Migrations:** none required.
- **Versioning:** storage key version unchanged.

## F) Implementation Sequence (Dependency-Explicit)
1. Add metadata normalization helpers.
   - Files: `queueModel.ts`
   - Preconditions: baseline green
   - Verify: `pnpm run test -- src/features/inbox/queueModel.test.ts`
   - Rollback: revert helper block if sorting/summary behavior regresses.
2. Add malformed metadata regression tests.
   - Files: `queueModel.test.ts`
   - Dependencies: step 1 complete
   - Verify: same targeted test command.
   - Rollback: revert test additions if unsupported assumptions.
3. Full frontend re-verification.
   - Verify: `pnpm run lint`, `pnpm run test`
   - Rollback: revert both modified files to baseline commit.

## G) Error Handling & Edge Cases
- Current pattern: fail-closed JSON parsing via try/catch; silent storage failures.
- Improvement: parse + structural validation, fallback defaults for malformed entries.
- Edge cases to test:
  - invalid priority/state strings
  - blank owner
  - invalid timestamp
  - non-object metadata entries

## H) Integration & Testing Strategy
- Integration points: localStorage load path consumed by queue screens.
- Unit tests: extend `queueModel.test.ts` for malformed metadata normalization.
- Regression suite: queueModel targeted test + full Vitest run.
- DoD: all baseline commands green, no contract/key changes, docs updated.

## I) Assumptions & Judgment Calls
- Assumption: localStorage payloads can become malformed through manual edits/legacy states.
- Judgment call: perform strict-enough runtime checks without introducing external schema libs.
