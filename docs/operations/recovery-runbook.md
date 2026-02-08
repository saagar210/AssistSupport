# Recovery Runbook

## Scope

Use this runbook for local data recovery after integrity, import, or operational failures.

## Failure Modes

- Import verification failure (`manifest.sig` mismatch or missing verification key)
- Encrypted snapshot import failure (missing/invalid decrypt key)
- Database corruption detected by `mk db integrity-check`

## Recovery Steps

1. Stop active write operations.
2. Collect diagnostics:
   - `mk db integrity-check`
   - command stderr from failed operation
3. Restore last known good backup:
   - `mk db restore --in <backup.sqlite3>`
4. Verify:
   - `mk db schema-version`
   - `mk db integrity-check`
5. Re-run failed import with explicit trust controls:
   - signed: `--verify-key-file`
   - encrypted: `--decrypt-key-file`
   - unsigned override only if approved: `--allow-unsigned`

## Post-Recovery Validation

1. Run deterministic query checks:
   - `mk query ask ...`
   - `mk query recall ...`
2. Confirm expected context package persistence:
   - `mk context show --context-package-id <id>`
