#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: verify_service_contract_source_of_truth.sh [options]

Validate canonical service contract source-of-truth alignment across runtime,
OpenAPI, producer manifest, and producer handoff payload.

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

source_of_truth="$memorykernel_root/contracts/integration/v1/service-contract-source-of-truth.json"
manifest="$memorykernel_root/contracts/integration/v1/producer-contract-manifest.json"
handoff="$memorykernel_root/docs/implementation/PRODUCER_RELEASE_HANDOFF_LATEST.json"
service_main="$memorykernel_root/crates/memory-kernel-service/src/main.rs"
openapi="$memorykernel_root/openapi/openapi.yaml"

for path in "$source_of_truth" "$manifest" "$handoff" "$service_main" "$openapi"; do
  if [[ ! -f "$path" ]]; then
    echo "required file missing: $path" >&2
    exit 1
  fi
done

echo "[check] canonical service contract source alignment"
python3 - "$source_of_truth" "$manifest" "$handoff" "$service_main" "$openapi" <<'PY'
import json
import re
import sys
from pathlib import Path

source_path, manifest_path, handoff_path, service_main_path, openapi_path = sys.argv[1:6]

source = json.loads(Path(source_path).read_text(encoding="utf-8"))
manifest = json.loads(Path(manifest_path).read_text(encoding="utf-8"))
handoff = json.loads(Path(handoff_path).read_text(encoding="utf-8"))
service_main = Path(service_main_path).read_text(encoding="utf-8")
openapi = Path(openapi_path).read_text(encoding="utf-8")

required_source_keys = {
    "service_contract_source_version",
    "expected_service_contract_version",
    "expected_api_contract_version",
    "integration_baseline",
    "liveness_endpoint",
    "readiness_endpoint",
    "error_code_enum",
    "non_2xx_envelope_policy",
}
missing = sorted(required_source_keys - set(source.keys()))
if missing:
    raise SystemExit(f"source-of-truth missing keys: {missing}")

for key in ("expected_service_contract_version", "expected_api_contract_version", "integration_baseline"):
    if source[key] != manifest[key]:
        raise SystemExit(f"manifest mismatch for {key}: source={source[key]} manifest={manifest[key]}")
    if source[key] != handoff[key]:
        raise SystemExit(f"handoff mismatch for {key}: source={source[key]} handoff={handoff[key]}")

source_codes = set(source["error_code_enum"])
manifest_codes = set(manifest["error_code_enum"])
handoff_codes = set(handoff["error_code_enum"])
if source_codes != manifest_codes:
    raise SystemExit("error_code_enum drift between source-of-truth and manifest")
if source_codes != handoff_codes:
    raise SystemExit("error_code_enum drift between source-of-truth and handoff payload")

source_policy = source["non_2xx_envelope_policy"]
for key in ("requires", "optional", "forbids"):
    if key not in source_policy:
        raise SystemExit(f"source non_2xx_envelope_policy missing key: {key}")

handoff_policy = handoff["non_2xx_envelope_policy"]
for mode in ("service_v3_stable", "service_v3_candidate"):
    if mode not in handoff_policy:
        raise SystemExit(f"handoff missing non_2xx policy mode: {mode}")
    mode_policy = handoff_policy[mode]
    for key in ("requires", "optional", "forbids"):
        if set(mode_policy.get(key, [])) != set(source_policy[key]):
            raise SystemExit(f"handoff policy mismatch for {mode}.{key}")

service_version = source["expected_service_contract_version"]
api_version = source["expected_api_contract_version"]

runtime_const = f'const SERVICE_CONTRACT_VERSION: &str = "{service_version}";'
if runtime_const not in service_main:
    raise SystemExit(f"runtime service version constant mismatch: expected {runtime_const}")

if f"version: {service_version}" not in openapi:
    raise SystemExit("openapi info.version does not match source-of-truth")

for endpoint in (source["liveness_endpoint"], source["readiness_endpoint"]):
    if endpoint not in openapi:
        raise SystemExit(f"openapi missing required endpoint from source-of-truth: {endpoint}")

for code in sorted(source_codes):
    if re.search(rf"-\s+{re.escape(code)}\b", openapi) is None:
        raise SystemExit(f"openapi missing error code enum entry: {code}")

if f"const: {api_version}" not in openapi:
    raise SystemExit("openapi is missing api contract const aligned to source-of-truth")

print("Canonical service contract source-of-truth checks passed.")
PY
