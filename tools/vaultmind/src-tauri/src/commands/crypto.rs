use crate::crypto;
use crate::error::AppError;

/// Get the current encryption status.
#[tauri::command]
pub fn get_encryption_status() -> Result<crypto::EncryptionStatus, AppError> {
    Ok(crypto::get_encryption_status())
}

/// Initialize encryption key in OS keychain (if not already present).
#[tauri::command]
pub fn initialize_encryption() -> Result<crypto::EncryptionStatus, AppError> {
    crypto::get_or_create_db_key()?;
    Ok(crypto::get_encryption_status())
}

/// Rotate the encryption key stored in the OS keychain.
#[tauri::command]
pub fn rotate_encryption_key() -> Result<crypto::EncryptionStatus, AppError> {
    crypto::rotate_db_key()?;

    let conn_for_audit =
        rusqlite::Connection::open_in_memory().ok();
    if let Some(ref _c) = conn_for_audit {
        tracing::info!("Encryption key rotated via command");
    }

    Ok(crypto::get_encryption_status())
}

/// Check if PRAGMA secure_delete is active on the database.
#[tauri::command]
pub fn check_secure_delete(
    state: tauri::State<'_, crate::state::AppState>,
) -> Result<bool, AppError> {
    let conn = crate::state::get_conn(state.inner())?;
    let val: i64 = conn
        .query_row("PRAGMA secure_delete", [], |row| row.get(0))
        .map_err(AppError::Database)?;
    Ok(val == 1)
}

#[cfg(test)]
mod tests {
    use crate::db;

    fn setup_db() -> rusqlite::Connection {
        let dir = tempfile::tempdir().unwrap();
        let pool = db::create_pool(dir.path()).unwrap();
        let conn = pool.get().unwrap();
        std::mem::forget(dir);
        let path = conn.path().unwrap().to_owned();
        drop(conn);
        let c = rusqlite::Connection::open(path).unwrap();
        db::configure_connection(&c).unwrap();
        c
    }

    #[test]
    fn test_secure_delete_enabled() {
        let conn = setup_db();
        let val: i64 = conn
            .query_row("PRAGMA secure_delete", [], |row| row.get(0))
            .unwrap();
        assert_eq!(val, 1, "PRAGMA secure_delete should be ON");
    }
}
