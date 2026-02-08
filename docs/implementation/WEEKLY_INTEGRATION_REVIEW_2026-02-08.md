# Weekly Integration Review: AssistSupport + MemoryKernel

Date: 2026-02-08
Review Type: Week 1 checkpoint (service.v2 steady-state window)

## Checklist
- [x] Approved baseline unchanged or intentionally updated.
- [x] Governance checks green in both repos.
- [x] Proposed producer contract changes reviewed.
- [x] No unresolved high-severity integration incidents.
- [x] Owners and next actions confirmed.

## Baseline Reviewed
- MemoryKernel release tag: `v0.3.1`
- MemoryKernel commit SHA: `cf331449e1589581a5dcbb3adecd3e9ae4509277`
- Service/API/baseline: `service.v2` / `api.v1` / `integration/v1`

## Evidence
- AssistSupport:
  - `pnpm run check:memorykernel-pin` -> PASS
  - `pnpm run test:memorykernel-contract` -> PASS
- MemoryKernel:
  - `./scripts/verify_producer_contract_manifest.sh --memorykernel-root /Users/d/Projects/MemoryKernel` -> PASS

## Decisions Confirmed
1. Checkpoint C = GO (14-day steady-state service.v2 window).
2. Checkpoint D = GO for planning only.
3. Hash validation for mirrored producer manifest is active in AssistSupport CI.

## Risks and Actions
| Risk | Action | Owner | Due |
|---|---|---|---|
| Remote manifest fetch transient CI failures | Retry policy + keep local hash parity check for dev loops | AssistSupport | Next checkpoint |
| service.v3 overlap drift | Execute rehearsal plan before any cutover | Joint | Before cutover |

## Next Review Target
- 2026-02-15 (or earlier if baseline changes)

