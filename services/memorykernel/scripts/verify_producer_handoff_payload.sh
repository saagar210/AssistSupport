#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: verify_producer_handoff_payload.sh [options]

Validate producer handoff payload correctness against canonical manifest and policy docs.

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

manifest="$memorykernel_root/contracts/integration/v1/producer-contract-manifest.json"
published="$memorykernel_root/docs/implementation/PRODUCER_RELEASE_HANDOFF_LATEST.json"
service_contract_doc="$memorykernel_root/docs/spec/service-contract.md"
cutover_gates_doc="$memorykernel_root/docs/implementation/SERVICE_V3_CUTOVER_GATES.md"
rfc_doc="$memorykernel_root/docs/implementation/SERVICE_V3_RFC_DRAFT.md"
decision_record_doc="$memorykernel_root/docs/implementation/RUNTIME_CUTOVER_DECISION_RECORD_PRODUCER_2026-02-08.md"
generator="$memorykernel_root/scripts/generate_producer_handoff_payload.sh"

for path in "$manifest" "$published" "$service_contract_doc" "$cutover_gates_doc" "$rfc_doc" "$decision_record_doc" "$generator"; do
  if [[ ! -f "$path" ]]; then
    echo "required file missing: $path" >&2
    exit 1
  fi
done

tmp_dir=$(mktemp -d)
trap 'rm -rf "$tmp_dir"' EXIT

stable_payload="$tmp_dir/handoff-stable.json"
candidate_payload="$tmp_dir/handoff-service-v3-candidate.json"

echo "[check] generate stable handoff payload"
"$generator" \
  --mode stable \
  --memorykernel-root "$memorykernel_root" \
  --out-json "$stable_payload" >/dev/null

echo "[check] generate service-v3 candidate handoff payload"
"$generator" \
  --mode service-v3-candidate \
  --memorykernel-root "$memorykernel_root" \
  --out-json "$candidate_payload" >/dev/null

echo "[check] payload schema and policy invariants"
python3 - "$manifest" "$stable_payload" "$candidate_payload" "$published" <<'PY'
import json
import sys

manifest_path, stable_path, candidate_path, published_path = sys.argv[1:5]

with open(manifest_path, "r", encoding="utf-8") as f:
    manifest = json.load(f)
with open(stable_path, "r", encoding="utf-8") as f:
    stable = json.load(f)
with open(candidate_path, "r", encoding="utf-8") as f:
    candidate = json.load(f)
with open(published_path, "r", encoding="utf-8") as f:
    published = json.load(f)

required_common = [
    "handoff_generated_at_utc",
    "handoff_mode",
    "release_tag",
    "commit_sha",
    "expected_service_contract_version",
    "expected_api_contract_version",
    "integration_baseline",
    "active_runtime_baseline",
    "error_code_enum",
    "non_2xx_envelope_policy",
    "manifest_contract_version",
    "manifest_path",
    "changelog_path",
    "verification_commands",
    "consumer_impact_statement_template",
]

def assert_keys(payload, keys, name):
    for key in keys:
        if key not in payload:
            raise SystemExit(f"{name}: missing required key: {key}")

def assert_baseline_alignment(payload, name):
    for key in ("release_tag", "commit_sha", "expected_api_contract_version", "integration_baseline"):
        if payload[key] != manifest[key]:
            raise SystemExit(f"{name}: {key} does not align with manifest")
    baseline = payload["active_runtime_baseline"]
    for key in ("release_tag", "commit_sha", "expected_service_contract_version", "expected_api_contract_version", "integration_baseline"):
        if key not in baseline:
            raise SystemExit(f"{name}: active_runtime_baseline missing key: {key}")
    if baseline["release_tag"] != manifest["release_tag"]:
        raise SystemExit(f"{name}: active_runtime_baseline.release_tag drift")
    if baseline["commit_sha"] != manifest["commit_sha"]:
        raise SystemExit(f"{name}: active_runtime_baseline.commit_sha drift")
    if baseline["expected_service_contract_version"] != manifest["expected_service_contract_version"]:
        raise SystemExit(f"{name}: active_runtime_baseline.expected_service_contract_version drift")
    if baseline["expected_api_contract_version"] != manifest["expected_api_contract_version"]:
        raise SystemExit(f"{name}: active_runtime_baseline.expected_api_contract_version drift")
    if baseline["integration_baseline"] != manifest["integration_baseline"]:
        raise SystemExit(f"{name}: active_runtime_baseline.integration_baseline drift")

def assert_policy(payload, name):
    policy = payload["non_2xx_envelope_policy"]
    for key in ("service_v3_stable", "service_v3_candidate"):
        if key not in policy:
            raise SystemExit(f"{name}: non_2xx_envelope_policy missing key: {key}")

    for key in ("service_v3_stable", "service_v3_candidate"):
        policy_entry = policy[key]
        if sorted(policy_entry.get("requires", [])) != sorted(
            ["service_contract_version", "error.code", "error.message"]
        ):
            raise SystemExit(f"{name}: invalid {key}.requires")
        if sorted(policy_entry.get("optional", [])) != ["error.details"]:
            raise SystemExit(f"{name}: invalid {key}.optional")
        if sorted(policy_entry.get("forbids", [])) != sorted(["legacy_error", "api_contract_version"]):
            raise SystemExit(f"{name}: invalid {key}.forbids")

def assert_common(payload, name):
    assert_keys(payload, required_common, name)
    if payload["manifest_contract_version"] != manifest["manifest_contract_version"]:
        raise SystemExit(f"{name}: manifest_contract_version drift")
    if payload["error_code_enum"] != manifest["error_code_enum"]:
        raise SystemExit(f"{name}: error_code_enum drift")
    assert_baseline_alignment(payload, name)
    assert_policy(payload, name)

def assert_mode(payload, expected_mode, expected_service, name):
    if payload["handoff_mode"] != expected_mode:
        raise SystemExit(f"{name}: handoff_mode expected {expected_mode}")
    if payload["expected_service_contract_version"] != expected_service:
        raise SystemExit(f"{name}: expected_service_contract_version expected {expected_service}")

assert_common(stable, "stable")
assert_mode(stable, "stable", manifest["expected_service_contract_version"], "stable")
if "rehearsal_candidate" in stable:
    raise SystemExit("stable payload must not include rehearsal_candidate")
if "required_consumer_validation_commands" in stable:
    raise SystemExit("stable payload must not include candidate-only required_consumer_validation_commands")

assert_common(candidate, "candidate")
assert_mode(candidate, "service-v3-candidate", manifest["expected_service_contract_version"], "candidate")
for key in ("rehearsal_candidate", "compatibility_expectations", "required_consumer_validation_commands"):
    if key not in candidate:
        raise SystemExit(f"candidate payload missing {key}")
if candidate["rehearsal_candidate"].get("requires_runtime_cutover") is not False:
    raise SystemExit("candidate payload requires_runtime_cutover must be false")

assert_common(published, "published")
published_mode = published["handoff_mode"]
if published_mode == "stable":
    assert_mode(published, "stable", manifest["expected_service_contract_version"], "published")
elif published_mode == "service-v3-candidate":
    assert_mode(published, "service-v3-candidate", "service.v3", "published")
else:
    raise SystemExit(f"published: unsupported handoff_mode {published_mode}")
PY

echo "[check] policy text is documented in normative docs"
rg -n --quiet -- 'Non-2xx responses intentionally do \*\*not\*\* include `api_contract_version`\.' "$service_contract_doc"
rg -n --quiet -- '`legacy_error` is removed in `service.v3`' "$service_contract_doc"
rg -n --quiet -- '^## Explicit Non-2xx Envelope Policy$' "$cutover_gates_doc"
rg -n --quiet -- '^Compatibility expectation \(candidate payload vs pinned runtime baseline\):$' "$rfc_doc"

expected_service_contract_version=$(python3 - "$manifest" <<'PY'
import json
import sys

with open(sys.argv[1], "r", encoding="utf-8") as f:
    manifest = json.load(f)

print(manifest["expected_service_contract_version"])
PY
)

if [[ "$expected_service_contract_version" == "service.v3" ]]; then
  rg -n --quiet -- '^2\. Runtime cutover execution: \*\*GO\*\*$' "$decision_record_doc"
  rg -n --quiet -- '^- Phase 8: \*\*Complete\*\*$' "$decision_record_doc"
else
  rg -n --quiet -- '^2\. Runtime cutover execution: \*\*NO-GO\*\*$' "$decision_record_doc"
  rg -n --quiet -- '^- Phase 7: \*\*Closed\*\* \(decision recorded\)$' "$decision_record_doc"
fi

echo "Producer handoff payload checks passed."
