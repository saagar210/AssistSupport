# AssistSupport Security Documentation

**Version**: 1.0
**Last Updated**: 2026-01-25
**Status**: Active

---

## Overview

AssistSupport is designed with a **local-first, security-conscious** architecture. This document outlines the security measures, threat model, and best practices for deployment.

---

## Security Architecture

### 1. Data at Rest Encryption

#### Database Encryption
- **Technology**: SQLCipher with AES-256-CBC
- **Key Management**: 256-bit master key (Keychain or passphrase-wrapped)
- **Database Location**: `~/Library/Application Support/AssistSupport/assistsupport.db`
- **File Permissions**: 0600 (owner read/write only)
- **Directory Permissions**: 0700 (owner access only)

#### Token Encryption
- **Technology**: AES-256-GCM with authenticated encryption
- **Storage**: Encrypted `tokens.json` file
- **Protected Tokens**: HuggingFace API token, Jira API token
- **Key Derivation**: Master key used for token encryption

### 2. Key Management

#### Master Key Generation
- **Algorithm**: Cryptographically secure random generation using `OsRng`
- **Key Size**: 256 bits (32 bytes)
- **Storage Format**: Hex-encoded (64 characters)

#### Key Storage Modes

AssistSupport supports two key storage modes:

| Mode | Storage | Security Level | Use Case |
|------|---------|----------------|----------|
| **Keychain** | macOS Keychain | OS-protected, biometric unlock | Default on macOS |
| **Passphrase** | Encrypted file | User-controlled, portable | Cross-platform, offline |

**Keychain Mode** (default on macOS):
- Key stored in system Keychain under `AssistSupport.master-key`
- Protected by macOS security (TouchID, password)
- Automatically unlocked when user is logged in

**Passphrase Mode**:
- Key wrapped with AES-256-GCM, protected by Argon2id-derived key
- Stored in `~/Library/Application Support/AssistSupport/master.key.wrap`
- User must enter passphrase on each app launch
- Portable across systems

#### Key Migration

Legacy plaintext keys are automatically migrated:
1. First, attempts migration to macOS Keychain
2. If Keychain unavailable, migrates to passphrase-protected file
3. Legacy plaintext file is securely deleted after migration

#### Key Rotation

Key rotation re-encrypts all protected data:
- Database is re-keyed with new master key
- Tokens are re-encrypted
- Audit event logged

#### Key Derivation (for exports and passphrase mode)
- **Algorithm**: Argon2id
- **Memory Cost**: 64 MiB
- **Time Cost**: 3 iterations
- **Parallelism**: 4 threads
- **Salt Size**: 32 bytes

### 3. Secure String Handling

- **Implementation**: `SecureString` type with `zeroize` trait
- **Behavior**: Memory is zeroed on drop to prevent exposure
- **Debug Output**: Redacted to prevent accidental logging

---

## Path Security

### Path Traversal Protection

All file operations are validated against path traversal attacks:

1. **Home Directory Restriction**: All user-accessible paths must be within `$HOME`
2. **Blocked Directories**:
   - `.ssh/` - SSH keys and config
   - `.gnupg/` - GPG keys
   - `Library/Keychains/` - macOS Keychain
   - System directories (`/System`, `/Library`, `/usr`)
3. **Symlink Validation**: Symlinks are resolved before validation

### Input Validation

- **Text Size Limits**: Maximum query size enforced
- **URL Validation**: SSRF prevention for web ingestion
- **Ticket ID Format**: Regex validation for Jira tickets

---

## Network Security

### Content Security Policy (CSP)

```
default-src 'self';
script-src 'self';
style-src 'self' 'unsafe-inline';
img-src 'self' data: blob:;
connect-src 'self' https: ipc: tauri:;
```

### Network Controls

- **Local-First**: All core functionality works offline
- **Explicit Opt-In**: Network requests require user action
- **HTTPS Required**: All external connections require HTTPS by default
- **HTTP Opt-In**: HTTP connections (e.g., for local Jira) require explicit user acknowledgment
- **No Telemetry**: No data sent to third parties

### Jira HTTPS Enforcement

Jira connections require HTTPS by default. HTTP connections:
- Require explicit `allow_http` parameter
- Trigger security audit log entry
- Store user's HTTP opt-in preference
- Display warning in UI

This protects credentials from network interception while allowing local/development Jira instances.

### External Connections

| Destination | Purpose | When |
|-------------|---------|------|
| huggingface.co | Model downloads | User-initiated |
| Jira instance | Ticket fetching | User-configured |
| YouTube | Transcript extraction | User-initiated |
| GitHub | Repo ingestion | User-initiated |

---

## Audit Logging

### Overview

AssistSupport maintains an audit log for security-relevant events. The log is stored as JSON lines at:
```
~/Library/Application Support/AssistSupport/audit.log
```

### Log Features

- **Format**: JSON lines (one JSON object per line)
- **Rotation**: Max 5MB per file, keeps 5 rotated files
- **Thread-safe**: Safe for concurrent writes
- **No Secrets**: Never logs tokens, keys, or passwords

### Logged Events

| Event Type | Severity | Description |
|------------|----------|-------------|
| `key_generated` | Info | Master key created |
| `key_migrated` | Info | Key moved between storage modes |
| `key_rotated` | Info | Master key rotated |
| `key_storage_mode_changed` | Info | Storage mode changed |
| `token_set` | Info | Token stored (token value not logged) |
| `token_cleared` | Info | Token removed |
| `jira_configured` | Info | Jira integration configured |
| `jira_http_opt_in` | Warning | User opted into insecure HTTP |
| `path_validation_failed` | Warning | Path security check failed |
| `app_initialized` | Info | Application started |
| `database_repaired` | Info | Database repair completed |

### Log Entry Format

```json
{
  "timestamp": "2026-01-25T12:00:00Z",
  "event": "token_set",
  "severity": "info",
  "message": "Token set: huggingface",
  "context": {"secure": true}
}
```

---

## Model Integrity

### SHA256 Verification

All downloaded models are verified against SHA256 checksums:

1. **Download Phase**: Hash calculated during download
2. **Verification**: Compared against expected hash (when available)
3. **Allowlist**: Known-good models have pre-verified hashes

### Model Allowlist

Verified models with known-good hashes:

| Model | Repository | Status |
|-------|------------|--------|
| Qwen2.5-7B-Instruct | Qwen/Qwen2.5-7B-Instruct-GGUF | Verified |
| Llama-3.2-3B-Instruct | bartowski/Llama-3.2-3B-Instruct-GGUF | Verified |
| nomic-embed-text-v1.5 | nomic-ai/nomic-embed-text-v1.5-GGUF | Verified |

### Custom Models

Users may load custom `.gguf` models. These are:
- Validated as valid GGUF format
- Hash calculated and displayed
- Marked as "Unverified" if not in allowlist

---

## Threat Model

### In Scope

| Threat | Mitigation |
|--------|------------|
| Unauthorized database access | SQLCipher encryption |
| Token exposure in memory | SecureString with zeroize |
| Path traversal attacks | Home directory restriction |
| Malicious model files | SHA256 verification + allowlist |
| SSRF via web ingestion | URL validation |
| XSS in rendered content | CSP + sanitization |

### Out of Scope

| Threat | Reason |
|--------|--------|
| Physical device access | Requires OS-level protection |
| Root/admin compromise | Requires OS-level protection |
| Memory forensics | Beyond app-level protection |
| Side-channel attacks | Requires hardware mitigation |

---

## Security Checklist

### Pre-Release

- [ ] All tests pass (security, path validation, encryption)
- [ ] No hardcoded credentials in codebase
- [ ] Dependencies audited for vulnerabilities
- [ ] File permissions set correctly
- [ ] CSP headers configured

### Periodic Audit

- [ ] Run `cargo audit` for Rust dependencies
- [ ] Run `npm audit` for frontend dependencies
- [ ] Review new model hashes for allowlist
- [ ] Check for new CVEs in dependencies
- [ ] Test path validation edge cases

---

## Dependency Security

### Rust Dependencies

Run security audit:
```bash
cargo install cargo-audit
cargo audit
```

### Frontend Dependencies

Run security audit:
```bash
npm audit
```

### Critical Dependencies

| Dependency | Purpose | Security Notes |
|------------|---------|----------------|
| aes-gcm | Encryption | Audited, widely used |
| argon2 | Key derivation | OWASP recommended |
| rusqlite | Database | SQLCipher build |
| reqwest | HTTP client | TLS verification |

---

## Incident Response

### If Credentials Exposed

1. Revoke exposed tokens immediately
2. Generate new tokens at source (HuggingFace, Jira)
3. Delete and regenerate master key:
   ```bash
   # Delete all key storage files
   rm ~/Library/Application\ Support/AssistSupport/master.key
   rm ~/Library/Application\ Support/AssistSupport/master.key.wrap
   rm ~/Library/Application\ Support/AssistSupport/tokens.json

   # If using Keychain mode, also delete from Keychain
   security delete-generic-password -s "AssistSupport" -a "master-key" 2>/dev/null
   ```
4. Restart application (new key generated automatically)
5. Review audit log for suspicious activity:
   ```bash
   cat ~/Library/Application\ Support/AssistSupport/audit.log | jq '.'
   ```

### If Database Compromised

1. Delete encrypted database
2. Re-index knowledge base
3. Review what data may have been exposed

---

## Security Contacts

For security issues, please report via:
- GitHub Security Advisories (private)
- Direct email to maintainers

**Do not** report security vulnerabilities in public issues.

---

---

## Diagnostics and Health Monitoring

### System Health Checks

AssistSupport includes built-in health monitoring for all critical components:

| Component | Health Check | Auto-Repair |
|-----------|--------------|-------------|
| Database | Integrity check, table validation | Repair command available |
| Vector Store | Index verification, count validation | Rebuild guidance |
| LLM Engine | Model loaded, memory check | Reload suggestion |
| Embedding Engine | Model loaded, test embedding | Reload suggestion |
| Filesystem | Permissions, disk space | N/A |

### Available Commands

- `get_system_health`: Comprehensive health report
- `run_quick_health_check`: Fast status overview
- `repair_database_cmd`: Attempt database repair
- `rebuild_vector_store`: Guidance for vector store rebuild
- `get_failure_modes_cmd`: Known issues with remediation steps

### Recovery Workflows

For the top 5 failure modes, AssistSupport provides clear remediation:

1. **Database corruption**: Run repair command, or delete and re-initialize
2. **Model load failure**: Verify GGUF integrity, check RAM, try smaller model
3. **Embedding failures**: Reload embedding model, verify vector consent
4. **Search degradation**: Rebuild vector index, clear FTS cache
5. **Token corruption**: Delete tokens.json, re-enter credentials

---

## Changelog

### v1.3 (2026-01-25)
- **SSRF Hardening**: Added IPv6-mapped IPv4 detection and cloud metadata endpoint blocking
- **Symlink Protection**: KB indexer now skips symlinks to prevent traversal attacks
- **Filter Injection Prevention**: Vector store sanitizes all filter inputs
- **SQLite Safety**: Enabled foreign keys, busy timeout, WAL mode, and secure delete
- **File Size Limits**: Local file indexing enforces size limits (10MB text, 50MB binary)
- **Streaming Memory Bounds**: Web ingestion and frontend streaming have size limits
- **KEK Zeroization**: Passphrase-derived keys are explicitly zeroized after use
- **Audit Coverage**: Token set/clear operations now logged
- **Namespace ID Validation**: Enforced slug format `[a-z0-9-]{1,64}` with auto-normalization
- **LLM Dynamic Batch Sizing**: Batch size scales with prompt length for better performance
- **Embedding Context Reuse**: Improved batch embedding performance

### v1.2 (2026-01-25)
- Added dual key storage modes (Keychain and Passphrase)
- Added audit logging for security events
- Added HTTPS enforcement for Jira with explicit HTTP opt-in
- Enhanced key migration and rotation support
- Updated incident response procedures

### v1.1 (2026-01-25)
- Added diagnostics and health monitoring section
- Documented self-repair commands
- Added recovery workflows for top 5 failure modes

### v1.0 (2026-01-25)
- Initial security documentation
- Model integrity verification
- File-based key storage
- Path traversal protection
