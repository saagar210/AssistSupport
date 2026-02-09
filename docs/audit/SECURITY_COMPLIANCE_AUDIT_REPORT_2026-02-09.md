# AssistSupport Security/Compliance Audit Report (2026-02-09)

Repo: `/Users/saagarpatel/AssistSupport`  
Branch: `master`  
HEAD (commit): `9b3672d4ec7b6d6ec934305a26cff26c5ca7c57f`  

This report is the Phase 1 "single artifact" write-up referenced by `/Users/saagarpatel/AssistSupport/docs/audit/AUDIT_CHECKPOINT_2026-02-09.md`. It enumerates the major findings, evidence anchors, and fix status.

## Executive Summary

Top risks identified in this audit were centered around the "Pilot" feature persisting operator-entered text without strong defaults, minimization, and export path validation. Those items have been remediated in the current working tree by:

- Default-off policy gate for Pilot logging/export (requires explicit opt-in via env var).
- Replacement of name/email collection with a pseudonymous operator ID.
- Best-effort redaction + truncation of persisted pilot text plus retention caps.
- Export path validation for Pilot CSV export (must be within `$HOME`, not sensitive subdirs).

Remaining open items are Search API minimization (AUD-003) and platform-scope enforcement (AUD-004).

## Baseline Evidence (Resume Gates)

Commands executed while resuming the audit (2026-02-09):

- `pnpm run check:monorepo-readiness` (pass)
- `pnpm run check:monorepo-readiness:full` (pass)
- `pnpm run test:ci` (pass)
- `pnpm audit --audit-level high` (pass)
- `cd src-tauri && cargo audit` (no vulns; warnings present, including GTK/GTK3-related advisories in Linux-only dependency trees)

## Findings

### AUD-001 (High) Pilot logging persisted PII-ish identifiers and raw query/response by default

Category: Security, Compliance, Defaults

Evidence (pre-fix behavior described in checkpoint):

- UI prompted for "Your name or email" and stored it in localStorage.
- Backend persisted raw `query`, raw `response`, and raw `user_id` into SQLite tables `pilot_query_logs` / `pilot_feedback`.

Remediation implemented in working tree:

1. Default-off Pilot policy gate:
   - Pilot commands now require `ASSISTSUPPORT_ENABLE_PILOT_LOGGING=1`.
   - Evidence: `/Users/saagarpatel/AssistSupport/src-tauri/src/commands/mod.rs:65-115` and `/Users/saagarpatel/AssistSupport/src-tauri/src/commands/mod.rs:6151-6205`.

2. Pseudonymous operator ID (no name/email):
   - Legacy localStorage key `pilot-user-id` is removed if present and replaced with `pilot-operator-id` generated as `op-<uuid>`.
   - Evidence: `/Users/saagarpatel/AssistSupport/src/components/Pilot/PilotQueryTester.tsx:14-45` and `/Users/saagarpatel/AssistSupport/src/components/Pilot/PilotQueryTester.tsx:119-138`.

3. Minimization and retention:
   - Persisted pilot text is truncated and redacted (best-effort) for emails, US phone numbers, SSNs, and common token prefixes.
   - Pilot retention and max rows are enforced on writes:
     - `ASSISTSUPPORT_PILOT_RETENTION_DAYS` (default 14, clamp 1..365)
     - `ASSISTSUPPORT_PILOT_MAX_ROWS` (default 500, clamp 50..50k)
   - Evidence: `/Users/saagarpatel/AssistSupport/src-tauri/src/feedback/mod.rs:14-134` and `/Users/saagarpatel/AssistSupport/src-tauri/src/feedback/mod.rs:203-259`.

Status: Addressed in working tree.

Notes:

- The SQLite schema column remains named `user_id`, but it now carries a pseudonymous operator ID, and the UI renders it as "Operator".

### AUD-002 (High) Pilot export accepted an arbitrary filesystem path

Category: Security

Risk:

- A compromised renderer could request exports to unexpected locations (within the user's OS permissions), potentially clobbering files.

Remediation implemented in working tree:

1. Output path validation and extension restriction:
   - `export_pilot_data` now requires a `.csv` path and validates it with `validate_output_file_within_home`.
   - Export paths outside `$HOME` or within sensitive subdirectories are rejected.
   - Evidence: `/Users/saagarpatel/AssistSupport/src-tauri/src/commands/mod.rs:6228-6258` and `/Users/saagarpatel/AssistSupport/src-tauri/src/validation.rs:281-320`.

2. Output-file validator added (does not auto-create directories):
   - `validate_output_file_within_home` validates canonical parent directories for non-existent files and does not create a directory at the file path.
   - Evidence: `/Users/saagarpatel/AssistSupport/src-tauri/src/validation.rs:281-360`.

Status: Addressed in working tree.

### AUD-003 (Medium) Search API query minimization

Category: Compliance, Assumptions

Risk:

- Search API persists raw query text (even if logs are structured/clean), which may violate minimization expectations in production.

Remediation implemented in working tree:

- Raw query persistence is now **default off**.
- Query text stored in `query_performance.query_text` is a deterministic SHA-256 fingerprint (`sha256:<hex>`) unless explicitly enabled.
- Opt-in override: set `ASSISTSUPPORT_SEARCH_API_STORE_RAW_QUERY_TEXT=1` to store raw query text (intended for local/dev analytics only).

Evidence:

- `/Users/saagarpatel/AssistSupport/search-api/runtime_config.py` (`store_raw_query_text` flag)
- `/Users/saagarpatel/AssistSupport/search-api/search_api.py` (engine wiring)
- `/Users/saagarpatel/AssistSupport/search-api/hybrid_search.py` (hashed query logging)

Status: Addressed in working tree.

Recommended fix:

- Make raw `query_text` persistence opt-in (default off), or store only a hash plus non-sensitive metadata.
- Add tests proving raw query text is not stored when disabled.

Primary evidence anchor (from checkpoint):

- `/Users/saagarpatel/AssistSupport/search-api/hybrid_search.py` (query persistence path).

### AUD-004 (Medium) Platform-scope enforcement for platform-specific advisories

Category: Defaults, Compliance

Risk:

- Rust advisory output includes Linux GTK/WebKitGTK-related crates in the dependency tree. If AssistSupport is macOS-only, the build/test/CI story should be explicit about platform scope and fail closed on unsupported platforms unless overridden.

Remediation implemented in working tree:

- Added a platform-scope guardrail and wired it into the monorepo readiness runner.
- Linux builds now fail closed by default, with an explicit override (`ASSISTSUPPORT_ALLOW_LINUX=1`) for intentional work.

Evidence:

- `/Users/saagarpatel/AssistSupport/scripts/check_assistsupport_platform_scope.sh`
- `/Users/saagarpatel/AssistSupport/scripts/run_monorepo_readiness.sh`

Status: Addressed in working tree.

## Next Steps (Recommended Order)

1. Implement AUD-003 (Search API minimization).
2. Implement AUD-004 (platform-scope enforcement).
3. Re-run the full gate suite:
   - `pnpm run check:monorepo-readiness:full`
   - `pnpm audit --audit-level high`
   - `cd src-tauri && cargo audit`
