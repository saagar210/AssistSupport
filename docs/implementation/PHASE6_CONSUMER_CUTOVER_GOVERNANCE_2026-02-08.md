# Phase 6 Consumer Cutover Governance Scaffold

Updated: 2026-02-08  
Owner: AssistSupport

## Policy State
Runtime cutover remains disabled until explicit joint approval from AssistSupport and MemoryKernel after all cutover gates are complete.

## Cutover Command Checklist (Policy-Disabled Until Joint Approval)
Do not execute this checklist for production cutover until joint GO is documented.

1. Confirm producer handoff payload and verification evidence are final for cutover candidate.
2. Prepare one atomic PR updating:
   - `config/memorykernel-integration-pin.json`
   - `config/memorykernel-producer-manifest.json`
   - `docs/MEMORYKERNEL_COMPATIBILITY_MATRIX.md`
3. Run:
   - `pnpm run check:memorykernel-pin`
   - `pnpm run check:memorykernel-governance`
   - `pnpm run check:memorykernel-handoff`
   - `pnpm run test:memorykernel-contract`
   - `pnpm run test:ci`
4. Execute rollback drill checklist and verify pass criteria.
5. Record joint GO/NO-GO and final decision owner sign-off.

## Rollback Success Checklist
1. Re-pin consumer to last approved baseline (`v0.3.2` / `cf331449e1589581a5dcbb3adecd3e9ae4509277`).
2. Re-run:
   - `pnpm run check:memorykernel-pin`
   - `pnpm run test:memorykernel-contract`
   - `pnpm run test:ci`
3. Confirm Draft flow remains available without enrichment.
4. Confirm fallback reasons still map deterministically for offline/timeout/malformed/version mismatch/non-2xx.
5. Publish rollback evidence artifact updates.

## Incident Communication Template

### Incident Header
- Title:
- Severity:
- Start time (UTC):
- Current status:
- Owner:

### Impact
- User-visible impact:
- Affected features:
- Whether Draft flow availability was impacted:

### Immediate Actions
1. Baseline frozen: yes/no
2. Rollback initiated: yes/no
3. Validation commands executed:

### Decision and Timeline
- Current decision: GO / NO-GO / ROLLBACK
- Next checkpoint time (UTC):
- Dependencies pending from MemoryKernel:

### Finalization
- Resolution summary:
- Follow-up actions:
- Owner assignments and due dates:
