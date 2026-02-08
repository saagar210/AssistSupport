#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: verify_contract_parity.sh [options]

Compare MemoryKernel integration contract pack against sibling repos.

Options:
  --canonical-root <path>   Path to MemoryKernel repo root (default: script/..)
  --outcome-root <path>     Path to OutcomeMemory repo root
  --multi-agent-root <path> Path to MultiAgentCenter repo root
  --allow-missing           Skip repos that are not present (default: strict)
  --strict-missing          Fail if sibling repo path is missing (default)
  -h, --help                Show this help
USAGE
}

canonical_root=""
outcome_root=""
multi_agent_root=""
strict_missing=1

while (($# > 0)); do
  case "$1" in
    --canonical-root)
      canonical_root="${2:-}"
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

resolve_path() {
  local path="$1"
  if [[ -d "$path" ]]; then
    (cd "$path" && pwd -P)
  elif [[ -f "$path" ]]; then
    local dir
    dir=$(cd "$(dirname "$path")" && pwd -P)
    printf "%s/%s\n" "$dir" "$(basename "$path")"
  else
    printf "%s\n" "$path"
  fi
}

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)
if [[ -z "$canonical_root" ]]; then
  canonical_root=$(resolve_path "$script_dir/..")
else
  canonical_root=$(resolve_path "$canonical_root")
fi

if [[ -z "$outcome_root" ]]; then
  outcome_root=$(resolve_path "$canonical_root/../OutcomeMemory")
else
  outcome_root=$(resolve_path "$outcome_root")
fi

if [[ -z "$multi_agent_root" ]]; then
  multi_agent_root=$(resolve_path "$canonical_root/../MultiAgentCenter")
else
  multi_agent_root=$(resolve_path "$multi_agent_root")
fi

canonical_contract_root="$canonical_root/contracts/integration/v1"
if [[ ! -d "$canonical_contract_root" ]]; then
  echo "Canonical contract root missing: $canonical_contract_root" >&2
  exit 1
fi

hash_file() {
  local file="$1"
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$file" | awk '{print $1}'
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$file" | awk '{print $1}'
  else
    echo "Missing checksum tool (sha256sum or shasum)" >&2
    exit 1
  fi
}

list_relative_files() {
  local root="$1"
  find "$root" -type f -name '*.json' | LC_ALL=C sort | sed "s#^$root/##"
}

failures=0

compare_repo() {
  local repo_label="$1"
  local repo_root="$2"
  local target_contract_root="$repo_root/contracts/integration/v1"

  if [[ ! -d "$target_contract_root" ]]; then
    if (( strict_missing == 1 )); then
      echo "[FAIL] $repo_label contract root missing: $target_contract_root" >&2
      failures=$((failures + 1))
    else
      echo "[SKIP] $repo_label contract root missing: $target_contract_root"
    fi
    return
  fi

  echo "[INFO] Comparing canonical contracts to $repo_label"
  echo "       canonical: $canonical_contract_root"
  echo "       target:    $target_contract_root"

  local canonical_files
  local target_files
  canonical_files=$(mktemp)
  target_files=$(mktemp)

  list_relative_files "$canonical_contract_root" > "$canonical_files"
  list_relative_files "$target_contract_root" > "$target_files"

  if ! diff -u "$canonical_files" "$target_files" >/dev/null; then
    echo "[FAIL] File-set mismatch for $repo_label" >&2
    diff -u "$canonical_files" "$target_files" || true
    failures=$((failures + 1))
  fi

  while IFS= read -r rel_path; do
    [[ -z "$rel_path" ]] && continue
    local canonical_file="$canonical_contract_root/$rel_path"
    local target_file="$target_contract_root/$rel_path"

    if [[ ! -f "$target_file" ]]; then
      continue
    fi

    local canonical_hash
    local target_hash
    canonical_hash=$(hash_file "$canonical_file")
    target_hash=$(hash_file "$target_file")

    if [[ "$canonical_hash" != "$target_hash" ]]; then
      echo "[FAIL] Content mismatch for $repo_label: $rel_path" >&2
      echo "       canonical=$canonical_hash" >&2
      echo "       target=$target_hash" >&2
      failures=$((failures + 1))
    fi
  done < "$canonical_files"

  rm -f "$canonical_files" "$target_files"
}

compare_repo "OutcomeMemory" "$outcome_root"
compare_repo "MultiAgentCenter" "$multi_agent_root"

if (( failures > 0 )); then
  echo "Contract parity check failed with $failures issue(s)." >&2
  exit 1
fi

echo "Contract parity check passed."
