-- codex-os-managed
CREATE EXTENSION IF NOT EXISTS pg_stat_statements;

WITH offenders AS (
  SELECT queryid, mean_exec_time, calls, query
  FROM pg_stat_statements
  WHERE calls >= 50 AND mean_exec_time > 100
  ORDER BY mean_exec_time DESC
)
SELECT * FROM offenders LIMIT 20;
