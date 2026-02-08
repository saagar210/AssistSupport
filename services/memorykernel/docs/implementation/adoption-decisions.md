# Adoption Decision Log

## 2026-02-07

- Status: Approved for local cross-project rollout
- Preconditions:
  - Phase 2 checklist complete
  - Phase 3 checklist complete
  - Phase 4-6 controls implemented and validated
  - Pilot report recorded at `docs/implementation/pilot-report-2026-02-07.md`
- Decision: Proceed with local integration across MemoryKernel, OutcomeMemory, and MultiAgentCenter.

## 2026-02-07 (Trilogy Release Gate)

- Status: Approved for release-candidate sequencing
- Preconditions:
  - Phase 7 convergence checklist complete
  - Contract parity and compatibility artifact checks passing
  - Trilogy smoke gate passing across MemoryKernel, OutcomeMemory, and MultiAgentCenter
- Decision: Proceed with release candidates in integration order: MemoryKernel -> OutcomeMemory -> MultiAgentCenter.
- Follow-up: Set `MEMORYKERNEL_CANONICAL_REPO` in OutcomeMemory hosted CI for deterministic canonical contract resolution.

## 2026-02-07 (Phase 10 Soak)

- Status: Completed
- Preconditions:
  - Trilogy compatibility artifact validation passing
  - Trilogy smoke gate passing
- Decision: Mark soak and runbook spot-check complete based on:
  - `./scripts/run_trilogy_soak.sh --iterations 3` (pass)
  - migration/recovery runbook command sequence spot-check (pass)
- Follow-up: Continue with Phase 8 hosted CI convergence evidence and Phase 9 RC version locking.

## 2026-02-07 (Phase 8/9/11 Coordination Status)

- Status: In progress with external dependencies
- Ready in MemoryKernel:
  - Phase 8-11 governance docs and checklists are in place.
  - Trilogy release gate and soak evidence are recorded.
- Pending outside MemoryKernel:
  - OutcomeMemory hosted CI variable `MEMORYKERNEL_CANONICAL_REPO` confirmation and evidence link.
  - Final RC version/sha lock entries from sibling repos.
  - Final release approvals/promotions across all three projects.

## 2026-02-07 (Sibling Quality Gate Revalidation)

- Status: Completed (local workspace evidence)
- Decision: Keep Phase 8 blocked only on hosted CI evidence, not on local code-health concerns.
- Evidence:
  - OutcomeMemory: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --workspace --all-targets --all-features` all pass.
  - MultiAgentCenter: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --workspace --all-targets --all-features` all pass.

## 2026-02-07 (Phase 8-11 Closeout Automation)

- Status: Completed (MemoryKernel-local)
- Decision: Adopt `scripts/run_trilogy_phase_8_11_closeout.sh` + `docs/implementation/trilogy-closeout-playbook.md` as the single deterministic closeout path for Phases 8-11.
- Rationale:
  - keeps hosted dependencies explicit instead of implicit
  - produces one report artifact (`trilogy-closeout-report-latest.md`) for phase evidence
  - prevents scope drift in release-status accounting
- Remaining external dependency:
  - hosted repository IDs and runs are still required to complete hosted evidence and final promotions.
- Evidence:
  - `./scripts/run_trilogy_phase_8_11_closeout.sh --soak-iterations 1` (pass)
  - `docs/implementation/trilogy-closeout-report-latest.md`

## 2026-02-08 (Hosted Convergence and RC Lock Approval)

- Status: Approved
- Decision:
  - Close Phase 8 hosted CI convergence and Phase 9 RC lock at trilogy baseline `0.1.0`.
  - Use locked commits recorded in `trilogy-compatibility-matrix.md` as release baseline.
- Hosted evidence:
  - MemoryKernel CI: `https://github.com/saagar210/MemoryKernel/actions/runs/21793009813`
  - MemoryKernel release workflow: `https://github.com/saagar210/MemoryKernel/actions/runs/21793014651`
  - OutcomeMemory Smoke: `https://github.com/saagar210/OutcomeMemory/actions/runs/21792820983`
  - OutcomeMemory Performance: `https://github.com/saagar210/OutcomeMemory/actions/runs/21792820986`
  - MultiAgentCenter trilogy-guard: `https://github.com/saagar210/MultiAgentCenter/actions/runs/21792988679`

## 2026-02-08 (Final Trilogy Release Approval)

- Status: Approved
- Preconditions:
  - Hosted and local closeout checks pass via `run_trilogy_phase_8_11_closeout.sh --require-hosted`.
  - RC lock metadata is finalized for all three projects.
- Decision:
  - Finalize trilogy release for current `v1` contract baseline.
  - Keep contract freeze policy: any semantic breaking change requires `contracts/integration/v2`.
- Promotion sequence confirmed:
  - `MemoryKernel` -> `OutcomeMemory` -> `MultiAgentCenter`
- Stabilization result:
  - Clean immediate post-promotion window; no blocking integration regressions observed in hosted validation runs.
