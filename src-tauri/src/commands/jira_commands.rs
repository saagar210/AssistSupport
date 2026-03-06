use super::*;
use crate::jira::JiraClient;
use crate::jira::{JiraConfig, JiraTicket};
use crate::kb::dns::PinnedDnsResolver;
use crate::kb::network::{validate_url_for_ssrf_with_pinning, SsrfConfig};

const JIRA_BASE_URL_SETTING: &str = "jira_base_url";
const JIRA_EMAIL_SETTING: &str = "jira_email";

fn get_jira_connection(
    db: &crate::db::Database,
    missing_token_msg: &str,
) -> Result<(String, String, String), String> {
    let base_url: String = db
        .conn()
        .query_row(
            "SELECT value FROM settings WHERE key = ?",
            rusqlite::params![JIRA_BASE_URL_SETTING],
            |row| row.get(0),
        )
        .map_err(|_| "Jira not configured")?;

    let email: String = db
        .conn()
        .query_row(
            "SELECT value FROM settings WHERE key = ?",
            rusqlite::params![JIRA_EMAIL_SETTING],
            |row| row.get(0),
        )
        .map_err(|_| "Jira not configured")?;

    let token = FileKeyStore::get_token(TOKEN_JIRA)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| missing_token_msg.to_string())?;

    Ok((base_url, email, token))
}

fn parse_comment_visibility(visibility: Option<String>) -> Option<crate::jira::CommentVisibility> {
    use crate::jira::CommentVisibility;

    visibility.map(|v| match v.as_str() {
        "internal" => CommentVisibility::Internal,
        "public" => CommentVisibility::Public,
        _ if v.starts_with("role:") => CommentVisibility::Role(v[5..].to_string()),
        _ if v.starts_with("group:") => CommentVisibility::Group(v[6..].to_string()),
        _ => CommentVisibility::Public,
    })
}

pub(crate) fn is_jira_configured_impl(state: State<'_, AppState>) -> Result<bool, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    let base_url: Result<String, _> = db.conn().query_row(
        "SELECT value FROM settings WHERE key = ?",
        rusqlite::params![JIRA_BASE_URL_SETTING],
        |row| row.get(0),
    );

    let has_token = FileKeyStore::get_token(TOKEN_JIRA)
        .map(|t| t.is_some())
        .unwrap_or(false);

    Ok(base_url.is_ok() && has_token)
}

pub(crate) fn get_jira_config_impl(
    state: State<'_, AppState>,
) -> Result<Option<JiraConfig>, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    let base_url: Result<String, _> = db.conn().query_row(
        "SELECT value FROM settings WHERE key = ?",
        rusqlite::params![JIRA_BASE_URL_SETTING],
        |row| row.get(0),
    );

    let email: Result<String, _> = db.conn().query_row(
        "SELECT value FROM settings WHERE key = ?",
        rusqlite::params![JIRA_EMAIL_SETTING],
        |row| row.get(0),
    );

    match (base_url, email) {
        (Ok(base_url), Ok(email)) => Ok(Some(JiraConfig { base_url, email })),
        _ => Ok(None),
    }
}

pub(crate) async fn configure_jira_impl(
    state: State<'_, AppState>,
    base_url: String,
    email: String,
    api_token: String,
    allow_http: Option<bool>,
) -> Result<(), String> {
    // Validate URL format + SSRF protection (blocks loopback/private by default).
    // Jira is expected to be a public SaaS endpoint; we do not allow pointing Jira
    // to localhost/private IPs to avoid SSRF-style abuse and credential leakage.
    let resolver = PinnedDnsResolver::new(SsrfConfig::default())
        .await
        .map_err(|e| e.to_string())?;
    validate_url_for_ssrf_with_pinning(&base_url, &resolver)
        .await
        .map_err(|e| e.to_string())?;

    // Enforce HTTPS by default
    let using_http = is_http_url(&base_url);
    if using_http {
        if allow_http != Some(true) {
            return Err(
                "HTTPS is required for Jira connections. HTTP connections expose credentials \
                 in transit. If you must use HTTP (e.g., local testing), enable the \
                 'allow_http' option explicitly."
                    .to_string(),
            );
        }
        // Log security warning for HTTP opt-in
        audit::audit_jira_http_opt_in(&base_url);
    }

    // Test connection first
    let client = JiraClient::new(&base_url, &email, &api_token);
    if !client.test_connection().await.map_err(|e| e.to_string())? {
        return Err("Connection failed - check credentials".to_string());
    }

    // Store token in file storage
    FileKeyStore::store_token(TOKEN_JIRA, &api_token).map_err(|e| e.to_string())?;
    audit::audit_token_set("jira");

    // Store config in DB
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    db.conn()
        .execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?, ?)",
            rusqlite::params![JIRA_BASE_URL_SETTING, &base_url],
        )
        .map_err(|e| e.to_string())?;

    db.conn()
        .execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?, ?)",
            rusqlite::params![JIRA_EMAIL_SETTING, &email],
        )
        .map_err(|e| e.to_string())?;

    // Store HTTP opt-in preference if used
    if using_http {
        db.conn()
            .execute(
                "INSERT OR REPLACE INTO settings (key, value) VALUES (?, ?)",
                rusqlite::params!["jira_http_opt_in", "true"],
            )
            .map_err(|e| e.to_string())?;
    } else {
        // Clear HTTP opt-in if switching to HTTPS
        db.conn()
            .execute(
                "DELETE FROM settings WHERE key = ?",
                rusqlite::params!["jira_http_opt_in"],
            )
            .map_err(|e| e.to_string())?;
    }

    // Audit log successful configuration
    audit::audit_jira_configured(!using_http);

    Ok(())
}

pub(crate) fn clear_jira_config_impl(state: State<'_, AppState>) -> Result<(), String> {
    // Delete token from file storage
    let _ = FileKeyStore::delete_token(TOKEN_JIRA);
    audit::audit_token_cleared("jira");

    // Delete config from DB
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    db.conn()
        .execute(
            "DELETE FROM settings WHERE key IN (?, ?)",
            rusqlite::params![JIRA_BASE_URL_SETTING, JIRA_EMAIL_SETTING],
        )
        .map_err(|e| e.to_string())?;

    Ok(())
}

pub(crate) async fn get_jira_ticket_impl(
    state: State<'_, AppState>,
    ticket_key: String,
) -> Result<JiraTicket, String> {
    validate_ticket_id(&ticket_key).map_err(|e| e.to_string())?;

    let (base_url, email, token) = {
        let db_lock = state.db.lock().map_err(|e| e.to_string())?;
        let db = db_lock.as_ref().ok_or("Database not initialized")?;
        get_jira_connection(db, "Jira token not found")?
    };

    let client = JiraClient::new(&base_url, &email, &token);
    client
        .get_ticket(&ticket_key)
        .await
        .map_err(|e| e.to_string())
}

pub(crate) async fn add_jira_comment_impl(
    state: State<'_, AppState>,
    ticket_key: String,
    comment_body: String,
    visibility: Option<String>,
) -> Result<String, String> {
    validate_ticket_id(&ticket_key).map_err(|e| e.to_string())?;

    let (base_url, email, token) = {
        let db_lock = state.db.lock().map_err(|e| e.to_string())?;
        let db = db_lock.as_ref().ok_or("Database not initialized")?;
        get_jira_connection(db, "Jira token not found")?
    };

    let vis = parse_comment_visibility(visibility);
    let client = JiraClient::new(&base_url, &email, &token);
    client
        .add_comment(&ticket_key, &comment_body, vis)
        .await
        .map_err(|e| e.to_string())
}

pub(crate) async fn push_draft_to_jira_impl(
    state: State<'_, AppState>,
    draft_id: String,
    ticket_key: String,
    visibility: Option<String>,
) -> Result<String, String> {
    use crate::jira::KbCitation;

    validate_ticket_id(&ticket_key).map_err(|e| e.to_string())?;

    let (response_text, sources_json) = {
        let db_lock = state.db.lock().map_err(|e| e.to_string())?;
        let db = db_lock.as_ref().ok_or("Database not initialized")?;
        let draft = db.get_draft(&draft_id).map_err(|e| e.to_string())?;
        let response = draft.response_text.ok_or("Draft has no response text")?;
        (response, draft.kb_sources_json)
    };

    let citations: Vec<KbCitation> = sources_json
        .and_then(|json| serde_json::from_str::<Vec<serde_json::Value>>(&json).ok())
        .map(|sources| {
            sources
                .iter()
                .map(|s| KbCitation {
                    title: s["title"].as_str().unwrap_or("Unknown").to_string(),
                    url: s["url"].as_str().map(|u| u.to_string()),
                    chunk_id: s["chunk_id"].as_str().map(|c| c.to_string()),
                })
                .collect()
        })
        .unwrap_or_default();

    let (base_url, email, token) = {
        let db_lock = state.db.lock().map_err(|e| e.to_string())?;
        let db = db_lock.as_ref().ok_or("Database not initialized")?;
        get_jira_connection(db, "Jira token not found")?
    };

    let vis = parse_comment_visibility(visibility);
    let client = JiraClient::new(&base_url, &email, &token);
    client
        .add_comment_with_citations(&ticket_key, &response_text, &citations, vis)
        .await
        .map_err(|e| e.to_string())
}

pub(crate) async fn get_jira_transitions_impl(
    state: State<'_, AppState>,
    ticket_key: String,
) -> Result<Vec<crate::jira::JiraTransition>, String> {
    validate_ticket_id(&ticket_key).map_err(|e| e.to_string())?;

    let (base_url, email, token) = {
        let db_guard = state
            .db
            .lock()
            .map_err(|e| format!("DB lock error: {}", e))?;
        let db = db_guard.as_ref().ok_or("Database not initialized")?;
        get_jira_connection(db, "Jira API token not found")?
    };

    let client = JiraClient::new(&base_url, &email, &token);
    client
        .get_transitions(&ticket_key)
        .await
        .map_err(|e| e.to_string())
}

pub(crate) async fn transition_jira_ticket_impl(
    state: State<'_, AppState>,
    ticket_key: String,
    transition_id: String,
    draft_id: Option<String>,
) -> Result<(), String> {
    validate_ticket_id(&ticket_key).map_err(|e| e.to_string())?;

    let (base_url, email, token) = {
        let db_guard = state
            .db
            .lock()
            .map_err(|e| format!("DB lock error: {}", e))?;
        let db = db_guard.as_ref().ok_or("Database not initialized")?;
        get_jira_connection(db, "Jira API token not found")?
    };

    let client = JiraClient::new(&base_url, &email, &token);

    let ticket = client
        .get_ticket(&ticket_key)
        .await
        .map_err(|e| e.to_string())?;
    let old_status = ticket.status.clone();

    let transitions = client
        .get_transitions(&ticket_key)
        .await
        .map_err(|e| e.to_string())?;
    let new_status = transitions
        .iter()
        .find(|t| t.id == transition_id)
        .map(|t| t.to_status.clone())
        .unwrap_or_else(|| "Unknown".to_string());

    client
        .transition_ticket(&ticket_key, &transition_id)
        .await
        .map_err(|e| e.to_string())?;

    let now = chrono::Utc::now().to_rfc3339();
    let transition_log = crate::db::JiraStatusTransition {
        id: uuid::Uuid::new_v4().to_string(),
        draft_id,
        ticket_key: ticket_key.clone(),
        old_status: Some(old_status),
        new_status,
        comment_id: None,
        transitioned_at: now,
    };

    let db_guard = state
        .db
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;
    db.save_jira_transition(&transition_log)
        .map_err(|e| e.to_string())?;

    Ok(())
}

pub(crate) async fn post_and_transition_impl(
    state: State<'_, AppState>,
    ticket_key: String,
    comment: String,
    transition_id: Option<String>,
    draft_id: Option<String>,
) -> Result<String, String> {
    validate_ticket_id(&ticket_key).map_err(|e| e.to_string())?;
    validate_non_empty(&comment).map_err(|e| e.to_string())?;

    let (base_url, email, token) = {
        let db_guard = state
            .db
            .lock()
            .map_err(|e| format!("DB lock error: {}", e))?;
        let db = db_guard.as_ref().ok_or("Database not initialized")?;
        get_jira_connection(db, "Jira API token not found")?
    };

    let client = JiraClient::new(&base_url, &email, &token);

    let comment_id = client
        .add_comment(&ticket_key, &comment, None)
        .await
        .map_err(|e| e.to_string())?;

    if let Some(tid) = transition_id {
        let ticket = client
            .get_ticket(&ticket_key)
            .await
            .map_err(|e| e.to_string())?;
        let old_status = ticket.status.clone();

        let transitions = client
            .get_transitions(&ticket_key)
            .await
            .map_err(|e| e.to_string())?;
        let new_status = transitions
            .iter()
            .find(|t| t.id == tid)
            .map(|t| t.to_status.clone())
            .unwrap_or_else(|| "Unknown".to_string());

        client
            .transition_ticket(&ticket_key, &tid)
            .await
            .map_err(|e| e.to_string())?;

        let now = chrono::Utc::now().to_rfc3339();
        let transition_log = crate::db::JiraStatusTransition {
            id: uuid::Uuid::new_v4().to_string(),
            draft_id,
            ticket_key: ticket_key.clone(),
            old_status: Some(old_status),
            new_status,
            comment_id: Some(comment_id.clone()),
            transitioned_at: now,
        };

        let db_guard = state
            .db
            .lock()
            .map_err(|e| format!("DB lock error: {}", e))?;
        let db = db_guard.as_ref().ok_or("Database not initialized")?;
        db.save_jira_transition(&transition_log)
            .map_err(|e| e.to_string())?;
    }

    Ok(comment_id)
}
