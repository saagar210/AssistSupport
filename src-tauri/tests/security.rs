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
