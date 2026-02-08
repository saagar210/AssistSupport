# Performance Budgets (Normative)

This document defines baseline performance gates for deterministic retrieval paths.

## Scope

- Policy resolver path (`build_context_package`)
- Recall resolver path (`build_recall_context_package`)

## CI-Enforced Baselines

- `TPERF-001` policy context package generation over 500 records, 25 iterations:
  - MUST complete within 4 seconds total in CI debug profile.
- `TPERF-002` recall context package generation over 500 records, 25 iterations:
  - MUST complete within 4 seconds total in CI debug profile.

## Benchmark Suite

- Criterion benchmark target:
  - `cargo bench -p memory-kernel-core --bench resolver_bench -- --sample-size 10 --warm-up-time 0.01 --measurement-time 0.02 --noplot`
- Benchmark smoke run is included in CI to detect severe regressions (`MKR-050`).
