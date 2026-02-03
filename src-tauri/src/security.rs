//! Security module for AssistSupport
//! Handles encryption, key management, and credential storage
//!
//! Key storage modes:
//! - Keychain: Master key stored in macOS Keychain (default, most secure)
//! - Passphrase: Master key wrapped with user passphrase (portable, offline backup)
//!
//! Credentials are stored under ~/Library/Application Support/AssistSupport/
//! with restrictive permissions. Tokens are encrypted at rest with the master key.

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use argon2::{Argon2, Params, Version};
use base64::{engine::general_purpose, Engine as _};
use rand::RngCore;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;
use thiserror::Error;
use zeroize::{Zeroize, ZeroizeOnDrop};

const SERVICE_NAME: &str = "AssistSupport";
const MASTER_KEY_ENTRY: &str = "master-key";
const HF_TOKEN_ENTRY: &str = "huggingface-token";
const JIRA_TOKEN_ENTRY: &str = "jira-api-token";

const ARGON2_MEMORY_COST: u32 = 65536; // 64 MiB
const ARGON2_TIME_COST: u32 = 3;
const ARGON2_PARALLELISM: u32 = 4;
const SALT_LEN: usize = 32;
const NONCE_LEN: usize = 12;
const KEY_LEN: usize = 32;

/// Permission mode for directories (owner rwx only)
pub const DIR_PERMISSIONS: u32 = 0o700;

/// Permission mode for private files (owner rw only)
pub const FILE_PERMISSIONS: u32 = 0o600;

/// Set secure permissions on a path (0o700 for directories, 0o600 for files)
/// On non-Unix systems this is a no-op.
#[cfg(unix)]
pub fn set_secure_permissions(path: &Path, mode: u32) -> std::io::Result<()> {
    let perms = fs::Permissions::from_mode(mode);
    fs::set_permissions(path, perms)
}

#[cfg(not(unix))]
pub fn set_secure_permissions(_path: &Path, _mode: u32) -> std::io::Result<()> {
    Ok(())
}

/// Create a directory with secure permissions (0o700).
/// Creates parent directories as needed.
pub fn create_secure_dir(path: &Path) -> std::io::Result<()> {
    fs::create_dir_all(path)?;
    set_secure_permissions(path, DIR_PERMISSIONS)?;
    Ok(())
}

/// Create a directory for private data with 0o700 permissions.
/// Returns the path for chaining.
pub fn ensure_secure_data_dir(path: &Path) -> std::io::Result<&Path> {
    create_secure_dir(path)?;
    Ok(path)
}

/// Key storage mode for master key
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KeyStorageMode {
    /// Store master key in macOS Keychain (default, most secure)
    #[default]
    Keychain,
    /// Store master key wrapped with user passphrase
    Passphrase,
}

impl std::fmt::Display for KeyStorageMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Keychain => write!(f, "keychain"),
            Self::Passphrase => write!(f, "passphrase"),
        }
    }
}

impl std::str::FromStr for KeyStorageMode {
    type Err = SecurityError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "keychain" => Ok(Self::Keychain),
            "passphrase" => Ok(Self::Passphrase),
            _ => Err(SecurityError::InvalidKeyFormat),
        }
    }
}

#[derive(Debug, Error)]
pub enum SecurityError {
    #[error("Keychain error: {0}")]
    Keychain(String),
    #[error("Encryption error: {0}")]
    Encryption(String),
    #[error("Decryption error: {0}")]
    Decryption(String),
    #[error("Key derivation error: {0}")]
    KeyDerivation(String),
    #[error("Master key not found")]
    MasterKeyNotFound,
    #[error("Invalid key format")]
    InvalidKeyFormat,
    #[error("Config directory not found")]
    ConfigDirNotFound,
    #[error("File I/O error: {0}")]
    FileIO(String),
    #[error("Token not found: {0}")]
    TokenNotFound(String),
    #[error("Passphrase required")]
    PassphraseRequired,
    #[error("Key rotation failed: {0}")]
    KeyRotationFailed(String),
}

/// Securely zeroed master key wrapper
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct MasterKey {
    key: [u8; KEY_LEN],
}

impl MasterKey {
    /// Generate a new random master key
    pub fn generate() -> Self {
        let mut key = [0u8; KEY_LEN];
        OsRng.fill_bytes(&mut key);
        Self { key }
    }

    /// Create from existing bytes (takes ownership)
    pub fn from_bytes(bytes: [u8; KEY_LEN]) -> Self {
        Self { key: bytes }
    }

    /// Get reference to key bytes
    pub fn as_bytes(&self) -> &[u8; KEY_LEN] {
        &self.key
    }

    /// Get hex-encoded key for SQLCipher.
    /// SECURITY: Caller MUST zeroize the returned String after use.
    pub fn to_hex(&self) -> String {
        hex::encode(self.key)
    }
}

/// Securely zeroed string wrapper for sensitive data like tokens
/// The string is zeroed when dropped, preventing memory exposure
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct SecureString {
    value: String,
}

impl SecureString {
    /// Create a new SecureString from a String (takes ownership)
    pub fn new(value: String) -> Self {
        Self { value }
    }

    /// Get reference to the inner string
    pub fn as_str(&self) -> &str {
        &self.value
    }

    /// Check if the string is empty
    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }
}

impl From<String> for SecureString {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl std::fmt::Debug for SecureString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Never expose the actual value in debug output
        f.debug_struct("SecureString")
            .field("value", &"[REDACTED]")
            .finish()
    }
}

/// Encrypted data with metadata for decryption
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EncryptedData {
    pub ciphertext: Vec<u8>,
    pub nonce: [u8; NONCE_LEN],
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct EncryptedTokensFile {
    encrypted: bool,
    nonce_b64: String,
    ciphertext_b64: String,
}

/// Key wrapping data for passphrase-based protection
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WrappedKey {
    pub encrypted_key: EncryptedData,
    pub salt: [u8; SALT_LEN],
    pub argon2_memory: u32,
    pub argon2_time: u32,
    pub argon2_parallelism: u32,
}

/// Keychain manager for macOS
pub struct KeychainManager;

impl KeychainManager {
    /// Store master key in Keychain
    pub fn store_master_key(key: &MasterKey) -> Result<(), SecurityError> {
        let entry = keyring::Entry::new(SERVICE_NAME, MASTER_KEY_ENTRY)
            .map_err(|e| SecurityError::Keychain(e.to_string()))?;

        entry
            .set_secret(key.as_bytes())
            .map_err(|e| SecurityError::Keychain(e.to_string()))
    }

    /// Retrieve master key from Keychain
    pub fn get_master_key() -> Result<MasterKey, SecurityError> {
        let entry = keyring::Entry::new(SERVICE_NAME, MASTER_KEY_ENTRY)
            .map_err(|e| SecurityError::Keychain(e.to_string()))?;

        let mut secret = entry.get_secret().map_err(|e| match e {
            keyring::Error::NoEntry => SecurityError::MasterKeyNotFound,
            _ => SecurityError::Keychain(e.to_string()),
        })?;

        if secret.len() != KEY_LEN {
            secret.zeroize();
            return Err(SecurityError::InvalidKeyFormat);
        }

        let mut key_bytes = [0u8; KEY_LEN];
        key_bytes.copy_from_slice(&secret);
        secret.zeroize();
        Ok(MasterKey::from_bytes(key_bytes))
    }

    /// Delete master key from Keychain
    pub fn delete_master_key() -> Result<(), SecurityError> {
        let entry = keyring::Entry::new(SERVICE_NAME, MASTER_KEY_ENTRY)
            .map_err(|e| SecurityError::Keychain(e.to_string()))?;

        entry
            .delete_credential()
            .map_err(|e| SecurityError::Keychain(e.to_string()))
    }

    /// Check if Keychain is available
    pub fn is_available() -> bool {
        keyring::Entry::new(SERVICE_NAME, "test")
            .map(|_| true)
            .unwrap_or(false)
    }

    /// Store HuggingFace token
    pub fn store_hf_token(token: &str) -> Result<(), SecurityError> {
        let entry = keyring::Entry::new(SERVICE_NAME, HF_TOKEN_ENTRY)
            .map_err(|e| SecurityError::Keychain(e.to_string()))?;

        entry
            .set_password(token)
            .map_err(|e| SecurityError::Keychain(e.to_string()))
    }

    /// Get HuggingFace token
    pub fn get_hf_token() -> Result<Option<String>, SecurityError> {
        let entry = keyring::Entry::new(SERVICE_NAME, HF_TOKEN_ENTRY)
            .map_err(|e| SecurityError::Keychain(e.to_string()))?;

        match entry.get_password() {
            Ok(token) => Ok(Some(token)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(SecurityError::Keychain(e.to_string())),
        }
    }

    /// Delete HuggingFace token
    pub fn delete_hf_token() -> Result<(), SecurityError> {
        let entry = keyring::Entry::new(SERVICE_NAME, HF_TOKEN_ENTRY)
            .map_err(|e| SecurityError::Keychain(e.to_string()))?;

        match entry.delete_credential() {
            Ok(()) => Ok(()),
            Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(SecurityError::Keychain(e.to_string())),
        }
    }

    /// Store Jira API token
    pub fn store_jira_token(token: &str) -> Result<(), SecurityError> {
        let entry = keyring::Entry::new(SERVICE_NAME, JIRA_TOKEN_ENTRY)
            .map_err(|e| SecurityError::Keychain(e.to_string()))?;

        entry
            .set_password(token)
            .map_err(|e| SecurityError::Keychain(e.to_string()))
    }

    /// Get Jira API token
    pub fn get_jira_token() -> Result<Option<String>, SecurityError> {
        let entry = keyring::Entry::new(SERVICE_NAME, JIRA_TOKEN_ENTRY)
            .map_err(|e| SecurityError::Keychain(e.to_string()))?;

        match entry.get_password() {
            Ok(token) => Ok(Some(token)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(SecurityError::Keychain(e.to_string())),
        }
    }

    /// Delete Jira API token
    pub fn delete_jira_token() -> Result<(), SecurityError> {
        let entry = keyring::Entry::new(SERVICE_NAME, JIRA_TOKEN_ENTRY)
            .map_err(|e| SecurityError::Keychain(e.to_string()))?;

        match entry.delete_credential() {
            Ok(()) => Ok(()),
            Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(SecurityError::Keychain(e.to_string())),
        }
    }
}

/// Token names for file storage
pub const TOKEN_HUGGINGFACE: &str = "huggingface_token";
pub const TOKEN_JIRA: &str = "jira_api_token";

/// Wrapped key file format (JSON)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct WrappedKeyFile {
    version: u32,
    wrapped_key: WrappedKey,
}

impl WrappedKeyFile {
    const CURRENT_VERSION: u32 = 1;

    fn new(wrapped_key: WrappedKey) -> Self {
        Self {
            version: Self::CURRENT_VERSION,
            wrapped_key,
        }
    }
}

/// Key storage configuration file (JSON)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct KeyStorageConfig {
    mode: KeyStorageMode,
}

/// Secure key storage with support for Keychain and passphrase modes
///
/// Storage location: ~/Library/Application Support/AssistSupport/
/// - key_storage.json: Storage mode configuration
/// - master.key.wrap: Wrapped key file (passphrase mode only)
/// - tokens.json: API tokens encrypted with the master key
///
/// Keychain mode: Master key stored in macOS Keychain (most secure)
/// Passphrase mode: Master key wrapped with user passphrase (portable)
pub struct FileKeyStore;

impl FileKeyStore {
    fn tokens_file_error(msg: impl Into<String>) -> SecurityError {
        SecurityError::FileIO(format!("Invalid tokens.json: {}", msg.into()))
    }

    fn set_permissions(path: &Path, mode: u32) -> Result<(), SecurityError> {
        #[cfg(unix)]
        {
            let perms = fs::Permissions::from_mode(mode);
            fs::set_permissions(path, perms).map_err(|e| SecurityError::FileIO(e.to_string()))?;
        }
        Ok(())
    }

    fn write_private_file(path: &Path, contents: &[u8]) -> Result<(), SecurityError> {
        let parent = path
            .parent()
            .ok_or_else(|| SecurityError::FileIO("Missing parent directory".into()))?;

        let mut temp =
            NamedTempFile::new_in(parent).map_err(|e| SecurityError::FileIO(e.to_string()))?;
        temp.write_all(contents)
            .map_err(|e| SecurityError::FileIO(e.to_string()))?;
        temp.as_file()
            .sync_all()
            .map_err(|e| SecurityError::FileIO(e.to_string()))?;
        #[cfg(unix)]
        {
            temp.as_file()
                .set_permissions(fs::Permissions::from_mode(0o600))
                .map_err(|e| SecurityError::FileIO(e.to_string()))?;
        }

        temp.persist(path)
            .map_err(|e| SecurityError::FileIO(e.to_string()))?;
        Self::set_permissions(path, 0o600)?;
        Ok(())
    }

    /// Get app data directory: ~/Library/Application Support/AssistSupport/
    fn get_app_data_dir() -> Result<PathBuf, SecurityError> {
        dirs::data_dir()
            .map(|d| d.join("AssistSupport"))
            .ok_or(SecurityError::ConfigDirNotFound)
    }

    /// Get path to legacy master key file (plaintext, deprecated)
    fn legacy_master_key_path() -> Result<PathBuf, SecurityError> {
        Ok(Self::get_app_data_dir()?.join("master.key"))
    }

    /// Get path to wrapped key file (passphrase mode)
    fn wrapped_key_path() -> Result<PathBuf, SecurityError> {
        Ok(Self::get_app_data_dir()?.join("master.key.wrap"))
    }

    /// Get path to storage config file
    fn storage_config_path() -> Result<PathBuf, SecurityError> {
        Ok(Self::get_app_data_dir()?.join("key_storage.json"))
    }

    /// Get path to tokens file
    pub(crate) fn tokens_path() -> Result<PathBuf, SecurityError> {
        Ok(Self::get_app_data_dir()?.join("tokens.json"))
    }

    /// Ensure app data directory exists with secure permissions
    fn ensure_dir() -> Result<PathBuf, SecurityError> {
        let dir = Self::get_app_data_dir()?;
        fs::create_dir_all(&dir).map_err(|e| SecurityError::FileIO(e.to_string()))?;
        Self::set_permissions(&dir, 0o700)?;
        Ok(dir)
    }

    /// Get current storage mode (defaults to Keychain)
    pub fn get_storage_mode() -> Result<KeyStorageMode, SecurityError> {
        let config_path = Self::storage_config_path()?;
        if !config_path.exists() {
            return Ok(KeyStorageMode::default());
        }

        let content =
            fs::read_to_string(&config_path).map_err(|e| SecurityError::FileIO(e.to_string()))?;
        let config: KeyStorageConfig = serde_json::from_str(&content)
            .map_err(|e| SecurityError::FileIO(format!("Invalid key_storage.json: {}", e)))?;
        Ok(config.mode)
    }

    /// Set storage mode (does not migrate existing key)
    fn set_storage_mode(mode: KeyStorageMode) -> Result<(), SecurityError> {
        Self::ensure_dir()?;
        let config_path = Self::storage_config_path()?;
        let config = KeyStorageConfig { mode };
        let content = serde_json::to_string_pretty(&config)
            .map_err(|e| SecurityError::FileIO(e.to_string()))?;
        Self::write_private_file(&config_path, content.as_bytes())
    }

    /// Check if any key storage exists
    pub fn has_any_key_storage() -> bool {
        // Check Keychain
        if KeychainManager::get_master_key().is_ok() {
            return true;
        }
        // Check wrapped key file
        if Self::wrapped_key_path()
            .map(|p| p.exists())
            .unwrap_or(false)
        {
            return true;
        }
        // Check legacy plaintext file
        if Self::legacy_master_key_path()
            .map(|p| p.exists())
            .unwrap_or(false)
        {
            return true;
        }
        false
    }

    pub(crate) fn read_tokens_map_with_key(
        master_key: &MasterKey,
    ) -> Result<HashMap<String, String>, SecurityError> {
        let tokens_path = Self::tokens_path()?;

        if !tokens_path.exists() {
            return Ok(HashMap::new());
        }

        let content =
            fs::read_to_string(&tokens_path).map_err(|e| SecurityError::FileIO(e.to_string()))?;
        let value: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| SecurityError::FileIO(format!("Invalid tokens.json: {}", e)))?;

        let (tokens, was_plaintext) = Self::decode_tokens_value(value, master_key)?;
        if was_plaintext {
            Self::store_tokens_map_with_key(master_key, &tokens)?;
        }

        Ok(tokens)
    }

    fn decode_tokens_value(
        value: serde_json::Value,
        master_key: &MasterKey,
    ) -> Result<(HashMap<String, String>, bool), SecurityError> {
        if value.get("encrypted").and_then(|v| v.as_bool()) == Some(true) {
            let encrypted: EncryptedTokensFile = serde_json::from_value(value)
                .map_err(|e| Self::tokens_file_error(e.to_string()))?;

            let nonce_bytes = general_purpose::STANDARD
                .decode(&encrypted.nonce_b64)
                .map_err(|e| Self::tokens_file_error(format!("Invalid nonce: {}", e)))?;
            if nonce_bytes.len() != NONCE_LEN {
                return Err(Self::tokens_file_error("Invalid nonce length"));
            }
            let mut nonce = [0u8; NONCE_LEN];
            nonce.copy_from_slice(&nonce_bytes);

            let ciphertext = general_purpose::STANDARD
                .decode(&encrypted.ciphertext_b64)
                .map_err(|e| Self::tokens_file_error(format!("Invalid ciphertext: {}", e)))?;

            let decrypted =
                Crypto::decrypt(master_key.as_bytes(), &EncryptedData { ciphertext, nonce })?;
            let tokens: HashMap<String, String> = serde_json::from_slice(&decrypted)
                .map_err(|e| Self::tokens_file_error(e.to_string()))?;

            return Ok((tokens, false));
        }

        let tokens: HashMap<String, String> =
            serde_json::from_value(value).map_err(|e| Self::tokens_file_error(e.to_string()))?;
        Ok((tokens, true))
    }

    pub(crate) fn store_tokens_map_with_key(
        master_key: &MasterKey,
        tokens: &HashMap<String, String>,
    ) -> Result<(), SecurityError> {
        Self::ensure_dir()?;
        let tokens_path = Self::tokens_path()?;

        let serialized =
            serde_json::to_vec(tokens).map_err(|e| SecurityError::FileIO(e.to_string()))?;
        let encrypted = Crypto::encrypt(master_key.as_bytes(), &serialized)?;
        let payload = EncryptedTokensFile {
            encrypted: true,
            nonce_b64: general_purpose::STANDARD.encode(encrypted.nonce),
            ciphertext_b64: general_purpose::STANDARD.encode(encrypted.ciphertext),
        };

        let content = serde_json::to_string_pretty(&payload)
            .map_err(|e| SecurityError::FileIO(e.to_string()))?;
        Self::write_private_file(&tokens_path, content.as_bytes())
    }

    fn store_token_with_key(
        master_key: &MasterKey,
        name: &str,
        value: &str,
    ) -> Result<(), SecurityError> {
        let mut tokens = Self::read_tokens_map_with_key(master_key)?;
        tokens.insert(name.to_string(), value.to_string());
        Self::store_tokens_map_with_key(master_key, &tokens)
    }

    /// Get master key using Keychain mode (default)
    ///
    /// Migration flow (idempotent):
    /// 1. If Keychain entry exists → use it, delete legacy files
    /// 2. If legacy master.key exists → migrate to Keychain, secure-delete legacy
    /// 3. If wrapped key exists → error (passphrase required)
    /// 4. Generate new key and store in Keychain
    pub fn get_master_key() -> Result<MasterKey, SecurityError> {
        let mode = Self::get_storage_mode()?;

        // If passphrase mode is configured, require passphrase
        if mode == KeyStorageMode::Passphrase {
            return Err(SecurityError::PassphraseRequired);
        }

        // 1. Try Keychain first
        if let Ok(key) = KeychainManager::get_master_key() {
            // Clean up any legacy files
            Self::cleanup_legacy_key_file(&key)?;
            return Ok(key);
        }

        // 2. Try migrating from legacy plaintext file
        let legacy_path = Self::legacy_master_key_path()?;
        if legacy_path.exists() {
            let key = Self::read_legacy_key_file(&legacy_path)?;

            // Migrate to Keychain
            KeychainManager::store_master_key(&key)?;
            Self::set_storage_mode(KeyStorageMode::Keychain)?;

            // Secure-delete legacy file
            secure_delete_file(&legacy_path).map_err(|e| SecurityError::FileIO(e.to_string()))?;

            // Migrate tokens from old Keychain storage
            Self::migrate_tokens_from_keychain(&key)?;

            return Ok(key);
        }

        // 3. Check if wrapped key exists (wrong mode)
        let wrapped_path = Self::wrapped_key_path()?;
        if wrapped_path.exists() {
            return Err(SecurityError::PassphraseRequired);
        }

        // 4. Generate new key (first run)
        let key = MasterKey::generate();
        KeychainManager::store_master_key(&key)?;
        Self::set_storage_mode(KeyStorageMode::Keychain)?;
        Self::migrate_tokens_from_keychain(&key)?;
        Ok(key)
    }

    /// Get master key with passphrase (passphrase mode)
    pub fn get_master_key_with_passphrase(passphrase: &str) -> Result<MasterKey, SecurityError> {
        let wrapped_path = Self::wrapped_key_path()?;

        if wrapped_path.exists() {
            let content = fs::read_to_string(&wrapped_path)
                .map_err(|e| SecurityError::FileIO(e.to_string()))?;
            let file: WrappedKeyFile = serde_json::from_str(&content)
                .map_err(|e| SecurityError::FileIO(format!("Invalid wrapped key file: {}", e)))?;

            return Crypto::unwrap_key(&file.wrapped_key, passphrase);
        }

        // Try migrating from legacy file with new passphrase
        let legacy_path = Self::legacy_master_key_path()?;
        if legacy_path.exists() {
            let key = Self::read_legacy_key_file(&legacy_path)?;

            // Store with passphrase
            Self::store_master_key_with_passphrase(&key, passphrase)?;

            // Secure-delete legacy file
            secure_delete_file(&legacy_path).map_err(|e| SecurityError::FileIO(e.to_string()))?;

            // Migrate tokens
            Self::migrate_tokens_from_keychain(&key)?;

            return Ok(key);
        }

        // Try migrating from Keychain with new passphrase
        if let Ok(key) = KeychainManager::get_master_key() {
            Self::store_master_key_with_passphrase(&key, passphrase)?;
            let _ = KeychainManager::delete_master_key();
            Self::migrate_tokens_from_keychain(&key)?;
            return Ok(key);
        }

        Err(SecurityError::MasterKeyNotFound)
    }

    /// Initialize new master key with passphrase (first run with passphrase mode)
    pub fn initialize_with_passphrase(passphrase: &str) -> Result<MasterKey, SecurityError> {
        // Don't overwrite existing key
        if Self::has_any_key_storage() {
            return Err(SecurityError::FileIO("Key storage already exists".into()));
        }

        let key = MasterKey::generate();
        Self::store_master_key_with_passphrase(&key, passphrase)?;
        Ok(key)
    }

    /// Store master key with passphrase protection
    pub fn store_master_key_with_passphrase(
        key: &MasterKey,
        passphrase: &str,
    ) -> Result<(), SecurityError> {
        Self::ensure_dir()?;

        let wrapped = Crypto::wrap_key(key, passphrase)?;
        let file = WrappedKeyFile::new(wrapped);
        let content = serde_json::to_string_pretty(&file)
            .map_err(|e| SecurityError::FileIO(e.to_string()))?;

        let wrapped_path = Self::wrapped_key_path()?;
        Self::write_private_file(&wrapped_path, content.as_bytes())?;
        Self::set_storage_mode(KeyStorageMode::Passphrase)?;

        Ok(())
    }

    /// Store master key in Keychain
    pub fn store_master_key_in_keychain(key: &MasterKey) -> Result<(), SecurityError> {
        KeychainManager::store_master_key(key)?;
        Self::set_storage_mode(KeyStorageMode::Keychain)?;
        Ok(())
    }

    /// Delete all key storage
    pub fn delete_all_key_storage() -> Result<(), SecurityError> {
        // Delete from Keychain
        let _ = KeychainManager::delete_master_key();

        // Delete wrapped key file
        let wrapped_path = Self::wrapped_key_path()?;
        if wrapped_path.exists() {
            secure_delete_file(&wrapped_path).map_err(|e| SecurityError::FileIO(e.to_string()))?;
        }

        // Delete legacy file
        let legacy_path = Self::legacy_master_key_path()?;
        if legacy_path.exists() {
            secure_delete_file(&legacy_path).map_err(|e| SecurityError::FileIO(e.to_string()))?;
        }

        // Delete config
        let config_path = Self::storage_config_path()?;
        if config_path.exists() {
            fs::remove_file(&config_path).map_err(|e| SecurityError::FileIO(e.to_string()))?;
        }

        Ok(())
    }

    /// Read legacy plaintext key file
    fn read_legacy_key_file(path: &Path) -> Result<MasterKey, SecurityError> {
        let mut hex = fs::read_to_string(path).map_err(|e| SecurityError::FileIO(e.to_string()))?;
        let mut bytes = hex::decode(hex.trim()).map_err(|_| {
            hex.zeroize();
            SecurityError::InvalidKeyFormat
        })?;
        hex.zeroize();

        if bytes.len() != KEY_LEN {
            bytes.zeroize();
            return Err(SecurityError::InvalidKeyFormat);
        }

        let mut key_bytes = [0u8; KEY_LEN];
        key_bytes.copy_from_slice(&bytes);
        bytes.zeroize();
        Ok(MasterKey::from_bytes(key_bytes))
    }

    /// Clean up legacy key file after successful Keychain migration
    fn cleanup_legacy_key_file(_key: &MasterKey) -> Result<(), SecurityError> {
        let legacy_path = Self::legacy_master_key_path()?;
        if legacy_path.exists() {
            secure_delete_file(&legacy_path).map_err(|e| SecurityError::FileIO(e.to_string()))?;
        }
        Ok(())
    }

    /// Get a token by name from tokens.json
    pub fn get_token(name: &str) -> Result<Option<String>, SecurityError> {
        let master_key = Self::get_master_key()?;
        let tokens = Self::read_tokens_map_with_key(&master_key)?;
        Ok(tokens.get(name).cloned())
    }

    /// Store a token by name to tokens.json
    pub fn store_token(name: &str, value: &str) -> Result<(), SecurityError> {
        let master_key = Self::get_master_key()?;
        Self::store_token_with_key(&master_key, name, value)
    }

    /// Delete a token by name from tokens.json
    pub fn delete_token(name: &str) -> Result<(), SecurityError> {
        let master_key = Self::get_master_key()?;
        let tokens_path = Self::tokens_path()?;
        if !tokens_path.exists() {
            return Ok(());
        }

        let mut tokens = Self::read_tokens_map_with_key(&master_key)?;
        tokens.remove(name);

        if tokens.is_empty() {
            secure_delete_file(&tokens_path).map_err(|e| SecurityError::FileIO(e.to_string()))?;
            return Ok(());
        }

        Self::store_tokens_map_with_key(&master_key, &tokens)
    }

    /// Migrate tokens from Keychain to file storage
    fn migrate_tokens_from_keychain(master_key: &MasterKey) -> Result<(), SecurityError> {
        // Migrate HuggingFace token
        if let Ok(Some(token)) = KeychainManager::get_hf_token() {
            Self::store_token_with_key(master_key, TOKEN_HUGGINGFACE, &token)?;
            let _ = KeychainManager::delete_hf_token();
        }

        // Migrate Jira token
        if let Ok(Some(token)) = KeychainManager::get_jira_token() {
            Self::store_token_with_key(master_key, TOKEN_JIRA, &token)?;
            let _ = KeychainManager::delete_jira_token();
        }

        Ok(())
    }

    /// Check if master key exists in any storage
    pub fn has_master_key() -> bool {
        Self::has_any_key_storage()
    }

    /// Check if passphrase mode is active
    pub fn is_passphrase_mode() -> bool {
        Self::get_storage_mode()
            .map(|m| m == KeyStorageMode::Passphrase)
            .unwrap_or(false)
    }
}

/// Key rotation utilities
pub struct KeyRotation;

impl KeyRotation {
    /// Rotate master key (generates new key, re-encrypts database and tokens)
    ///
    /// For Keychain mode:
    /// - Get current key from Keychain
    /// - Generate new key
    /// - Re-encrypt tokens with new key
    /// - Store new key in Keychain
    ///
    /// Returns the old and new keys for database re-keying
    pub fn rotate_keychain_key() -> Result<(MasterKey, MasterKey), SecurityError> {
        // Get current key
        let old_key = KeychainManager::get_master_key()?;

        // Generate new key
        let new_key = MasterKey::generate();

        // Re-encrypt tokens with new key
        Self::reencrypt_tokens(&old_key, &new_key)?;

        // Store new key in Keychain
        KeychainManager::store_master_key(&new_key)?;

        Ok((old_key, new_key))
    }

    /// Rotate key with passphrase (generates new key, re-encrypts)
    pub fn rotate_passphrase_key(
        old_passphrase: &str,
        new_passphrase: &str,
    ) -> Result<(MasterKey, MasterKey), SecurityError> {
        // Get current key
        let old_key = FileKeyStore::get_master_key_with_passphrase(old_passphrase)?;

        // Generate new key
        let new_key = MasterKey::generate();

        // Re-encrypt tokens with new key
        Self::reencrypt_tokens(&old_key, &new_key)?;

        // Store new key with new passphrase
        FileKeyStore::store_master_key_with_passphrase(&new_key, new_passphrase)?;

        Ok((old_key, new_key))
    }

    /// Change passphrase without rotating the key
    pub fn change_passphrase(
        old_passphrase: &str,
        new_passphrase: &str,
    ) -> Result<(), SecurityError> {
        // Get current key
        let key = FileKeyStore::get_master_key_with_passphrase(old_passphrase)?;

        // Re-wrap with new passphrase
        FileKeyStore::store_master_key_with_passphrase(&key, new_passphrase)?;

        Ok(())
    }

    /// Re-encrypt tokens.json with a new key
    fn reencrypt_tokens(old_key: &MasterKey, new_key: &MasterKey) -> Result<(), SecurityError> {
        let tokens_path = FileKeyStore::tokens_path()?;

        if !tokens_path.exists() {
            return Ok(());
        }

        // Read with old key
        let tokens = FileKeyStore::read_tokens_map_with_key(old_key)?;

        // Write with new key
        FileKeyStore::store_tokens_map_with_key(new_key, &tokens)?;

        Ok(())
    }

    /// Migrate from Keychain to passphrase mode
    pub fn migrate_to_passphrase(passphrase: &str) -> Result<(), SecurityError> {
        // Get key from Keychain
        let key = KeychainManager::get_master_key()?;

        // Store with passphrase
        FileKeyStore::store_master_key_with_passphrase(&key, passphrase)?;

        // Delete from Keychain
        let _ = KeychainManager::delete_master_key();

        Ok(())
    }

    /// Migrate from passphrase to Keychain mode
    pub fn migrate_to_keychain(passphrase: &str) -> Result<(), SecurityError> {
        // Get key with passphrase
        let key = FileKeyStore::get_master_key_with_passphrase(passphrase)?;

        // Store in Keychain
        KeychainManager::store_master_key(&key)?;
        FileKeyStore::set_storage_mode(KeyStorageMode::Keychain)?;

        // Delete wrapped key file
        let wrapped_path = FileKeyStore::wrapped_key_path()?;
        if wrapped_path.exists() {
            secure_delete_file(&wrapped_path).map_err(|e| SecurityError::FileIO(e.to_string()))?;
        }

        Ok(())
    }
}

/// AES-256-GCM encryption utilities
pub struct Crypto;

impl Crypto {
    /// Encrypt data with AES-256-GCM
    pub fn encrypt(key: &[u8; KEY_LEN], plaintext: &[u8]) -> Result<EncryptedData, SecurityError> {
        let cipher =
            Aes256Gcm::new_from_slice(key).map_err(|e| SecurityError::Encryption(e.to_string()))?;

        let mut nonce_bytes = [0u8; NONCE_LEN];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| SecurityError::Encryption(e.to_string()))?;

        Ok(EncryptedData {
            ciphertext,
            nonce: nonce_bytes,
        })
    }

    /// Decrypt data with AES-256-GCM
    pub fn decrypt(
        key: &[u8; KEY_LEN],
        encrypted: &EncryptedData,
    ) -> Result<Vec<u8>, SecurityError> {
        let cipher =
            Aes256Gcm::new_from_slice(key).map_err(|e| SecurityError::Decryption(e.to_string()))?;

        let nonce = Nonce::from_slice(&encrypted.nonce);

        cipher
            .decrypt(nonce, encrypted.ciphertext.as_ref())
            .map_err(|e| SecurityError::Decryption(e.to_string()))
    }

    /// Derive key from passphrase using Argon2id
    pub fn derive_key_from_passphrase(
        passphrase: &str,
        salt: &[u8; SALT_LEN],
    ) -> Result<[u8; KEY_LEN], SecurityError> {
        let params = Params::new(
            ARGON2_MEMORY_COST,
            ARGON2_TIME_COST,
            ARGON2_PARALLELISM,
            Some(KEY_LEN),
        )
        .map_err(|e| SecurityError::KeyDerivation(e.to_string()))?;

        let argon2 = Argon2::new(argon2::Algorithm::Argon2id, Version::V0x13, params);

        let mut key = [0u8; KEY_LEN];
        argon2
            .hash_password_into(passphrase.as_bytes(), salt, &mut key)
            .map_err(|e| SecurityError::KeyDerivation(e.to_string()))?;

        Ok(key)
    }

    /// Generate random salt
    pub fn generate_salt() -> [u8; SALT_LEN] {
        let mut salt = [0u8; SALT_LEN];
        OsRng.fill_bytes(&mut salt);
        salt
    }

    /// Wrap master key with passphrase-derived key
    ///
    /// Security: The KEK (Key Encryption Key) is zeroized after use.
    pub fn wrap_key(master_key: &MasterKey, passphrase: &str) -> Result<WrappedKey, SecurityError> {
        let salt = Self::generate_salt();
        let mut kek = Self::derive_key_from_passphrase(passphrase, &salt)?;
        let encrypted_key = Self::encrypt(&kek, master_key.as_bytes());

        // Zeroize the KEK immediately after use
        kek.zeroize();

        let encrypted_key = encrypted_key?;

        Ok(WrappedKey {
            encrypted_key,
            salt,
            argon2_memory: ARGON2_MEMORY_COST,
            argon2_time: ARGON2_TIME_COST,
            argon2_parallelism: ARGON2_PARALLELISM,
        })
    }

    /// Unwrap master key with passphrase
    ///
    /// Security: The KEK and intermediate key bytes are zeroized after use.
    pub fn unwrap_key(wrapped: &WrappedKey, passphrase: &str) -> Result<MasterKey, SecurityError> {
        let mut kek = Self::derive_key_from_passphrase(passphrase, &wrapped.salt)?;
        let decrypted = Self::decrypt(&kek, &wrapped.encrypted_key);

        // Zeroize the KEK immediately after use
        kek.zeroize();

        let mut key_bytes = decrypted?;

        if key_bytes.len() != KEY_LEN {
            key_bytes.zeroize();
            return Err(SecurityError::InvalidKeyFormat);
        }

        let mut key = [0u8; KEY_LEN];
        key.copy_from_slice(&key_bytes);

        // Zeroize the intermediate buffer
        key_bytes.zeroize();

        Ok(MasterKey::from_bytes(key))
    }
}

/// Export encryption utilities (Argon2id + AES-256-GCM with separate password)
/// Export encryption result type (ciphertext, salt, nonce)
pub type ExportEncryptResult = (Vec<u8>, [u8; SALT_LEN], [u8; NONCE_LEN]);

pub struct ExportCrypto;

impl ExportCrypto {
    /// Encrypt data for export with user-provided password
    ///
    /// Security: The derived key is zeroized after use.
    pub fn encrypt_for_export(
        data: &[u8],
        password: &str,
    ) -> Result<ExportEncryptResult, SecurityError> {
        let salt = Crypto::generate_salt();
        let mut key = Crypto::derive_key_from_passphrase(password, &salt)?;
        let result = Crypto::encrypt(&key, data);

        // Zeroize the key after use
        key.zeroize();

        let encrypted = result?;
        Ok((encrypted.ciphertext, salt, encrypted.nonce))
    }

    /// Decrypt exported data
    ///
    /// Security: The derived key is zeroized after use.
    pub fn decrypt_export(
        ciphertext: &[u8],
        salt: &[u8; SALT_LEN],
        nonce: &[u8; NONCE_LEN],
        password: &str,
    ) -> Result<Vec<u8>, SecurityError> {
        let mut key = Crypto::derive_key_from_passphrase(password, salt)?;
        let encrypted = EncryptedData {
            ciphertext: ciphertext.to_vec(),
            nonce: *nonce,
        };
        let result = Crypto::decrypt(&key, &encrypted);

        // Zeroize the key after use
        key.zeroize();

        result
    }
}

/// Secure file deletion (best-effort)
pub fn secure_delete_file(path: &Path) -> std::io::Result<()> {
    use std::fs::{self, OpenOptions};
    use std::io::Write;

    if path.exists() {
        // Overwrite with zeros (best-effort on SSD)
        if let Ok(metadata) = fs::metadata(path) {
            let size = metadata.len() as usize;
            if let Ok(mut file) = OpenOptions::new().write(true).open(path) {
                let zeros = vec![0u8; size.min(1024 * 1024)];
                let mut remaining = size;
                while remaining > 0 {
                    let to_write = remaining.min(zeros.len());
                    if file.write_all(&zeros[..to_write]).is_err() {
                        break;
                    }
                    remaining -= to_write;
                }
                let _ = file.sync_all();
            }
        }
        fs::remove_file(path)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_master_key_generation() {
        let key1 = MasterKey::generate();
        let key2 = MasterKey::generate();
        assert_ne!(key1.as_bytes(), key2.as_bytes());
    }

    #[test]
    fn test_encryption_roundtrip() {
        let key = MasterKey::generate();
        let plaintext = b"Hello, World!";

        let encrypted = Crypto::encrypt(key.as_bytes(), plaintext).unwrap();
        let decrypted = Crypto::decrypt(key.as_bytes(), &encrypted).unwrap();

        assert_eq!(plaintext.as_slice(), decrypted.as_slice());
    }

    #[test]
    fn test_key_wrapping() {
        let master_key = MasterKey::generate();
        let passphrase = "test-passphrase-123";

        let wrapped = Crypto::wrap_key(&master_key, passphrase).unwrap();
        let unwrapped = Crypto::unwrap_key(&wrapped, passphrase).unwrap();

        assert_eq!(master_key.as_bytes(), unwrapped.as_bytes());
    }

    #[test]
    fn test_wrong_passphrase_fails() {
        let master_key = MasterKey::generate();
        let wrapped = Crypto::wrap_key(&master_key, "correct-passphrase").unwrap();

        let result = Crypto::unwrap_key(&wrapped, "wrong-passphrase");
        assert!(result.is_err());
    }

    #[test]
    fn test_export_crypto() {
        let data = b"Export test data";
        let password = "export-password";

        let (ciphertext, salt, nonce) = ExportCrypto::encrypt_for_export(data, password).unwrap();
        let decrypted = ExportCrypto::decrypt_export(&ciphertext, &salt, &nonce, password).unwrap();

        assert_eq!(data.as_slice(), decrypted.as_slice());
    }

    #[test]
    fn property_encrypt_decrypt_roundtrip_random_payloads() {
        let key = MasterKey::generate();
        let mut rng = rand::thread_rng();

        for size in [0usize, 1, 7, 32, 255, 1024, 4096] {
            let mut plaintext = vec![0u8; size];
            rng.fill_bytes(&mut plaintext);
            let encrypted =
                Crypto::encrypt(key.as_bytes(), &plaintext).expect("encryption should succeed");
            let decrypted =
                Crypto::decrypt(key.as_bytes(), &encrypted).expect("decryption should succeed");
            assert_eq!(decrypted, plaintext);
        }

        for _ in 0..64 {
            let mut plaintext = vec![0u8; (rng.next_u32() as usize % 2048) + 1];
            rng.fill_bytes(&mut plaintext);
            let encrypted =
                Crypto::encrypt(key.as_bytes(), &plaintext).expect("encryption should succeed");
            let decrypted =
                Crypto::decrypt(key.as_bytes(), &encrypted).expect("decryption should succeed");
            assert_eq!(decrypted, plaintext);
        }
    }

    #[test]
    fn property_wrong_key_cannot_decrypt_random_payloads() {
        let key_a = MasterKey::from_bytes([0x11u8; KEY_LEN]);
        let key_b = MasterKey::from_bytes([0x22u8; KEY_LEN]);
        let mut rng = rand::thread_rng();

        for _ in 0..64 {
            let mut plaintext = vec![0u8; (rng.next_u32() as usize % 1024) + 1];
            rng.fill_bytes(&mut plaintext);
            let encrypted =
                Crypto::encrypt(key_a.as_bytes(), &plaintext).expect("encryption should succeed");
            let decrypted = Crypto::decrypt(key_b.as_bytes(), &encrypted);
            assert!(decrypted.is_err());
        }
    }

    // FileKeyStore tests use a test subdirectory to avoid touching real credentials
    mod file_key_store_tests {
        use super::*;

        // Note: These tests don't actually call FileKeyStore methods that use
        // the real data dir. Instead, we test the logic indirectly through
        // the public interfaces or test internal functions separately.

        #[test]
        fn test_master_key_hex_roundtrip() {
            let key = MasterKey::generate();
            let hex = hex::encode(key.as_bytes());
            let decoded = hex::decode(&hex).unwrap();
            assert_eq!(key.as_bytes().as_slice(), decoded.as_slice());
        }

        #[test]
        fn test_tokens_json_format() {
            let mut tokens: HashMap<String, String> = HashMap::new();
            tokens.insert("huggingface_token".to_string(), "hf_test123".to_string());
            tokens.insert("jira_api_token".to_string(), "jira_test456".to_string());

            let json = serde_json::to_string_pretty(&tokens).unwrap();
            let parsed: HashMap<String, String> = serde_json::from_str(&json).unwrap();

            assert_eq!(
                parsed.get("huggingface_token"),
                Some(&"hf_test123".to_string())
            );
            assert_eq!(
                parsed.get("jira_api_token"),
                Some(&"jira_test456".to_string())
            );
        }

        #[test]
        fn test_token_update_preserves_others() {
            let mut tokens: HashMap<String, String> = HashMap::new();
            tokens.insert("token_a".to_string(), "value_a".to_string());
            tokens.insert("token_b".to_string(), "value_b".to_string());

            // Update token_a
            tokens.insert("token_a".to_string(), "new_value_a".to_string());

            assert_eq!(tokens.get("token_a"), Some(&"new_value_a".to_string()));
            assert_eq!(tokens.get("token_b"), Some(&"value_b".to_string()));
        }

        #[test]
        fn test_token_delete() {
            let mut tokens: HashMap<String, String> = HashMap::new();
            tokens.insert("token_a".to_string(), "value_a".to_string());
            tokens.insert("token_b".to_string(), "value_b".to_string());

            tokens.remove("token_a");

            assert_eq!(tokens.get("token_a"), None);
            assert_eq!(tokens.get("token_b"), Some(&"value_b".to_string()));
        }
    }
}
