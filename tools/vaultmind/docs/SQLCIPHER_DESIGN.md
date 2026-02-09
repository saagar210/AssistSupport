# SQLCipher At-Rest Encryption — Design Document

## Status: DESIGN ONLY (Not Implemented)

## Problem

VaultMind currently uses `PRAGMA secure_delete = ON` and `PRAGMA journal_mode = WAL`, but the SQLite database file is **not encrypted**. Any process with filesystem access can read all user documents, embeddings, chat history, and audit logs.

AssistSupport already ships SQLCipher (AES-256-CBC). VaultMind needs parity for monorepo integration.

## Proposed Architecture

### Option A: SQLCipher via rusqlite feature flag (Recommended)

**Cargo.toml change:**
```toml
# FROM:
rusqlite = { version = "0.38.0", features = ["bundled", "modern-full"] }
# TO:
rusqlite = { version = "0.38.0", features = ["bundled-sqlcipher", "modern-full"] }
```

**Key management:** Already implemented via `keyring` crate (macOS Keychain). The `crypto.rs` module has `get_or_create_db_key()` and `rotate_db_key()` — currently used for PRAGMA secure_delete only. Would be extended to apply `PRAGMA key` on connection open.

**Connection flow:**
1. Pool checkout via `ConnectionCustomizer`
2. Apply `PRAGMA key = '{key_from_keychain}'`
3. Apply existing PRAGMAs (WAL, FK, busy_timeout, secure_delete)
4. Connection ready

**Migration from unencrypted:**
```sql
-- One-time migration
ATTACH DATABASE 'encrypted.db' AS encrypted KEY 'key';
SELECT sqlcipher_export('encrypted');
DETACH DATABASE encrypted;
-- Swap files
```

### Option B: Application-level encryption (AES-256-GCM per field)

Encrypt sensitive columns (document text, chat messages) at the application layer. Leaves metadata, schemas, and indexes readable.

**Not recommended** — partial coverage, complex key management per field, breaks FTS5 and vector search.

### Risk Assessment

| Risk | Mitigation |
|------|------------|
| `bundled-sqlcipher` changes compiled SQLite | Pin exact rusqlite version, test thoroughly |
| r2d2_sqlite 0.32 compatibility | Uses same rusqlite 0.38 — should work |
| Performance impact (AES overhead) | ~5-10% on writes, negligible on reads (hardware AES on M-series) |
| Key loss = data loss | Keychain backup warning in onboarding, key export option |
| Existing unencrypted DB migration | Automated one-time migration on first launch post-update |

### Blockers

1. **Testing required** — `bundled-sqlcipher` feature must compile clean with existing test suite
2. **r2d2_sqlite compatibility** — needs verification that `ConnectionCustomizer` can apply PRAGMA key
3. **CI/CD** — SQLCipher adds ~30s to compile time (OpenSSL dependency)

### Estimated Effort

2-3 days for implementation + testing. Low code change volume but high blast radius.
