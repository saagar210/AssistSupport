# OutcomeMemory Performance Program

## Harness
The benchmark harness is exposed via:

`mk outcome benchmark run`

It generates a versioned JSON payload (`benchmark_report.v1`) with p50/p95 latency metrics for:
- append
- replay
- gate preview

## Baseline and Thresholds
- Baseline snapshots:
  - `/Users/d/Projects/OutcomeMemory/benchmarks/baseline.macos.v1.json`
  - `/Users/d/Projects/OutcomeMemory/benchmarks/baseline.linux.v1.json`
- Platform-specific CI thresholds:
  - Linux (`ubuntu-latest`): `append_p95_ms_max=8.0`, `replay_p95_ms_max=250.0`, `gate_p95_ms_max=8.0`
  - macOS (`macos-latest`): `append_p95_ms_max=12.0`, `replay_p95_ms_max=350.0`, `gate_p95_ms_max=12.0`
- Stress nightly thresholds (high-volume benchmark profile):
  - `append_p95_ms_max=25.0`
  - `replay_p95_ms_max=1500.0`
  - `gate_p95_ms_max=25.0`

The Linux baseline volume values are seeded and must be replaced from the first Linux perf artifact.

## Replace Linux Baseline From CI Artifact
After the first successful `Performance` workflow on `ubuntu-latest`, download the artifact and update the baseline:

```bash
gh run download <run-id> -n benchmark-report-ubuntu-latest -D /tmp/outcome-perf
./scripts/update_linux_baseline_from_artifact.sh \
  /tmp/outcome-perf/benchmark-report-ubuntu-latest.json \
  benchmarks/baseline.linux.v1.json
```

The updater validates `benchmark_report.v1` and writes `benchmark_baseline.v1` for Linux.

## Example
```bash
cargo run -p memory-kernel-outcome-cli -- \
  --db /tmp/ignore.sqlite3 \
  outcome benchmark run \
  --volume 100 --volume 500 --volume 2000 \
  --repetitions 3 \
  --append-p95-max-ms 5 \
  --replay-p95-max-ms 200 \
  --gate-p95-max-ms 5 \
  --output benchmark-report.json \
  --json
```

When thresholds are provided, the command exits non-zero on violations.

Threshold enforcement semantics:
- Thresholds are optional, but when used they must be provided as a full set:
  - `--append-p95-max-ms`
  - `--replay-p95-max-ms`
  - `--gate-p95-max-ms`
- Any single threshold breach in any volume marks the run failed (`within_thresholds=false`) and exits non-zero.
- CI should always run with thresholds so regressions fail deterministically.
- CI additionally runs a deliberate violation case (all thresholds `0`) and asserts non-zero exit.

## Nightly Stress Monitoring and Tuning
Store nightly stress artifacts under `/Users/d/Projects/OutcomeMemory/benchmarks/stress-history` with filenames matching `benchmark-stress-report*.json`, then summarize:

```bash
./scripts/summarize_stress_reports.sh benchmarks/stress-history
```

Tuning policy:
- Do not tune thresholds for a single failure.
- Tune only after repeated nightly failures and confirming no deterministic regression.
- Keep at least 50% margin over observed max p95 values.

## 2026-02-07 Local Workflow-Equivalent Run
- Smoke suite passed (`cli_contracts_v1`, `projector_smoke`).
- Perf guardrails passed for both Linux and macOS threshold sets.
- Stress suite passed (`prop_`, long-stream determinism, scale guardrails, stress benchmark).
