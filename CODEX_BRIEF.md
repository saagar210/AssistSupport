# CODEX_BRIEF.md — Senior Architect Handover

**Author:** Claude Code (Junior Implementation Agent)
**Date:** 2026-01-25
**Commit:** Security Roadmap Phases 0-6 Implementation

---

## A. State Transition

- **From:** AssistSupport v0.3.0 with basic security (SQLCipher, path validation, HTTPS enforcement) but gaps in SSRF bypass vectors, symlink traversal, filter injection, and incomplete audit coverage.

- **To:** AssistSupport v0.3.1 with hardened security: IPv6-mapped SSRF protection, symlink traversal prevention, vector store injection defense, KEK zeroization, namespace ID validation, dynamic LLM batching, streaming memory bounds, and complete audit event coverage.

---

## B. Change Manifest (Evidence Anchors)

### Backend (Rust)

| File | Logic Change |
|------|--------------|
| `src-tauri/src/security.rs` | Added `set_secure_permissions()`, `create_secure_dir()`, `ensure_secure_data_dir()` public helpers. Added explicit KEK zeroization in `wrap_key()` and `unwrap_key()` using `Zeroize` trait. Updated `ExportCrypto` to zeroize derived keys after use. |
| `src-tauri/src/validation.rs` | Added namespace ID validation: `normalize_namespace_id()`, `validate_namespace_id()`, `normalize_and_validate_namespace_id()` enforcing slug format `[a-z0-9-]{1,64}`. Added `InvalidNamespaceId` error variant. |
| `src-tauri/src/error.rs` | Added `VALIDATION_INVALID_NAMESPACE_ID` error code constant and handler in `From<ValidationError>` impl. |
| `src-tauri/src/kb/network.rs` | Added `get_ipv4_from_mapped()` to detect IPv6-mapped IPv4 addresses. Updated `is_ip_blocked()` to check mapped addresses. Added cloud metadata endpoint blocking (`169.254.169.254`). |
| `src-tauri/src/kb/vectors.rs` | Added `sanitize_filter_value()` and `sanitize_id()` functions to prevent filter injection. Applied sanitization to all filter operations. |
| `src-tauri/src/kb/indexer.rs` | Added `FileTooLarge` and `SymlinkNotAllowed` error variants. Added `is_symlink()`, `get_file_size_safe()`, `max_file_size()` helpers. Updated `scan_recursive()` to skip symlinks. Updated `parse_document()` to enforce size limits. Changed `file_hash()` to streaming hash. |
| `src-tauri/src/kb/ingest/web.rs` | Added `read_body_with_limit()` for stream-safe content downloading with early abort. |
| `src-tauri/src/kb/embeddings.rs` | Added `EMBEDDING_CTX_SIZE` constant. Added `ctx_params` caching in state. Updated `embed_batch()` to reuse context and batch across texts. Added token truncation warning. |
| `src-tauri/src/llm.rs` | Changed fixed `LlamaBatch::new(512, 1)` to dynamic sizing based on prompt length + max tokens. |
| `src-tauri/src/db/mod.rs` | Added SQLite pragmas: `foreign_keys = ON`, `busy_timeout = 5000`, `journal_mode = WAL`, `secure_delete = ON`. |
| `src-tauri/src/audit.rs` | Updated storage path from `com.d.assistsupport` to `AssistSupport`. Uses `create_secure_dir()` and `set_secure_permissions()`. |
| `src-tauri/src/commands/mod.rs` | Added namespace validation to `ingest_url`, `ingest_youtube`, `ingest_github`, `add_namespace_rule`. Uses `create_secure_dir()` in `initialize_app`. Added audit calls for `set_hf_token`, `clear_hf_token`, Jira token set/clear. |
| `src-tauri/src/commands/diagnostics.rs` | Updated storage path from `com.d.assistsupport` to `AssistSupport`. |
| `src-tauri/src/bin/assistsupport-cli.rs` | Updated storage path from `com.d.assistsupport` to `AssistSupport`. |

### Frontend (TypeScript)

| File | Logic Change |
|------|--------------|
| `src/hooks/useLlm.ts` | Added `MAX_STREAMING_TEXT_SIZE` constant (500KB). Updated token listener to truncate from beginning when buffer exceeds limit, preventing memory exhaustion. |

### Documentation

| File | Logic Change |
|------|--------------|
| `docs/SECURITY.md` | Fixed incident response paths from `com.d.assistsupport` to `AssistSupport`. Added v1.3 changelog documenting all security improvements. |
| `docs/ARCHITECTURE.md` | Added namespace ID policy documentation (slug format, auto-normalization). |

---

## C. Trade-Off Defense

### 1. IPv6-Mapped IPv4 Detection vs. Performance
**Decision:** Check every IPv6 address for IPv4-mapped format before allowing.
**Rationale:** SSRF bypasses using `::ffff:127.0.0.1` are a known attack vector. The additional CPU cost (~nanoseconds) is negligible compared to the security risk of allowing internal network access.

### 2. Symlink Rejection vs. Symlink Following
**Decision:** Skip all symlinks in KB indexer rather than resolving and validating.
**Rationale:** Symlink resolution introduces TOCTOU race conditions. The simpler "reject all" approach eliminates an entire class of attacks. Users wanting to index symlinked content can copy files instead.

### 3. Filter Sanitization vs. Parameterized Queries
**Decision:** Input sanitization with pattern rejection rather than attempting parameterized LanceDB queries.
**Rationale:** LanceDB doesn't support parameterized filters. Sanitization with suspicious pattern detection (`' OR`, `UNION`, etc.) plus quote escaping provides defense-in-depth.

### 4. Streaming Memory Bound vs. Unlimited Accumulation
**Decision:** Hard cap at 500KB for streaming text with truncation from beginning.
**Rationale:** Prevents memory exhaustion from extremely long LLM responses. Truncating from the beginning preserves the most recent (and likely most relevant) content.

### 5. KEK Zeroization Placement
**Decision:** Zeroize KEK immediately after encrypt/decrypt call, before checking result.
**Rationale:** Even if decryption fails, the KEK should not linger in memory. Early zeroization reduces the window of exposure.

### 6. Dynamic Batch Sizing Formula
**Decision:** `batch_size = (tokens.len() + max_tokens).max(512).min(n_ctx)`
**Rationale:** Ensures batch can hold prompt + expected output, with floor of 512 for small prompts and ceiling at context window to prevent allocation failures.

---

## D. The Audit Mandate (Instructions for Codex)

> **Codex, please review my work and generate these 7 specific reports based on FACTS and LOGIC, not assumptions:**
>
> 1. **Security Issues Summary** — Identify any remaining vulnerabilities, bypasses, or insufficient mitigations in the implemented security controls.
>
> 2. **Code Quality Report** — Assess code clarity, error handling completeness, idiomatic Rust/TypeScript usage, and adherence to project conventions.
>
> 3. **Architecture Assessment** — Evaluate whether the security boundaries are properly enforced, separation of concerns is maintained, and the changes integrate cleanly with existing architecture.
>
> 4. **Test Coverage Analysis** — Identify gaps in test coverage, especially for edge cases in SSRF bypass, symlink handling, filter injection, and streaming limits.
>
> 5. **Performance Observation** — Flag any changes that could cause performance regressions, especially in the embedding context reuse, dynamic batching, and streaming paths.
>
> 6. **Risk Assessment** — Enumerate risks introduced by these changes, including backward compatibility, migration issues, and potential for user confusion.
>
> 7. **Improvement Recommendations** — Provide actionable suggestions for hardening, cleanup, or architectural improvements that should be addressed in future iterations.

---

## Files Changed Summary

**17 files modified**, implementing:
- 6 security phases from SECURITY_ROADMAP.md
- 171 tests passing
- 0 new dependencies added
- Backward compatible (no breaking API changes)
