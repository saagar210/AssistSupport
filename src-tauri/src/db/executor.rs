//! Database executor for non-blocking database operations
//!
//! Provides an async-friendly interface to the database that doesn't
//! hold locks across await points. Uses a dedicated thread and channels
//! to execute database operations.
//!
//! # Usage
//!
//! ```ignore
//! let executor = DbExecutor::new(db);
//!
//! // Run a query asynchronously
//! let count = executor.run(|conn| {
//!     conn.query_row("SELECT COUNT(*) FROM kb_documents", [], |r| r.get(0))
//! }).await?;
//! ```

use crate::db::Database;
use std::sync::mpsc;
use std::thread;
use tokio::sync::oneshot;

/// A database executor that runs operations on a dedicated thread
pub struct DbExecutor {
    sender: mpsc::Sender<DbOperation>,
    _handle: thread::JoinHandle<()>,
}

type DbResult<T> = Result<T, rusqlite::Error>;
type BoxedDbOp = Box<dyn FnOnce(&rusqlite::Connection) -> BoxedResult + Send + 'static>;
type BoxedResult = Box<dyn std::any::Any + Send + 'static>;

struct DbOperation {
    op: BoxedDbOp,
    response: oneshot::Sender<BoxedResult>,
}

impl DbExecutor {
    /// Create a new database executor
    ///
    /// Takes ownership of the database connection and runs all operations
    /// on a dedicated thread.
    pub fn new(db: Database) -> Self {
        let (sender, receiver) = mpsc::channel::<DbOperation>();

        let handle = thread::spawn(move || {
            let conn = db.conn();

            while let Ok(operation) = receiver.recv() {
                let result = (operation.op)(conn);
                let _ = operation.response.send(result);
            }
        });

        Self {
            sender,
            _handle: handle,
        }
    }

    /// Run a database operation asynchronously
    ///
    /// The operation is executed on the dedicated database thread and
    /// the result is returned through a oneshot channel.
    pub async fn run<F, T>(&self, op: F) -> Result<T, DbExecutorError>
    where
        F: FnOnce(&rusqlite::Connection) -> DbResult<T> + Send + 'static,
        T: Send + 'static,
    {
        let (response_tx, response_rx) = oneshot::channel();

        // Wrap the operation to return a boxed result
        let boxed_op: BoxedDbOp = Box::new(move |conn| {
            let result = op(conn);
            Box::new(result) as BoxedResult
        });

        let operation = DbOperation {
            op: boxed_op,
            response: response_tx,
        };

        self.sender
            .send(operation)
            .map_err(|_| DbExecutorError::ChannelClosed)?;

        let boxed_result = response_rx
            .await
            .map_err(|_| DbExecutorError::ChannelClosed)?;

        // Downcast the result back to the expected type
        let result = boxed_result
            .downcast::<DbResult<T>>()
            .map_err(|_| DbExecutorError::TypeMismatch)?;

        result.map_err(DbExecutorError::Database)
    }

    /// Run a database operation synchronously (blocking)
    ///
    /// Use this when you need the result immediately and can't use async.
    pub fn run_blocking<F, T>(&self, op: F) -> Result<T, DbExecutorError>
    where
        F: FnOnce(&rusqlite::Connection) -> DbResult<T> + Send + 'static,
        T: Send + 'static,
    {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(self.run(op))
        })
    }
}

/// Errors that can occur when using the database executor
#[derive(Debug, thiserror::Error)]
pub enum DbExecutorError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Channel closed - executor may have shut down")]
    ChannelClosed,

    #[error("Type mismatch in result - internal error")]
    TypeMismatch,
}

impl From<DbExecutorError> for crate::error::AppError {
    fn from(e: DbExecutorError) -> Self {
        match e {
            DbExecutorError::Database(db_err) => crate::error::AppError::db_query_failed(db_err.to_string()),
            DbExecutorError::ChannelClosed => {
                crate::error::AppError::internal("Database executor channel closed")
            }
            DbExecutorError::TypeMismatch => {
                crate::error::AppError::internal("Database result type mismatch")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use crate::security::MasterKey;

    #[tokio::test]
    async fn test_executor_basic_query() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let key = MasterKey::generate();

        let db = Database::open(&db_path, &key).unwrap();
        db.initialize().unwrap();

        let executor = DbExecutor::new(db);

        // Test basic query
        let count: i64 = executor
            .run(|conn| conn.query_row("SELECT 1", [], |r| r.get(0)))
            .await
            .unwrap();

        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_executor_insert_and_query() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let key = MasterKey::generate();

        let db = Database::open(&db_path, &key).unwrap();
        db.initialize().unwrap();

        let executor = DbExecutor::new(db);

        // Insert a setting
        executor
            .run(|conn| {
                conn.execute(
                    "INSERT INTO settings (key, value) VALUES (?, ?)",
                    ["test_key", "test_value"],
                )
            })
            .await
            .unwrap();

        // Query it back
        let value: String = executor
            .run(|conn| {
                conn.query_row(
                    "SELECT value FROM settings WHERE key = ?",
                    ["test_key"],
                    |r| r.get(0),
                )
            })
            .await
            .unwrap();

        assert_eq!(value, "test_value");
    }
}
