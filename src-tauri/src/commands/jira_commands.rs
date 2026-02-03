use super::*;

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

pub(crate) fn get_jira_config_impl(state: State<'_, AppState>) -> Result<Option<JiraConfig>, String> {
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
    // Validate URL format
    validate_url(&base_url).map_err(|e| e.to_string())?;

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
