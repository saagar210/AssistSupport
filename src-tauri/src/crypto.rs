use crate::error::AppError;

const SERVICE_NAME: &str = "com.vaultmind.app";
const KEY_ENTRY_NAME: &str = "db-encryption-key";

/// Get or create the database encryption key from the OS keychain.
/// On macOS, this uses the Keychain. On other platforms, uses the
/// platform-native credential store.
///
/// The key is a 64-character hex string (256 bits) suitable for
/// AES-256 or SQLCipher PRAGMA key.
pub fn get_or_create_db_key() -> Result<String, AppError> {
    let entry = keyring::Entry::new(SERVICE_NAME, KEY_ENTRY_NAME)
        .map_err(|e| AppError::Crypto(format!("Failed to create keyring entry: {}", e)))?;

    match entry.get_password() {
        Ok(key) => Ok(key),
        Err(keyring::Error::NoEntry) => {
            let key = generate_key();
            entry
                .set_password(&key)
                .map_err(|e| AppError::Crypto(format!("Failed to store key in keychain: {}", e)))?;
            tracing::info!("Generated and stored new encryption key in OS keychain");
            Ok(key)
        }
        Err(e) => Err(AppError::Crypto(format!(
            "Failed to retrieve key from keychain: {}",
            e
        ))),
    }
}

/// Rotate the encryption key. Generates a new key and stores it
/// in the OS keychain, replacing the old one.
/// Returns the new key.
pub fn rotate_db_key() -> Result<String, AppError> {
    let entry = keyring::Entry::new(SERVICE_NAME, KEY_ENTRY_NAME)
        .map_err(|e| AppError::Crypto(format!("Failed to create keyring entry: {}", e)))?;

    let new_key = generate_key();
    entry
        .set_password(&new_key)
        .map_err(|e| AppError::Crypto(format!("Failed to store rotated key: {}", e)))?;

    tracing::info!("Encryption key rotated successfully");
    Ok(new_key)
}

/// Check whether an encryption key exists in the OS keychain.
pub fn has_db_key() -> bool {
    let entry = match keyring::Entry::new(SERVICE_NAME, KEY_ENTRY_NAME) {
        Ok(e) => e,
        Err(_) => return false,
    };
    entry.get_password().is_ok()
}

/// Delete the encryption key from the OS keychain.
/// Used during "erase all data" operations.
pub fn delete_db_key() -> Result<(), AppError> {
    let entry = keyring::Entry::new(SERVICE_NAME, KEY_ENTRY_NAME)
        .map_err(|e| AppError::Crypto(format!("Failed to create keyring entry: {}", e)))?;

    match entry.delete_credential() {
        Ok(()) => {
            tracing::info!("Encryption key deleted from OS keychain");
            Ok(())
        }
        Err(keyring::Error::NoEntry) => Ok(()), // Already gone
        Err(e) => Err(AppError::Crypto(format!(
            "Failed to delete key from keychain: {}",
            e
        ))),
    }
}

/// Get encryption status information.
pub fn get_encryption_status() -> EncryptionStatus {
    EncryptionStatus {
        key_in_keychain: has_db_key(),
        secure_delete_enabled: true, // Always enabled via PRAGMA
        encryption_method: "PRAGMA secure_delete + OS Keychain".to_string(),
    }
}

/// Generate a cryptographically random 256-bit key as a hex string.
fn generate_key() -> String {
    use std::fmt::Write;
    let mut bytes = [0u8; 32];
    // Use getrandom (available via std on all supported platforms)
    getrandom::fill(&mut bytes).expect("Failed to generate random bytes");
    let mut hex = String::with_capacity(64);
    for b in &bytes {
        write!(hex, "{:02x}", b).expect("hex write failed");
    }
    hex
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EncryptionStatus {
    pub key_in_keychain: bool,
    pub secure_delete_enabled: bool,
    pub encryption_method: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_key_length() {
        let key = generate_key();
        assert_eq!(key.len(), 64, "Key should be 64 hex chars (256 bits)");
    }

    #[test]
    fn test_generate_key_is_hex() {
        let key = generate_key();
        assert!(
            key.chars().all(|c| c.is_ascii_hexdigit()),
            "Key should be valid hex"
        );
    }

    #[test]
    fn test_generate_key_uniqueness() {
        let key1 = generate_key();
        let key2 = generate_key();
        assert_ne!(key1, key2, "Two generated keys should differ");
    }

    #[test]
    fn test_encryption_status() {
        let status = get_encryption_status();
        assert!(status.secure_delete_enabled);
        assert!(!status.encryption_method.is_empty());
    }
}
