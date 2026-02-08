# Trilogy Closeout Playbook (Phases 8-11)

## Purpose

Provide one deterministic procedure to close Phases 8-11 across MemoryKernel, OutcomeMemory, and MultiAgentCenter with no implicit steps.

## Scope

- In-scope:
  - hosted CI convergence evidence (Phase 8)
  - release-candidate lock records (Phase 9)
  - soak/ops evidence (Phase 10)
  - final approval/promotion/stabilization evidence (Phase 11)
- Out-of-scope:
  - new feature work
  - integration contract breaking changes (`integration/v2` required)

## Inputs

- Local workspaces available:
  - `/Users/d/Projects/MemoryKernel`
  - `/Users/d/Projects/OutcomeMemory`
  - `/Users/d/Projects/MultiAgentCenter`
- Hosted repository IDs (required for hosted evidence capture):
  - `<owner>/MemoryKernel`
  - `<owner>/OutcomeMemory`
  - `<owner>/MultiAgentCenter`

## Step 1: Run Deterministic Local Closeout

Run from MemoryKernel:

```bash
./scripts/run_trilogy_phase_8_11_closeout.sh \
  --memorykernel-root /Users/d/Projects/MemoryKernel \
  --outcome-root /Users/d/Projects/OutcomeMemory \
  --multi-agent-root /Users/d/Projects/MultiAgentCenter \
  --soak-iterations 1
```

Expected output artifact:

- `docs/implementation/trilogy-closeout-report-latest.md`

## Step 2: Configure OutcomeMemory Hosted Canonical Pointer

Set once on hosted OutcomeMemory repo:

```bash
gh variable set MEMORYKERNEL_CANONICAL_REPO \
  -R <owner>/OutcomeMemory \
  -b <owner>/MemoryKernel
```

Verify:

```bash
gh variable list -R <owner>/OutcomeMemory | rg '^MEMORYKERNEL_CANONICAL_REPO\\s'
```

## Step 3: Capture Hosted Evidence

Re-run closeout with hosted repositories:

```bash
./scripts/run_trilogy_phase_8_11_closeout.sh \
  --memorykernel-root /Users/d/Projects/MemoryKernel \
  --outcome-root /Users/d/Projects/OutcomeMemory \
  --multi-agent-root /Users/d/Projects/MultiAgentCenter \
  --memorykernel-repo <owner>/MemoryKernel \
  --outcome-repo <owner>/OutcomeMemory \
  --multi-agent-repo <owner>/MultiAgentCenter \
  --require-hosted \
  --soak-iterations 1
```

This captures:

- OutcomeMemory `smoke.yml` run list
- MultiAgentCenter `trilogy-guard.yml` run list
- MemoryKernel `release.yml` run list
- OutcomeMemory variable presence (`MEMORYKERNEL_CANONICAL_REPO`)

## Step 4: Record RC Lock Metadata (Phase 9)

Record for each project:

- release version (SemVer)
- immutable commit SHA for the promoted RC
- gate evidence reference (run URL or report timestamp)

Record location:

- `docs/implementation/trilogy-compatibility-matrix.md`
- `docs/implementation/trilogy-execution-status-2026-02-07.md`

## Step 5: Final Promotion and Stabilization (Phase 11)

Promotion order:

1. MemoryKernel
2. OutcomeMemory
3. MultiAgentCenter

Rollback order:

1. MultiAgentCenter
2. OutcomeMemory
3. MemoryKernel

Stabilization evidence to capture:

- incident summary (or clean window confirmation)
- final release approval decisions per project
- reaffirmation that `contracts/integration/v1/*` remains frozen

## Exit Criteria

- Phase 8: hosted CI evidence recorded with canonical pointer configured.
- Phase 9: RC versions and commit SHAs locked for all three repos.
- Phase 10: soak and operations checks recorded as passing.
- Phase 11: approvals, promotions, and stabilization report recorded.
