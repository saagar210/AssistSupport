#!/usr/bin/env bash
set -euo pipefail

reports_dir="${1:-benchmarks/stress-history}"

if [[ ! -d "$reports_dir" ]]; then
  echo "error: reports directory not found: $reports_dir" >&2
  exit 1
fi

shopt -s nullglob
report_files=("$reports_dir"/benchmark-stress-report*.json)
shopt -u nullglob

if [[ ${#report_files[@]} -eq 0 ]]; then
  echo "error: no stress reports found in $reports_dir (expected benchmark-stress-report*.json)" >&2
  exit 1
fi

jq -s '
  def ceil3: ((. * 1000.0) | ceil) / 1000.0;
  def max_append: (map(.volumes[]?.append_p95_ms) | max);
  def max_replay: (map(.volumes[]?.replay_p95_ms) | max);
  def max_gate: (map(.volumes[]?.gate_p95_ms) | max);
  {
    contract_version: "stress_summary.v1",
    report_count: length,
    generated_at_window: {
      first: (map(.generated_at) | min),
      last: (map(.generated_at) | max)
    },
    max_observed_p95_ms: {
      append: max_append,
      replay: max_replay,
      gate: max_gate
    },
    suggested_thresholds_p95_ms: {
      append: ((max_append * 1.50) | ceil3),
      replay: ((max_replay * 1.50) | ceil3),
      gate: ((max_gate * 1.50) | ceil3)
    },
    guidance: "Tune only when repeated nightly failures occur; keep >=50% headroom over max observed p95."
  }
' "${report_files[@]}"
