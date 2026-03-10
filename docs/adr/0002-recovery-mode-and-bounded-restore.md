# 0002: Recovery Mode and Bounded Restore Flow

- Status: Accepted
- Date: 2026-03-10

## Context

The stabilization program identified three related operator risks:

1. A corrupted or partially migrated SQLite workspace could fail during normal startup before any repair path was available.
2. Backup import accepted archives too eagerly, which increased the chance of oversized or malformed restore inputs consuming memory before practical limits were enforced.
3. Migration conflicts were visible in logs, but they did not produce a first-class operator flow that a junior engineer or support engineer could follow safely.

These risks needed one explicit design decision so the frontend, Rust backend, and runbooks all describe the same failure and recovery path.

## Decision

- Startup may enter a dedicated recovery mode before normal DB initialization completes.
- Recovery mode is entered for:
  - database integrity failures,
  - foreign-key consistency failures,
  - migration conflicts that require operator action,
  - restore scenarios where the normal DB cannot be trusted.
- Recovery diagnostics must run both `PRAGMA integrity_check` and `PRAGMA foreign_key_check`.
- Backup preview and restore flows must validate on-disk size bounds before accepting full archive contents.
- Recovery restore writes into a fresh encrypted database and only replaces the active DB after import succeeds.
- Existing DB files are archived during restore so the operator has a rollback path if a restore attempt fails.

## Consequences

### Positive

- Corruption and migration conflicts now produce a deterministic, user-visible recovery path.
- Backup restore is more resilient against oversized or malformed inputs.
- Support engineers have one documented path for repair vs restore decisions.

### Tradeoffs

- Startup logic is more branched because recovery mode exists before normal shell initialization.
- Some malformed archives that previously slipped through will now be rejected earlier.
- Recovery UI and runbooks become part of the supported surface and must stay current.

## Follow-up

- Keep `docs/runbooks/safe-mode-recovery.md` aligned with the shipped UI and backend behavior.
- Add future tests for manual conflict resolution flows if migration complexity grows further.
