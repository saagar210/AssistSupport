#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: generate_service_v3_rehearsal_payload.sh [options]

Generate a planning-only producer rehearsal handoff payload for service.v3.

Options:
  --memorykernel-root <path>  Path to MemoryKernel root (default: script/..)
  --out-json <path>           Output JSON path (default: stdout only)
  -h, --help                  Show this help
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
out_json=""

while (($# > 0)); do
  case "$1" in
    --memorykernel-root)
      memorykernel_root="${2:-}"
      shift 2
      ;;
    --out-json)
      out_json="${2:-}"
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

manifest_path="$memorykernel_root/contracts/integration/v1/producer-contract-manifest.json"
cutover_gates_path="$memorykernel_root/docs/implementation/SERVICE_V3_CUTOVER_GATES.md"

if [[ ! -f "$manifest_path" ]]; then
  echo "manifest file missing: $manifest_path" >&2
  exit 1
fi

if [[ ! -f "$cutover_gates_path" ]]; then
  echo "cutover-gates file missing: $cutover_gates_path" >&2
  exit 1
fi

payload=$(
  python3 - "$manifest_path" "$cutover_gates_path" "$memorykernel_root" <<'PY'
import datetime as dt
import json
import pathlib
import subprocess
import sys

manifest_path = pathlib.Path(sys.argv[1])
cutover_gates_path = pathlib.Path(sys.argv[2])
memorykernel_root = pathlib.Path(sys.argv[3])
manifest = json.loads(manifest_path.read_text(encoding="utf-8"))

required = (
    "release_tag",
    "commit_sha",
    "expected_service_contract_version",
    "expected_api_contract_version",
    "integration_baseline",
    "error_code_enum",
)
for key in required:
    if key not in manifest:
        raise SystemExit(f"manifest missing required key: {key}")

today_utc = dt.datetime.now(dt.timezone.utc).date()
not_before = today_utc + dt.timedelta(days=14)

planning_commit = "unknown"
try:
    planning_commit = subprocess.check_output(
        ["git", "-C", str(memorykernel_root), "rev-parse", "HEAD"],
        text=True,
    ).strip()
except Exception:
    pass

payload = {
    "payload_contract_version": "service-v3-rehearsal-handoff.v1",
    "generated_at_utc": dt.datetime.now(dt.timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z"),
    "producer": {
        "repo": "saagar210/MemoryKernel",
        "planning_commit_seen_by_consumer": planning_commit,
        "active_runtime_baseline": {
            "release_tag": manifest["release_tag"],
            "commit_sha": manifest["commit_sha"],
            "service_contract_version": manifest["expected_service_contract_version"],
            "api_contract_version": manifest["expected_api_contract_version"],
            "integration_baseline": manifest["integration_baseline"],
        },
    },
    "rehearsal_candidate": {
        "mode": "planning_only_no_runtime_cutover",
        "target_service_contract_version": "service.v3",
        "target_api_contract_version": "api.v1",
        "requires_runtime_cutover": False,
        "cutover_not_before_utc_date": str(not_before),
    },
    "non_2xx_envelope_policy": {
        "service_v2_stable": {
            "requires": [
                "service_contract_version",
                "error.code",
                "error.message",
                "legacy_error",
            ],
            "forbids": [
                "api_contract_version",
            ],
        },
        "service_v3_candidate": {
            "requires": [
                "service_contract_version",
                "error.code",
                "error.message",
            ],
            "optional": [
                "error.details",
            ],
            "forbids": [
                "legacy_error",
                "api_contract_version",
            ],
        },
    },
    "error_code_expectations": {
        "validation_mode": "set_equality",
        "codes": manifest["error_code_enum"],
        "additive_change_notice_policy": {
            "standard_business_days": 10,
            "emergency_notice_hours": 24,
            "emergency_requires_same_day_docs_spec_tests": True,
        },
    },
    "migration_overlap_assumptions": {
        "duration_days": 14,
        "service_v2_stability_required": True,
        "consumer_enrichment_must_remain_optional_non_blocking": True,
        "consumer_fallback_must_remain_deterministic": True,
    },
    "cutover_gates_doc": str(cutover_gates_path),
    "required_producer_checks": [
        "cargo fmt --all -- --check",
        "cargo clippy --workspace --all-targets --all-features -- -D warnings",
        "cargo test --workspace --all-targets --all-features",
        "./scripts/verify_service_contract_alignment.sh --memorykernel-root /Users/d/Projects/MemoryKernel",
        "./scripts/verify_contract_parity.sh --canonical-root /Users/d/Projects/MemoryKernel",
        "./scripts/verify_trilogy_compatibility_artifacts.sh --memorykernel-root /Users/d/Projects/MemoryKernel",
        "./scripts/run_trilogy_smoke.sh --memorykernel-root /Users/d/Projects/MemoryKernel",
        "./scripts/run_trilogy_compliance_suite.sh --memorykernel-root /Users/d/Projects/MemoryKernel --skip-baseline",
    ],
    "required_consumer_checks": [
        "pnpm run check:memorykernel-pin",
        "pnpm run test:memorykernel-contract",
        "pnpm run test:ci",
    ],
    "rollback_policy": {
        "triggers": [
            "consumer_contract_suite_red",
            "non_2xx_envelope_shape_mismatch",
            "deterministic_fallback_regression",
        ],
        "action": "repin consumer to last approved service.v2 tag/sha and reopen overlap window",
    },
}

print(json.dumps(payload, indent=2))
PY
)

if [[ -n "$out_json" ]]; then
  mkdir -p "$(dirname "$out_json")"
  printf "%s\n" "$payload" > "$out_json"
fi

printf "%s\n" "$payload"
