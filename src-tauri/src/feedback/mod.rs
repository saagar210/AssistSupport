//! Pilot feedback collection module for AssistSupport
//! Logs queries and collects user feedback (accuracy, clarity, helpfulness)
//! with SQLite persistence via the existing Database layer.

pub mod export;

use crate::db::Database;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A logged query-response pair
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryLog {
    pub id: String,
    pub query: String,
    pub response: String,
    pub category: QueryCategory,
    pub user_id: String,
    pub created_at: String,
}

/// Category detected from query text
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum QueryCategory {
    Policy,
    Procedure,
    Reference,
    Unknown,
}

impl std::fmt::Display for QueryCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryCategory::Policy => write!(f, "policy"),
            QueryCategory::Procedure => write!(f, "procedure"),
            QueryCategory::Reference => write!(f, "reference"),
            QueryCategory::Unknown => write!(f, "unknown"),
        }
    }
}

/// User feedback on a query response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserFeedback {
    pub id: String,
    pub query_log_id: String,
    pub user_id: String,
    pub accuracy: i32,
    pub clarity: i32,
    pub helpfulness: i32,
    pub comment: Option<String>,
    pub created_at: String,
}

/// Aggregate stats for the pilot dashboard
#[derive(Debug, Clone, Serialize)]
pub struct PilotStats {
    pub total_queries: usize,
    pub total_feedback: usize,
    pub accuracy_pct: f64,
    pub clarity_avg: f64,
    pub helpfulness_avg: f64,
    pub by_category: Vec<CategoryStat>,
}

/// Per-category statistics
#[derive(Debug, Clone, Serialize)]
pub struct CategoryStat {
    pub category: String,
    pub query_count: usize,
    pub feedback_count: usize,
    pub accuracy_avg: f64,
    pub clarity_avg: f64,
    pub helpfulness_avg: f64,
}

/// Log a query and its response to the database
pub fn log_query(
    db: &Database,
    query: &str,
    response: &str,
    user_id: &str,
) -> Result<String, String> {
    let id = Uuid::new_v4().to_string();
    let category = detect_query_category(query);
    let now = Utc::now().to_rfc3339();

    db.conn()
        .execute(
            "INSERT INTO pilot_query_logs (id, query, response, category, user_id, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![id, query, response, category.to_string(), user_id, now],
        )
        .map_err(|e| format!("Failed to log query: {}", e))?;

    Ok(id)
}

/// Submit user feedback on a logged query
pub fn submit_feedback(
    db: &Database,
    query_log_id: &str,
    user_id: &str,
    accuracy: i32,
    clarity: i32,
    helpfulness: i32,
    comment: Option<&str>,
) -> Result<String, String> {
    // Validate ratings
    let accuracy = accuracy.clamp(1, 5);
    let clarity = clarity.clamp(1, 5);
    let helpfulness = helpfulness.clamp(1, 5);

    // Verify query log exists
    let exists: bool = db
        .conn()
        .query_row(
            "SELECT COUNT(*) > 0 FROM pilot_query_logs WHERE id = ?1",
            rusqlite::params![query_log_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("Failed to check query log: {}", e))?;

    if !exists {
        return Err("Query log not found".to_string());
    }

    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    db.conn()
        .execute(
            "INSERT INTO pilot_feedback (id, query_log_id, user_id, accuracy, clarity, helpfulness, comment, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![id, query_log_id, user_id, accuracy, clarity, helpfulness, comment, now],
        )
        .map_err(|e| format!("Failed to save feedback: {}", e))?;

    Ok(id)
}

/// Get all query logs
pub fn get_query_logs(db: &Database) -> Result<Vec<QueryLog>, String> {
    let mut stmt = db
        .conn()
        .prepare(
            "SELECT id, query, response, category, user_id, created_at
             FROM pilot_query_logs ORDER BY created_at DESC",
        )
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let logs = stmt
        .query_map([], |row| {
            let cat_str: String = row.get(3)?;
            Ok(QueryLog {
                id: row.get(0)?,
                query: row.get(1)?,
                response: row.get(2)?,
                category: parse_category(&cat_str),
                user_id: row.get(4)?,
                created_at: row.get(5)?,
            })
        })
        .map_err(|e| format!("Failed to query logs: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to read log row: {}", e))?;

    Ok(logs)
}

/// Get all feedback entries
pub fn get_all_feedback(db: &Database) -> Result<Vec<UserFeedback>, String> {
    let mut stmt = db
        .conn()
        .prepare(
            "SELECT id, query_log_id, user_id, accuracy, clarity, helpfulness, comment, created_at
             FROM pilot_feedback ORDER BY created_at DESC",
        )
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let feedback = stmt
        .query_map([], |row| {
            Ok(UserFeedback {
                id: row.get(0)?,
                query_log_id: row.get(1)?,
                user_id: row.get(2)?,
                accuracy: row.get(3)?,
                clarity: row.get(4)?,
                helpfulness: row.get(5)?,
                comment: row.get(6)?,
                created_at: row.get(7)?,
            })
        })
        .map_err(|e| format!("Failed to query feedback: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to read feedback row: {}", e))?;

    Ok(feedback)
}

/// Calculate pilot stats for the dashboard
pub fn get_pilot_stats(db: &Database) -> Result<PilotStats, String> {
    let total_queries: usize = db
        .conn()
        .query_row("SELECT COUNT(*) FROM pilot_query_logs", [], |row| {
            row.get(0)
        })
        .map_err(|e| format!("Failed to count queries: {}", e))?;

    let total_feedback: usize = db
        .conn()
        .query_row("SELECT COUNT(*) FROM pilot_feedback", [], |row| row.get(0))
        .map_err(|e| format!("Failed to count feedback: {}", e))?;

    // Overall averages
    let (accuracy_pct, clarity_avg, helpfulness_avg) = if total_feedback > 0 {
        let row: (f64, f64, f64) = db
            .conn()
            .query_row(
                "SELECT
                    CAST(SUM(CASE WHEN accuracy >= 4 THEN 1 ELSE 0 END) AS REAL) / COUNT(*) * 100.0,
                    AVG(clarity),
                    AVG(helpfulness)
                 FROM pilot_feedback",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .map_err(|e| format!("Failed to calculate averages: {}", e))?;
        row
    } else {
        (0.0, 0.0, 0.0)
    };

    // Per-category stats
    let mut stmt = db
        .conn()
        .prepare(
            "SELECT
                q.category,
                COUNT(DISTINCT q.id) as query_count,
                COUNT(f.id) as feedback_count,
                COALESCE(AVG(f.accuracy), 0) as avg_accuracy,
                COALESCE(AVG(f.clarity), 0) as avg_clarity,
                COALESCE(AVG(f.helpfulness), 0) as avg_helpfulness
             FROM pilot_query_logs q
             LEFT JOIN pilot_feedback f ON f.query_log_id = q.id
             GROUP BY q.category
             ORDER BY query_count DESC",
        )
        .map_err(|e| format!("Failed to prepare category stats: {}", e))?;

    let by_category = stmt
        .query_map([], |row| {
            Ok(CategoryStat {
                category: row.get(0)?,
                query_count: row.get(1)?,
                feedback_count: row.get(2)?,
                accuracy_avg: row.get(3)?,
                clarity_avg: row.get(4)?,
                helpfulness_avg: row.get(5)?,
            })
        })
        .map_err(|e| format!("Failed to query category stats: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to read category stat: {}", e))?;

    Ok(PilotStats {
        total_queries,
        total_feedback,
        accuracy_pct,
        clarity_avg,
        helpfulness_avg,
        by_category,
    })
}

/// Detect query category from text
pub fn detect_query_category(query: &str) -> QueryCategory {
    let q = query.to_lowercase();

    if q.contains("can i")
        || q.contains("am i allowed")
        || q.contains("is it okay")
        || q.contains("forbidden")
        || q.contains("permitted")
        || q.contains("allowed")
        || q.contains("exception")
    {
        QueryCategory::Policy
    } else if q.contains("how do i")
        || q.contains("how to")
        || q.contains("how can i")
        || q.contains("steps to")
        || q.contains("process for")
        || q.contains("what's the process")
    {
        QueryCategory::Procedure
    } else if q.contains("what is")
        || q.contains("what are")
        || q.contains("explain")
        || q.contains("tell me about")
        || q.contains("who do i contact")
        || q.contains("what options")
    {
        QueryCategory::Reference
    } else {
        QueryCategory::Unknown
    }
}

fn parse_category(s: &str) -> QueryCategory {
    match s {
        "policy" => QueryCategory::Policy,
        "procedure" => QueryCategory::Procedure,
        "reference" => QueryCategory::Reference,
        _ => QueryCategory::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::MasterKey;
    use tempfile::TempDir;

    fn setup_test_db() -> (TempDir, Database) {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("test.db");
        let key = MasterKey::generate();
        let db = Database::open(&db_path, &key).unwrap();
        db.initialize().unwrap();
        (dir, db)
    }

    #[test]
    fn test_log_query() {
        let (_dir, db) = setup_test_db();
        let id = log_query(
            &db,
            "Can I get a flash drive?",
            "No, policy forbids it.",
            "alice",
        )
        .unwrap();
        assert!(!id.is_empty());

        let logs = get_query_logs(&db).unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].query, "Can I get a flash drive?");
        assert_eq!(logs[0].category, QueryCategory::Policy);
    }

    #[test]
    fn test_submit_feedback() {
        let (_dir, db) = setup_test_db();
        let log_id = log_query(&db, "Flash drive?", "No.", "alice").unwrap();

        let fb_id =
            submit_feedback(&db, &log_id, "alice", 5, 4, 5, Some("Clear response")).unwrap();
        assert!(!fb_id.is_empty());

        let feedback = get_all_feedback(&db).unwrap();
        assert_eq!(feedback.len(), 1);
        assert_eq!(feedback[0].accuracy, 5);
        assert_eq!(feedback[0].comment, Some("Clear response".to_string()));
    }

    #[test]
    fn test_feedback_validates_log_exists() {
        let (_dir, db) = setup_test_db();
        let result = submit_feedback(&db, "nonexistent", "alice", 5, 5, 5, None);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_feedback_clamps_ratings() {
        let (_dir, db) = setup_test_db();
        let log_id = log_query(&db, "Test query", "Response", "bob").unwrap();

        submit_feedback(&db, &log_id, "bob", 10, 0, -1, None).unwrap();

        let feedback = get_all_feedback(&db).unwrap();
        assert_eq!(feedback[0].accuracy, 5);
        assert_eq!(feedback[0].clarity, 1);
        assert_eq!(feedback[0].helpfulness, 1);
    }

    #[test]
    fn test_detect_query_category() {
        assert_eq!(
            detect_query_category("Can I get a flash drive?"),
            QueryCategory::Policy
        );
        assert_eq!(
            detect_query_category("Am I allowed to install Slack?"),
            QueryCategory::Policy
        );
        assert_eq!(
            detect_query_category("How do I request a laptop?"),
            QueryCategory::Procedure
        );
        assert_eq!(
            detect_query_category("How to reset password?"),
            QueryCategory::Procedure
        );
        assert_eq!(
            detect_query_category("What is the VPN policy?"),
            QueryCategory::Reference
        );
        assert_eq!(
            detect_query_category("Who do I contact for help?"),
            QueryCategory::Reference
        );
        assert_eq!(detect_query_category("hello"), QueryCategory::Unknown);
    }

    #[test]
    fn test_pilot_stats() {
        let (_dir, db) = setup_test_db();

        // Log 3 queries
        let id1 = log_query(&db, "Can I use a USB?", "No.", "alice").unwrap();
        let id2 = log_query(&db, "How to request laptop?", "Submit via portal.", "bob").unwrap();
        let _id3 = log_query(&db, "What VPN options?", "Cisco AnyConnect.", "alice").unwrap();

        // Submit feedback for 2
        submit_feedback(&db, &id1, "alice", 5, 5, 4, None).unwrap();
        submit_feedback(&db, &id2, "bob", 4, 3, 4, None).unwrap();

        let stats = get_pilot_stats(&db).unwrap();
        assert_eq!(stats.total_queries, 3);
        assert_eq!(stats.total_feedback, 2);
        assert!(stats.accuracy_pct > 99.0); // Both rated >= 4
        assert!(stats.by_category.len() >= 2);
    }
}
