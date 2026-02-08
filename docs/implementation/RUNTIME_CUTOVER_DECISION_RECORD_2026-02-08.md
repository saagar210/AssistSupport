# Runtime Cutover Decision Record (Joint)

Updated: 2026-02-08  
Owner: AssistSupport + MemoryKernel (bilateral)

## Decision Scope
This record is the explicit bilateral decision artifact for entering Phase 8 runtime cutover execution mode.

## Decision Outcome
1. Rehearsal continuation: **GO**
2. Runtime cutover execution: **NO-GO**

## Approved Runtime Cutover Window
- Status: **Not approved**
- Reason: immutable `service.v3` runtime release target is not yet published and approved.

## Ownership (Named Roles)
- AssistSupport incident commander role: Support Platform On-Call Lead
- MemoryKernel incident commander role: MemoryKernel Producer On-Call Lead
- AssistSupport rollback owner role: AssistSupport Runtime Integrations Owner
- MemoryKernel rollback owner role: MemoryKernel Release Owner
- Joint decision log owner role: Integration Program Owner

## Decision Inputs
- Consumer checkpoint packet:
  - `/Users/d/Projects/AssistSupport/docs/implementation/SERVICE_V3_CUTOVER_DECISION_CHECKPOINT_2026-02-08.md`
- Producer checkpoint packet:
  - `/Users/d/Projects/MemoryKernel/docs/implementation/SERVICE_V3_CUTOVER_DECISION_CHECKPOINT_PRODUCER_2026-02-08.md`
- Producer addendum:
  - `/Users/d/Projects/MemoryKernel/docs/implementation/JOINT_DECISION_STATUS_ADDENDUM_2026-02-08.md`
- Consumer baseline commit:
  - `008dfc8`
- Producer baseline commit:
  - `cf451a7`

## Required Stop Conditions (Immediate Abort / NO-GO)
1. Any contract mismatch on non-2xx envelope semantics.
2. Any deterministic fallback regression in AssistSupport.
3. Any unresolved rollback-owner ambiguity at decision time.
4. Any missing immutable runtime target release evidence.

## Remaining Blockers Before Phase 8 Start
1. Immutable `service.v3` runtime release tag + commit SHA are not yet published and approved.
2. Runtime-target evidence bundle is not yet captured against a real runtime target:
   - producer release handoff packet for runtime mode
   - consumer atomic repin evidence (pin + matrix + mirrored manifest)
3. Bilateral rollback execution evidence for the runtime switch window is not yet logged as complete.

## Gate Re-entry Criteria for Runtime Decision
1. MemoryKernel publishes immutable runtime target (`release_tag` + `commit_sha`) for `service.v3`.
2. AssistSupport executes atomic repin PR and records green verification suite.
3. Both repos log rollback execution evidence for the same runtime target.
4. Bilateral GO record is signed in both decision-checkpoint packets.
