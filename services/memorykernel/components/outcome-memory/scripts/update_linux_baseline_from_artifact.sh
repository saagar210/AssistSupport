#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 || $# -gt 2 ]]; then
  echo "usage: $0 <benchmark-report.json> [output-path]" >&2
  exit 2
fi

report_path="$1"
output_path="${2:-benchmarks/baseline.linux.v1.json}"

if [[ ! -f "$report_path" ]]; then
  echo "error: report not found: $report_path" >&2
  exit 1
fi

contract_version="$(jq -r '.contract_version // empty' "$report_path")"
if [[ "$contract_version" != "benchmark_report.v1" ]]; then
  echo "error: expected contract_version benchmark_report.v1, got: ${contract_version:-<missing>}" >&2
  exit 1
fi

jq -e '
  (.volumes | type == "array" and length > 0) and
  (.volumes[] | has("event_count") and has("append_p95_ms") and has("replay_p95_ms") and has("gate_p95_ms"))
' "$report_path" >/dev/null

jq -e '
  .thresholds != null and
  (.thresholds | has("append_p95_ms_max") and has("replay_p95_ms_max") and has("gate_p95_ms_max"))
' "$report_path" >/dev/null

mkdir -p "$(dirname "$output_path")"
tmp_path="$(mktemp "${output_path}.tmp.XXXXXX")"
source_label="$(basename "$report_path")"

jq --arg source_label "$source_label" '
  {
    contract_version: "benchmark_baseline.v1",
    platform: "linux",
    captured_at: (.generated_at // "unknown"),
    source: ("artifact:" + $source_label),
    volumes: [
      .volumes[]
      | {
          event_count: .event_count,
          append_p95_ms: .append_p95_ms,
          replay_p95_ms: .replay_p95_ms,
          gate_p95_ms: .gate_p95_ms
        }
    ],
    ci_thresholds: {
      append_p95_ms_max: .thresholds.append_p95_ms_max,
      replay_p95_ms_max: .thresholds.replay_p95_ms_max,
      gate_p95_ms_max: .thresholds.gate_p95_ms_max
    },
    notes: "Updated from benchmark_report.v1 artifact via scripts/update_linux_baseline_from_artifact.sh"
  }
' "$report_path" >"$tmp_path"

mv "$tmp_path" "$output_path"
echo "updated $output_path from $report_path"
