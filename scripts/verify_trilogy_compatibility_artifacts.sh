#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: verify_trilogy_compatibility_artifacts.sh [options]

Validate trilogy compatibility artifacts produced by sibling repositories.

Options:
  --memorykernel-root <path> Path to MemoryKernel repo root (default: script/..)
  --outcome-root <path>      Path to OutcomeMemory repo root
  --multi-agent-root <path>  Path to MultiAgentCenter repo root
  --allow-missing            Skip missing sibling repos (default: strict)
  --strict-missing           Fail when sibling repo path is missing (default)
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
outcome_root=""
multi_agent_root=""
strict_missing=1

while (($# > 0)); do
  case "$1" in
    --memorykernel-root)
      memorykernel_root="${2:-}"
      shift 2
      ;;
    --outcome-root)
      outcome_root="${2:-}"
      shift 2
      ;;
    --multi-agent-root)
      multi_agent_root="${2:-}"
      shift 2
      ;;
    --allow-missing)
      strict_missing=0
      shift
      ;;
    --strict-missing)
      strict_missing=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
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

if [[ -z "$outcome_root" ]]; then
  outcome_root=$(resolve_path "$memorykernel_root/../OutcomeMemory")
else
  outcome_root=$(resolve_path "$outcome_root")
fi

if [[ -z "$multi_agent_root" ]]; then
  multi_agent_root=$(resolve_path "$memorykernel_root/../MultiAgentCenter")
else
  multi_agent_root=$(resolve_path "$multi_agent_root")
fi

validate_outcome_artifact() {
  local artifact_path="$1"
  python3 - "$artifact_path" <<'PY'
import json
import sys

path = sys.argv[1]
with open(path, "r", encoding="utf-8") as f:
    doc = json.load(f)

errors = []

version = str(doc.get("artifact_version", ""))
if "v1" not in version:
    errors.append("artifact_version must indicate v1 compatibility")

project = doc.get("project", {})
if project.get("name") != "OutcomeMemory":
    errors.append("project.name must be OutcomeMemory")

if doc.get("supported_memorykernel_contract_baseline") != "integration/v1":
    errors.append("supported_memorykernel_contract_baseline must be integration/v1")

api = doc.get("required_stable_embed_api")
required_api = {"run_cli", "run_outcome_with_db", "run_outcome", "run_benchmark"}
if not isinstance(api, list):
    errors.append("required_stable_embed_api must be a list")
else:
    api_set = set(api)
    missing = sorted(required_api - api_set)
    if missing:
        errors.append(f"required_stable_embed_api missing: {', '.join(missing)}")

semantics = doc.get("benchmark_threshold_semantics", {})
if semantics.get("threshold_triplet_required") is not True:
    errors.append("benchmark_threshold_semantics.threshold_triplet_required must be true")
if semantics.get("non_zero_exit_on_any_violation") is not True:
    errors.append("benchmark_threshold_semantics.non_zero_exit_on_any_violation must be true")
required_flags = semantics.get("required_flags")
if not isinstance(required_flags, list):
    errors.append("benchmark_threshold_semantics.required_flags must be a list")
else:
    flag_set = set(required_flags)
    expected = {"--append-p95-max-ms", "--replay-p95-max-ms", "--gate-p95-max-ms"}
    missing = sorted(expected - flag_set)
    if missing:
        errors.append(f"benchmark_threshold_semantics.required_flags missing: {', '.join(missing)}")

if errors:
    print("OutcomeMemory trilogy compatibility artifact validation failed:", file=sys.stderr)
    for err in errors:
        print(f"- {err}", file=sys.stderr)
    sys.exit(1)
PY
}

validate_multi_agent_artifact() {
  local artifact_path="$1"
  python3 - "$artifact_path" <<'PY'
import json
import sys

path = sys.argv[1]
with open(path, "r", encoding="utf-8") as f:
    doc = json.load(f)

errors = []

version = str(doc.get("artifact_version", ""))
if "v1" not in version:
    errors.append("artifact_version must indicate v1 compatibility")

project = doc.get("project", {})
if project.get("name") != "MultiAgentCenter":
    errors.append("project.name must be MultiAgentCenter")

mk = doc.get("memory_kernel", {})
if mk.get("contract_baseline") != "integration/v1":
    errors.append("memory_kernel.contract_baseline must be integration/v1")

api = mk.get("api_retrieval", {})
mode = api.get("context_queries_mode", {})
supported = mode.get("supported")
if not isinstance(supported, list):
    errors.append("memory_kernel.api_retrieval.context_queries_mode.supported must be a list")
else:
    supported_set = set(supported)
    needed = {"policy", "recall"}
    missing = sorted(needed - supported_set)
    if missing:
        errors.append(f"context_queries_mode.supported missing: {', '.join(missing)}")

if mode.get("default") != "policy":
    errors.append("context_queries_mode.default must be policy")

recall = api.get("recall", {})
default_types = recall.get("default_record_types")
if not isinstance(default_types, list):
    errors.append("recall.default_record_types must be a list")
else:
    needed = ["decision", "preference", "event", "outcome"]
    missing = [name for name in needed if name not in default_types]
    if missing:
        errors.append(f"recall.default_record_types missing: {', '.join(missing)}")

if recall.get("missing_record_types_behavior") != "use_default_scope":
    errors.append("recall.missing_record_types_behavior must be use_default_scope")
if recall.get("empty_record_types_behavior") != "use_default_scope":
    errors.append("recall.empty_record_types_behavior must be use_default_scope")
if recall.get("invalid_record_types_behavior") != "validation_error":
    errors.append("recall.invalid_record_types_behavior must be validation_error")

trust = doc.get("trust", {})
identity = trust.get("memory_ref_identity_required_fields")
if not isinstance(identity, list):
    errors.append("trust.memory_ref_identity_required_fields must be a list")
else:
    needed = ["memory_id", "version", "memory_version_id"]
    missing = [name for name in needed if name not in identity]
    if missing:
        errors.append(
            "trust.memory_ref_identity_required_fields missing: " + ", ".join(missing)
        )

if errors:
    print("MultiAgentCenter trilogy compatibility artifact validation failed:", file=sys.stderr)
    for err in errors:
        print(f"- {err}", file=sys.stderr)
    sys.exit(1)
PY
}

failures=0

check_artifact() {
  local label="$1"
  local repo_root="$2"
  local validator="$3"

  if [[ ! -d "$repo_root" ]]; then
    if (( strict_missing == 1 )); then
      echo "[FAIL] $label repo root missing: $repo_root" >&2
      failures=$((failures + 1))
    else
      echo "[SKIP] $label repo root missing: $repo_root"
    fi
    return
  fi

  local artifact_path="$repo_root/trilogy-compatibility.v1.json"
  if [[ ! -f "$artifact_path" ]]; then
    echo "[FAIL] $label artifact missing: $artifact_path" >&2
    failures=$((failures + 1))
    return
  fi

  echo "[INFO] Validating $label artifact: $artifact_path"
  if ! "$validator" "$artifact_path"; then
    failures=$((failures + 1))
  fi
}

check_artifact "OutcomeMemory" "$outcome_root" validate_outcome_artifact
check_artifact "MultiAgentCenter" "$multi_agent_root" validate_multi_agent_artifact

if (( failures > 0 )); then
  echo "Trilogy compatibility artifact validation failed with $failures issue(s)." >&2
  exit 1
fi

echo "Trilogy compatibility artifacts validated successfully."
