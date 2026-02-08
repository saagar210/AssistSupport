use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::error::AppError;

/// Actions that are tracked in the audit log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditAction {
    CollectionCreate,
    CollectionUpdate,
    CollectionDelete,
    DocumentIngest,
    DocumentDelete,
    DocumentReingest,
    ConversationCreate,
    ConversationDelete,
    ConversationRename,
    ChatMessage,
    SearchExecute,
    SettingUpdate,
    GraphBuild,
}

impl AuditAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            AuditAction::CollectionCreate => "collection.create",
            AuditAction::CollectionUpdate => "collection.update",
            AuditAction::CollectionDelete => "collection.delete",
            AuditAction::DocumentIngest => "document.ingest",
            AuditAction::DocumentDelete => "document.delete",
            AuditAction::DocumentReingest => "document.reingest",
            AuditAction::ConversationCreate => "conversation.create",
            AuditAction::ConversationDelete => "conversation.delete",
            AuditAction::ConversationRename => "conversation.rename",
            AuditAction::ChatMessage => "chat.message",
            AuditAction::SearchExecute => "search.execute",
            AuditAction::SettingUpdate => "setting.update",
            AuditAction::GraphBuild => "graph.build",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: String,
    pub timestamp: String,
    pub action: String,
    pub entity_type: Option<String>,
    pub entity_id: Option<String>,
    pub details: String,
    pub ip_address: Option<String>,
    pub user_agent: String,
}

/// Log an audit event to the audit_log table.
pub fn log_audit(
    conn: &Connection,
    action: AuditAction,
    entity_type: Option<&str>,
    entity_id: Option<&str>,
    details: &serde_json::Value,
) -> Result<(), AppError> {
    let id = uuid::Uuid::new_v4().to_string();
    let timestamp = chrono::Utc::now().to_rfc3339();
    let details_str = details.to_string();

    conn.execute(
        "INSERT INTO audit_log (id, timestamp, action, entity_type, entity_id, details, ip_address, user_agent)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, '127.0.0.1', 'desktop')",
        rusqlite::params![id, timestamp, action.as_str(), entity_type, entity_id, details_str],
    )?;

    Ok(())
}

/// Query the audit log with optional filters.
pub fn query_audit_log(
    conn: &Connection,
    action_filter: Option<&str>,
    entity_type_filter: Option<&str>,
    start_date: Option<&str>,
    end_date: Option<&str>,
    page: usize,
    page_size: usize,
) -> Result<(Vec<AuditEntry>, i64), AppError> {
    let mut conditions = Vec::new();
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
    let mut param_idx = 1;

    if let Some(action) = action_filter {
        conditions.push(format!("action = ?{}", param_idx));
        params.push(Box::new(action.to_string()));
        param_idx += 1;
    }

    if let Some(entity_type) = entity_type_filter {
        conditions.push(format!("entity_type = ?{}", param_idx));
        params.push(Box::new(entity_type.to_string()));
        param_idx += 1;
    }

    if let Some(start) = start_date {
        conditions.push(format!("timestamp >= ?{}", param_idx));
        params.push(Box::new(start.to_string()));
        param_idx += 1;
    }

    if let Some(end) = end_date {
        conditions.push(format!("timestamp <= ?{}", param_idx));
        params.push(Box::new(end.to_string()));
        param_idx += 1;
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    // Count total
    let count_sql = format!("SELECT COUNT(*) FROM audit_log {}", where_clause);
    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let total: i64 = conn.query_row(&count_sql, param_refs.as_slice(), |row| row.get(0))?;

    // Query with pagination
    let offset = (page.saturating_sub(1)) * page_size;
    let query_sql = format!(
        "SELECT id, timestamp, action, entity_type, entity_id, details, ip_address, user_agent
         FROM audit_log {} ORDER BY timestamp DESC LIMIT ?{} OFFSET ?{}",
        where_clause, param_idx, param_idx + 1
    );

    let mut query_params: Vec<Box<dyn rusqlite::types::ToSql>> = params;
    query_params.push(Box::new(page_size as i64));
    query_params.push(Box::new(offset as i64));

    let query_param_refs: Vec<&dyn rusqlite::types::ToSql> =
        query_params.iter().map(|p| p.as_ref()).collect();

    let mut stmt = conn.prepare(&query_sql)?;
    let entries = stmt
        .query_map(query_param_refs.as_slice(), |row| {
            Ok(AuditEntry {
                id: row.get(0)?,
                timestamp: row.get(1)?,
                action: row.get(2)?,
                entity_type: row.get(3)?,
                entity_id: row.get(4)?,
                details: row.get(5)?,
                ip_address: row.get(6)?,
                user_agent: row.get(7)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok((entries, total))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE audit_log (
                id TEXT PRIMARY KEY,
                timestamp TEXT NOT NULL,
                action TEXT NOT NULL,
                entity_type TEXT,
                entity_id TEXT,
                details TEXT DEFAULT '{}',
                ip_address TEXT,
                user_agent TEXT DEFAULT 'desktop'
            );
            CREATE INDEX idx_audit_log_timestamp ON audit_log(timestamp);
            CREATE INDEX idx_audit_log_action ON audit_log(action);
            CREATE INDEX idx_audit_log_entity ON audit_log(entity_type, entity_id);",
        )
        .unwrap();
        conn
    }

    #[test]
    fn test_log_audit_inserts_row() {
        let conn = setup_db();
        let details = serde_json::json!({"name": "Test Collection"});
        log_audit(
            &conn,
            AuditAction::CollectionCreate,
            Some("collection"),
            Some("col-123"),
            &details,
        )
        .unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM audit_log", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);

        let action: String = conn
            .query_row("SELECT action FROM audit_log LIMIT 1", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(action, "collection.create");
    }

    #[test]
    fn test_query_audit_log_filters_by_action() {
        let conn = setup_db();
        log_audit(
            &conn,
            AuditAction::CollectionCreate,
            Some("collection"),
            Some("c1"),
            &serde_json::json!({}),
        )
        .unwrap();
        log_audit(
            &conn,
            AuditAction::DocumentIngest,
            Some("document"),
            Some("d1"),
            &serde_json::json!({}),
        )
        .unwrap();

        let (entries, total) =
            query_audit_log(&conn, Some("collection.create"), None, None, None, 1, 50).unwrap();
        assert_eq!(total, 1);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].action, "collection.create");
    }

    #[test]
    fn test_query_audit_log_filters_by_date_range() {
        let conn = setup_db();
        // Insert with specific timestamps
        conn.execute(
            "INSERT INTO audit_log (id, timestamp, action, details, user_agent) VALUES ('a1', '2025-01-01T00:00:00Z', 'collection.create', '{}', 'desktop')",
            [],
        ).unwrap();
        conn.execute(
            "INSERT INTO audit_log (id, timestamp, action, details, user_agent) VALUES ('a2', '2025-06-01T00:00:00Z', 'document.ingest', '{}', 'desktop')",
            [],
        ).unwrap();

        let (entries, total) = query_audit_log(
            &conn,
            None,
            None,
            Some("2025-05-01T00:00:00Z"),
            Some("2025-12-31T23:59:59Z"),
            1,
            50,
        )
        .unwrap();
        assert_eq!(total, 1);
        assert_eq!(entries[0].action, "document.ingest");
    }

    #[test]
    fn test_query_audit_log_pagination() {
        let conn = setup_db();
        for i in 0..10 {
            log_audit(
                &conn,
                AuditAction::SearchExecute,
                None,
                None,
                &serde_json::json!({"query": format!("q{}", i)}),
            )
            .unwrap();
        }

        let (entries, total) = query_audit_log(&conn, None, None, None, None, 1, 3).unwrap();
        assert_eq!(total, 10);
        assert_eq!(entries.len(), 3);

        let (entries_p2, _) = query_audit_log(&conn, None, None, None, None, 2, 3).unwrap();
        assert_eq!(entries_p2.len(), 3);
    }

    #[test]
    fn test_audit_log_details_valid_json() {
        let conn = setup_db();
        let details = serde_json::json!({"filename": "test.pdf", "chunks": 42});
        log_audit(
            &conn,
            AuditAction::DocumentIngest,
            Some("document"),
            Some("doc-1"),
            &details,
        )
        .unwrap();

        let stored_details: String = conn
            .query_row("SELECT details FROM audit_log LIMIT 1", [], |row| {
                row.get(0)
            })
            .unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&stored_details).unwrap();
        assert_eq!(parsed["filename"], "test.pdf");
        assert_eq!(parsed["chunks"], 42);
    }
}
