# Service.v3 Cutover Decision Checkpoint (Consumer)

Updated: 2026-02-08  
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
- [ ] `pnpm run check:memorykernel-handoff:service-v3-candidate` passes.
- [ ] `pnpm run check:memorykernel-pin` passes.
- [ ] `pnpm run test:memorykernel-contract` passes.
- [ ] Consumer deterministic fallback behavior remains green for:
  - offline
  - timeout
  - malformed payload
  - version mismatch
  - non-2xx response

### Governance Gates
- [ ] Producer manifest, compatibility matrix, and consumer pin remain synchronized.
- [ ] Producer handoff payload correctness gate evidence is available.
- [ ] Bilateral sign-off artifacts are linked in status docs.
- [ ] Runtime cutover remains policy-blocked pending explicit joint approval.

### Rollback Gates
- [ ] Prior approved baseline rollback target is documented (`v0.3.2` / `cf331449...`).
- [ ] Rollback drill evidence remains valid:
  - `/Users/d/Projects/AssistSupport/docs/implementation/MEMORYKERNEL_ROLLBACK_DRILL_2026-02-08.md`
- [ ] Incident communication protocol references are current on both repos.

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

## Required Evidence Links
1. `/Users/d/Projects/AssistSupport/docs/implementation/JOINT_CHECKPOINT_STATUS_2026-02-08.md`
2. `/Users/d/Projects/AssistSupport/docs/implementation/PHASE_EXECUTION_STATUS_2026-02-08.md`
3. `/Users/d/Projects/AssistSupport/artifacts/memorykernel-handoff-evidence.json`
4. `/Users/d/Projects/AssistSupport/artifacts/memorykernel-contract-evidence.json`
5. `/Users/d/Projects/MemoryKernel/docs/implementation/JOINT_SIGNOFF_CHECKPOINT_PACKET_PRODUCER_2026-02-08.md`

## Current Policy Confirmation
1. Runtime cutover remains disabled.
2. Enrichment remains optional and non-blocking.
3. Deterministic fallback remains preserved.
4. Adapter boundary remains unchanged (no direct MemoryKernel calls outside boundary).
