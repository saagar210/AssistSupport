# AssistSupport v0.3.0 Improvement Roadmap (Security + Quality + Performance)

Audience: Claude Code implementation agent
Owner: Senior/Principal Engineer (this document)
Priority drivers: Security, efficiency, productivity

This roadmap translates the full audit into phased, actionable work. Each phase includes tasks, acceptance criteria, and implementation notes. Keep changes tightly scoped and avoid unrelated refactors. All work must preserve local‑first, offline operation and user privacy.

---

## Phase 0 — Alignment and Guardrails (1–2 days)
Goal: Establish a consistent storage root, confirm ID policy, and set coding guardrails before touching critical paths.

Tasks:
1) Choose canonical app data root
   - Decision: Use `~/Library/Application Support/AssistSupport/` for all runtime data.
   - Update all path builders to use this root (DB, tokens, logs, vectors, models, downloads, cache).
   - Acceptance: All modules resolve paths under the same root; no `com.d.assistsupport` remnants.

2) Define namespace ID policy (see Open Questions section)
   - Decision: IDs are slug‑safe `[a-z0-9-]{1,64}`; store a separate display name.
   - Auto‑normalize user input into IDs; reject invalid IDs.
   - Document in `docs/ARCHITECTURE.md` and `docs/OPERATIONS.md`.
   - Acceptance: A single `validate_namespace_id()` exists and is used everywhere.

3) Define network policy for HTTP
   - Decision: HTTP is allowed via user preference (explicit opt‑in + UI warning).
   - Keep SSRF controls unchanged except to close bypasses.
   - Acceptance: Policy is clearly stated in `docs/SECURITY.md`.

---

## Phase 1 — Critical Security Hardening (SSRF + Path Traversal + Injection) (3–5 days)
Goal: Close high‑risk vectors in network and filesystem ingestion.

Tasks:
1) SSRF: IPv6‑mapped IPv4 normalization
   - Update IP check to detect `::ffff:127.0.0.1` and other mapped IPv4 addresses.
   - Apply `to_ipv4()`/`is_ipv4_mapped()` checks before allowing IPv6 addresses.
   - Files: `src-tauri/src/kb/network.rs`
   - Acceptance: Tests cover IPv6‑mapped local/private ranges and fail as expected.

2) SSRF: DNS rebinding mitigation for requests
   - Resolve host to IP(s) and pin for the request, or validate the resolved IP used by the client.
   - Options:
     - Use reqwest with a custom DNS resolver and connect‑to‑IP.
     - Pre‑resolve and replace URL with IP + Host header, and enforce TLS SNI where applicable.
   - Files: `src-tauri/src/kb/network.rs`, `src-tauri/src/kb/ingest/web.rs`
   - Acceptance: Rebinding attempt is blocked; tests include staged host resolution.

3) SSRF: Redirect validation
   - Ensure redirects re‑validate against SSRF rules and host allowlist.
   - Cap redirect count; return explicit error if exceeded.
   - Files: `src-tauri/src/kb/ingest/web.rs`
   - Acceptance: Redirects to private ranges are blocked even if initial URL is public.

4) KB scan: avoid symlink traversal
   - In recursive scan, skip symlinked files and directories by default.
   - If you must allow symlinks, require canonical path to stay within allowed root.
   - Files: `src-tauri/src/kb/indexer.rs`
   - Acceptance: Symlink to `/etc` under KB folder is skipped; test added.

5) Vector store filter injection
   - Sanitize namespace/document IDs before building LanceDB filters.
   - Consider strict ID validation (slug) or safe escaping.
   - Files: `src-tauri/src/kb/vectors.rs`
   - Acceptance: IDs containing quotes cannot alter filters; tests added.

---

## Phase 2 — Storage Consistency + Permissions + SQLite Safety (2–4 days)
Goal: Align storage paths with docs and harden filesystem permissions.

Tasks:
1) Standardize storage root
   - Update `get_app_data_dir()` in DB module to match security storage root.
   - Ensure Security module and DB module agree on paths.
   - Files: `src-tauri/src/db/mod.rs`, `src-tauri/src/security.rs`, `src-tauri/src/commands/mod.rs`
   - Acceptance: All runtime data is under one root.

2) Update docs to match storage layout
   - Fix filenames and paths for key storage, tokens, vectors, database location.
   - Files: `docs/SECURITY.md`, `docs/INSTALLATION.md`, `docs/OPERATIONS.md`
   - Acceptance: Docs match code exactly.

3) Enforce directory/file permissions
   - Set `0700` on root data dirs, `0600` on sensitive files (DB, tokens, audit log).
   - Apply at creation time for DB, tokens, audit logs, models, vectors.
   - Files: `src-tauri/src/security.rs`, `src-tauri/src/audit.rs`, `src-tauri/src/db/mod.rs`
   - Acceptance: Permissions are correct under default umask; tests verify on unix.

4) Enable SQLite foreign keys
   - Execute `PRAGMA foreign_keys = ON;` on connection open.
   - Consider `busy_timeout` and `journal_mode=WAL` if not already set.
   - Files: `src-tauri/src/db/mod.rs`
   - Acceptance: ON DELETE CASCADE works in tests.

---

## Phase 3 — KB and Ingestion Robustness (3–5 days)
Goal: Improve safety/performance during ingestion and indexing.

Tasks:
1) Stream‑safe web ingestion
   - Enforce content size while streaming; stop reading when limit is hit.
   - Avoid buffering entire response in memory.
   - Files: `src-tauri/src/kb/ingest/web.rs`
   - Acceptance: Large pages abort early; memory remains bounded.

2) File hashing and parsing limits
   - Add size limits for local files indexed via KB folder.
   - Use streaming hash for large files (or skip by size).
   - Files: `src-tauri/src/kb/indexer.rs`
   - Acceptance: Files over limit are skipped with a recorded error.

3) Namespace rules enforcement (if intended)
   - If namespace rules are security‑relevant, enforce them in ingestion commands.
   - If not, remove/flag as non‑enforcing to avoid false security signals.
   - Files: `src-tauri/src/db/mod.rs`, `src-tauri/src/commands/mod.rs`
   - Acceptance: Rules are consistently applied or explicitly documented as advisory only.

4) Search snippet sanitization usage
   - Apply `post_process_results()` consistently before returning search results.
   - Files: `src-tauri/src/kb/search.rs`, `src-tauri/src/commands/mod.rs`
   - Acceptance: Snippets are sanitized server‑side, regardless of UI rendering.

---

## Phase 4 — LLM + Embedding Performance & Stability (3–5 days)
Goal: Make inference robust for large prompts and improve throughput.

Tasks:
1) Dynamic batch sizing for LLM
   - Set batch size based on prompt length / context window (avoid fixed 512).
   - Ensure correct handling for long prompts.
   - Files: `src-tauri/src/llm.rs`
   - Acceptance: Long prompts no longer error or truncate unexpectedly.

2) Embedding context reuse
   - Reuse embedding context or pool contexts to avoid per‑call initialization cost.
   - Files: `src-tauri/src/kb/embeddings.rs`
   - Acceptance: Batch embeddings run faster with same outputs.

3) Bound streaming memory
   - Put a hard cap on `streamingText` size in frontend to avoid memory spikes.
   - Files: `src/hooks/useLlm.ts`, `src/components/Draft/ResponsePanel.tsx`
   - Acceptance: Streaming continues but UI never accumulates unbounded text buffers.

---

## Phase 5 — Audit Logging & Sensitive Data Handling (2–3 days)
Goal: Ensure secrets are never logged and audit logs are properly secured.

Tasks:
1) Harden audit log permissions
   - Ensure audit log file is created with `0600` and directory `0700`.
   - Files: `src-tauri/src/audit.rs`
   - Acceptance: Log is not world‑readable.

2) Zeroize derived secrets
   - Apply `Zeroize` to passphrase‑derived KEKs and decrypted key bytes.
   - Files: `src-tauri/src/security.rs`
   - Acceptance: KEKs are wiped immediately after use.

3) Audit logging coverage review
   - Ensure key rotation, HTTP opt‑in, and token changes are logged.
   - Add missing audit events if needed.
   - Files: `src-tauri/src/audit.rs`, `src-tauri/src/commands/mod.rs`

---

## Phase 6 — Tests, Benchmarks, Documentation Sync (2–4 days)
Goal: Close coverage gaps and keep docs in sync with code.

Tasks:
1) SSRF tests
   - Add tests for IPv6‑mapped addresses and rebinding scenario.
   - Files: `src-tauri/tests/security.rs`, `src-tauri/src/kb/network.rs`

2) Symlink traversal tests
   - Add KB scan tests that verify symlinked dirs/files are skipped.
   - Files: `src-tauri/tests/path_validation.rs`, `src-tauri/src/kb/indexer.rs`

3) Vector filter injection tests
   - Validate escaping/validation behavior for namespace and document IDs.
   - Files: `src-tauri/src/kb/vectors.rs`

4) Docs update
   - Align storage paths and key file names with code.
   - Files: `docs/SECURITY.md`, `docs/INSTALLATION.md`, `docs/ARCHITECTURE.md`

---

## Phase 7 — Optional Hardening (if time) (3–6 days)
Goal: Long‑term improvements for resilience and maintainability.

Tasks:
1) Storage encryption for vectors
   - Track LanceDB encryption progress; consider OS‑level volume encryption or app‑level encryption layer.

2) Consistent error surfaces
   - Replace `unwrap_or_default()` with explicit error handling for observability.
   - Files: `src-tauri/src/commands/mod.rs`

3) Structured telemetry (local only)
   - Optional: local performance counters, no remote telemetry.

---

## Decisions (Locked In)
1) Namespace ID policy
   - Enforce slug `[a-z0-9-]{1,64}` and normalize on input.
   - Store a separate display name for user‑friendly labels.

2) Canonical app data root
   - `~/Library/Application Support/AssistSupport/`.

3) HTTP ingestion policy
   - Allowed via explicit user preference (opt‑in), enforced in UI and backend.

---

## Implementation Notes for Claude Code
- Keep changes small and verified with tests.
- Do not refactor unrelated modules.
- Update docs whenever file paths or security behavior changes.
- Add targeted tests with minimal fixtures.
- Avoid new dependencies unless necessary for SSRF DNS pinning.
