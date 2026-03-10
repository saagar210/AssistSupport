#!/usr/bin/env python3
"""Run pg_stat_statements checks without requiring a local psql install."""

from __future__ import annotations

import json
import os
import sys
from pathlib import Path

import psycopg2
from psycopg2.extras import RealDictCursor

SEARCH_API_DIR = Path(__file__).resolve().parents[3] / "search-api"
if str(SEARCH_API_DIR) not in sys.path:
  sys.path.insert(0, str(SEARCH_API_DIR))

from db_config import get_db_conn_kwargs


CHECK_SQL = """
WITH offenders AS (
  SELECT
    queryid,
    mean_exec_time,
    calls,
    query
  FROM pg_stat_statements
  WHERE calls >= %s
    AND mean_exec_time > %s
  ORDER BY mean_exec_time DESC
)
SELECT * FROM offenders LIMIT 20;
"""

COUNT_SQL = """
SELECT COUNT(*) AS offender_count
FROM (
  SELECT queryid
  FROM pg_stat_statements
  WHERE calls >= %s
    AND mean_exec_time > %s
) offenders;
"""

BM25_EXPLAIN_SQL = """
EXPLAIN (ANALYZE, FORMAT JSON)
SELECT id, ts_rank(fts_content, query) AS bm25_score
FROM kb_articles, plainto_tsquery('english', %s) query
WHERE fts_content @@ query AND is_active = true
ORDER BY bm25_score DESC
LIMIT 50;
"""


def _run_explain_fallback(cursor, *, max_mean_ms: float, mode: str) -> int:
  cursor.execute(BM25_EXPLAIN_SQL, ("vpn",))
  row = cursor.fetchone()
  plan_root = row["QUERY PLAN"][0]
  execution_time_ms = float(plan_root.get("Execution Time", 0.0))
  planning_time_ms = float(plan_root.get("Planning Time", 0.0))
  summary = {
    "mode": "explain_fallback",
    "query": "bm25_search_vpn",
    "execution_time_ms": execution_time_ms,
    "planning_time_ms": planning_time_ms,
    "threshold_ms": max_mean_ms,
  }

  if mode == "check":
    print(json.dumps(summary, indent=2))
    return 0

  if execution_time_ms > max_mean_ms:
    print(
      f"DB performance gate failed: BM25 explain execution "
      f"{execution_time_ms:.1f}ms > {max_mean_ms:.1f}ms.",
      file=sys.stderr,
    )
    return 1

  print(
    f"DB performance gate passed via EXPLAIN fallback: "
    f"{execution_time_ms:.1f}ms <= {max_mean_ms:.1f}ms."
  )
  return 0


def main() -> int:
  if len(sys.argv) != 2 or sys.argv[1] not in {"check", "enforce"}:
    print("Usage: query_pg_stat.py <check|enforce>", file=sys.stderr)
    return 1

  min_calls = int(os.environ.get("DB_MIN_CALLS", "50"))
  max_mean_ms = float(os.environ.get("DB_MAX_MEAN_MS", "100"))
  mode = sys.argv[1]

  database_url = os.environ.get("DATABASE_URL")
  connection = psycopg2.connect(database_url) if database_url else psycopg2.connect(**get_db_conn_kwargs())

  with connection:
    connection.autocommit = True
    with connection.cursor(cursor_factory=RealDictCursor) as cursor:
      cursor.execute("CREATE EXTENSION IF NOT EXISTS pg_stat_statements;")

      try:
        if mode == "check":
          cursor.execute(CHECK_SQL, (min_calls, max_mean_ms))
          rows = cursor.fetchall()
          print(json.dumps(rows, indent=2, default=str))
          return 0

        cursor.execute(COUNT_SQL, (min_calls, max_mean_ms))
        count = int(cursor.fetchone()["offender_count"])
        if count > 0:
          print(
            f"DB performance gate failed: {count} offender queries "
            f"(calls>={min_calls}, mean_exec_time>{max_mean_ms}ms).",
            file=sys.stderr,
          )
          return 1

        print("DB performance gate passed: no offenders.")
        return 0
      except psycopg2.Error as exc:
        message = str(exc)
        if "pg_stat_statements must be loaded via shared_preload_libraries" not in message:
          raise
        connection.rollback()
        return _run_explain_fallback(cursor, max_mean_ms=max_mean_ms, mode=mode)


if __name__ == "__main__":
  raise SystemExit(main())
