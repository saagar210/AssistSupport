# Runtime Cutover Governance Packet (Monorepo Context)

Owner: Joint Runtime Governance  
Last updated: 2026-02-08

## Purpose
Consolidate the runtime cutover governance links and controls in the unified monorepo model.

## Canonical Runtime Baseline
1. release_tag: `v0.4.0`
2. commit_sha: `7e4806a34b98e6c06ee33fa9f11499a975e7b922`
3. service contract: `service.v3`
4. api contract: `api.v1`
5. integration baseline: `integration/v1`

## Required Consumer Controls
1. Enrichment remains optional and non-blocking.
2. Deterministic fallback remains preserved.
3. Adapter boundary remains enforced.
4. Pin/matrix/manifest updates are atomic.

## Required Governance Checks
1. `pnpm run check:memorykernel-pin`
2. `pnpm run check:memorykernel-governance`
3. `pnpm run check:memorykernel-handoff`
4. `pnpm run check:memorykernel-handoff:service-v3-candidate`
5. `pnpm run check:memorykernel-boundary`
6. `pnpm run check:memorykernel-cutover-policy`

## Bilateral Decision Artifacts
1. Consumer decision packet:
   - `/Users/d/Projects/AssistSupport/docs/implementation/RUNTIME_CUTOVER_DECISION_RECORD_2026-02-08.md`
2. Producer decision packet (embedded):
   - `/Users/d/Projects/AssistSupport/services/memorykernel/docs/implementation/RUNTIME_CUTOVER_DECISION_RECORD_PRODUCER_2026-02-08.md`
3. Joint closure packets:
   - `/Users/d/Projects/AssistSupport/docs/implementation/JOINT_RUNTIME_CUTOVER_CLOSURE_2026-02-08.md`
   - `/Users/d/Projects/AssistSupport/services/memorykernel/docs/implementation/JOINT_RUNTIME_CUTOVER_CLOSURE_PRODUCER_2026-02-08.md`

## Operational Guardrails
1. Any contract drift requires explicit pin+matrix+manifest update.
2. Any fallback regression blocks release.
3. Any non-green compliance suite blocks release.
4. Any cross-repo governance mismatch blocks release.
