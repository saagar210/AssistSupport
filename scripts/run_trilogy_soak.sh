#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: run_trilogy_soak.sh [options]

Execute repeated trilogy convergence checks from MemoryKernel workspace.

Options:
  --memorykernel-root <path> Path to MemoryKernel repo root (default: script/..)
  --outcome-root <path>      Path to OutcomeMemory repo root (default: ../OutcomeMemory)
  --multi-agent-root <path>  Path to MultiAgentCenter repo root (default: ../MultiAgentCenter)
  --iterations <n>           Number of iterations (default: 3)
  --sleep-seconds <n>        Sleep between iterations (default: 0)
  --full-gate                Run fmt/clippy/tests each iteration
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
iterations=3
sleep_seconds=0
full_gate=0

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
    --iterations)
      iterations="${2:-}"
      shift 2
      ;;
    --sleep-seconds)
      sleep_seconds="${2:-}"
      shift 2
      ;;
    --full-gate)
      full_gate=1
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

if ! [[ "$iterations" =~ ^[0-9]+$ ]] || (( iterations < 1 )); then
  echo "--iterations must be a positive integer" >&2
  exit 2
fi
if ! [[ "$sleep_seconds" =~ ^[0-9]+$ ]]; then
  echo "--sleep-seconds must be a non-negative integer" >&2
  exit 2
fi

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

run_benchmark_gate() {
  cargo run --manifest-path "$memorykernel_root/Cargo.toml" -p memory-kernel-cli -- outcome benchmark run \
    --volume 100 --volume 500 --volume 2000 --repetitions 3 \
    --append-p95-max-ms 8 --replay-p95-max-ms 250 --gate-p95-max-ms 8 --json >/dev/null
}

echo "Starting trilogy soak: iterations=$iterations full_gate=$full_gate sleep_seconds=$sleep_seconds"
for ((i=1; i<=iterations; i++)); do
  echo "[ITERATION $i/$iterations] contract parity"
  "$memorykernel_root/scripts/verify_contract_parity.sh" \
    --canonical-root "$memorykernel_root" \
    --outcome-root "$outcome_root" \
    --multi-agent-root "$multi_agent_root"

  echo "[ITERATION $i/$iterations] compatibility artifacts"
  "$memorykernel_root/scripts/verify_trilogy_compatibility_artifacts.sh" \
    --memorykernel-root "$memorykernel_root" \
    --outcome-root "$outcome_root" \
    --multi-agent-root "$multi_agent_root"

  echo "[ITERATION $i/$iterations] trilogy smoke"
  "$memorykernel_root/scripts/run_trilogy_smoke.sh" \
    --memorykernel-root "$memorykernel_root" \
    --outcome-root "$outcome_root" \
    --multi-agent-root "$multi_agent_root"

  echo "[ITERATION $i/$iterations] benchmark gate"
  run_benchmark_gate

  if (( full_gate == 1 )); then
    echo "[ITERATION $i/$iterations] full rust gate"
    cargo fmt --manifest-path "$memorykernel_root/Cargo.toml" --all -- --check
    cargo clippy --manifest-path "$memorykernel_root/Cargo.toml" --workspace --all-targets --all-features -- -D warnings
    cargo test --manifest-path "$memorykernel_root/Cargo.toml" --workspace --all-targets --all-features
  fi

  if (( i < iterations && sleep_seconds > 0 )); then
    sleep "$sleep_seconds"
  fi
done

echo "Trilogy soak completed successfully ($iterations iteration(s))."
