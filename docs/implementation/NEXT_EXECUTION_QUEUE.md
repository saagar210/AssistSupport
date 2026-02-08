# Next Execution Queue (Single-Threaded Program Backlog)

Updated: 2026-02-08
Owner: AssistSupport (joint execution driver)

Purpose: convert remaining work into a fixed execution queue to avoid prompt-by-prompt ambiguity.

## Section 1: Monorepo Consolidation Readiness
Objective: eliminate dual-repo operational friction while preserving release safety.

- [x] Finalize consolidation authority model document (source-of-truth, mirrored components, ownership map).
- [x] Create migration gate checklist with explicit pass/fail criteria for each migration gate.
- [x] Add compatibility guard that blocks migration if governance artifacts are stale.
- [x] Produce pre-migration evidence packet (tests, compliance suite, rollback readiness).

Exit Criteria:
- Consolidation authority model approved.
- Migration gate checklist is executable with no ambiguous steps.
- Pre-migration evidence packet is complete and green.

## Section 2: MemoryKernel Backend Hardening Extension
Objective: increase reliability/security observability before broader AssistSupport iteration.

- [x] Add request-level telemetry counters for service timeout/failure classes in producer service.
- [x] Add explicit service timeout SLO definitions and alerting trigger docs.
- [x] Expand failure-mode tests for malformed and partial payload edge cases across service endpoints.
- [x] Validate governance scripts still detect pin/matrix/manifest drift under negative test fixtures.

Exit Criteria:
- Timeout/error telemetry surfaced in docs and test evidence.
- Failure-mode regression tests expanded and green.
- Governance negative tests documented and passing.

## Section 3: AssistSupport Product Reliability Iteration
Objective: improve user-facing reliability and diagnostics while keeping deterministic fallback.

- [x] Improve diagnostics UX for enrichment disable reasons and remediation hints.
- [x] Add integration health panel traceability to pin + handoff payload state.
- [x] Strengthen non-blocking fallback copy and response quality heuristics.
- [ ] Add smoke tests for degraded-mode Draft flow end-to-end behavior.

Exit Criteria:
- Draft flow remains fully usable under producer offline/timeout/schema mismatch.
- Diagnostics provide actionable operator guidance without exposing sensitive internals.
- Regression suite proves deterministic fallback and non-blocking behavior.

## Section 4: Cutover-Program Closeout and Transfer
Objective: prepare safe transfer to new machine and repeatable operation.

- [ ] Generate one-click workstation bootstrap verification packet from current baseline.
- [ ] Run full monorepo readiness command set and archive artifacts.
- [ ] Produce operator handoff runbook for normal operations + incident rollback.
- [ ] Freeze baseline and publish final checkpoint summary.

Exit Criteria:
- New machine bootstrap path is documented and validated.
- Complete evidence bundle exists for security/compliance review.
- Final checkpoint summary is published with explicit steady-state verdict.

## Mandatory Verification Set (for each section closure)
```bash
pnpm run typecheck
pnpm run test
pnpm run test:memorykernel-governance-negative
pnpm run check:memorykernel-pin
pnpm run check:memorykernel-governance
pnpm run check:memorykernel-handoff:service-v3-candidate
pnpm run test:memorykernel-contract
pnpm run test:ci
cd services/memorykernel && ./scripts/test_producer_governance_negative.sh --memorykernel-root "$(pwd)"
```

## Program Rule
No section is marked complete without:
1. explicit exit-criteria evidence,
2. command outputs captured,
3. rollback implications documented.
