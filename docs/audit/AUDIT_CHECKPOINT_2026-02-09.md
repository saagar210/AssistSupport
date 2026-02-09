# AssistSupport Security/Compliance Audit Checkpoint (2026-02-09)

Repo: `/Users/d/Projects/AssistSupport`  
Branch: `master`  
HEAD: `ce9991ae032c936c8bea5e45bbb4421b8fa872c2`  
Remote: `origin` (GitHub)  

This file is a hard checkpoint so we can resume the security/compliance audit without losing context.

## Status Update (2026-02-09)

Phase 1 report artifact is now written at:

- `/Users/saagarpatel/AssistSupport/docs/audit/SECURITY_COMPLIANCE_AUDIT_REPORT_2026-02-09.md`

High-severity Pilot items have been addressed in the working tree (AUD-001, AUD-002). Remaining open items are AUD-003 and AUD-004.

## Where We Stopped

- Phase 0 (Baseline + Repo Map): completed (all required gates were green at time of work).
- Phase 1 (Exhaustive Audit Report): started, not fully written up as a single report artifact yet.
- Phase 2 (Fix in controlled batches): started and multiple batches landed on `master` (see commits below).

Next intended work (not done yet): continue Phase 2 with the remaining high-risk items (Pilot logging/export, Search API minimization, platform-scope enforcement).

## Mandatory Gates (Resume Commands)

Run these first when resuming to ensure we still start from a green baseline:

```bash
cd /Users/d/Projects/AssistSupport
pnpm run check:monorepo-readiness
pnpm run check:monorepo-readiness:full
pnpm run test:ci
cargo audit
pnpm audit
```

Notes:
- `cargo audit` may surface Linux GTK/WebKitGTK advisories depending on dependency graph; treat as platform-scope risk unless Linux is explicitly supported and patched.

## Fix Batches Already Landed (Phase 2)

These commits are already on `master`:

1) `89fd938` `fix(security): tighten loopback base URL validation`
- Tightened URL validation to fail closed and fixed a clippy issue.

2) `9a7f6d6` `fix(deps): patch tantivy to remove unsound lru`
- Vendored `src-tauri/vendor/tantivy` and bumped `lru` to remove RustSec UNSOUND advisory path.

3) `82a69e0` `fix(security): remove session token auto-unlock`
- Removed session-token "auto-unlock" control (security theater / misleading) across frontend + tauri + DB migration.

4) `ce9991a` `fix(security): harden search-api logging and input`
- Replaced runtime `print()` usage with structured logging.
- Production error behavior: avoids emitting raw exception strings.
- Added request body size cap (`MAX_CONTENT_LENGTH`, env override supported).
- Added allowlist validation for `fusion_strategy`.
- Added pytest coverage.

## Open Issues (Phase 1 Findings Not Yet Fixed)

### AUD-001
- Severity: **High**
- Category: **Security**, **Compliance**, **Defaults**
- Compliance Mapping:
  - Standards: SOC 2, ISO 27001, NIST SP 800-53, GDPR, HIPAA, FISMA
  - Controls (representative): NIST AU-2/AU-3 (audit event content), AU-11 (retention), PL-8 (security and privacy plans); ISO A.5.34/A.8.12 (logging/monitoring); SOC2 CC7.2/CC7.3; GDPR Art. 5(1)(c) (data minimization), Art. 32; HIPAA 164.312(b) (audit controls)
- Location:
  - `/Users/d/Projects/AssistSupport/src/components/Pilot/PilotQueryTester.tsx:14-56`
  - `/Users/d/Projects/AssistSupport/src-tauri/src/feedback/mod.rs:79-170`
  - `/Users/d/Projects/AssistSupport/src-tauri/src/db/mod.rs:927-953`
- Evidence:
  - UI asks for "Your name or email" and persists it in localStorage key `pilot-user-id`.
  - Backend stores raw `query`, raw `response`, and raw `user_id` in SQLite tables `pilot_query_logs` and `pilot_feedback`.
  - No explicit retention policy, minimization, or "pilot logging enabled" gate.
- Fix (planned):
  - Default-off gate for Pilot logging/export (`ASSISTSUPPORT_ENABLE_PILOT_LOGGING=1`), enforced in tauri command layer.
  - Replace user identifier with pseudonymous ID (no names/emails); validate strongly.
  - Redact / minimize stored data (hashing and/or truncation) and add retention caps (days + max rows).
- Verification (planned):
  - `pnpm run test:ci`
  - `pnpm run check:monorepo-readiness:full`
  - Add Rust tests covering gating + redaction/validation.

Status: addressed in working tree (default-off gate + pseudonymous operator ID + redaction/truncation + retention caps). See `/Users/saagarpatel/AssistSupport/docs/audit/SECURITY_COMPLIANCE_AUDIT_REPORT_2026-02-09.md`.

### AUD-002
- Severity: **High**
- Category: **Security**
- Compliance Mapping:
  - Standards: SOC 2, ISO 27001, NIST SP 800-53, FISMA
  - Controls (representative): NIST AC-6 (least privilege), CM-7 (least functionality)
- Location:
  - `/Users/d/Projects/AssistSupport/src-tauri/src/commands/mod.rs:6154-6161`
  - `/Users/d/Projects/AssistSupport/src-tauri/src/feedback/export.rs:8-15`
- Evidence:
  - `export_pilot_data(path)` passes unvalidated `path` to `File::create(path)` via `export_to_csv`.
  - This is a tauri command surface that can write to arbitrary paths (within the user’s OS permissions).
- Fix (planned):
  - Validate export path using existing path sandboxing (e.g., `validate_within_home`) in command layer before file creation.
  - Potentially restrict to a dedicated export directory (stronger than "within home") if required by policy.
- Verification (planned):
  - Add unit tests ensuring unsafe paths are rejected.
  - Run `pnpm run test:ci` and `pnpm run check:monorepo-readiness:full`.

Status: addressed in working tree (export path validation using `validate_output_file_within_home` + `.csv` enforcement). See `/Users/saagarpatel/AssistSupport/docs/audit/SECURITY_COMPLIANCE_AUDIT_REPORT_2026-02-09.md`.

### AUD-003
- Severity: **Medium** (can be **High** if we claim strict minimization)
- Category: **Compliance**, **Assumptions**
- Compliance Mapping:
  - Standards: GDPR, HIPAA, SOC 2, ISO 27001, NIST SP 800-53, FISMA
  - Controls (representative): NIST AU-2/AU-3; GDPR Art. 5; HIPAA 164.306(b)
- Location:
  - `/Users/d/Projects/AssistSupport/search-api/hybrid_search.py:323-363`
- Evidence:
  - Search API persists raw `query` into `query_performance.query_text`.
  - Even if logs are clean, storage may violate “data minimization / local-only / no sensitive content retention” expectations.
- Fix (planned):
  - Make query_text storage opt-in (default off), or store only a hash + metadata (counts/timing) in production.
  - Add tests to prove no raw query is stored when disabled.

Status: addressed in working tree (query_text stored as `sha256:<hex>` by default; opt-in via `ASSISTSUPPORT_SEARCH_API_STORE_RAW_QUERY_TEXT=1`). See `/Users/saagarpatel/AssistSupport/docs/audit/SECURITY_COMPLIANCE_AUDIT_REPORT_2026-02-09.md`.

### AUD-004
- Severity: **Medium**
- Category: **Defaults**, **Compliance**
- Compliance Mapping:
  - Standards: SOC 2, ISO 27001, NIST SP 800-53, FISMA
  - Controls (representative): NIST CM-2/CM-6 (configuration), CM-7 (least functionality)
- Location: repo-wide (policy scripts + CI)
- Evidence:
  - `cargo audit` may report GTK/WebKitGTK advisories on non-macOS platforms; if AssistSupport is macOS-only, we should fail closed on unsupported platforms in CI/build unless explicitly overridden.
- Fix (planned):
  - Add an AssistSupport platform-scope enforcement script (similar to the VaultMind approach) and wire it into monorepo readiness.

Status: addressed in working tree (platform-scope guardrail added and wired into readiness). See `/Users/saagarpatel/AssistSupport/docs/audit/SECURITY_COMPLIANCE_AUDIT_REPORT_2026-02-09.md`.

## Next Fix Batch Plan (Phase 2)

Batch A (Pilot logging minimization + gating):
- Add tauri-side env gate (default off) for all Pilot commands.
- Replace PII-ish “name/email” UX with pseudonymous operator id.
- Add retention + data minimization/redaction.

Batch B (Pilot export path validation):
- Enforce `validate_within_home` (or stronger) for export path.

Batch C (Search API minimization):
- Optional/hashed query_text; default off for production.

Batch D (Platform-scope enforcement):
- Fail closed on unsupported OS builds unless an explicit override is present.

## Local Git Remotes Present (FYI)

`origin` is GitHub. There are local remotes named `memorykernel` and `vaultmind` pointing at local filesystem repos; do not assume those are pushed unless explicitly done.
