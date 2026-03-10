#!/usr/bin/env bash
set -euo pipefail

sql_file="scripts/perf/db/check-pg-stat.sql"

if command -v psql >/dev/null 2>&1; then
  if [[ -z "${DATABASE_URL:-}" ]]; then
    echo "DATABASE_URL is required when using the psql-backed perf:db path."
    exit 1
  fi
  exec psql "$DATABASE_URL" -f "$sql_file"
fi

if [[ -x "search-api/venv/bin/python3" ]]; then
  exec search-api/venv/bin/python3 scripts/perf/db/query_pg_stat.py check
fi

if ! command -v docker >/dev/null 2>&1; then
  echo "psql is not installed, search-api/venv is unavailable, and Docker is unavailable. Install a PostgreSQL client, restore the venv, or install Docker to run perf:db."
  exit 1
fi

if [[ -z "${DATABASE_URL:-}" ]]; then
  echo "DATABASE_URL is required when using the Docker-backed perf:db path."
  exit 1
fi

exec docker run --rm \
  -e DATABASE_URL="$DATABASE_URL" \
  -v "$PWD":/work \
  -w /work \
  postgres:16-alpine \
  psql "$DATABASE_URL" -f "/work/$sql_file"
