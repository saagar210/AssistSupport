# Week 3 Carry-Forward Plan

Date: 2026-02-22
Source: `docs/reports/week1-2-stabilization-readiness.md`
Status: **Execution complete for engineering items; governance closure pending required approval + merged-branch checks**

## Objective

Close temporary waivers/debt from Week 1-2 stabilization and lock release governance for the next delivery phase.

## Required Week 3 Items

1. Close Rust advisory waiver debt before expiry (2026-03-01)

- Create mitigation tracking issue(s) for all temporary cargo-audit ignores.
- Decide one of: dependency upgrade path, feature-flag/platform pruning, or approved long-term risk acceptance.
- Acceptance: `pnpm test:security:audit:rust` remains green with updated allowlist and linked mitigation issue IDs.
- Outcome: **Completed (partial reduction + tracked blockers)**. Ignore set reduced from 20 -> 18; issue map: `#11` umbrella, `#12`–`#15` child tracks.

2. Replace temporary "Unknown" mitigation metadata in security waiver with actual owners/issues

- Update `scripts/security/run-cargo-audit.sh` comments to include real owner and real issue references.
- Acceptance: no "Unknown" placeholders remain for waiver ownership or mitigation tracking.
- Outcome: **Completed**. `Unknown` count is 0 in waiver metadata.

3. Resolve frontend bundle warning debt (`index` chunk > 500kB)

- Split heavy modules or add chunk strategy to reduce single-chunk risk.
- Acceptance: `pnpm perf:bundle` and `pnpm perf:build` pass with no new regressions; warning reduced or intentionally documented.
- Outcome: **Completed**. Main chunk reduced 529.46 kB -> 333.64 kB; perf gates pass.

4. Validate CI on remote branch with stabilization gates enforced

- Confirm `ci.yml` and `quality-gates.yml` run green in GitHub Actions on the merged branch.
- Acceptance: all required lanes pass with no pass-by-skip semantics.
- Outcome: **In progress / blocked by governance preconditions**. PR head gates are running on latest SHA, but merge to `master` is blocked by required approval and branch governance.

5. Finalize readiness handoff for feature-phase restart

- Refresh readiness report with actual CI run links, final gate matrix, and release recommendation.
- Acceptance: go/no-go decision documented with explicit residual risks and owners.
- Outcome: **In progress**. `docs/reports/phase3-closeout-readiness.md` updated with current evidence and explicit merge blockers.

## Stretch (If Capacity Allows)

- Start reducing transitive GTK/Tauri advisory footprint by upgrading or feature-scoping dependencies.
- Add targeted unit tests around LLM streaming token decode behavior after migration to `token_to_piece`.
