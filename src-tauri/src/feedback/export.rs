//! CSV export for pilot feedback data

use crate::db::Database;
use std::io::Write;
use std::path::Path;

/// Export pilot query logs and feedback to a CSV file
pub fn export_to_csv(db: &Database, path: &Path) -> Result<usize, String> {
    let logs = super::get_query_logs(db)?;
    let feedback = super::get_all_feedback(db)?;

    let mut file =
        std::fs::File::create(path).map_err(|e| format!("Failed to create export file: {}", e))?;

    // Query logs section
    writeln!(file, "# Query Logs").map_err(|e| format!("Write error: {}", e))?;
    writeln!(file, "id,query,response,category,user_id,created_at")
        .map_err(|e| format!("Write error: {}", e))?;

    for log in &logs {
        writeln!(
            file,
            "{},{},{},{},{},{}",
            csv_escape(&log.id),
            csv_escape(&log.query),
            csv_escape(&log.response),
            log.category,
            csv_escape(&log.user_id),
            csv_escape(&log.created_at),
        )
        .map_err(|e| format!("Write error: {}", e))?;
    }

    // Feedback section
    writeln!(file).map_err(|e| format!("Write error: {}", e))?;
    writeln!(file, "# Feedback").map_err(|e| format!("Write error: {}", e))?;
    writeln!(
        file,
        "id,query_log_id,user_id,accuracy,clarity,helpfulness,comment,created_at"
    )
    .map_err(|e| format!("Write error: {}", e))?;

    for fb in &feedback {
        writeln!(
            file,
            "{},{},{},{},{},{},{},{}",
            csv_escape(&fb.id),
            csv_escape(&fb.query_log_id),
            csv_escape(&fb.user_id),
            fb.accuracy,
            fb.clarity,
            fb.helpfulness,
            csv_escape(fb.comment.as_deref().unwrap_or("")),
            csv_escape(&fb.created_at),
        )
        .map_err(|e| format!("Write error: {}", e))?;
    }

    Ok(logs.len() + feedback.len())
}

fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::MasterKey;
    use tempfile::TempDir;

    #[test]
    fn test_export_to_csv() {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("test.db");
        let key = MasterKey::generate();
        let db = Database::open(&db_path, &key).unwrap();
        db.initialize().unwrap();

        let log_id = super::super::log_query(&db, "Can I use USB?", "No.", "alice").unwrap();
        super::super::submit_feedback(&db, &log_id, "alice", 5, 4, 5, Some("Great")).unwrap();

        let csv_path = dir.path().join("export.csv");
        let count = export_to_csv(&db, &csv_path).unwrap();
        assert_eq!(count, 2); // 1 log + 1 feedback

        let content = std::fs::read_to_string(&csv_path).unwrap();
        assert!(content.contains("Can I use USB?"));
        assert!(content.contains("alice"));
    }
}
