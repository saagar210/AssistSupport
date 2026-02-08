use std::sync::RwLock;

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

use crate::error::AppError;
use crate::metrics::AppMetrics;
use crate::vector_index::VectorIndex;

pub struct AppState {
    pub db_pool: Pool<SqliteConnectionManager>,
    pub vector_index: RwLock<VectorIndex>,
    pub metrics: AppMetrics,
}

/// Get a connection from the pool, returning a clean AppError on failure.
pub fn get_conn(
    state: &AppState,
) -> Result<r2d2::PooledConnection<SqliteConnectionManager>, AppError> {
    state.db_pool.get().map_err(|e| {
        AppError::LockFailed(format!("Failed to get DB connection from pool: {}", e))
    })
}
