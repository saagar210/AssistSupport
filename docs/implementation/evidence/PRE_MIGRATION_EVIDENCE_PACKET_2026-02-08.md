# Pre-Migration Evidence Packet

Updated: 2026-02-08
Owner: AssistSupport

## Scope

Pre-migration readiness evidence for monorepo consolidation and baseline governance hardening.

## Required Evidence

1. Governance alignment:
   - `pnpm run check:memorykernel-pin`
   - `pnpm run check:memorykernel-governance`
2. Handoff validation:
   - `pnpm run check:memorykernel-handoff`
   - `pnpm run check:memorykernel-handoff:service-v3-candidate`
3. Boundary/fallback safety:
   - `pnpm run check:memorykernel-boundary`
   - `pnpm run check:memorykernel-cutover-policy`
   - `pnpm run test:memorykernel-contract`
4. Drift negative fixtures:
   - `pnpm run test:memorykernel-governance-negative`
   - `services/memorykernel/scripts/test_producer_governance_negative.sh`
5. Full suites:
   - `pnpm run typecheck`
   - `pnpm run test`
   - `pnpm run test:ci`
   - embedded MemoryKernel full verification set

## Baseline

- release_tag: `v0.4.0`
- commit_sha: `7e4806a34b98e6c06ee33fa9f11499a975e7b922`
- service/api/baseline: `service.v3` / `api.v1` / `integration/v1`

## Verdict Criteria

- All required checks pass.
- No drift findings remain.
- Rollback workflow documented and executable.
