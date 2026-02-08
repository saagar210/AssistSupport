#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: generate_producer_handoff_payload.sh [options]

Generate a producer release handoff payload JSON from the canonical manifest.

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
changelog_path="$memorykernel_root/CHANGELOG.md"

if [[ ! -f "$manifest_path" ]]; then
  echo "manifest file missing: $manifest_path" >&2
  exit 1
fi

if [[ ! -f "$changelog_path" ]]; then
  echo "changelog missing: $changelog_path" >&2
  exit 1
fi

payload=$(
  python3 - "$manifest_path" "$changelog_path" <<'PY'
import json
import pathlib
import sys
from datetime import datetime, timezone

manifest_path = pathlib.Path(sys.argv[1])
changelog_path = pathlib.Path(sys.argv[2])

manifest = json.loads(manifest_path.read_text(encoding="utf-8"))

required = [
    "release_tag",
    "commit_sha",
    "expected_service_contract_version",
    "expected_api_contract_version",
    "integration_baseline",
    "error_code_enum",
]
for key in required:
    if key not in manifest:
        raise SystemExit(f"manifest missing required key: {key}")

payload = {
    "handoff_generated_at_utc": datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z"),
    "release_tag": manifest["release_tag"],
    "commit_sha": manifest["commit_sha"],
    "expected_service_contract_version": manifest["expected_service_contract_version"],
    "expected_api_contract_version": manifest["expected_api_contract_version"],
    "integration_baseline": manifest["integration_baseline"],
    "error_code_enum": manifest["error_code_enum"],
    "manifest_contract_version": manifest.get("manifest_contract_version", "producer-contract-manifest.v1"),
    "manifest_path": str(manifest_path),
    "changelog_path": str(changelog_path),
    "verification_commands": [
        "cargo fmt --all -- --check",
        "cargo clippy --workspace --all-targets --all-features -- -D warnings",
        "cargo test --workspace --all-targets --all-features",
        "./scripts/verify_service_contract_alignment.sh --memorykernel-root /Users/d/Projects/MemoryKernel",
        "./scripts/verify_contract_parity.sh --canonical-root /Users/d/Projects/MemoryKernel",
        "./scripts/verify_trilogy_compatibility_artifacts.sh --memorykernel-root /Users/d/Projects/MemoryKernel",
        "./scripts/run_trilogy_smoke.sh --memorykernel-root /Users/d/Projects/MemoryKernel",
        "./scripts/run_trilogy_compliance_suite.sh --memorykernel-root /Users/d/Projects/MemoryKernel --skip-baseline",
    ],
    "consumer_impact_statement_template": {
        "runtime_behavior_changes": "<none|describe>",
        "required_consumer_mapping_changes": "<none|describe>",
        "required_test_updates": "<none|describe>",
        "repin_required": "<yes/no>",
        "rollback_instruction": "repin to <previous tag/sha>",
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
