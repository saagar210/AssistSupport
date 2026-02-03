use super::*;

pub(crate) fn has_hf_token_impl() -> Result<bool, String> {
    FileKeyStore::get_token(TOKEN_HUGGINGFACE)
        .map(|t| t.is_some())
        .map_err(|e| e.to_string())
}

pub(crate) fn set_hf_token_impl(token: String) -> Result<(), String> {
    FileKeyStore::store_token(TOKEN_HUGGINGFACE, &token).map_err(|e| e.to_string())?;
    audit::audit_token_set("huggingface");
    Ok(())
}

pub(crate) fn clear_hf_token_impl() -> Result<(), String> {
    FileKeyStore::delete_token(TOKEN_HUGGINGFACE).map_err(|e| e.to_string())?;
    audit::audit_token_cleared("huggingface");
    Ok(())
}

pub(crate) fn set_github_token_impl(host: String, token: String) -> Result<(), String> {
    let host = normalize_github_host(&host)?;
    let token = token.trim();
    if token.is_empty() {
        return Err("Token cannot be empty".to_string());
    }
    let key = format!("{}{}", GITHUB_TOKEN_PREFIX, host);
    FileKeyStore::store_token(&key, token).map_err(|e| e.to_string())?;
    audit::audit_token_set(&format!("github:{}", host));
    Ok(())
}

pub(crate) fn clear_github_token_impl(host: String) -> Result<(), String> {
    let host = normalize_github_host(&host)?;
    let key = format!("{}{}", GITHUB_TOKEN_PREFIX, host);
    FileKeyStore::delete_token(&key).map_err(|e| e.to_string())?;
    audit::audit_token_cleared(&format!("github:{}", host));
    Ok(())
}

pub(crate) fn has_github_token_impl(host: String) -> Result<bool, String> {
    let host = normalize_github_host(&host)?;
    let key = format!("{}{}", GITHUB_TOKEN_PREFIX, host);
    Ok(FileKeyStore::get_token(&key)
        .map_err(|e| e.to_string())?
        .is_some())
}

pub(crate) fn get_audit_entries_impl(limit: Option<usize>) -> Result<Vec<crate::audit::AuditEntry>, String> {
    crate::audit::read_audit_entries(limit).map_err(|e| e.to_string())
}

pub(crate) fn export_audit_log_impl(export_path: String) -> Result<String, String> {
    use std::path::Path;

    let path = Path::new(&export_path);
    let validated = validate_within_home(path).map_err(|e| match e {
        ValidationError::PathTraversal => {
            "Export path must be within your home directory".to_string()
        }
        _ => format!("Invalid export path: {}", e),
    })?;

    let entries = crate::audit::read_audit_entries(None).map_err(|e| e.to_string())?;
    let json = serde_json::to_string_pretty(&entries).map_err(|e| e.to_string())?;
    std::fs::write(&validated, json).map_err(|e| e.to_string())?;
    let _ = crate::security::set_secure_permissions(&validated, crate::security::FILE_PERMISSIONS);

    Ok(validated.to_string_lossy().to_string())
}

pub(crate) fn create_session_token_impl(
    state: State<'_, AppState>,
) -> Result<String, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    // Clean up expired tokens first
    let _ = db.cleanup_expired_sessions();

    // Generate a secure session ID
    let session_id = uuid::Uuid::new_v4().to_string();

    // Hash the session ID for storage (using HMAC-like approach with master key)
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(session_id.as_bytes());
    hasher.update(b"assistsupport-session-salt");
    let token_hash = hasher.finalize().to_vec();

    // 24-hour expiry
    let expires_at = (chrono::Utc::now() + chrono::Duration::hours(24)).to_rfc3339();

    // Device identifier (hostname + username)
    let device_id = get_device_identifier();

    db.store_session_token(&session_id, &token_hash, &expires_at, &device_id)
        .map_err(|e| e.to_string())?;

    Ok(session_id)
}

pub(crate) fn validate_session_token_impl(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<bool, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    let device_id = get_device_identifier();
    db.validate_session_token(&session_id, &device_id)
        .map_err(|e| e.to_string())
}

pub(crate) fn clear_session_token_impl(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<(), String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.delete_session_token(&session_id)
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub(crate) fn lock_app_impl(
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    // Delete all sessions for this device
    db.conn()
        .execute("DELETE FROM session_tokens WHERE device_id = ?1", rusqlite::params![get_device_identifier()])
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn get_device_identifier() -> String {
    let username = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "unknown".to_string());
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| "unknown".to_string());
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(home.as_bytes());
    let hash = hex::encode(&hasher.finalize()[..8]);
    format!("{}@{}", username, hash)
}
