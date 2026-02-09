# Service.v3 Cutover Gates

Updated: 2026-02-09

This document defines the explicit, testable gates for migrating AssistSupport's runtime baseline
from MemoryKernel `service.v2` to `service.v3`.

Runtime cutover: requires explicit separate joint approval after rehearsal completion.

## Active Runtime Baseline (Pinned)
- release_tag: `v0.3.2`
- commit_sha: `cf331449e1589581a5dcbb3adecd3e9ae4509277`
- service_contract_version: `service.v2`
- api_contract_version: `api.v1`
- integration_baseline: `integration/v1`

## Candidate Target (Rehearsal Only; Not Runtime)
- release_tag: `v0.4.0`
- commit_sha: `7e4806a34b98e6c06ee33fa9f11499a975e7b922`
- service_contract_version: `service.v3`
- api_contract_version: `api.v1`
- integration_baseline: `integration/v1`

## Required Producer Artifacts (Before Any Runtime Cutover)
1. Producer contract manifest aligned to candidate runtime target.
2. Candidate producer handoff payload (`handoff_mode=service-v3-candidate`) that:
   - retains `active_runtime_baseline` pointing to the pinned `service.v2` baseline
   - sets `rehearsal_candidate.requires_runtime_cutover=false`
3. Green producer verification suite evidence.
4. Explicit producer-side decision record stating runtime cutover posture (expected `NO-GO` until joint approval).

## Required Consumer Evidence (Before Any Runtime Cutover)
1. Candidate-mode validation (no repin):
   - `pnpm run check:memorykernel-handoff:service-v3-candidate`
   - `pnpm run test:memorykernel-contract`
2. Rollback drill evidence against current pinned runtime baseline (`v0.3.2` / `service.v2`).
3. Explicit consumer-side rehearsal closeout:
   - Rehearsal continuation verdict: `GO`/`NO-GO`
   - Runtime cutover verdict: `NO-GO` unless explicit joint approval exists

## Runtime Cutover Approval Gate (Bilateral)
Runtime cutover is allowed only when all conditions are true:
1. Producer publishes immutable runtime target tag+SHA for `service.v3`.
2. Consumer repin evidence is ready (pin + compatibility matrix + mirrored manifest update).
3. Both repos have full validation suites green against the runtime target.
4. Bilateral rollback execution evidence is recorded and validated.
5. Bilateral GO/NO-GO record explicitly marks runtime cutover as `GO`.

## Non-2xx Envelope Policy (Pinned Baseline + Candidate)
### Stable baseline (`service.v2`)
- Required fields: `service_contract_version`, `error.code`, `error.message`, `legacy_error`
- Forbidden fields: `api_contract_version`

### Candidate (`service.v3`)
- Required fields: `service_contract_version`, `error.code`, `error.message`
- Optional fields: `error.details`
- Forbidden fields: `legacy_error`, `api_contract_version`

## Fail-Fast Rollback Conditions (Always Enforced)
Rollback immediately to last approved baseline (`v0.3.2` / `cf331449...`) if any condition is true:
1. Contract mismatch or schema drift.
2. Deterministic fallback regression.
3. Preflight no longer passes against intended runtime target.
4. Any CI gate failure in cutover validation scope.

