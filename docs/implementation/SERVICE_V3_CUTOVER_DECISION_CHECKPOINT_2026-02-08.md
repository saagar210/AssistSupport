# Service.v3 Cutover Decision Checkpoint (Consumer)

Updated: 2026-02-08 (checkpoint decision logged)  
Owner: AssistSupport (joint decision with MemoryKernel)

## Scope
Define the explicit decision gate for entering runtime cutover discussion.  
This checkpoint is planning/validation only and does **not** authorize runtime cutover.

## Baseline
- release_tag: `v0.3.2`
- commit_sha: `cf331449e1589581a5dcbb3adecd3e9ae4509277`
- service_contract_version: `service.v2`
- api_contract_version: `api.v1`
- integration baseline: `integration/v1`

## Entry Criteria (Joint Preconditions)
1. Bilateral rehearsal sign-off packets are complete:
   - `/Users/d/Projects/AssistSupport/docs/implementation/JOINT_SIGNOFF_CHECKPOINT_PACKET_2026-02-08.md`
   - `/Users/d/Projects/MemoryKernel/docs/implementation/JOINT_SIGNOFF_CHECKPOINT_PACKET_PRODUCER_2026-02-08.md`
2. Producer baseline remains immutable and unchanged during checkpoint review.
3. Consumer pin/matrix/manifest atomic governance remains green.
4. Both teams explicitly agree this checkpoint is decision-mode only (no runtime switch execution).

## Hard Gate Checklist

### Technical Gates
- [x] `pnpm run check:memorykernel-handoff:service-v3-candidate` passes.
- [x] `pnpm run check:memorykernel-pin` passes.
- [x] `pnpm run test:memorykernel-contract` passes.
- [x] Consumer deterministic fallback behavior remains green for:
  - offline
  - timeout
  - malformed payload
  - version mismatch
  - non-2xx response

### Governance Gates
- [x] Producer manifest, compatibility matrix, and consumer pin remain synchronized.
- [x] Producer handoff payload correctness gate evidence is available.
- [x] Bilateral sign-off artifacts are linked in status docs.
- [x] Runtime cutover remains policy-blocked pending explicit joint approval.

### Rollback Gates
- [x] Prior approved baseline rollback target is documented (`v0.3.2` / `cf331449...`).
- [x] Rollback drill evidence remains valid:
  - `/Users/d/Projects/AssistSupport/docs/implementation/MEMORYKERNEL_ROLLBACK_DRILL_2026-02-08.md`
- [x] Incident communication protocol references are current on both repos.

## Explicit NO-GO Criteria
Any one of the following is an automatic NO-GO:
1. Any required technical/gov command fails.
2. Any deterministic fallback regression is detected.
3. Any contract mismatch between handoff payload and consumer expectations remains unresolved.
4. Any request to perform runtime cutover without an explicit new joint approval record.
5. Any unresolved disagreement on rollback ownership or execution sequencing.

## Decision Log Template
- Checkpoint date:
- Participants (AssistSupport / MemoryKernel):
- Producer artifact commit/tag reviewed:
- Consumer artifact commit reviewed:
- Commands executed + result summary:
- Technical gate verdict:
- Governance gate verdict:
- Rollback gate verdict:
- Final rehearsal posture: `GO` / `NO-GO`
- Runtime cutover decision: `GO` / `NO-GO`
- If `NO-GO`, blockers + owners + target resolution date:

## Decision Log (Current Session)
- Checkpoint date: 2026-02-08
- Participants (AssistSupport / MemoryKernel): AssistSupport Codex + MemoryKernel Codex
- Producer artifact commit/tag reviewed: `fda52ed627648a912f9c17a26f8f65a023e40f42` (producer cutover-decision packet), baseline `v0.3.2` / `cf331449e1589581a5dcbb3adecd3e9ae4509277`
- Consumer artifact commit reviewed: `7b1d1fc` (pre-checkpoint run)
- Commands executed + result summary:
  - `pnpm run check:memorykernel-handoff:service-v3-candidate` PASS
  - `pnpm run check:memorykernel-pin` PASS
  - `pnpm run test:memorykernel-contract` PASS
  - `pnpm run test:ci` PASS
- Technical gate verdict: GO
- Governance gate verdict: GO
- Rollback gate verdict: GO
- Final rehearsal posture: GO
- Runtime cutover decision: NO-GO
- If `NO-GO`, blockers + owners + target resolution date:
  1. Explicit bilateral runtime-cutover approval record is not yet granted (Joint, next cutover decision session).
  2. Runtime baseline remains intentionally pinned to `service.v2` by governance policy (Joint, unchanged until new approval).
  3. No immutable service.v3 runtime promotion tag/sha is approved for cutover execution (MemoryKernel + AssistSupport, pending future decision checkpoint).

## Final Decision Record (Bilateral)
- Decision artifact:
  - `/Users/d/Projects/AssistSupport/docs/implementation/RUNTIME_CUTOVER_DECISION_RECORD_2026-02-08.md`
- Decision summary:
  - Rehearsal continuation: `GO`
  - Runtime cutover: `NO-GO`
- Checkpoint closure status:
  - Phase 7 decision checkpoint: `CLOSED`
  - Phase 8 runtime cutover: `NOT STARTED`

## Required Evidence Links
1. `/Users/d/Projects/AssistSupport/docs/implementation/JOINT_CHECKPOINT_STATUS_2026-02-08.md`
2. `/Users/d/Projects/AssistSupport/docs/implementation/PHASE_EXECUTION_STATUS_2026-02-08.md`
3. `/Users/d/Projects/AssistSupport/artifacts/memorykernel-handoff-evidence.json`
4. `/Users/d/Projects/AssistSupport/artifacts/memorykernel-contract-evidence.json`
5. `/Users/d/Projects/MemoryKernel/docs/implementation/JOINT_SIGNOFF_CHECKPOINT_PACKET_PRODUCER_2026-02-08.md`
6. `/Users/d/Projects/MemoryKernel/docs/implementation/SERVICE_V3_CUTOVER_DECISION_CHECKPOINT_PRODUCER_2026-02-08.md`

## Producer Alignment (Received)
- Producer cutover-decision packet:
  - `/Users/d/Projects/MemoryKernel/docs/implementation/SERVICE_V3_CUTOVER_DECISION_CHECKPOINT_PRODUCER_2026-02-08.md`
- Producer decision addendum:
  - `/Users/d/Projects/MemoryKernel/docs/implementation/JOINT_DECISION_STATUS_ADDENDUM_2026-02-08.md`
- Producer runtime decision mirror:
  - `/Users/d/Projects/MemoryKernel/docs/implementation/RUNTIME_CUTOVER_DECISION_RECORD_PRODUCER_2026-02-08.md`
- Producer alignment commit:
  - `cf451a7`
- Producer reported verdicts:
  - Rehearsal continuation: `GO`
  - Runtime cutover: `NO-GO`

## Current Policy Confirmation
1. Runtime cutover remains disabled.
2. Enrichment remains optional and non-blocking.
3. Deterministic fallback remains preserved.
4. Adapter boundary remains unchanged (no direct MemoryKernel calls outside boundary).
