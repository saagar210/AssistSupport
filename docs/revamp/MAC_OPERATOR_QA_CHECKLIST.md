# Mac Operator QA Checklist (Revamp)

Status: Active  
Owner: AssistSupport revamp program  
Platform: macOS (MacBook Pro target environment)

## Purpose
This checklist is the required manual validation pass for queue-first and workspace-revamp operator workflows before rollout to a new machine.

## Preconditions
1. Build and tests are green:
   - `pnpm run typecheck`
   - `pnpm run test`
   - `pnpm run test:memorykernel-contract`
   - `pnpm run test:ci`
2. Revamp rehearsal checks are green:
   - `pnpm run test:revamp-queue-rehearsal`
3. Feature flags enabled for rehearsal:
   - `ASSISTSUPPORT_REVAMP_INBOX=1`
   - `ASSISTSUPPORT_REVAMP_WORKSPACE=1`
   - `ASSISTSUPPORT_REVAMP_COMMAND_PALETTE_V2=1`

## Manual Validation Scenarios

### A. Queue Keyboard Triage
1. Open `Follow-ups` tab in queue-first mode.
2. Verify keyboard shortcuts:
   - `J/K` and arrow keys move selected item.
   - `C` claims unassigned item.
   - `X` resolves active item.
   - `O` reopens resolved item.
   - `Enter` opens selected draft.
3. Expected result:
   - Selection state changes without lag.
   - Queue state/owner labels update immediately.
   - No application error banners.

### B. Shift Handoff Snapshot and Delta
1. In queue analytics panel, review priority mix and owner workload cards.
2. Click `Copy Handoff Snapshot`.
3. Make queue state changes (claim/resolve/reopen), click `Copy Handoff Snapshot` again.
4. Expected result:
   - Clipboard copy succeeds.
   - Trend delta section updates against previous snapshot baseline.
   - Owner delta rows reflect latest workload movement.

### C. Workspace Queue Quick Actions
1. Go to Draft workspace revamp view.
2. In `Live queue context`, use:
   - `Open At-Risk Queue`
   - `Open Unassigned Queue`
   - `Open In-Progress Queue`
3. Expected result:
   - App navigates to Follow-ups queue-first view.
   - Corresponding filter is pre-selected.
   - No stale/dead state after navigation.

### D. Core Non-Blocking Invariants
1. Simulate MemoryKernel unavailable and attempt response generation.
2. Expected result:
   - Enrichment remains optional/non-blocking.
   - Deterministic fallback path still produces usable output.
   - No hard failure in Draft flow.

### E. Accessibility and Usability
1. Run full keyboard-only pass for queue triage and draft open.
2. Confirm visible focus indicator on interactive queue list.
3. Verify text density/readability on MacBook Pro resolution.
4. Expected result:
   - No keyboard traps.
   - Focus state and selected-state are visually clear.
   - Operators can complete triage and open draft without mouse.

## Sign-off Record
Record each validation with:
1. Build/test evidence reference (command output file or CI run link).
2. Operator initials.
3. Result (`PASS` / `FAIL`).
4. Notes and follow-up issue IDs for any failures.

## Exit Criteria
Checklist is complete only when:
1. All scenarios A-E are `PASS`.
2. No unresolved `FAIL` entries remain.
3. Required command checks remain green on latest commit.
