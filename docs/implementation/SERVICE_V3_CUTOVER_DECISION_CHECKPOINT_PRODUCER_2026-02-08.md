# Service.v3 Cutover Decision Checkpoint (Producer)

Updated: 2026-02-08  
Scope: Planning + gate validation only (no runtime cutover execution)

## Baseline Lock (must remain unchanged during this checkpoint)
- `release_tag`: `v0.3.2`
- `commit_sha`: `cf331449e1589581a5dcbb3adecd3e9ae4509277`
- `service_contract_version`: `service.v2`
- `api_contract_version`: `api.v1`
- `integration_baseline`: `integration/v1`

## Producer Entry Criteria
All required:
1. Producer rehearsal sign-off packet is published:
   - `/Users/d/Projects/MemoryKernel/docs/implementation/JOINT_SIGNOFF_CHECKPOINT_PACKET_PRODUCER_2026-02-08.md`
2. Consumer rehearsal sign-off packet is confirmed.
3. Producer verification stack is green on `main`.
4. Candidate handoff payload validation is green.
5. No open SEV-1/SEV-2 contract regressions.

## Immutable Artifact Requirements Before Any Runtime Switch Consideration
1. Immutable release tag + commit for candidate/release target.
2. Canonical producer manifest aligned:
   - `/Users/d/Projects/MemoryKernel/contracts/integration/v1/producer-contract-manifest.json`
3. Producer handoff payload aligned and verified:
   - `/Users/d/Projects/MemoryKernel/docs/implementation/PRODUCER_RELEASE_HANDOFF_LATEST.json`
4. Service.v3 rehearsal/cutover governance docs present and current.
5. Changelog entry present with consumer impact statement.

## Required Schema/Spec/Manifest Guarantees
1. OpenAPI, service contract docs, and versioning policy are internally consistent.
2. Non-2xx envelope policy is explicit and machine-verifiable for:
   - current stable `service.v2`
   - candidate `service.v3`
3. `error_code_enum` remains canonical and synchronized with manifest.
4. Integration contract parity checks remain green against sibling components.
5. Producer handoff payload checker passes for both modes:
   - `stable`
   - `service-v3-candidate`

## Rollback Preconditions (must exist before cutover can be approved)
1. Valid rollback target is pinned (`service.v2` stable baseline).
2. Rollback communication protocol exists and is reviewed:
   - `/Users/d/Projects/MemoryKernel/docs/implementation/SERVICE_V3_ROLLBACK_COMMUNICATION_PROTOCOL.md`
3. Cutover-day checklist includes immediate rollback triggers and evidence bundle requirements:
   - `/Users/d/Projects/MemoryKernel/docs/implementation/SERVICE_V3_CUTOVER_DAY_CHECKLIST.md`
4. Producer + consumer command sets for rollback verification are documented.

## Abort Triggers (immediate NO-GO)
1. Any mismatch in non-2xx envelope contract assertions.
2. Candidate payload field drift from canonical manifest expectations.
3. Missing immutable release evidence (tag/sha/proof).
4. Consumer contract suite regressions related to pin/matrix/manifest sync.
5. Missing rollback path evidence.

## Explicit NO-GO Criteria
Any single criterion blocks runtime cutover:
1. Producer verification stack not fully green.
2. Consumer verification stack not fully green.
3. Joint decision record incomplete or unsigned.
4. Baseline lock values unexpectedly changed.
5. Critical incident unresolved at decision time.

## Joint Decision Template (Producer-Side Copy)
Use this template to align with consumer checkpoint packet:

1. Decision timestamp (UTC):
2. Producer decision:
   - `GO (rehearsal continuation)` or `NO-GO`
   - `GO (runtime cutover)` or `NO-GO`
3. Consumer decision:
   - `GO (rehearsal continuation)` or `NO-GO`
   - `GO (runtime cutover)` or `NO-GO`
4. Baseline confirmation:
   - tag/sha/service/api/integration values
5. Evidence links:
   - producer command outputs
   - consumer command outputs
   - payload + manifest + docs
6. Residual risks accepted:
7. Rollback target and trigger owner:

## Producer Checkpoint Verdict (current)
- Rehearsal posture: **GO**
- Runtime cutover: **NO-GO** (blocked pending explicit joint cutover gate completion)
