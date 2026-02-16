#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LEAN_TMP_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/assistsupport-lean-dev.XXXXXX")"

cleanup() {
  local exit_code="$1"

  # Clean heavy repo-local artifacts on exit to keep workspace size low.
  "${ROOT_DIR}/scripts/clean-heavy-artifacts.sh" >/dev/null 2>&1 || true

  if [[ -d "${LEAN_TMP_ROOT}" ]]; then
    rm -rf -- "${LEAN_TMP_ROOT}"
  fi

  return "${exit_code}"
}

on_exit() {
  local exit_code=$?
  trap - EXIT INT TERM
  cleanup "${exit_code}"
}

on_interrupt() {
  trap - EXIT INT TERM
  cleanup 130
  exit 130
}

trap on_exit EXIT
trap on_interrupt INT TERM

export VITE_CACHE_DIR="${LEAN_TMP_ROOT}/vite-cache"
export CARGO_TARGET_DIR="${LEAN_TMP_ROOT}/cargo-target"

echo "lean dev temp root: ${LEAN_TMP_ROOT}"
echo "using VITE_CACHE_DIR=${VITE_CACHE_DIR}"
echo "using CARGO_TARGET_DIR=${CARGO_TARGET_DIR}"

tauri_args=("$@")
if [[ ${#tauri_args[@]} -gt 0 && "${tauri_args[0]}" == "--" ]]; then
  tauri_args=("${tauri_args[@]:1}")
fi

cd "${ROOT_DIR}"
pnpm tauri dev "${tauri_args[@]}"
