#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# Heavy build outputs that are safe to regenerate.
paths=(
  "dist"
  "dist-ssr"
  "out"
  "build"
  "src-tauri/target"
  "node_modules/.vite"
)

removed_any=0
for rel_path in "${paths[@]}"; do
  abs_path="${ROOT_DIR}/${rel_path}"
  if [[ -e "${abs_path}" ]]; then
    rm -rf -- "${abs_path}"
    echo "removed ${rel_path}"
    removed_any=1
  fi
done

if [[ "${removed_any}" -eq 0 ]]; then
  echo "no heavy build artifacts found"
fi
