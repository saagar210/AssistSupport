#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: test_producer_governance_negative.sh [options]

Run negative-fixture tests proving producer governance scripts fail under drift.

Options:
  --memorykernel-root <path> Path to MemoryKernel root (default: script/..)
  -h, --help                 Show help
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

require_file() {
  if [[ ! -f "$1" ]]; then
    echo "required file missing: $1" >&2
    exit 1
  fi
}

run_expect_pass() {
  local name="$1"
  shift
  if ! "$@" >/tmp/mk-neg-pass.log 2>&1; then
    echo "expected PASS for $name, but failed:" >&2
    cat /tmp/mk-neg-pass.log >&2
    exit 1
  fi
}

run_expect_fail() {
  local name="$1"
  local needle="$2"
  shift 2
  if "$@" >/tmp/mk-neg-fail.log 2>&1; then
    echo "expected FAIL for $name, but command passed" >&2
    exit 1
  fi
  if [[ -n "$needle" ]] && ! rg -n --quiet -- "$needle" /tmp/mk-neg-fail.log; then
    echo "expected failure output for $name to include: $needle" >&2
    cat /tmp/mk-neg-fail.log >&2
    exit 1
  fi
}

create_fixture() {
  local fixture_root="$1"
  mkdir -p "$fixture_root"

  local files=(
    "CHANGELOG.md"
    "contracts/integration/v1/producer-contract-manifest.json"
    "crates/memory-kernel-service/src/main.rs"
    "openapi/openapi.yaml"
    "docs/spec/service-contract.md"
    "docs/spec/versioning.md"
    "docs/implementation/ACTIVE_RUNTIME_STATUS_PRODUCER_2026-02-08.md"
    "docs/implementation/NEXT_EXECUTION_QUEUE_PRODUCER.md"
    "docs/implementation/PRODUCER_RELEASE_HANDOFF_LATEST.json"
    "docs/implementation/SERVICE_V3_CUTOVER_GATES.md"
    "docs/implementation/SERVICE_V3_RFC_DRAFT.md"
    "docs/implementation/RUNTIME_CUTOVER_DECISION_RECORD_PRODUCER_2026-02-08.md"
    "scripts/generate_producer_handoff_payload.sh"
  )

  local rel src dst
  for rel in "${files[@]}"; do
    src="$memorykernel_root/$rel"
    dst="$fixture_root/$rel"
    require_file "$src"
    mkdir -p "$(dirname "$dst")"
    cp "$src" "$dst"
  done

  chmod +x "$fixture_root/scripts/generate_producer_handoff_payload.sh"
}

mutate_json_field() {
  local file="$1"
  local key="$2"
  local value="$3"
  python3 - "$file" "$key" "$value" <<'PY'
import json
import sys

path, key, value = sys.argv[1:4]
with open(path, 'r', encoding='utf-8') as f:
    data = json.load(f)

data[key] = value

with open(path, 'w', encoding='utf-8') as f:
    json.dump(data, f, indent=2)
    f.write('\n')
PY
}

replace_or_fail() {
  local file="$1"
  local from="$2"
  local to="$3"
  if ! rg -n --quiet -- "$from" "$file"; then
    echo "replace_or_fail: expected pattern not found in $file: $from" >&2
    exit 1
  fi
  perl -0pi -e "s/\Q$from\E/$to/" "$file"
}

tmp_dir=$(mktemp -d)
trap 'rm -rf "$tmp_dir"' EXIT

# Baseline fixture must pass both checks.
baseline="$tmp_dir/baseline"
create_fixture "$baseline"
run_expect_pass "baseline-manifest" \
  "$memorykernel_root/scripts/verify_producer_contract_manifest.sh" --memorykernel-root "$baseline"
run_expect_pass "baseline-handoff" \
  "$memorykernel_root/scripts/verify_producer_handoff_payload.sh" --memorykernel-root "$baseline"

# Manifest drift: wrong service contract version.
manifest_drift="$tmp_dir/manifest-drift"
create_fixture "$manifest_drift"
mutate_json_field \
  "$manifest_drift/contracts/integration/v1/producer-contract-manifest.json" \
  "expected_service_contract_version" \
  "service.v2"
run_expect_fail "manifest-drift" "expected_service_contract_version must be service.v3" \
  "$memorykernel_root/scripts/verify_producer_contract_manifest.sh" --memorykernel-root "$manifest_drift"

# Versioning doc drift: service contract version mismatch.
status_drift="$tmp_dir/status-drift"
create_fixture "$status_drift"
replace_or_fail \
  "$status_drift/docs/spec/versioning.md" \
  'Service contract: `service.v3`' \
  'Service contract: `service.v2`'
run_expect_fail "versioning-drift" "required pattern not found" \
  "$memorykernel_root/scripts/verify_producer_contract_manifest.sh" --memorykernel-root "$status_drift"

# Handoff drift: published payload expected service contract mismatch.
handoff_drift="$tmp_dir/handoff-drift"
create_fixture "$handoff_drift"
mutate_json_field \
  "$handoff_drift/docs/implementation/PRODUCER_RELEASE_HANDOFF_LATEST.json" \
  "expected_service_contract_version" \
  "service.v2"
run_expect_fail "handoff-drift" "expected_service_contract_version expected" \
  "$memorykernel_root/scripts/verify_producer_handoff_payload.sh" --memorykernel-root "$handoff_drift"

echo "Producer governance negative-fixture tests passed."
