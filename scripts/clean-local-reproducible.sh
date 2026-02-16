#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

"${ROOT_DIR}/scripts/clean-heavy-artifacts.sh"

# Reproducible local caches and reports (dependencies can be reinstalled).
paths=(
  "node_modules"
  "coverage"
  "test-results"
  "playwright-report"
  "artifacts"
  ".cache"
  "tmp"
  "search-api/venv"
  "search-api/.venv"
  "search-api/__pycache__"
  "search-api/.pytest_cache"
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
  echo "no additional reproducible caches found"
fi
