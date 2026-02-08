use std::sync::Mutex;

use crate::error::AppError;

pub struct AppState {
    pub db: Mutex<rusqlite::Connection>,
}

/// Lock the database mutex, returning a clean AppError on poisoned mutex.
pub fn lock_db(state: &AppState) -> Result<std::sync::MutexGuard<'_, rusqlite::Connection>, AppError> {
    state.db.lock().map_err(|e| AppError::LockFailed(format!("DB mutex lock failed: {}", e)))
}
