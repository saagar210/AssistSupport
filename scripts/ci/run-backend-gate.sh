#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -lt 3 ]; then
  echo "Usage: $0 <gate-name> <log-path> <command>"
  exit 2
fi

gate_name="$1"
log_path="$2"
command="$3"
summary_file="${4:-artifacts/backend/gates-summary.md}"

summary_dir="$(dirname "$summary_file")"
mkdir -p "$summary_dir"
mkdir -p "$(dirname "$log_path")"

if [ ! -f "$summary_file" ]; then
  printf '| Gate | Status | Duration (s) |\n|---|---|---:|\n' > "$summary_file"
fi

start_ts=$(date +%s)
if bash -lc "$command" 2>&1 | tee "$log_path"; then
  status="PASS"
else
  status="FAIL"
fi
duration=$(( $(date +%s) - start_ts ))

printf '| %s | %s | %s |\n' "$gate_name" "$status" "$duration" >> "$summary_file"
[ "$status" = "PASS" ]
