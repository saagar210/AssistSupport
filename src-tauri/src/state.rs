use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

use crate::error::AppError;

pub struct AppState {
    pub db_pool: Pool<SqliteConnectionManager>,
}

/// Get a connection from the pool, returning a clean AppError on failure.
pub fn get_conn(
    state: &AppState,
) -> Result<r2d2::PooledConnection<SqliteConnectionManager>, AppError> {
    state.db_pool.get().map_err(|e| {
        AppError::LockFailed(format!("Failed to get DB connection from pool: {}", e))
    })
}
