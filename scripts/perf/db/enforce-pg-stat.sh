#!/usr/bin/env bash
set -euo pipefail

min_calls="${DB_MIN_CALLS:-50}"
max_mean_ms="${DB_MAX_MEAN_MS:-100}"

if [[ -x "search-api/venv/bin/python3" ]]; then
  exec search-api/venv/bin/python3 scripts/perf/db/query_pg_stat.py enforce
fi

run_psql() {
  if command -v psql >/dev/null 2>&1; then
    if [[ -z "${DATABASE_URL:-}" ]]; then
      echo "DATABASE_URL is required when using the psql-backed perf:db:enforce path."
      exit 1
    fi
    psql "$DATABASE_URL" "$@"
    return
  fi

  if ! command -v docker >/dev/null 2>&1; then
    echo "psql is not installed, search-api/venv is unavailable, and Docker is unavailable. Install a PostgreSQL client, restore the venv, or install Docker to run perf:db:enforce."
    exit 1
  fi

  if [[ -z "${DATABASE_URL:-}" ]]; then
    echo "DATABASE_URL is required when using the Docker-backed perf:db:enforce path."
    exit 1
  fi

  docker run --rm \
    -e DATABASE_URL="$DATABASE_URL" \
    postgres:16-alpine \
    psql "$DATABASE_URL" "$@"
}

run_psql -q -c "CREATE EXTENSION IF NOT EXISTS pg_stat_statements;" >/dev/null

count="$(run_psql -At -c "SELECT COUNT(*) FROM (
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
