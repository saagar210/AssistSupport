#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: verify_producer_contract_manifest.sh [options]

Validate the machine-readable producer contract manifest used by consumers.

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

require_grep() {
  local pattern="$1"
  local file="$2"
  if ! rg -n --quiet -- "$pattern" "$file"; then
    echo "required pattern not found: '$pattern' in $file" >&2
    exit 1
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

manifest="$memorykernel_root/contracts/integration/v1/producer-contract-manifest.json"
service_main="$memorykernel_root/crates/memory-kernel-service/src/main.rs"
openapi="$memorykernel_root/openapi/openapi.yaml"
service_contract_doc="$memorykernel_root/docs/spec/service-contract.md"
versioning_doc="$memorykernel_root/docs/spec/versioning.md"

for path in "$manifest" "$service_main" "$openapi" "$service_contract_doc" "$versioning_doc"; do
  if [[ ! -f "$path" ]]; then
    echo "required file missing: $path" >&2
    exit 1
  fi
done

echo "[check] manifest JSON structure and policy fields"
python3 - "$manifest" <<'PY'
import json
import re
import sys

path = sys.argv[1]
with open(path, "r", encoding="utf-8") as f:
    data = json.load(f)

required = [
    "manifest_contract_version",
    "release_tag",
    "commit_sha",
    "expected_service_contract_version",
    "expected_api_contract_version",
    "integration_baseline",
    "error_code_enum",
    "stability_window",
    "additive_error_code_notice_policy",
    "service_v3_transition_guard",
]
for key in required:
    if key not in data:
        raise SystemExit(f"missing required key: {key}")

if data["manifest_contract_version"] != "producer-contract-manifest.v1":
    raise SystemExit("unexpected manifest_contract_version")

if not re.fullmatch(r"v\d+\.\d+\.\d+", data["release_tag"]):
    raise SystemExit("release_tag must be semver-style (vX.Y.Z)")

if not re.fullmatch(r"[0-9a-f]{40}", data["commit_sha"]):
    raise SystemExit("commit_sha must be a 40-char lowercase hex sha")

if data["expected_service_contract_version"] != "service.v3":
    raise SystemExit("expected_service_contract_version must be service.v3")

if data["expected_api_contract_version"] != "api.v1":
    raise SystemExit("expected_api_contract_version must be api.v1")

if data["integration_baseline"] != "integration/v1":
    raise SystemExit("integration_baseline must be integration/v1")

expected_codes = [
    "invalid_json",
    "validation_error",
    "context_package_not_found",
    "write_conflict",
    "write_failed",
    "schema_unavailable",
    "migration_failed",
    "query_failed",
    "context_lookup_failed",
    "internal_error",
]
if data["error_code_enum"] != expected_codes:
    raise SystemExit("error_code_enum does not match expected service.v3 canonical ordering")

window = data["stability_window"]
if window.get("minimum_sprint_days") != 14:
    raise SystemExit("stability_window.minimum_sprint_days must be 14")

notice = data["additive_error_code_notice_policy"]
if notice.get("standard_business_days") != 10:
    raise SystemExit("standard_business_days must be 10")
if notice.get("emergency_notice_hours") != 24:
    raise SystemExit("emergency_notice_hours must be 24")
if notice.get("emergency_requires_same_day_docs_spec_tests") is not True:
    raise SystemExit("emergency_requires_same_day_docs_spec_tests must be true")

guard = data["service_v3_transition_guard"]
for key in (
    "legacy_error_must_remain_in_service_v2",
    "legacy_error_removal_requires_service_v3",
    "consumer_green_repin_required_before_legacy_error_removal",
):
    if guard.get(key) is not True:
        raise SystemExit(f"{key} must be true")
PY

echo "[check] manifest service/api versions align with runtime and docs"
require_grep 'const SERVICE_CONTRACT_VERSION: &str = "service.v3";' "$service_main"
require_grep '^  version: service.v3$' "$openapi"
require_grep 'Service contract version: `service.v3`' "$service_contract_doc"
require_grep 'API contract version surfaced in envelopes: `api.v1`' "$service_contract_doc"
require_grep 'Service contract: `service.v3`' "$versioning_doc"
require_grep 'API envelope contract: `api.v1`' "$versioning_doc"

echo "[check] manifest error code enum aligns with OpenAPI"
for code in \
  invalid_json \
  validation_error \
  context_package_not_found \
  write_conflict \
  write_failed \
  schema_unavailable \
  migration_failed \
  query_failed \
  context_lookup_failed \
  internal_error; do
  require_grep "\\- ${code}" "$openapi"
  require_grep "\"${code}\"" "$manifest"
done

echo "[check] manifest policy aligns with service-v3 lifecycle docs"
require_grep '`legacy_error` is removed in `service.v3`' "$service_contract_doc"
require_grep 'service.v3' "$service_contract_doc"
require_grep 'service.v3' "$versioning_doc"

echo "Producer contract manifest checks passed."
