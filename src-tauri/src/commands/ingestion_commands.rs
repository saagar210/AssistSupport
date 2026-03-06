use super::*;

const NETWORK_INGEST_POLICY_ENV: &str = "ASSISTSUPPORT_ENABLE_NETWORK_INGEST";

fn network_ingest_enabled_by_policy() -> bool {
    std::env::var(NETWORK_INGEST_POLICY_ENV)
        .ok()
        .map(|value| value.trim().to_ascii_lowercase())
        .map(|value| matches!(value.as_str(), "1" | "true" | "yes" | "on" | "enabled"))
        .unwrap_or(false)
}

pub(crate) fn ingest_kb_from_disk_impl(
    state: State<'_, AppState>,
    folder_path: String,
    namespace_id: String,
) -> Result<DiskIngestResultResponse, String> {
    use crate::kb::ingest::disk::DiskIngester;
    use std::path::Path;

    let namespace_id =
        normalize_and_validate_namespace_id(&namespace_id).map_err(|e| e.to_string())?;

    let validated_path = validate_within_home(Path::new(&folder_path)).map_err(|e| match e {
        ValidationError::PathTraversal => "Folder must be within your home directory".to_string(),
        ValidationError::InvalidFormat(msg) if msg.contains("sensitive") => {
            "This directory cannot be used as it contains sensitive data".to_string()
        }
        _ => format!("Invalid folder path: {}", e),
    })?;

    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    db.ensure_namespace_exists(&namespace_id)
        .map_err(|e| e.to_string())?;

    let ingester = DiskIngester::new();
    let result = ingester
        .ingest_folder(db, &validated_path, &namespace_id)
        .map_err(|e| e.to_string())?;

    Ok(DiskIngestResultResponse {
        total_files: result.total_files,
        ingested: result.ingested,
        skipped: result.skipped,
        errors: result.errors,
        documents: result
            .documents
            .into_iter()
            .map(|d| IngestResult {
                document_id: d.id,
                title: d.title,
                source_uri: d.source_uri,
                chunk_count: d.chunk_count,
                word_count: d.word_count,
            })
            .collect(),
    })
}

pub(crate) fn ingest_url_impl(
    state: State<'_, AppState>,
    url: String,
    namespace_id: String,
) -> Result<IngestResult, String> {
    use crate::kb::ingest::web::{WebIngestConfig, WebIngester};
    use crate::kb::ingest::CancellationToken;

    if !network_ingest_enabled_by_policy() {
        return Err(
            "Network ingestion is disabled by policy. Set ASSISTSUPPORT_ENABLE_NETWORK_INGEST=1 and restart AssistSupport to enable."
                .to_string(),
        );
    }

    let namespace_id =
        normalize_and_validate_namespace_id(&namespace_id).map_err(|e| e.to_string())?;

    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    db.ensure_namespace_exists(&namespace_id)
        .map_err(|e| e.to_string())?;

    let config = WebIngestConfig::default();
    let cancel_token = CancellationToken::new();

    let result = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            let ingester = WebIngester::new(config).await.map_err(|e| e.to_string())?;
            ingester
                .ingest_page(db, &url, &namespace_id, &cancel_token, None)
                .await
                .map_err(|e| e.to_string())
        })
    })?;

    Ok(IngestResult {
        document_id: result.id,
        title: result.title,
        source_uri: result.source_uri,
        chunk_count: result.chunk_count,
        word_count: result.word_count,
    })
}

pub(crate) fn ingest_youtube_impl(
    state: State<'_, AppState>,
    url: String,
    namespace_id: String,
) -> Result<IngestResult, String> {
    use crate::kb::ingest::youtube::{YouTubeIngestConfig, YouTubeIngester};
    use crate::kb::ingest::CancellationToken;

    if !network_ingest_enabled_by_policy() {
        return Err(
            "Network ingestion is disabled by policy. Set ASSISTSUPPORT_ENABLE_NETWORK_INGEST=1 and restart AssistSupport to enable."
                .to_string(),
        );
    }

    let namespace_id =
        normalize_and_validate_namespace_id(&namespace_id).map_err(|e| e.to_string())?;

    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    db.ensure_namespace_exists(&namespace_id)
        .map_err(|e| e.to_string())?;

    let config = YouTubeIngestConfig::default();
    let ingester = YouTubeIngester::new(config);

    if !ingester.check_ytdlp_available() {
        return Err("yt-dlp not found. Install with: brew install yt-dlp".to_string());
    }

    let cancel_token = CancellationToken::new();

    let result = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            ingester
                .ingest_video(db, &url, &namespace_id, &cancel_token, None)
                .await
        })
    })
    .map_err(|e| e.to_string())?;

    Ok(IngestResult {
        document_id: result.id,
        title: result.title,
        source_uri: result.source_uri,
        chunk_count: result.chunk_count,
        word_count: result.word_count,
    })
}

pub(crate) fn ingest_github_impl(
    state: State<'_, AppState>,
    repo_path: String,
    namespace_id: String,
) -> Result<Vec<IngestResult>, String> {
    use crate::kb::ingest::github::{GitHubIngestConfig, GitHubIngester};
    use crate::kb::ingest::CancellationToken;
    use std::path::Path;

    let namespace_id =
        normalize_and_validate_namespace_id(&namespace_id).map_err(|e| e.to_string())?;

    let validated_path = validate_within_home(Path::new(&repo_path)).map_err(|e| match e {
        ValidationError::PathTraversal => {
            "Repository must be within your home directory".to_string()
        }
        ValidationError::InvalidFormat(msg) if msg.contains("sensitive") => {
            "This directory cannot be used as it contains sensitive data".to_string()
        }
        _ => format!("Invalid repository path: {}", e),
    })?;

    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    db.ensure_namespace_exists(&namespace_id)
        .map_err(|e| e.to_string())?;

    let config = GitHubIngestConfig::default();
    let ingester = GitHubIngester::new(config);
    let cancel_token = CancellationToken::new();

    let results = ingester
        .ingest_local_repo(db, &validated_path, &namespace_id, &cancel_token, None)
        .map_err(|e| e.to_string())?;

    Ok(results
        .into_iter()
        .map(|r| IngestResult {
            document_id: r.id,
            title: r.title,
            source_uri: r.source_uri,
            chunk_count: r.chunk_count,
            word_count: r.word_count,
        })
        .collect())
}

pub(crate) fn ingest_github_remote_impl(
    state: State<'_, AppState>,
    repo_url: String,
    namespace_id: String,
) -> Result<Vec<IngestResult>, String> {
    use crate::kb::ingest::github::{parse_https_repo_url, GitHubIngestConfig, GitHubIngester};
    use crate::kb::ingest::CancellationToken;

    if !network_ingest_enabled_by_policy() {
        return Err(
            "Network ingestion is disabled by policy. Set ASSISTSUPPORT_ENABLE_NETWORK_INGEST=1 and restart AssistSupport to enable."
                .to_string(),
        );
    }

    let namespace_id =
        normalize_and_validate_namespace_id(&namespace_id).map_err(|e| e.to_string())?;

    let remote = parse_https_repo_url(&repo_url).map_err(|e| e.to_string())?;
    let host_key = normalize_github_host(&remote.host_port)?;
    let token_key = format!("{}{}", GITHUB_TOKEN_PREFIX, host_key);
    let token = FileKeyStore::get_token(&token_key).map_err(|e| e.to_string())?;

    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    db.ensure_namespace_exists(&namespace_id)
        .map_err(|e| e.to_string())?;

    let config = GitHubIngestConfig::default();
    let ingester = GitHubIngester::new(config);
    let cancel_token = CancellationToken::new();

    let results = ingester
        .ingest_remote_repo(
            db,
            &repo_url,
            token.as_deref(),
            &namespace_id,
            &cancel_token,
            None,
        )
        .map_err(|e| e.to_string())?;

    Ok(results
        .into_iter()
        .map(|r| IngestResult {
            document_id: r.id,
            title: r.title,
            source_uri: r.source_uri,
            chunk_count: r.chunk_count,
            word_count: r.word_count,
        })
        .collect())
}

pub(crate) fn process_source_file_impl(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<BatchIngestResult, String> {
    use crate::kb::ingest::batch::{BatchIngestConfig, BatchIngester};
    use crate::kb::ingest::CancellationToken;
    use crate::sources::SourceFile;
    use std::path::Path;

    let path = Path::new(&file_path);
    if !path.exists() {
        return Err(format!("Source file not found: {}", file_path));
    }

    let validated_path = validate_within_home(path).map_err(|e| match e {
        ValidationError::PathTraversal => {
            "Source file must be within your home directory".to_string()
        }
        ValidationError::InvalidFormat(msg) if msg.contains("sensitive") => {
            "This path is blocked because it contains sensitive data".to_string()
        }
        _ => format!("Invalid source file path: {}", e),
    })?;

    if !validated_path.is_file() {
        return Err("Source file path is not a file".into());
    }

    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    let source_file = SourceFile::from_path(&validated_path).map_err(|e| e.to_string())?;

    db.ensure_namespace_exists(&source_file.namespace)
        .map_err(|e| e.to_string())?;

    let sources: Vec<String> = source_file
        .enabled_sources()
        .map(|s| s.uri.clone())
        .collect();

    let config = BatchIngestConfig::default();
    let cancel_token = CancellationToken::new();
    let namespace = source_file.namespace.clone();

    let result = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            let ingester = BatchIngester::new(config)
                .await
                .map_err(|e| e.to_string())?;
            Ok::<_, String>(
                ingester
                    .ingest_from_strings(db, &sources, &namespace, &cancel_token, None)
                    .await,
            )
        })
    })?;

    Ok(BatchIngestResult {
        successful: result
            .successful
            .into_iter()
            .map(|r| IngestResult {
                document_id: r.id,
                title: r.title,
                source_uri: r.source_uri,
                chunk_count: r.chunk_count,
                word_count: r.word_count,
            })
            .collect(),
        failed: result
            .failed
            .into_iter()
            .map(|f| FailedSource {
                source: f.source,
                error: f.error,
            })
            .collect(),
        cancelled: result.cancelled,
    })
}

pub(crate) fn list_namespaces_impl(
    state: State<'_, AppState>,
) -> Result<Vec<crate::db::Namespace>, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.list_namespaces().map_err(|e| e.to_string())
}

pub(crate) fn list_namespaces_with_counts_impl(
    state: State<'_, AppState>,
) -> Result<Vec<crate::db::NamespaceWithCounts>, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.list_namespaces_with_counts().map_err(|e| e.to_string())
}

pub(crate) fn create_namespace_impl(
    state: State<'_, AppState>,
    name: String,
    description: Option<String>,
    color: Option<String>,
) -> Result<crate::db::Namespace, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.create_namespace(&name, description.as_deref(), color.as_deref())
        .map_err(|e| e.to_string())
}

pub(crate) fn rename_namespace_impl(
    state: State<'_, AppState>,
    old_name: String,
    new_name: String,
) -> Result<(), String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.rename_namespace(&old_name, &new_name)
        .map_err(|e| e.to_string())
}

pub(crate) fn delete_namespace_impl(
    state: State<'_, AppState>,
    name: String,
) -> Result<(), String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.delete_namespace(&name).map_err(|e| e.to_string())
}

pub(crate) fn list_ingest_sources_impl(
    state: State<'_, AppState>,
    namespace_id: Option<String>,
) -> Result<Vec<crate::db::IngestSource>, String> {
    let namespace_id = namespace_id
        .map(|ns| normalize_and_validate_namespace_id(&ns))
        .transpose()
        .map_err(|e| e.to_string())?;

    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.list_ingest_sources(namespace_id.as_deref())
        .map_err(|e| e.to_string())
}

pub(crate) fn delete_ingest_source_impl(
    state: State<'_, AppState>,
    source_id: String,
) -> Result<(), String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.delete_ingest_source(&source_id)
        .map_err(|e| e.to_string())
}

pub(crate) fn get_source_health_impl(
    state: State<'_, AppState>,
    namespace_id: Option<String>,
) -> Result<SourceHealthSummary, String> {
    let namespace_id = namespace_id
        .map(|ns| normalize_and_validate_namespace_id(&ns))
        .transpose()
        .map_err(|e| e.to_string())?;

    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    let sql = r#"
        SELECT
            s.id,
            s.source_type,
            s.source_uri,
            s.title,
            s.status,
            s.error_message,
            s.last_ingested_at,
            COUNT(d.id) as document_count,
            CASE
                WHEN s.last_ingested_at IS NOT NULL
                THEN julianday('now') - julianday(s.last_ingested_at)
                ELSE NULL
            END as days_since
        FROM ingest_sources s
        LEFT JOIN kb_documents d ON d.source_id = s.id
        WHERE (?1 IS NULL OR s.namespace_id = ?1)
        GROUP BY s.id
        ORDER BY s.updated_at DESC
    "#;

    let mut stmt = db.conn().prepare(sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([namespace_id], |row| {
            Ok(SourceHealth {
                id: row.get(0)?,
                source_type: row.get(1)?,
                source_uri: row.get(2)?,
                title: row.get(3)?,
                status: row.get(4)?,
                error_message: row.get(5)?,
                last_ingested_at: row.get(6)?,
                document_count: row.get::<_, i64>(7)? as u32,
                days_since_refresh: row.get(8)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let sources: Vec<SourceHealth> = rows.filter_map(|r| r.ok()).collect();

    let mut summary = SourceHealthSummary {
        total_sources: sources.len() as u32,
        active_sources: 0,
        stale_sources: 0,
        error_sources: 0,
        pending_sources: 0,
        sources,
    };

    for source in &summary.sources {
        match source.status.as_str() {
            "active" => summary.active_sources += 1,
            "stale" => summary.stale_sources += 1,
            "error" => summary.error_sources += 1,
            "pending" => summary.pending_sources += 1,
            _ => {}
        }
    }

    Ok(summary)
}

pub(crate) fn retry_source_impl(
    state: State<'_, AppState>,
    source_id: String,
) -> Result<IngestResult, String> {
    let source = {
        let db_lock = state.db.lock().map_err(|e| e.to_string())?;
        let db = db_lock.as_ref().ok_or("Database not initialized")?;
        db.get_ingest_source(&source_id)
            .map_err(|e| e.to_string())?
    };

    {
        let db_lock = state.db.lock().map_err(|e| e.to_string())?;
        let db = db_lock.as_ref().ok_or("Database not initialized")?;
        db.update_ingest_source_status(&source_id, "pending", None)
            .map_err(|e| e.to_string())?;
    }

    match source.source_type.as_str() {
        "web" => ingest_url_impl(state, source.source_uri, source.namespace_id),
        "youtube" => ingest_youtube_impl(state, source.source_uri, source.namespace_id),
        "github" => {
            let results: Vec<IngestResult> =
                ingest_github_impl(state, source.source_uri.clone(), source.namespace_id)?;
            Ok(IngestResult {
                document_id: source_id,
                title: source.title.unwrap_or_else(|| "Repository".to_string()),
                source_uri: source.source_uri,
                chunk_count: results.iter().map(|r| r.chunk_count).sum(),
                word_count: results.iter().map(|r| r.word_count).sum(),
            })
        }
        _ => Err(format!("Unknown source type: {}", source.source_type)),
    }
}

pub(crate) fn mark_stale_sources_impl(
    state: State<'_, AppState>,
    days_threshold: Option<u32>,
) -> Result<u32, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    let days = days_threshold.unwrap_or(7) as i64;

    let sql = r#"
        UPDATE ingest_sources
        SET status = 'stale', updated_at = datetime('now')
        WHERE status = 'active'
        AND last_ingested_at IS NOT NULL
        AND julianday('now') - julianday(last_ingested_at) > ?
    "#;

    let count = db.conn().execute(sql, [days]).map_err(|e| e.to_string())?;
    Ok(count as u32)
}

pub(crate) fn get_document_chunks_impl(
    state: State<'_, AppState>,
    document_id: String,
) -> Result<Vec<DocumentChunk>, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    let chunks: Vec<DocumentChunk> = db
        .conn()
        .prepare(
            "SELECT id, chunk_index, heading_path, content, word_count
             FROM kb_chunks WHERE document_id = ? ORDER BY chunk_index",
        )
        .map_err(|e| e.to_string())?
        .query_map([&document_id], |row| {
            Ok(DocumentChunk {
                id: row.get(0)?,
                chunk_index: row.get(1)?,
                heading_path: row.get(2)?,
                content: row.get(3)?,
                word_count: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(chunks)
}

pub(crate) fn delete_kb_document_impl(
    state: State<'_, AppState>,
    document_id: String,
) -> Result<(), String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    db.conn()
        .execute("DELETE FROM kb_documents WHERE id = ?", [&document_id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub(crate) fn clear_knowledge_data_impl(
    state: State<'_, AppState>,
    namespace_id: Option<String>,
) -> Result<(), String> {
    let namespace_id = namespace_id
        .map(|ns| normalize_and_validate_namespace_id(&ns))
        .transpose()
        .map_err(|e| e.to_string())?;

    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    match namespace_id {
        Some(ns) => {
            db.conn()
                .execute("DELETE FROM kb_documents WHERE namespace_id = ?", [&ns])
                .map_err(|e| e.to_string())?;
            db.conn()
                .execute("DELETE FROM ingest_sources WHERE namespace_id = ?", [&ns])
                .map_err(|e| e.to_string())?;
        }
        None => {
            db.conn()
                .execute("DELETE FROM kb_chunks", [])
                .map_err(|e| e.to_string())?;
            db.conn()
                .execute("DELETE FROM kb_documents", [])
                .map_err(|e| e.to_string())?;
            db.conn()
                .execute("DELETE FROM ingest_runs", [])
                .map_err(|e| e.to_string())?;
            db.conn()
                .execute("DELETE FROM ingest_sources", [])
                .map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

pub(crate) fn check_ytdlp_available_impl() -> Result<bool, String> {
    use std::process::Command;

    Ok(Command::new("yt-dlp")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false))
}

pub(crate) fn list_document_versions_impl(
    state: State<'_, AppState>,
    document_id: String,
) -> Result<Vec<crate::db::DocumentVersion>, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    db.list_document_versions(&document_id)
        .map_err(|e| e.to_string())
}

pub(crate) fn rollback_document_impl(
    state: State<'_, AppState>,
    document_id: String,
    version_id: String,
) -> Result<(), String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    db.rollback_document(&document_id, &version_id)
        .map_err(|e| e.to_string())
}

pub(crate) fn update_source_trust_impl(
    state: State<'_, AppState>,
    source_id: String,
    trust_score: f64,
) -> Result<(), String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    db.update_source_trust(&source_id, trust_score)
        .map_err(|e| e.to_string())
}

pub(crate) fn set_source_pinned_impl(
    state: State<'_, AppState>,
    source_id: String,
    pinned: bool,
) -> Result<(), String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    db.set_source_pinned(&source_id, pinned)
        .map_err(|e| e.to_string())
}

pub(crate) fn set_source_review_status_impl(
    state: State<'_, AppState>,
    source_id: String,
    status: String,
) -> Result<(), String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    db.set_source_review_status(&source_id, &status)
        .map_err(|e| e.to_string())
}

pub(crate) fn get_stale_sources_impl(
    state: State<'_, AppState>,
    namespace_id: Option<String>,
) -> Result<Vec<crate::db::IngestSource>, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    db.get_stale_sources(namespace_id.as_deref())
        .map_err(|e| e.to_string())
}

pub(crate) fn add_namespace_rule_impl(
    state: State<'_, AppState>,
    namespace_id: String,
    rule_type: String,
    pattern_type: String,
    pattern: String,
    reason: Option<String>,
) -> Result<String, String> {
    let namespace_id =
        normalize_and_validate_namespace_id(&namespace_id).map_err(|e| e.to_string())?;

    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    db.add_namespace_rule(
        &namespace_id,
        &rule_type,
        &pattern_type,
        &pattern,
        reason.as_deref(),
    )
    .map_err(|e| e.to_string())
}

pub(crate) fn delete_namespace_rule_impl(
    state: State<'_, AppState>,
    rule_id: String,
) -> Result<(), String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    db.delete_namespace_rule(&rule_id)
        .map_err(|e| e.to_string())
}

pub(crate) fn list_namespace_rules_impl(
    state: State<'_, AppState>,
    namespace_id: String,
) -> Result<Vec<crate::db::NamespaceRule>, String> {
    let namespace_id =
        normalize_and_validate_namespace_id(&namespace_id).map_err(|e| e.to_string())?;

    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    db.list_namespace_rules(&namespace_id)
        .map_err(|e| e.to_string())
}
