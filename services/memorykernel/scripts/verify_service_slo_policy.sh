#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: verify_service_slo_policy.sh [options]

Validate benchmark SLO policy alignment across canonical policy artifact,
normative docs, and CI/release workflows.

Options:
  --memorykernel-root <path> Path to MemoryKernel repo root (default: script/..)
  -h, --help                 Show this help
USAGE
}

resolve_path() {
  local path="$1"
  if [[ -d "$path" ]]; then
    (cd "$path" && pwd -P)
  else
    printf "%s\n" "$path"
  fi
}

memorykernel_root=""

while (($# > 0)); do
  case "$1" in
    --memorykernel-root)
      memorykernel_root="${2:-}"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)
if [[ -z "$memorykernel_root" ]]; then
  memorykernel_root=$(resolve_path "$script_dir/..")
else
  memorykernel_root=$(resolve_path "$memorykernel_root")
fi

policy="$memorykernel_root/contracts/integration/v1/service-slo-policy.json"
policy_doc="$memorykernel_root/docs/spec/service-slo-policy.md"
ci_workflow="$memorykernel_root/.github/workflows/ci.yml"
release_workflow="$memorykernel_root/.github/workflows/release.yml"

for path in "$policy" "$policy_doc" "$ci_workflow" "$release_workflow"; do
  if [[ ! -f "$path" ]]; then
    echo "required file missing: $path" >&2
    exit 1
  fi
done

echo "[check] service SLO policy alignment"
python3 - "$policy" "$policy_doc" "$ci_workflow" "$release_workflow" <<'PY'
import json
import re
import sys
from pathlib import Path

policy_path, doc_path, ci_path, release_path = sys.argv[1:5]

policy = json.loads(Path(policy_path).read_text(encoding="utf-8"))
doc = Path(doc_path).read_text(encoding="utf-8")
ci = Path(ci_path).read_text(encoding="utf-8")
release = Path(release_path).read_text(encoding="utf-8")

required_keys = {
    "slo_policy_version",
    "benchmark_command",
    "workloads",
    "repetitions",
    "thresholds_ms",
    "operational_thresholds",
    "required_output",
}
missing = sorted(required_keys - set(policy.keys()))
if missing:
    raise SystemExit(f"service-slo-policy missing keys: {missing}")

thresholds = policy["thresholds_ms"]
for key in ("append_p95_max", "replay_p95_max", "gate_p95_max"):
    if key not in thresholds:
        raise SystemExit(f"service-slo-policy threshold missing: {key}")

operational_thresholds = policy["operational_thresholds"]
for key in ("timeout_rate_max_percent", "failure_rate_max_percent", "schema_unavailable_max_per_1000_requests"):
    if key not in operational_thresholds:
        raise SystemExit(f"service-slo-policy operational threshold missing: {key}")

required_doc_snippets = [
    "Service SLO Policy (Normative)",
    "service-slo-policy.json",
    str(thresholds["append_p95_max"]),
    str(thresholds["replay_p95_max"]),
    str(thresholds["gate_p95_max"]),
    str(operational_thresholds["timeout_rate_max_percent"]),
    str(operational_thresholds["failure_rate_max_percent"]),
    str(operational_thresholds["schema_unavailable_max_per_1000_requests"]),
]
for snippet in required_doc_snippets:
    if snippet not in doc:
        raise SystemExit(f"policy doc missing snippet: {snippet}")

workloads = policy["workloads"]
if not isinstance(workloads, list) or not workloads:
    raise SystemExit("policy workloads must be a non-empty list")

checks = [
    f"--repetitions {policy['repetitions']}",
    f"--append-p95-max-ms {thresholds['append_p95_max']}",
    f"--replay-p95-max-ms {thresholds['replay_p95_max']}",
    f"--gate-p95-max-ms {thresholds['gate_p95_max']}",
    f"--{policy['required_output']}",
]
checks.extend([f"--volume {workload}" for workload in workloads])

for workflow_name, workflow in (("ci", ci), ("release", release)):
    if "Outcome benchmark guardrails (host CLI)" not in workflow:
        raise SystemExit(f"{workflow_name} workflow missing outcome benchmark step")
    for check in checks:
        if check not in workflow:
            raise SystemExit(f"{workflow_name} workflow missing benchmark policy token: {check}")

print("Service SLO policy checks passed.")
PY
