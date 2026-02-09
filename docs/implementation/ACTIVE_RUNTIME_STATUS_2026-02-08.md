# Active Runtime Status (Authoritative)

Updated: 2026-02-09
Owner: AssistSupport

This document is the authoritative consumer-side status for current MemoryKernel runtime integration.

## Active Baseline
- MemoryKernel release tag: `v0.3.2`
- MemoryKernel commit SHA: `cf331449e1589581a5dcbb3adecd3e9ae4509277`
- Service/API/baseline: `service.v2` / `api.v1` / `integration/v1`
- Consumer pin file: `/Users/d/Projects/AssistSupport/config/memorykernel-integration-pin.json`
- Producer manifest mirror: `/Users/d/Projects/AssistSupport/config/memorykernel-producer-manifest.json`

## Runtime Posture
- Rehearsal posture: **GO**
- Runtime cutover posture: **NO-GO**
- Operational posture: **STEADY-STATE (service.v2)**

## Source-of-Truth and Boundary Rule
- Canonical producer runtime/service contract source of truth: `/Users/d/Projects/MemoryKernel` on `main`.
- AssistSupport must not treat embedded MemoryKernel code as an independent contract authority.
- Any producer behavior change requires a producer handoff payload + consumer governance checks before adoption.

## Drift-Control Rule
- Required atomic governance update set when adopting a new producer baseline:
  1. `/Users/d/Projects/AssistSupport/config/memorykernel-integration-pin.json`
  2. `/Users/d/Projects/AssistSupport/config/memorykernel-producer-manifest.json`
  3. `/Users/d/Projects/AssistSupport/docs/MEMORYKERNEL_COMPATIBILITY_MATRIX.md`
- CI must fail if these artifacts drift.

## Supersession Note
- Historical rehearsal NO-GO artifacts are retained for audit history.
- Current operational decisions are governed by:
  - `/Users/d/Projects/AssistSupport/docs/implementation/RUNTIME_CUTOVER_DECISION_RECORD_2026-02-08.md`
  - `/Users/d/Projects/AssistSupport/docs/implementation/JOINT_RUNTIME_CUTOVER_CLOSURE_2026-02-08.md`
  - `/Users/d/Projects/AssistSupport/docs/implementation/ACTIVE_RUNTIME_STATUS_2026-02-08.md`
