# Week 3 Carry-Forward Plan

Date: 2026-02-22
Source: `docs/reports/week1-2-stabilization-readiness.md`

## Objective
Close temporary waivers/debt from Week 1-2 stabilization and lock release governance for the next delivery phase.

## Required Week 3 Items

1. Close Rust advisory waiver debt before expiry (2026-03-01)
- Create mitigation tracking issue(s) for all temporary cargo-audit ignores.
- Decide one of: dependency upgrade path, feature-flag/platform pruning, or approved long-term risk acceptance.
- Acceptance: `pnpm test:security:audit:rust` remains green with updated allowlist and linked mitigation issue IDs.

2. Replace temporary "Unknown" mitigation metadata in security waiver with actual owners/issues
- Update `scripts/security/run-cargo-audit.sh` comments to include real owner and real issue references.
- Acceptance: no "Unknown" placeholders remain for waiver ownership or mitigation tracking.

3. Resolve frontend bundle warning debt (`index` chunk > 500kB)
- Split heavy modules or add chunk strategy to reduce single-chunk risk.
- Acceptance: `pnpm perf:bundle` and `pnpm perf:build` pass with no new regressions; warning reduced or intentionally documented.

4. Validate CI on remote branch with stabilization gates enforced
- Confirm `ci.yml` and `quality-gates.yml` run green in GitHub Actions on the merged branch.
- Acceptance: all required lanes pass with no pass-by-skip semantics.

5. Finalize readiness handoff for feature-phase restart
- Refresh readiness report with actual CI run links, final gate matrix, and release recommendation.
- Acceptance: go/no-go decision documented with explicit residual risks and owners.

## Stretch (If Capacity Allows)

- Start reducing transitive GTK/Tauri advisory footprint by upgrading or feature-scoping dependencies.
- Add targeted unit tests around LLM streaming token decode behavior after migration to `token_to_piece`.
