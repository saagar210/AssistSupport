# Phase A Baseline Scorecard

Date: 2026-03-05  
Phase: A (Guardrails + Baselines)

## What Is Enforced

- Contract fixture drift gate (Rust + Search API).
- Golden journey lane (`@journey`) for behavior parity.
- Program scorecard generation for performance and DORA snapshots.
- Command lifecycle policy gate for all registered Tauri commands.
- Rust lock-await policy gate (`clippy::await_holding_lock`) with exception report output.

## How To Capture Baselines

Run:

```bash
pnpm perf:baseline:capture
```

Artifacts:

- `.perf-results/summary.json`
- `.perf-results/program-scorecard.json`
- `.perf-results/program-scorecard.md`
- `.perf-results/startup.json`
- `.perf-results/generation.json`
- `.perf-results/search.json`
- `.perf-results/dora.json`

## CI Integration

- `ci.yml`:
  - `check:contract-fixtures` in frontend static lane.
  - `check:command-lifecycle` in backend lane.
  - `ci:backend:gate:lock-policy` in backend lane.
  - `test:contracts:rust` in backend lane.
  - `test:e2e:journey` in UI regression lane.
- `quality-gates.yml`:
  - `check:contract-fixtures`.
  - `check:command-lifecycle`.

## Runtime Capture Policy

Runtime captures are mandatory for phase-exit gating. `pnpm perf:baseline:capture` now runs:

- `pnpm perf:runtime` (startup/generation/search probe artifacts)
- `pnpm perf:dora` (git + CI metadata DORA snapshot)
