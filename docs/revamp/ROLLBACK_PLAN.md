# Revamp Rollback Plan

Status: Active  
Date: 2026-02-08

## 1) Rollback Anchors
1. Baseline tag: `revamp-baseline-2026-02-08`
2. Baseline commit: `bc77ae5286e6a9fdabe6b6e64322202f3cb9f989`
3. Primary branch: `master`

## 2) Trigger Conditions
Rollback is mandatory when any of the following occurs:
1. Canonical test/governance suite fails and cannot be corrected within current phase scope.
2. Core user flow becomes unavailable or non-deterministic.
3. MemoryKernel adapter boundary or fallback invariants are violated.
4. Security-critical regression detected.

## 3) Rollback Procedures
### Procedure A: Full branch rollback to baseline tag
```bash
git fetch --all --tags
git checkout master
git reset --hard revamp-baseline-2026-02-08
pnpm install
pnpm run test:ci
```

### Procedure B: Feature-flag rollback (preferred when possible)
1. Disable relevant revamp flags.
2. Re-run canonical suite.
3. Keep branch history, avoid full hard reset.

## 4) Post-Rollback Validation (Required)
```bash
pnpm run typecheck
pnpm run test
pnpm run check:memorykernel-pin
pnpm run check:memorykernel-governance
pnpm run check:memorykernel-handoff
pnpm run test:memorykernel-contract
pnpm run test:ci
```

## 5) Evidence Requirements
After rollback, log:
1. Trigger condition.
2. Chosen rollback procedure.
3. Command outputs.
4. Follow-up corrective action.

Evidence location:
- `docs/revamp/evidence/`
