# Runtime Cutover Decision Record (Joint)

Updated: 2026-02-08  
Owner: AssistSupport + MemoryKernel (bilateral)

## Decision Scope
This record is the bilateral decision artifact for Phase 8 runtime cutover execution.

## Decision Outcome
1. Rehearsal continuation: **GO**
2. Runtime cutover execution: **GO**

## Approved Runtime Cutover Window
- Approved runtime target:
  - release_tag: `v0.4.0`
  - commit_sha: `7e4806a34b98e6c06ee33fa9f11499a975e7b922`
  - service_contract_version: `service.v3`
  - api_contract_version: `api.v1`
  - integration_baseline: `integration/v1`

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
- Producer handoff payload:
  - `/Users/d/Projects/MemoryKernel/docs/implementation/PRODUCER_RELEASE_HANDOFF_LATEST.json`

## Stop Conditions (Immediate Abort / Rollback)
1. Any contract mismatch on non-2xx envelope semantics.
2. Any deterministic fallback regression in AssistSupport.
3. Any unresolved rollback-owner ambiguity at cutover time.
4. Any failed consumer contract or CI suite command during cutover validation.

## Runtime Cutover Completion Evidence
1. MemoryKernel published immutable runtime target (`v0.4.0`, tag pushed).
2. AssistSupport repinned atomically (pin + matrix + mirrored producer manifest).
3. Consumer verification suite passed against runtime baseline.
4. Rollback drill evidence refreshed with runtime baseline references.

## Status
- Phase 8 runtime cutover: **COMPLETE**
- Operational posture: **GO**
