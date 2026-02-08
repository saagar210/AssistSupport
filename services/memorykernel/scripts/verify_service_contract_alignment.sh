#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: verify_service_contract_alignment.sh [options]

Verify MemoryKernel service contract alignment across runtime code, OpenAPI, and docs.

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

service_main="$memorykernel_root/crates/memory-kernel-service/src/main.rs"
openapi="$memorykernel_root/openapi/openapi.yaml"
service_contract_doc="$memorykernel_root/docs/spec/service-contract.md"
versioning_doc="$memorykernel_root/docs/spec/versioning.md"
schema_dir="$memorykernel_root/contracts/integration/v1/schemas"

for path in "$service_main" "$openapi" "$service_contract_doc" "$versioning_doc"; do
  if [[ ! -f "$path" ]]; then
    echo "required file missing: $path" >&2
    exit 1
  fi
done

echo "[check] service runtime version"
require_grep 'const SERVICE_CONTRACT_VERSION: &str = "service.v3";' "$service_main"

echo "[check] openapi version"
require_grep '^  version: service.v3$' "$openapi"

echo "[check] documented service version"
require_grep 'Service contract version: `service.v3`' "$service_contract_doc"
require_grep 'Non-2xx responses intentionally do \*\*not\*\* include `api_contract_version`\.' "$service_contract_doc"
require_grep '`legacy_error` is removed in `service.v3`' "$service_contract_doc"
require_grep 'Service contract: `service.v3`' "$versioning_doc"

echo "[check] openapi includes structured error envelope"
require_grep 'ServiceErrorEnvelope' "$openapi"

echo "[check] openapi includes non-2xx responses on query endpoints"
require_grep '/v1/query/ask:' "$openapi"
require_grep '/v1/query/recall:' "$openapi"
require_grep '"400":' "$openapi"
require_grep '"500":' "$openapi"
require_grep '"503":' "$openapi"

echo "[check] error code taxonomy present"
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
done

echo "[check] ServiceErrorEnvelope policy invariants"
error_block_start=$(rg -n '^    ServiceErrorEnvelope:' "$openapi" | cut -d: -f1 | head -n1 || true)
if [[ -z "$error_block_start" ]]; then
  echo "ServiceErrorEnvelope schema block missing in $openapi" >&2
  exit 1
fi

error_block=$(tail -n +"$error_block_start" "$openapi")
if printf "%s\n" "$error_block" | rg -n --quiet -- '^\s*api_contract_version:'; then
  echo "ServiceErrorEnvelope must not define api_contract_version in $openapi" >&2
  exit 1
fi

if printf "%s\n" "$error_block" | rg -n --quiet -- '^\s*-\s+legacy_error$'; then
  echo "ServiceErrorEnvelope must not require legacy_error in $openapi" >&2
  exit 1
fi

if printf "%s\n" "$error_block" | rg -n --quiet -- '^\s*legacy_error:'; then
  echo "ServiceErrorEnvelope must not define legacy_error in $openapi" >&2
  exit 1
fi

echo "[check] integration schemas use \$id metadata"
if rg -n --quiet -- '"":' "$schema_dir"; then
  echo "invalid empty-key schema metadata detected in $schema_dir" >&2
  rg -n -- '"":' "$schema_dir" >&2 || true
  exit 1
fi

for schema in "$schema_dir"/*.json; do
  require_grep '"\$id":' "$schema"
done

echo "Service contract alignment checks passed."
