use rusqlite::Connection;

use crate::error::AppError;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IntegrityReport {
    pub is_ok: bool,
    pub integrity_check: String,
    pub foreign_key_violations: Vec<ForeignKeyViolation>,
    pub db_size_bytes: u64,
    pub page_count: i64,
    pub page_size: i64,
    pub wal_mode: bool,
    pub secure_delete: bool,
    pub schema_version: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ForeignKeyViolation {
    pub table: String,
    pub rowid: i64,
    pub parent: String,
    pub fkid: i64,
}

/// Run PRAGMA integrity_check on the database.
pub fn check_database_integrity(conn: &Connection) -> Result<IntegrityReport, AppError> {
    let integrity_result: String =
        conn.query_row("PRAGMA integrity_check", [], |row| row.get(0))?;

    let fk_violations = check_foreign_keys(conn)?;

    let page_count: i64 = conn.query_row("PRAGMA page_count", [], |row| row.get(0))?;
    let page_size: i64 = conn.query_row("PRAGMA page_size", [], |row| row.get(0))?;
    let db_size_bytes = (page_count * page_size) as u64;

    let journal_mode: String = conn.query_row("PRAGMA journal_mode", [], |row| row.get(0))?;
    let secure_delete: i64 = conn
        .query_row("PRAGMA secure_delete", [], |row| row.get(0))
        .unwrap_or(0);

    let schema_version: i64 = conn
        .query_row(
            "SELECT CAST(value AS INTEGER) FROM settings WHERE key = 'schema_version'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let is_ok = integrity_result == "ok" && fk_violations.is_empty();

    Ok(IntegrityReport {
        is_ok,
        integrity_check: integrity_result,
        foreign_key_violations: fk_violations,
        db_size_bytes,
        page_count,
        page_size,
        wal_mode: journal_mode == "wal",
        secure_delete: secure_delete == 1,
        schema_version,
    })
}

/// Check for foreign key violations.
pub fn check_foreign_keys(conn: &Connection) -> Result<Vec<ForeignKeyViolation>, AppError> {
    let mut stmt = conn.prepare("PRAGMA foreign_key_check")?;
    let violations = stmt
        .query_map([], |row| {
            Ok(ForeignKeyViolation {
                table: row.get(0)?,
                rowid: row.get(1)?,
                parent: row.get(2)?,
                fkid: row.get(3)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(violations)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_db() -> Connection {
        let dir = tempfile::tempdir().unwrap();
        let pool = crate::db::create_pool(dir.path()).unwrap();
        let conn = pool.get().unwrap();
        std::mem::forget(dir);
        let path = conn.path().unwrap().to_owned();
        drop(conn);
        let c = Connection::open(path).unwrap();
        crate::db::configure_connection(&c).unwrap();
        c
    }

    #[test]
    fn test_integrity_check_healthy_db() {
        let conn = setup_db();
        let report = check_database_integrity(&conn).unwrap();
        assert!(report.is_ok);
        assert_eq!(report.integrity_check, "ok");
    }

    #[test]
    fn test_integrity_check_reports_pragmas() {
        let conn = setup_db();
        let report = check_database_integrity(&conn).unwrap();
        assert!(report.wal_mode);
        assert!(report.secure_delete);
    }

    #[test]
    fn test_foreign_key_check_clean() {
        let conn = setup_db();
        let violations = check_foreign_keys(&conn).unwrap();
        assert!(violations.is_empty());
    }

    #[test]
    fn test_integrity_report_has_db_size() {
        let conn = setup_db();
        let report = check_database_integrity(&conn).unwrap();
        assert!(report.db_size_bytes > 0);
    }
}
