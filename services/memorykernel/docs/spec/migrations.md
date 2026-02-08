# Migration Policy (Normative)

- Migrations MUST be forward-only (`MKR-025`).
- Applied versions MUST be recorded in `schema_migrations`.
- DB schema MUST enforce enum/check constraints and foreign keys (`MKR-023`).
- SQLite runtime MUST set:
  - `PRAGMA journal_mode = WAL`
  - `PRAGMA foreign_keys = ON`
  - `PRAGMA busy_timeout = 5000`

## v1 Schema Objects

- `memory_records`
- `memory_links`
- payload tables:
  - `constraint_payloads`
  - `decision_payloads`
  - `preference_payloads`
  - `event_payloads`
  - `outcome_payloads`
- `context_packages`

## Keying and Foreign Keys

- `memory_records` MUST use `memory_version_id` as primary key.
- `memory_records` MUST enforce `UNIQUE(memory_id, version)`.
- `memory_links` endpoints MUST be `from_memory_version_id` and `to_memory_version_id`.
- payload tables MUST key on `memory_version_id` and reference `memory_records(memory_version_id)`.

## Compatibility

- Changes to Context Package JSON require schema version bump and migration note.
- Startup MUST fail explicitly when required version-id columns are missing (`MKR-031`).
- CLI MUST support non-mutating schema inspection and dry-run migration planning (`MKR-032`).
- CLI MUST support snapshot portability through NDJSON export/import plus manifest metadata (`MKR-033`).
- CLI MUST support database backup/restore and integrity-check operations (`MKR-034`).
- Snapshot exports SHOULD support manifest signatures and optional encryption sidecars for trust controls (`MKR-051`, `MKR-053`).
