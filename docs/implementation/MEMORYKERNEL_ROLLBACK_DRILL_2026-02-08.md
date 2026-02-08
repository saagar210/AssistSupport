# MemoryKernel Rollback Drill Record

Date: 2026-02-08
Owner: AssistSupport
Scenario: Validate rollback readiness while preserving Draft flow and contract guarantees.

## Drill Objective
Prove that AssistSupport can safely hold/revert integration baseline without losing core Draft functionality.

## Baseline Under Test
- release_tag: `v0.3.1`
- commit_sha: `cf331449e1589581a5dcbb3adecd3e9ae4509277`
- service/api/baseline: `service.v2` / `api.v1` / `integration/v1`

## Drill Steps
1. Validate governance alignment (`pin + matrix + manifest + hash`).
2. Validate contract behavior and deterministic fallback paths.
3. Confirm evidence artifact generation for incident auditing.
4. Confirm rollback procedure references are complete in operations docs.

## Command Evidence
```bash
pnpm run check:memorykernel-pin
pnpm run test:memorykernel-contract
```

Result summary:
- Governance validation: PASS
- Contract/fallback suite: PASS
- Evidence artifact generated: `/Users/d/Projects/AssistSupport/artifacts/memorykernel-contract-evidence.json`

## Rollback Decision
- Status: READY
- If emergency rollback is required, revert to last approved `pin + matrix + mirrored manifest` trio and rerun validation commands above before promoting.

## Follow-up Actions
1. Keep this drill cadence at least once per quarter.
2. Add service.v3 pre-cutover rollback rehearsal before runtime transition.

