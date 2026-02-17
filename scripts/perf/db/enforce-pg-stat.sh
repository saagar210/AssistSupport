#!/usr/bin/env bash
set -euo pipefail

if [[ -z "${DATABASE_URL:-}" ]]; then
  echo "DATABASE_URL is required."
  exit 1
fi

min_calls="${DB_MIN_CALLS:-50}"
max_mean_ms="${DB_MAX_MEAN_MS:-100}"

psql "$DATABASE_URL" -q -c "CREATE EXTENSION IF NOT EXISTS pg_stat_statements;" >/dev/null

count="$(psql "$DATABASE_URL" -At -c "SELECT COUNT(*) FROM (
  SELECT queryid
  FROM pg_stat_statements
  WHERE calls >= ${min_calls}
    AND mean_exec_time > ${max_mean_ms}
) offenders;")"

count="${count//$'\n'/}"
count="${count// /}"

if [[ -z "$count" ]]; then
  echo "Could not compute offender count from pg_stat_statements."
  exit 1
fi

if ((count > 0)); then
  echo "DB performance gate failed: ${count} offender queries (calls>=${min_calls}, mean_exec_time>${max_mean_ms}ms)."
  exit 1
fi

echo "DB performance gate passed: no offenders."
