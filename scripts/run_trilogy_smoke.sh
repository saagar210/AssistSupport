#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: run_trilogy_smoke.sh [options]

Runs a local trilogy smoke validation from the MemoryKernel workspace:
- contract parity check against sibling repos
- deterministic policy + recall retrieval checks
- Outcome host benchmark and gate preview checks
- optional MultiAgentCenter API-backed run check

Options:
  --memorykernel-root <path> Path to MemoryKernel repo root (default: script/..)
  --outcome-root <path>      Path to OutcomeMemory repo root (default: ../OutcomeMemory)
  --multi-agent-root <path>  Path to MultiAgentCenter repo root (default: ../MultiAgentCenter)
  --skip-contract-parity     Skip contract parity step
  --skip-multi-agent         Skip MultiAgentCenter run step
  --allow-missing-siblings   Allow missing sibling repos for parity check
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
outcome_root=""
multi_agent_root=""
skip_contract_parity=0
skip_multi_agent=0
allow_missing_siblings=0

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
    --skip-contract-parity)
      skip_contract_parity=1
      shift
      ;;
    --skip-multi-agent)
      skip_multi_agent=1
      shift
      ;;
    --allow-missing-siblings)
      allow_missing_siblings=1
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

run_mk_json() {
  local out_file="$1"
  shift
  cargo run --manifest-path "$memorykernel_root/Cargo.toml" -p memory-kernel-cli -- "$@" > "$out_file"
}

assert_normalized_json_equal() {
  local left_file="$1"
  local right_file="$2"
  local label="$3"

  python3 - "$left_file" "$right_file" "$label" <<'PY'
import json
import sys

left_path, right_path, label = sys.argv[1], sys.argv[2], sys.argv[3]

DYNAMIC_KEYS = {
    "context_package_id",
    "snapshot_id",
    "generated_at",
    "as_of",
    "created_at",
    "effective_at",
}


def normalize(node):
    if isinstance(node, dict):
        normalized = {}
        for key, value in node.items():
            if key in DYNAMIC_KEYS:
                normalized[key] = "<dynamic>"
            else:
                normalized[key] = normalize(value)
        return normalized
    if isinstance(node, list):
        return [normalize(item) for item in node]
    return node

with open(left_path, "r", encoding="utf-8") as f:
    left = normalize(json.load(f))
with open(right_path, "r", encoding="utf-8") as f:
    right = normalize(json.load(f))

if left != right:
    print(f"normalized mismatch for {label}", file=sys.stderr)
    sys.exit(1)
PY
}

extract_candidate_ref() {
  local decision_file="$1"
  python3 - "$decision_file" <<'PY'
import json
import sys

with open(sys.argv[1], "r", encoding="utf-8") as f:
    payload = json.load(f)
memory_id = payload["memory_id"]
version = payload["version"]
print(f"{memory_id}:{version}")
PY
}

sandbox=$(mktemp -d)
trap 'rm -rf "$sandbox"' EXIT

db_path="$sandbox/memorykernel.sqlite3"
db_clone_path="$sandbox/memorykernel.clone.sqlite3"
trace_db="$sandbox/multi-agent-center.trace.sqlite3"
as_of="2026-02-07T12:00:00Z"

if (( skip_contract_parity == 0 )); then
  echo "[STEP] Contract parity check"
  parity_args=("--canonical-root" "$memorykernel_root" "--outcome-root" "$outcome_root" "--multi-agent-root" "$multi_agent_root")
  if (( allow_missing_siblings == 1 )); then
    parity_args+=("--allow-missing")
  fi
  "$memorykernel_root/scripts/verify_contract_parity.sh" "${parity_args[@]}"
fi

echo "[STEP] Trilogy compatibility artifact check"
artifact_args=("--memorykernel-root" "$memorykernel_root" "--outcome-root" "$outcome_root" "--multi-agent-root" "$multi_agent_root")
if (( allow_missing_siblings == 1 )); then
  artifact_args+=("--allow-missing")
fi
"$memorykernel_root/scripts/verify_trilogy_compatibility_artifacts.sh" "${artifact_args[@]}"

echo "[STEP] MemoryKernel deterministic policy/recall smoke"
run_mk_json "$sandbox/migrate.json" --db "$db_path" db migrate
run_mk_json "$sandbox/constraint.json" --db "$db_path" memory add constraint \
  --actor user --action use --resource usb_drive --effect deny \
  --writer smoke --justification "phase7 smoke" --source-uri file:///phase7/policy.md \
  --truth-status asserted --authority authoritative --confidence 0.95
run_mk_json "$sandbox/decision.json" --db "$db_path" memory add decision \
  --summary "Decision: USB usage requires approval" \
  --writer smoke --justification "phase7 smoke" --source-uri file:///phase7/decision.md \
  --truth-status observed --authority derived --confidence 0.80

cp "$db_path" "$db_clone_path"

run_mk_json "$sandbox/ask_a.json" --db "$db_path" query ask \
  --text "Am I allowed to use a USB drive?" --actor user --action use --resource usb_drive --as-of "$as_of"
run_mk_json "$sandbox/ask_b.json" --db "$db_clone_path" query ask \
  --text "Am I allowed to use a USB drive?" --actor user --action use --resource usb_drive --as-of "$as_of"
assert_normalized_json_equal "$sandbox/ask_a.json" "$sandbox/ask_b.json" "query ask"

run_mk_json "$sandbox/recall_a.json" --db "$db_path" query recall --text "usb approval" --as-of "$as_of"
run_mk_json "$sandbox/recall_b.json" --db "$db_clone_path" query recall --text "usb approval" --as-of "$as_of"
assert_normalized_json_equal "$sandbox/recall_a.json" "$sandbox/recall_b.json" "query recall"

echo "[STEP] Outcome host checks"
run_mk_json "$sandbox/projector_status.json" --db "$db_path" outcome projector status --json
candidate_ref=$(extract_candidate_ref "$sandbox/decision.json")
run_mk_json "$sandbox/gate_preview.json" --db "$db_path" outcome gate preview --mode safe --as-of "$as_of" --candidate "$candidate_ref" --json
run_mk_json "$sandbox/benchmark.json" --db "$db_path" outcome benchmark run \
  --volume 100 --volume 500 --volume 2000 --repetitions 3 \
  --append-p95-max-ms 8 --replay-p95-max-ms 250 --gate-p95-max-ms 8 --json

if (( skip_multi_agent == 0 )); then
  if [[ ! -f "$multi_agent_root/Cargo.toml" ]]; then
    echo "[WARN] MultiAgentCenter workspace not found at $multi_agent_root; skipping."
  else
    echo "[STEP] MultiAgentCenter API-backed context source smoke"
    cargo run --manifest-path "$multi_agent_root/Cargo.toml" -p multi-agent-center-cli -- run \
      --workflow "$multi_agent_root/examples/workflow.memory.yaml" \
      --trace-db "$trace_db" \
      --memory-db "$db_path" \
      --non-interactive > "$sandbox/multi_agent_run.txt"
  fi
fi

echo "Trilogy smoke checks passed."
