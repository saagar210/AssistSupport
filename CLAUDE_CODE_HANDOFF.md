# Claude Code Handoff (AssistSupport)

This file summarizes what was done in this session, current working tree status, and a prioritized implementation plan for Claude Code (Opus 4.5) to move the project forward.

## Scope Recap
- Reviewed frontend + backend structure and ran tests.
- Audited contracts between Rust commands and TS hooks/types.
- Fixed a handful of contract mismatches and robustness issues.
- Identified remaining improvement areas and security hardening needs.
- DirectURN: no references found in this repo.

## What Was Accomplished (Code Changes Applied)
### Backend (Rust/Tauri)
- Added `id` to LLM `ModelInfo` and ensured model id is included when loading models.
  - `src-tauri/src/llm.rs`
  - `src-tauri/src/commands.rs`
- Added input size validation for `generate_with_context` and `generate_streaming`.
  - `src-tauri/src/commands.rs`
- Implemented download cancellation flag and wired it to `cancel_download`.
  - `src-tauri/src/commands.rs`
- Hardened download resume logic: if server ignores Range (200 OK), restart from scratch to avoid file corruption.
  - `src-tauri/src/downloads.rs`

### Frontend (React/TypeScript)
- Fixed embedding model load to use `path` param expected by backend.
  - `src/hooks/useEmbedding.ts`
- Fixed LLM loaded model id tracking to use backend `id`.
  - `src/hooks/useLlm.ts`
- Aligned KB stats/types with backend schema (document_count/chunk_count).
  - `src/types/index.ts`
  - `src/hooks/useKb.ts`
  - `src/hooks/useKb.test.ts`
- Fixed download progress event handling to match backend event payload shape.
  - `src/hooks/useDownload.ts`
- Stabilized SettingsTab tests by preventing hook mock function identity churn.
  - `src/components/Settings/SettingsTab.test.tsx`
- Updated LLM tests for new `ModelInfo.id`.
  - `src/hooks/useLlm.test.ts`

## Tests Run
- `pnpm test` (passed; previously hit an OOM due to test timeouts, resolved by stabilizing SettingsTab mocks)
- `cargo test` in `src-tauri` (passed; 1 ignored)

## Notes on the Working Tree
The following files were modified in this session:
- `src-tauri/src/commands.rs`
- `src-tauri/src/downloads.rs`
- `src-tauri/src/llm.rs`
- `src/components/Settings/SettingsTab.test.tsx`
- `src/hooks/useDownload.ts`
- `src/hooks/useEmbedding.ts`
- `src/hooks/useKb.test.ts`
- `src/hooks/useKb.ts`
- `src/hooks/useLlm.test.ts`
- `src/hooks/useLlm.ts`
- `src/types/index.ts`

If you want all changes re-applied via Claude Code, either:
1) keep these changes and continue from here, or
2) revert these changes and re-implement in Claude Code.

## Priority Backlog (Recommended)
### P0 (Release Safety / Security)
- Jira validation in backend commands: validate base URL + ticket key and surface clear errors.
  - Use `validate_url` and `validate_ticket_id`.
  - `src-tauri/src/commands.rs`
- OCR base64 size cap: reject oversized payloads to prevent memory spikes.
  - `src-tauri/src/commands.rs` (use `validate_text_size` or add a dedicated max size constant)
- CSP hardening: avoid `csp: null` in production. Set a minimal allowlist CSP.
  - `src-tauri/tauri.conf.json`

### P1 (Data Safety / UX)
- Backup encryption: use `ExportCrypto` for password-protected backups and update import flow.
  - `src-tauri/src/backup.rs`, `src-tauri/src/security.rs`
- Download UX: per-model progress and a cancel button wired to `cancel_download`.
  - `src/components/Settings/SettingsTab.tsx`, `src/hooks/useDownload.ts`

### P2 (Capabilities / Quality)
- Add UI support for loading custom GGUF models.
  - `src/components/Settings/SettingsTab.tsx`, `src-tauri/src/commands.rs`
- Prompt budget enforcement: guard against prompt size exceeding model context window.
  - `src-tauri/src/prompts.rs`, `src-tauri/src/commands.rs`

## Implementation Plan (Claude Code)
### Phase 1: Security & Input Hardening (P0)
1. Validate Jira config:
   - Apply `validate_url` in `configure_jira`.
   - Apply `validate_ticket_id` in `get_jira_ticket`.
   - Ensure error messages are user-facing and consistent.
2. Add OCR base64 size cap:
   - Introduce a max constant (ex: 10MB) and reject inputs above it.
   - Return a friendly error string.
3. CSP hardening:
   - Replace `csp: null` with a minimal CSP that still allows app assets.

Acceptance:
- Invalid Jira URLs and ticket IDs fail fast with clear errors.
- OCR paste of large images returns a controlled error.
- App still loads with CSP enabled.

### Phase 2: Backup Encryption (P1)
1. Add optional password in export flow:
   - Prompt user for password via dialog (if feasible).
   - Encrypt backup data using `ExportCrypto`.
2. Import flow:
   - Support decrypting backup when password is provided.

Acceptance:
- Backups can be created with/without password.
- Encrypted backups import only with correct password.

### Phase 3: Download UX + Cancel (P1)
1. Add cancel button in Settings download UI.
2. Wire to `cancel_download` and ensure UI state updates on cancel.
3. Update progress UI to show accurate totals and speeds.

Acceptance:
- Download can be cancelled; UI reflects cancellation state.
- Progress bar is accurate and per-model.

### Phase 4: Custom Model UI + Prompt Budget (P2)
1. Add a file picker to load custom GGUF.
2. Surface validation errors from `validate_gguf_file`.
3. Add context window enforcement in prompt builder.

Acceptance:
- Users can load custom GGUF, validation errors are shown.
- Prompt building respects configured context window.

## Progress Notes

### Phase 1 Complete (2026-01-25)
**Security & Input Hardening (P0)** - All items implemented:

1. **Jira validation in backend commands:**
   - Added `validate_url(&base_url)` check to `configure_jira` command
   - Added `validate_ticket_id(&ticket_key)` check to `get_jira_ticket` command
   - Both return clear user-facing error messages on validation failure

2. **OCR base64 size cap:**
   - Added `MAX_OCR_BASE64_BYTES` constant (10MB)
   - Added size check in `process_ocr_bytes` before decoding
   - Returns friendly error: "Image too large: X bytes exceeds limit of Y bytes. Please use a smaller image."

3. **CSP hardening:**
   - Replaced `"csp": null` with minimal CSP:
     `default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data: blob:; font-src 'self' data:; connect-src 'self' https: ipc: tauri:`
   - Allows: app assets, inline styles (for UI libs), data URIs for images/fonts, HTTPS connections (HuggingFace, Jira), and Tauri IPC

### Phase 2 Complete (2026-01-25)
**Backup Encryption (P1)** - All items implemented:

1. **Optional password-based encryption for backup export:**
   - Added `EncryptedBackupHeader` struct with magic marker, version, salt, and nonce
   - Modified `export_backup()` to accept optional password parameter
   - Encrypted backups use Argon2id + AES-256-GCM via `ExportCrypto`
   - Encrypted files saved with `.enc` extension, unencrypted as `.zip`
   - `ExportSummary` now includes `encrypted: bool` field

2. **Decryption support for import:**
   - Added `is_encrypted_backup()` and `decrypt_backup()` helper functions
   - Modified `preview_import()` to detect encrypted backups and require password
   - Modified `import_backup()` to decrypt with provided password
   - `ImportPreview` now includes `encrypted: bool` and optional `path` field
   - Returns `BackupError::EncryptionRequired` when password needed but not provided
   - Returns `BackupError::DecryptionFailed` for wrong password

3. **Backend command updates:**
   - `export_backup`, `preview_backup_import`, `import_backup` now accept optional `password` parameter
   - File dialogs updated to show both `.zip` and `.enc` file filters

### Phase 3 Complete (2026-01-25)
**Download UX + Cancel (P1)** - All items implemented:

1. **Cancel button added to download UI:**
   - Added `cancelDownload()` function to `useDownload` hook
   - Wired to backend `cancel_download` command
   - Cancel button appears during active downloads

2. **Enhanced download progress display:**
   - Added `formatBytes()` and `formatSpeed()` helper functions
   - Progress UI now shows downloaded/total bytes and speed (e.g., "1.2 MB / 2.0 GB" and "5.6 MB/s")
   - Applied to both LLM model downloads and embedding model downloads

3. **CSS enhancements:**
   - Added `.download-progress-container`, `.download-info`, `.download-size`, `.download-speed`, `.download-cancel-btn` styles

### Phase 4 Complete (2026-01-25)
**Custom Model UI + Prompt Budget (P2)** - All items implemented:

1. **Custom GGUF model file picker:**
   - Added `handleLoadCustomModel()` handler in SettingsTab
   - Uses file dialog with `.gguf` filter
   - Validates file with `validateGgufFile()` before loading
   - Shows validation errors if file is invalid
   - Added `validateGgufFile()` and `loadCustomModel()` to `useLlm` hook
   - Added `GgufFileInfo` type to frontend types

2. **UI for custom model loading:**
   - Added "Custom Model" section in Settings with "Select GGUF File..." button
   - Added CSS for `.custom-model-section`

3. **Context window enforcement in prompt builder:**
   - Added `PromptBudgetError` enum for context window violations
   - Added `context_window: Option<usize>` field to `PromptContext`
   - Added `with_context_window()` method to `PromptBuilder`
   - Added `build_with_budget()` method that:
     - Reserves 25% of context window for model response
     - Progressively removes lowest-scoring KB results if prompt exceeds budget
     - Returns `PromptBudgetError::ExceedsContextWindow` if minimum prompt exceeds limit

**All Tests Pass:** 59 Rust tests, 72 frontend tests.

## Summary
All phases (P0, P1, P2) from the implementation plan have been completed:
- ✅ Phase 1: Security & Input Hardening (P0)
- ✅ Phase 2: Backup Encryption (P1)
- ✅ Phase 3: Download UX + Cancel (P1)
- ✅ Phase 4: Custom Model UI + Prompt Budget (P2)

## Additional Notes
- DirectURN: no occurrences in this repo; if it lives elsewhere, provide reference.
- Tests currently pass after the applied fixes.
- Consider adding a small set of integration tests for download progress events and Jira validation.
