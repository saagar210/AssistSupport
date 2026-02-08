# Weekly Integration Review Plan: 2026-02-15

Owner: Joint (AssistSupport + MemoryKernel)
Window: Week 2 steady-state service.v2 check
Execution note: Consumer closeout run executed early on 2026-02-08 using current producer closeout artifacts.

## Objective
Confirm service.v2 remains stable through the 14-day window with no drift/regressions and prepare transition to service.v3 rehearsal execution.

## Inputs Required Before Review
1. Latest AssistSupport CI run URLs for:
   - MemoryKernel contract gate
   - Full CI run
2. Latest MemoryKernel manifest/alignment verification outputs.
3. Open change list affecting integration contracts since 2026-02-08.

## Checklist
- [x] Baseline unchanged (`v0.3.2` / `cf331449...`) or intentionally repinned with full governance evidence.
- [x] `pin + matrix + mirrored manifest` remain synchronized.
- [x] Local manifest hash validation in consumer CI green.
- [x] Governance bundle validation (`pnpm run check:memorykernel-governance`) green.
- [x] Remote manifest hash validation green when `MEMORYKERNEL_REPO_READ_TOKEN` is configured.
- [x] Deterministic fallback behavior remains regression-free.
- [x] No unresolved high-severity integration incidents.

## Verification Commands (AssistSupport)
```bash
pnpm run check:memorykernel-pin
pnpm run check:memorykernel-governance
pnpm run test:memorykernel-contract
pnpm run test:ci
```

## Verification Commands (MemoryKernel)
```bash
./scripts/verify_producer_contract_manifest.sh --memorykernel-root /Users/d/Projects/MemoryKernel
./scripts/verify_service_contract_alignment.sh --memorykernel-root /Users/d/Projects/MemoryKernel
```

## Expected Outputs
1. Week-2 GO verdict for closing steady-state window (consumer scope) recorded in `/Users/d/Projects/AssistSupport/docs/implementation/PHASE1_STEADY_STATE_CLOSEOUT_2026-02-08.md`.
2. Service.v3 rehearsal remains planning-active pending producer candidate baseline publication.
3. Updated risk log with owner + due dates.
