# Phase 0B: Platform + Security Spike Results

**Date**: 2026-01-24
**Status**: VALIDATED

## SQLCipher + FTS5

### Build Configuration
- Using `rusqlite` with `bundled-sqlcipher-vendored-openssl` feature
- FTS5 enabled via bundled SQLCipher build

### Verification Tests
| Test | Result | Notes |
|------|--------|-------|
| `test_fts5_available` | PASS | FTS5 virtual tables work |
| `test_fts5_indexing` | PASS | Search returns correct results |
| `test_fts5_triggers` | PASS | INSERT/UPDATE/DELETE triggers sync FTS5 |
| `test_sqlcipher_encryption` | PASS | Plaintext not visible in raw file |
| `test_wrong_key_fails` | PASS | Cannot open with incorrect key |

### FTS5 Verification Command
```rust
// In-app verification (release gate)
#[tauri::command]
pub fn check_fts5_enabled(state: State<'_, AppState>) -> Result<bool, String>
```

## Keychain Integration

### Configuration
- Service: `com.d.assistsupport`
- Uses `keyring` crate v3 (macOS Keychain backend)

### Entries
| Entry | Purpose |
|-------|---------|
| `master-key` | 32-byte SQLCipher encryption key |
| `huggingface-token` | Optional HuggingFace API token |

### Verification
| Test | Result |
|------|--------|
| `test_keychain_available` | PASS |
| Store/retrieve master key | PASS |
| Delete credentials | PASS |

## LanceDB Encryption

### Status: NOT SUPPORTED
LanceDB 0.17 does not support native encryption for data at rest.

### Fallback Strategy
1. Vector search **disabled by default**
2. Explicit user consent required to enable unencrypted storage
3. Consent persisted in `vector_consent` table
4. Settings UI will show warning and require opt-in

### Verification
| Test | Result |
|------|--------|
| `test_encryption_not_supported` | PASS |
| `test_vector_store_disabled_by_default` | PASS |
| `test_enable_requires_consent` | PASS |

## OCR Provider Architecture

### macOS Default: Vision
- Uses Vision framework (macOS 10.15+)
- Helper binary approach (bundled in Resources)
- Fallback: AppleScript/osascript (limited)

### Optional: Tesseract
- Feature-gated (`tesseract` Cargo feature)
- Requires bundled tessdata (English only)
- Separate DMG build if needed

### Verification
| Test | Result |
|------|--------|
| `test_ocr_manager_creation` | PASS |
| `test_vision_availability` | PASS (macOS) |

### Verification
| Test | Result | Notes |
|------|--------|-------|
| `test_vision_availability` | PASS | Vision reports available on macOS |
| `test_vision_ocr_integration` | PASS | 100% confidence on test image |
| Helper binary compiled | PASS | `helpers/vision_ocr` (116KB) |
| JSON output parsing | PASS | Returns text, confidence, bounding boxes |

### TODO
- [x] Build Vision helper Swift binary
- [ ] Bundle PDFium dylib
- [ ] Create OCR benchmark dataset
- [ ] Run WER evaluation

## PDFium Integration

### Status: VALIDATED
- Using `pdfium-render` crate v0.8
- Bundled `libpdfium.dylib` (5.5MB) from bblanchon/pdfium-binaries
- No system dependency required

### Verification
| Test | Result | Notes |
|------|--------|-------|
| `test_pdfium_availability` | PASS | Library found at resources/pdfium/libpdfium.dylib |
| `test_pdf_text_extraction` | PASS | Extracted text correctly from test PDF |
| `test_pdf_page_count` | PASS | Page count accurate |

### Configuration
- Library source: https://github.com/bblanchon/pdfium-binaries/releases
- Platform: macOS arm64
- Bundled in: `resources/pdfium/libpdfium.dylib`

## Unsigned DMG Distribution

### Status: VALIDATED
- Sandbox disabled for v1
- No notarization required for internal distribution
- Quarantine removal documented in docs/INSTALLATION.md

### Build Output
- DMG: `AssistSupport_0.1.0_aarch64.dmg` (7.7MB)
- App Bundle: `AssistSupport.app`
- Bundled Resources: Vision OCR helper, PDFium dylib

### Verification
| Test | Result | Notes |
|------|--------|-------|
| DMG created | PASS | 7.7MB, signed checksum valid |
| DMG mounts | PASS | Contains .app + Applications symlink |
| Resources bundled | PASS | helpers (116KB), pdfium (5.5MB) |
| App launches from DMG | PASS | Process starts successfully |

### Installation Methods
1. Right-click > Open (recommended)
2. `xattr -d com.apple.quarantine /Applications/AssistSupport.app`
3. System Settings > Privacy & Security > Open Anyway

## Security Dependencies

All dependencies verified on crates.io:
- `aes-gcm` 0.10 - AES-256-GCM encryption
- `argon2` 0.5 - Argon2id key derivation
- `rand` 0.8 - Cryptographic RNG
- `zeroize` 1.x - Memory zeroization
- `keyring` 3.x - Keychain access

## Test Summary

**Total Tests**: 19
**Passed**: 19
**Failed**: 0

```
test kb::ocr::tests::test_ocr_manager_creation ... ok
test kb::ocr::tests::test_vision_availability ... ok
test kb::vectors::tests::test_encryption_not_supported ... ok
test kb::vectors::tests::test_vector_store_disabled_by_default ... ok
test kb::vectors::tests::test_enable_requires_consent ... ok
test kb::vectors::tests::test_encryption_status ... ok
test security::tests::test_encryption_roundtrip ... ok
test security::tests::test_master_key_generation ... ok
test security::tests::test_key_wrapping ... ok
test security::tests::test_wrong_passphrase_fails ... ok
test security::tests::test_export_crypto ... ok
test db::tests::test_fts5_available ... ok
test db::tests::test_fts5_indexing ... ok
test db::tests::test_fts5_triggers ... ok
test db::tests::test_sqlcipher_encryption ... ok
test db::tests::test_wrong_key_fails ... ok
test db::tests::test_database_creation ... ok
test db::tests::test_vector_consent ... ok
test tests::test_keychain_available ... ok
```

## Next Steps

1. Build Vision OCR helper binary
2. Bundle PDFium dylib
3. Create OCR benchmark dataset (>=50 pages, >=30% image-only)
4. Run WER evaluation with jiwer
5. Validate unsigned DMG installation
