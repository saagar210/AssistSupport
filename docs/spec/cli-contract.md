# CLI Contract (Normative)

## Binary Name

`mk`

## Global Flags

- `--db <path>` default `./memory_kernel.sqlite3`

## Output Contract Version

- Every successful JSON output MUST include top-level `contract_version`.
- Current contract version is `cli.v1`.
- JSON Schemas for `cli.v1` outputs MUST be stored under `contracts/v1/`.

## Commands

### `mk db schema-version`
Required:
- none

Output:
- MUST print JSON with `current_version`, `target_version`, `pending_versions`, `up_to_date`, and `inferred_from_legacy`.
- MUST NOT mutate schema state.

### `mk db migrate`
Required:
- none

Optional:
- `--dry-run` (if set, command MUST output migration plan and MUST NOT mutate schema state)

Output:
- For non-dry-run, MUST apply pending migrations then print before/after version details.

### `mk db export`
Required:
- `--out <path>`

Optional:
- `--signing-key-file <path>` (`32-byte` hex key file for manifest signature generation)
- `--encrypt-key-file <path>` (`32-byte` hex key file for snapshot file encryption)

Output:
- MUST create export files under `--out`:
  - `memory_records.ndjson`
  - `context_packages.ndjson`
  - `manifest.json`
- When signing is enabled, MUST also write `manifest.sig`.
- When encryption and/or signing is enabled, MUST write `manifest.security.json`.
- MUST print JSON including export path and manifest details.

### `mk db import`
Required:
- `--in <path>`

Optional:
- `--skip-existing` (default `true`)
- `--verify-key-file <path>` (`32-byte` hex key file for manifest signature verification)
- `--decrypt-key-file <path>` (`32-byte` hex key file for encrypted snapshot files)
- `--allow-unsigned` (default `false`; explicit override for unsigned imports)

Output:
- MUST import NDJSON snapshot from `--in` and print import summary.
- When `--skip-existing=true`, duplicate `memory_version_id` and duplicate `context_package_id` rows MUST be skipped and reported.
- Signed snapshots MUST fail import if signature verification is missing or invalid.
- Encrypted snapshots MUST fail import unless a valid decrypt key is provided.

### `mk db backup`
Required:
- `--out <path>`

Output:
- MUST create a SQLite backup file at `--out`.
- MUST print JSON confirming backup path and success status.

### `mk db restore`
Required:
- `--in <path>` (SQLite backup file path)

Output:
- MUST restore database state from backup file.
- MUST print resulting schema version status.

### `mk db integrity-check`
Required:
- none

Output:
- MUST print structured report containing:
  - `quick_check_ok`
  - `quick_check_message`
  - `foreign_key_violations[]`
  - `schema_status`

### `mk memory add constraint`
Required:
- `--actor --action --resource --effect`
- `--writer --justification`
- `--source-uri --truth-status --authority`

Optional:
- `--memory-id` (ULID; if omitted, a new `memory_id` is generated)
- `--source-hash --evidence ... --confidence`
- `--created-at --effective-at`
- `--supersedes ... --contradicts ...` (values MUST be `memory_version_id` ULIDs)

### `mk memory add decision|preference|event|outcome`
Required:
- `--summary`
- same required accountability/provenance flags as above

Optional:
- `--memory-id` (ULID; if omitted, a new `memory_id` is generated)
- `--supersedes ... --contradicts ...` (values MUST be `memory_version_id` ULIDs)

### `mk memory link`
Required:
- `--from --to --relation` (`--from` and `--to` MUST be `memory_version_id` ULIDs)
- `--writer --justification`

Output:
- MUST include `from_memory_version_id` and `to_memory_version_id`.

### `mk query ask`
Required:
- `--text --actor --action --resource`
Optional:
- `--as-of` (UTC RFC3339); if omitted CLI sets UTC now.

Output:
- MUST print Context Package JSON.
- MUST persist package for retrieval.

### `mk query recall`
Required:
- `--text`

Optional:
- `--record-type <constraint|decision|preference|event|outcome>` (repeatable; if omitted defaults to `decision|preference|event|outcome`)
- `--as-of` (UTC RFC3339); if omitted CLI sets UTC now.

Output:
- MUST print Context Package JSON.
- MUST persist package for retrieval.
- MUST use deterministic recall ordering metadata from `docs/spec/resolver.md`.

### `mk context show`
Required:
- `--context-package-id`

Output:
- MUST print stored Context Package JSON.

### `mk outcome ...`
Outcome command surface is hosted under the same `mk` binary and MUST remain contract-compatible
with OutcomeMemory v1 command semantics:

- `mk outcome log ...`
- `mk outcome manual ...`
- `mk outcome system ...`
- `mk outcome trust show ...`
- `mk outcome replay ...`
- `mk outcome benchmark run ...`
- `mk outcome projector status|check|stale-keys ...`
- `mk outcome gate preview ...`
- `mk outcome events list ...`

Host integration MUST preserve Outcome JSON contract versions (for example `gate_preview.v1`,
`projector_status.v1`, `projector_check.v1`, `benchmark_report.v1`).

## Error Model

- Missing required args -> non-zero exit
- Invalid enum/time/id formats -> non-zero exit
- Unknown context package id -> non-zero exit
- Any rejected write MUST return actionable error text (`MKR-022`, `MKR-026`)
