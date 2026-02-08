#!/usr/bin/env bash
set -euo pipefail

resolve_path() {
  local path="$1"
  if [[ -d "$path" ]]; then
    (cd "$path" && pwd -P)
  else
    printf "%s\n" "$path"
  fi
}

parse_memorykernel_root() {
  local provided_root="${1:-}"
  local script_dir
  script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)
  if [[ -n "$provided_root" ]]; then
    resolve_path "$provided_root"
  else
    resolve_path "$script_dir/../.."
  fi
}

require_file() {
  local path="$1"
  if [[ ! -f "$path" ]]; then
    echo "missing required file: $path" >&2
    exit 1
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

require_json_expr() {
  local file="$1"
  local expr="$2"
  if ! jq -e "$expr" "$file" >/dev/null; then
    echo "json expression failed: $expr in $file" >&2
    exit 1
  fi
}

run_step() {
  local label="$1"
  shift
  echo "[check] $label"
  "$@"
}
