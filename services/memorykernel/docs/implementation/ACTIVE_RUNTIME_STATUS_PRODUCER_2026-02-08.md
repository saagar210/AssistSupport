# Active Runtime Status (Producer Authoritative)

Updated: 2026-02-08
Owner: MemoryKernel

This document is the authoritative producer-side runtime status for AssistSupport integration.

## Active Baseline
- release_tag: `v0.4.0`
- commit_sha: `7e4806a34b98e6c06ee33fa9f11499a975e7b922`
- service/api/integration: `service.v3` / `api.v1` / `integration/v1`
- canonical producer manifest: `/Users/d/Projects/MemoryKernel/contracts/integration/v1/producer-contract-manifest.json`
- published handoff payload: `/Users/d/Projects/MemoryKernel/docs/implementation/PRODUCER_RELEASE_HANDOFF_LATEST.json`

## Runtime Posture
- Rehearsal continuation: **GO**
- Runtime cutover: **GO**
- Runtime posture: **STEADY-STATE GO**

## Source-of-Truth Rule
- MemoryKernel repository remains the canonical producer contract authority.
- Any baseline change must update in lockstep:
  1. `contracts/integration/v1/producer-contract-manifest.json`
  2. `docs/implementation/PRODUCER_RELEASE_HANDOFF_LATEST.json`
  3. normative docs (`docs/spec/*`) and OpenAPI (`openapi/openapi.yaml`)

## Consumer Coordination Rule
- AssistSupport must adopt baseline changes only via pinned release + commit + manifest synchronization.
- Producer-side additive behavior changes require prior notification and updated handoff payload evidence.

## Supersession Note
- Historical pre-cutover NO-GO artifacts are retained for audit history.
- Current operational authority is:
  - `/Users/d/Projects/MemoryKernel/docs/implementation/RUNTIME_CUTOVER_DECISION_RECORD_PRODUCER_2026-02-08.md`
  - `/Users/d/Projects/MemoryKernel/docs/implementation/JOINT_RUNTIME_CUTOVER_CLOSURE_PRODUCER_2026-02-08.md`
  - `/Users/d/Projects/MemoryKernel/docs/implementation/ACTIVE_RUNTIME_STATUS_PRODUCER_2026-02-08.md`
