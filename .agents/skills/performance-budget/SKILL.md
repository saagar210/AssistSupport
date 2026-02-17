---
name: performance-budget
description: Run bundle/build/Lighthouse/API/DB/memory/asset checks, compare against baselines, classify drift, and enforce profile-specific gates.
---

# Performance Budget Skill

## Objective

Catch regressions at PR-time with low-flake, deterministic checks and clear remediation advice.

## Inputs

- Profile: `side` or `production`.
- Current branch and baseline files under `.perf-baselines/`.

## Metrics

- Bundle size (`totalBytes` and key asset sizes).
- Build time (`buildMs`).
- Lighthouse scores.
- API latency/error thresholds (k6).
- DB query performance (`pg_stat_statements` offenders).
- Memory growth smoke check.
- Asset size caps.

## Default thresholds

- Side:
  - Bundle regression max: `+10%`
  - Build time regression max: `+25%`
  - Lighthouse performance min: `0.85`
  - API p95 max: `350ms`
  - DB offender threshold: `mean_exec_time > 120ms with calls >= 50`
  - Asset file max: `350KB`
- Production:
  - Bundle regression max: `+8%`
  - Build time regression max: `+15%`
  - Lighthouse performance min: `0.90`
  - API p95 max: `250ms`
  - DB offender threshold: `mean_exec_time > 100ms with calls >= 50`
  - Asset file max: `250KB`

## Procedure

1. Run checks:

- `pnpm perf:bundle`
- `pnpm perf:build`
- `pnpm perf:lhci`
- `pnpm perf:api`
- `pnpm perf:db`
- `pnpm perf:memory`
- `pnpm perf:assets`

2. Compare with baselines:

- Use `scripts/perf/compare-metric.mjs` for ratio-based checks.
- Mark each metric as `pass`, `warn`, or `fail`.

3. Diagnose meaningful regressions:

- Point to changed files likely responsible.
- Suggest one minimal remediation per failing metric.

4. Gate decision:

- Side profile:
  - Fail only on severe regressions or repeated drift.
- Production profile:
  - Fail on any required metric `fail` or `not-run`.

5. Output report:

- Single markdown table with baseline/current/delta/status.
- Top 3 root-cause suspects.
- Exact commands to reproduce locally.

## Outputs

- Machine-readable summary in `.perf-results/summary.json`.
- Human report for PR comment.
- Gate verdict: `pass`, `warn`, or `fail`.
