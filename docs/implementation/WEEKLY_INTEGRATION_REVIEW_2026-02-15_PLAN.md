# Weekly Integration Review Plan: 2026-02-15

Owner: Joint (AssistSupport + MemoryKernel)
Window: Week 2 steady-state service.v2 check

## Objective
Confirm service.v2 remains stable through the 14-day window with no drift/regressions and prepare transition to service.v3 rehearsal execution.

## Inputs Required Before Review
1. Latest AssistSupport CI run URLs for:
   - MemoryKernel contract gate
   - Full CI run
2. Latest MemoryKernel manifest/alignment verification outputs.
3. Open change list affecting integration contracts since 2026-02-08.

## Checklist
- [ ] Baseline unchanged (`v0.3.2` / `cf331449...`) or intentionally repinned with full governance evidence.
- [ ] `pin + matrix + mirrored manifest` remain synchronized.
- [ ] Local manifest hash validation in consumer CI green.
- [ ] Remote manifest hash validation green when `MEMORYKERNEL_REPO_READ_TOKEN` is configured.
- [ ] Deterministic fallback behavior remains regression-free.
- [ ] No unresolved high-severity integration incidents.

## Verification Commands (AssistSupport)
```bash
pnpm run check:memorykernel-pin
pnpm run test:memorykernel-contract
pnpm run test:ci
```

## Verification Commands (MemoryKernel)
```bash
./scripts/verify_producer_contract_manifest.sh --memorykernel-root /Users/d/Projects/MemoryKernel
./scripts/verify_service_contract_alignment.sh --memorykernel-root /Users/d/Projects/MemoryKernel
```

## Expected Outputs
1. Week-2 GO/NO-GO verdict for closing steady-state window.
2. Confirmed service.v3 rehearsal start date.
3. Updated risk log with owner + due dates.

