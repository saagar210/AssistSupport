//! Security integration tests
//!
//! Tests for encryption, key handling, FileKeyStore operations,
//! and database encryption verification.

mod common;

use assistsupport_lib::db::Database;
use assistsupport_lib::security::{
    Crypto, ExportCrypto, FileKeyStore, MasterKey, TOKEN_HUGGINGFACE, TOKEN_JIRA,
};
use std::fs;
use tempfile::TempDir;

// ============================================================================
// Master Key Tests
// ============================================================================

#[test]
fn test_master_key_generation_unique() {
    // Generate multiple keys and ensure they're all different
    let keys: Vec<MasterKey> = (0..5).map(|_| MasterKey::generate()).collect();

    for i in 0..keys.len() {
        for j in (i + 1)..keys.len() {
            assert_ne!(
                keys[i].as_bytes(),
                keys[j].as_bytes(),
                "Generated keys should be unique"
            );
        }
    }
}

#[test]
fn test_master_key_length() {
    let key = MasterKey::generate();
    assert_eq!(key.as_bytes().len(), 32, "Master key should be 32 bytes");
}

#[test]
fn test_master_key_hex_encoding() {
    let key = MasterKey::generate();
    let hex = key.to_hex();

    // Hex encoding should be 64 characters (32 bytes * 2)
    assert_eq!(hex.len(), 64, "Hex encoding should be 64 characters");

    // Should be valid hex
    assert!(
        hex.chars().all(|c| c.is_ascii_hexdigit()),
        "Should be valid hexadecimal"
    );

    // Should be lowercase
    assert_eq!(hex, hex.to_lowercase(), "Hex should be lowercase");
}

#[test]
fn test_master_key_from_bytes() {
    let original = MasterKey::generate();
    let bytes = *original.as_bytes();

    let restored = MasterKey::from_bytes(bytes);
    assert_eq!(
        original.as_bytes(),
        restored.as_bytes(),
        "Key should be restorable from bytes"
    );
}

// ============================================================================
// Database Encryption Tests
// ============================================================================

#[test]
fn test_database_encrypted_on_disk() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("encrypted.db");
    let key = MasterKey::generate();

    // Create and initialize database
    {
        let db = Database::open(&db_path, &key).expect("Failed to open database");
        db.initialize().expect("Failed to initialize");

        // Insert some test data
        db.conn()
            .execute("INSERT INTO settings (key, value) VALUES (?, ?)", ["test_key", "test_value"])
            .expect("Failed to insert");
    }

    // Read raw file content
    let raw_content = fs::read(&db_path).expect("Failed to read db file");

    // SQLite3 unencrypted files start with "SQLite format 3\0"
    assert!(
        !raw_content.starts_with(b"SQLite format 3"),
        "Database file should be encrypted (not start with SQLite magic)"
    );

    // The encrypted content should not contain our test string in plain text
    let content_str = String::from_utf8_lossy(&raw_content);
    assert!(
        !content_str.contains("test_value"),
        "Test data should not be visible in encrypted file"
    );
}

#[test]
fn test_database_wrong_key_fails() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");
    let correct_key = MasterKey::generate();
    let wrong_key = MasterKey::generate();

    // Create database with correct key
    {
        let db = Database::open(&db_path, &correct_key).expect("Failed to open database");
        db.initialize().expect("Failed to initialize");
    }

    // Try to open with wrong key - should fail
    let result = Database::open(&db_path, &wrong_key);
    assert!(
        result.is_err(),
        "Opening with wrong key should fail"
    );
}

#[test]
fn test_database_integrity_check() {
    let ctx = common::TestContext::new().expect("Failed to create context");

    // Insert some data
    ctx.db
        .conn()
        .execute("INSERT INTO settings (key, value) VALUES (?, ?)", ["test", "value"])
        .expect("Failed to insert");

    // Check integrity - returns Ok(()) if successful
    ctx.db.check_integrity().expect("Database integrity check should pass");
}

// ============================================================================
// AES-256-GCM Encryption Tests
// ============================================================================

#[test]
fn test_encryption_roundtrip() {
    let key = MasterKey::generate();
    let plaintext = b"Hello, World! This is a test message.";

    let encrypted = Crypto::encrypt(key.as_bytes(), plaintext).expect("Encryption failed");
    let decrypted = Crypto::decrypt(key.as_bytes(), &encrypted).expect("Decryption failed");

    assert_eq!(
        plaintext.as_slice(),
        decrypted.as_slice(),
        "Decrypted data should match original"
    );
}

#[test]
fn test_encryption_produces_different_ciphertext() {
    let key = MasterKey::generate();
    let plaintext = b"Test message";

    // Encrypt the same plaintext twice
    let encrypted1 = Crypto::encrypt(key.as_bytes(), plaintext).expect("Encryption failed");
    let encrypted2 = Crypto::encrypt(key.as_bytes(), plaintext).expect("Encryption failed");

    // Ciphertext should be different (due to random nonce)
    assert_ne!(
        encrypted1.ciphertext, encrypted2.ciphertext,
        "Same plaintext should produce different ciphertext"
    );

    // Nonces should be different
    assert_ne!(
        encrypted1.nonce, encrypted2.nonce,
        "Nonces should be different"
    );
}

#[test]
fn test_encryption_with_wrong_key_fails() {
    let key1 = MasterKey::generate();
    let key2 = MasterKey::generate();
    let plaintext = b"Secret message";

    let encrypted = Crypto::encrypt(key1.as_bytes(), plaintext).expect("Encryption failed");

    // Decrypt with wrong key should fail
    let result = Crypto::decrypt(key2.as_bytes(), &encrypted);
    assert!(
        result.is_err(),
        "Decryption with wrong key should fail"
    );
}

#[test]
fn test_encryption_empty_plaintext() {
    let key = MasterKey::generate();
    let plaintext = b"";

    let encrypted = Crypto::encrypt(key.as_bytes(), plaintext).expect("Encryption failed");
    let decrypted = Crypto::decrypt(key.as_bytes(), &encrypted).expect("Decryption failed");

    assert!(decrypted.is_empty(), "Empty plaintext should decrypt to empty");
}

#[test]
fn test_encryption_large_data() {
    let key = MasterKey::generate();
    let plaintext: Vec<u8> = (0..1_000_000).map(|i| (i % 256) as u8).collect();

    let encrypted = Crypto::encrypt(key.as_bytes(), &plaintext).expect("Encryption failed");
    let decrypted = Crypto::decrypt(key.as_bytes(), &encrypted).expect("Decryption failed");

    assert_eq!(
        plaintext, decrypted,
        "Large data should encrypt/decrypt correctly"
    );
}

// ============================================================================
// Key Wrapping Tests
// ============================================================================

#[test]
fn test_key_wrapping_with_passphrase() {
    let master_key = MasterKey::generate();
    let passphrase = "my-secure-passphrase-123";

    let wrapped = Crypto::wrap_key(&master_key, passphrase).expect("Wrapping failed");
    let unwrapped = Crypto::unwrap_key(&wrapped, passphrase).expect("Unwrapping failed");

    assert_eq!(
        master_key.as_bytes(),
        unwrapped.as_bytes(),
        "Unwrapped key should match original"
    );
}

#[test]
fn test_key_wrapping_wrong_passphrase_fails() {
    let master_key = MasterKey::generate();
    let wrapped = Crypto::wrap_key(&master_key, "correct-passphrase").expect("Wrapping failed");

    let result = Crypto::unwrap_key(&wrapped, "wrong-passphrase");
    assert!(
        result.is_err(),
        "Unwrapping with wrong passphrase should fail"
    );
}

#[test]
fn test_key_wrapping_produces_different_output() {
    let master_key = MasterKey::generate();
    let passphrase = "test-passphrase";

    // Wrap the same key twice
    let wrapped1 = Crypto::wrap_key(&master_key, passphrase).expect("Wrapping failed");
    let wrapped2 = Crypto::wrap_key(&master_key, passphrase).expect("Wrapping failed");

    // Salts should be different
    assert_ne!(
        wrapped1.salt, wrapped2.salt,
        "Wrapped keys should have different salts"
    );

    // Both should unwrap correctly
    let unwrapped1 = Crypto::unwrap_key(&wrapped1, passphrase).expect("Unwrapping failed");
    let unwrapped2 = Crypto::unwrap_key(&wrapped2, passphrase).expect("Unwrapping failed");

    assert_eq!(unwrapped1.as_bytes(), master_key.as_bytes());
    assert_eq!(unwrapped2.as_bytes(), master_key.as_bytes());
}

// ============================================================================
// Export Crypto Tests
// ============================================================================

#[test]
fn test_export_crypto_roundtrip() {
    let data = b"Export test data with various content: 123!@#";
    let password = "export-password-456";

    let (ciphertext, salt, nonce) =
        ExportCrypto::encrypt_for_export(data, password).expect("Export encryption failed");

    let decrypted =
        ExportCrypto::decrypt_export(&ciphertext, &salt, &nonce, password).expect("Decryption failed");

    assert_eq!(
        data.as_slice(),
        decrypted.as_slice(),
        "Export data should decrypt correctly"
    );
}

#[test]
fn test_export_crypto_wrong_password_fails() {
    let data = b"Secret export data";
    let (ciphertext, salt, nonce) =
        ExportCrypto::encrypt_for_export(data, "correct-password").expect("Export encryption failed");

    let result = ExportCrypto::decrypt_export(&ciphertext, &salt, &nonce, "wrong-password");
    assert!(
        result.is_err(),
        "Export decryption with wrong password should fail"
    );
}

// ============================================================================
// FileKeyStore Tests (isolated using temp directories)
// ============================================================================

// Note: These tests verify the FileKeyStore logic indirectly.
// Direct testing would require modifying the actual app data directory.

#[test]
fn test_token_constants() {
    // Verify token constants are what we expect
    assert_eq!(TOKEN_HUGGINGFACE, "huggingface_token");
    assert_eq!(TOKEN_JIRA, "jira_api_token");
}

#[test]
fn test_file_key_store_has_master_key_false_initially() {
    // This tests the real system, but FileKeyStore::has_master_key()
    // just checks if the file exists, which is safe to call.
    // Note: This may return true if app has been run before.
    let _has_key = FileKeyStore::has_master_key();
    // We just verify it doesn't panic
}

// ============================================================================
// Key Storage Mode Tests
// ============================================================================

#[test]
fn test_key_storage_mode_serialization() {
    use assistsupport_lib::security::KeyStorageMode;

    // Test Display/ToString
    assert_eq!(KeyStorageMode::Keychain.to_string(), "keychain");
    assert_eq!(KeyStorageMode::Passphrase.to_string(), "passphrase");

    // Test serde serialization
    let json_keychain = serde_json::to_string(&KeyStorageMode::Keychain).unwrap();
    let json_passphrase = serde_json::to_string(&KeyStorageMode::Passphrase).unwrap();
    assert_eq!(json_keychain, "\"keychain\"");
    assert_eq!(json_passphrase, "\"passphrase\"");

    // Test serde deserialization
    let mode: KeyStorageMode = serde_json::from_str("\"keychain\"").unwrap();
    assert_eq!(mode, KeyStorageMode::Keychain);
    let mode: KeyStorageMode = serde_json::from_str("\"passphrase\"").unwrap();
    assert_eq!(mode, KeyStorageMode::Passphrase);
}

// ============================================================================
// Audit Logging Tests
// ============================================================================

#[test]
fn test_audit_entry_serialization() {
    use assistsupport_lib::audit::{AuditEntry, AuditEventType, AuditSeverity};

    let entry = AuditEntry::new(
        AuditEventType::TokenSet,
        AuditSeverity::Info,
        "Token set: huggingface",
    );

    let json = serde_json::to_string(&entry).unwrap();
    assert!(json.contains("\"event\":\"token_set\""));
    assert!(json.contains("\"severity\":\"info\""));
    assert!(json.contains("Token set: huggingface"));
    // Most importantly: should not contain actual token value
    assert!(!json.contains("hf_"));
}

#[test]
fn test_audit_entry_with_context() {
    use assistsupport_lib::audit::{AuditEntry, AuditEventType, AuditSeverity};

    let entry = AuditEntry::new(
        AuditEventType::JiraConfigured,
        AuditSeverity::Info,
        "Jira configured",
    )
    .with_context(serde_json::json!({"secure": true}));

    let json = serde_json::to_string(&entry).unwrap();
    assert!(json.contains("\"context\":{\"secure\":true}"));
}

#[test]
fn test_audit_severity_levels() {
    use assistsupport_lib::audit::AuditSeverity;

    // Test display
    assert_eq!(AuditSeverity::Info.to_string(), "info");
    assert_eq!(AuditSeverity::Warning.to_string(), "warning");
    assert_eq!(AuditSeverity::Error.to_string(), "error");
    assert_eq!(AuditSeverity::Critical.to_string(), "critical");
}

#[test]
fn test_audit_event_types() {
    use assistsupport_lib::audit::AuditEventType;

    // Test key events
    assert_eq!(AuditEventType::KeyGenerated.to_string(), "key_generated");
    assert_eq!(AuditEventType::KeyMigrated.to_string(), "key_migrated");
    assert_eq!(AuditEventType::KeyRotated.to_string(), "key_rotated");

    // Test token events
    assert_eq!(AuditEventType::TokenSet.to_string(), "token_set");
    assert_eq!(AuditEventType::TokenCleared.to_string(), "token_cleared");

    // Test Jira events
    assert_eq!(AuditEventType::JiraConfigured.to_string(), "jira_configured");
    assert_eq!(AuditEventType::JiraHttpOptIn.to_string(), "jira_http_opt_in");

    // Test custom events
    assert_eq!(
        AuditEventType::Custom("test".to_string()).to_string(),
        "custom:test"
    );
}

// ============================================================================
// HTTPS Validation Tests
// ============================================================================

#[test]
fn test_validate_https_url() {
    use assistsupport_lib::validation::{validate_https_url, is_http_url};

    // HTTPS should pass
    assert!(validate_https_url("https://example.com").is_ok());
    assert!(validate_https_url("https://jira.company.com/rest/api").is_ok());

    // HTTP should fail
    assert!(validate_https_url("http://example.com").is_err());
    assert!(validate_https_url("http://localhost:8080").is_err());

    // is_http_url helper
    assert!(is_http_url("http://localhost:8080"));
    assert!(!is_http_url("https://example.com"));
    assert!(!is_http_url("ftp://example.com")); // Not HTTP or HTTPS
}

// ============================================================================
// Key Rotation Tests
// ============================================================================

#[test]
fn test_key_wrapping_passphrase_change_simulation() {
    // Simulate changing passphrase by rewrapping with different passphrase
    let master_key = MasterKey::generate();
    let old_passphrase = "old-secure-passphrase";
    let new_passphrase = "new-different-passphrase";

    // Wrap with old passphrase
    let wrapped_old = Crypto::wrap_key(&master_key, old_passphrase).expect("Wrapping failed");

    // Unwrap with old passphrase
    let unwrapped = Crypto::unwrap_key(&wrapped_old, old_passphrase).expect("Unwrapping failed");

    // Wrap with new passphrase (simulates passphrase change)
    let wrapped_new = Crypto::wrap_key(&unwrapped, new_passphrase).expect("Re-wrapping failed");

    // Verify new passphrase works
    let final_key = Crypto::unwrap_key(&wrapped_new, new_passphrase).expect("Final unwrap failed");
    assert_eq!(
        master_key.as_bytes(),
        final_key.as_bytes(),
        "Key should be preserved after passphrase change"
    );

    // Old passphrase should no longer work for new wrap
    let result = Crypto::unwrap_key(&wrapped_new, old_passphrase);
    assert!(result.is_err(), "Old passphrase should not work for new wrapped key");
}

#[test]
fn test_token_encryption_with_different_keys() {
    // Simulate token re-encryption during key rotation
    let old_key = MasterKey::generate();
    let new_key = MasterKey::generate();
    let token = b"hf_secrettoken123456789";

    // Encrypt with old key
    let encrypted = Crypto::encrypt(old_key.as_bytes(), token).expect("Encryption failed");

    // Decrypt with old key
    let decrypted = Crypto::decrypt(old_key.as_bytes(), &encrypted).expect("Decryption failed");
    assert_eq!(token.as_slice(), decrypted.as_slice());

    // Re-encrypt with new key (simulates rotation)
    let re_encrypted = Crypto::encrypt(new_key.as_bytes(), &decrypted).expect("Re-encryption failed");

    // Verify new key works
    let final_decrypted = Crypto::decrypt(new_key.as_bytes(), &re_encrypted).expect("Final decryption failed");
    assert_eq!(token.as_slice(), final_decrypted.as_slice());

    // Old key should not decrypt new ciphertext
    let result = Crypto::decrypt(old_key.as_bytes(), &re_encrypted);
    assert!(result.is_err(), "Old key should not decrypt re-encrypted data");
}

#[test]
fn test_wrapped_key_components() {
    let master_key = MasterKey::generate();
    let passphrase = "test-passphrase";

    let wrapped = Crypto::wrap_key(&master_key, passphrase).expect("Wrapping failed");

    // Verify components have expected sizes
    assert_eq!(wrapped.salt.len(), 32, "Salt should be 32 bytes");
    assert_eq!(wrapped.encrypted_key.nonce.len(), 12, "Nonce should be 12 bytes");
    assert!(!wrapped.encrypted_key.ciphertext.is_empty(), "Ciphertext should not be empty");

    // Verify Argon2 parameters are set
    assert_eq!(wrapped.argon2_memory, 65536, "Argon2 memory should be 64 MiB");
    assert_eq!(wrapped.argon2_time, 3, "Argon2 time should be 3 iterations");
    assert_eq!(wrapped.argon2_parallelism, 4, "Argon2 parallelism should be 4");
}

// ============================================================================
// SSRF Protection Tests (Network Module)
// ============================================================================

#[test]
fn test_ssrf_blocks_localhost_variants() {
    use assistsupport_lib::kb::network::{validate_url_for_ssrf, SsrfConfig};

    let config = SsrfConfig::default();

    // Standard localhost
    assert!(validate_url_for_ssrf("http://localhost/", &config).is_err());
    assert!(validate_url_for_ssrf("http://localhost:8080/", &config).is_err());
    assert!(validate_url_for_ssrf("https://localhost/", &config).is_err());

    // IPv4 loopback
    assert!(validate_url_for_ssrf("http://127.0.0.1/", &config).is_err());
    assert!(validate_url_for_ssrf("http://127.0.0.1:3000/", &config).is_err());

    // IPv6 loopback
    assert!(validate_url_for_ssrf("http://[::1]/", &config).is_err());
    assert!(validate_url_for_ssrf("http://[::1]:8080/", &config).is_err());
}

#[test]
fn test_ssrf_blocks_private_ranges() {
    use assistsupport_lib::kb::network::{validate_url_for_ssrf, SsrfConfig};

    let config = SsrfConfig::default();

    // 10.0.0.0/8
    assert!(validate_url_for_ssrf("http://10.0.0.1/", &config).is_err());
    assert!(validate_url_for_ssrf("http://10.255.255.255/", &config).is_err());

    // 172.16.0.0/12
    assert!(validate_url_for_ssrf("http://172.16.0.1/", &config).is_err());
    assert!(validate_url_for_ssrf("http://172.31.255.255/", &config).is_err());

    // 192.168.0.0/16
    assert!(validate_url_for_ssrf("http://192.168.0.1/", &config).is_err());
    assert!(validate_url_for_ssrf("http://192.168.255.255/", &config).is_err());
}

#[test]
fn test_ssrf_blocks_invalid_schemes() {
    use assistsupport_lib::kb::network::{validate_url_for_ssrf, SsrfConfig};

    let config = SsrfConfig::default();

    // File protocol (dangerous!)
    assert!(validate_url_for_ssrf("file:///etc/passwd", &config).is_err());
    assert!(validate_url_for_ssrf("file:///Users/secret", &config).is_err());

    // FTP
    assert!(validate_url_for_ssrf("ftp://example.com/file.txt", &config).is_err());

    // Gopher (used in SSRF attacks)
    assert!(validate_url_for_ssrf("gopher://localhost:9000/_test", &config).is_err());
}

#[test]
fn test_ssrf_allowlist_bypass() {
    use assistsupport_lib::kb::network::{validate_url_for_ssrf, SsrfConfig};

    let mut config = SsrfConfig::default();
    config.allowlist.push("localhost".into());

    // Allowlisted hosts should be allowed
    assert!(validate_url_for_ssrf("http://localhost/", &config).is_ok());
}

#[test]
fn test_ssrf_metadata_endpoints() {
    use assistsupport_lib::kb::network::{validate_url_for_ssrf, SsrfConfig};

    let config = SsrfConfig::default();

    // AWS metadata endpoint (link-local)
    assert!(validate_url_for_ssrf("http://169.254.169.254/latest/meta-data/", &config).is_err());
}
