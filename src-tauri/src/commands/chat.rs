use std::collections::HashMap;
use std::sync::Mutex as StdMutex;

use tauri::{AppHandle, Manager, State};
use tauri::Emitter;
use tokio_util::sync::CancellationToken;

use crate::audit::{self, AuditAction};
use crate::error::AppError;
use crate::models::{Citation, Conversation, Message, PaginatedResponse};
use crate::ollama::{self, ChatMessage};
use crate::state::{get_conn, AppState};

use super::search::SearchResult;

// Global map of active generation cancel tokens
static CANCEL_TOKENS: std::sync::LazyLock<StdMutex<HashMap<String, CancellationToken>>> =
    std::sync::LazyLock::new(|| StdMutex::new(HashMap::new()));

/// Send a chat message: run RAG pipeline (search + context + stream LLM response).
#[tauri::command]
pub async fn send_chat_message(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    conversation_id: String,
    collection_id: String,
    user_message: String,
    model_override: Option<String>,
) -> Result<(), AppError> {
    let now = chrono::Utc::now().to_rfc3339();
    let user_msg_id = uuid::Uuid::new_v4().to_string();

    // 1. Save user message to DB + read all settings we need, then drop connection
    let (host, port, embedding_model, chat_model, context_chunks, rrf_k, vector_top_k, keyword_top_k) = {
        let conn = get_conn(state.inner())?;

        conn.execute(
            "INSERT INTO messages (id, conversation_id, role, content, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![user_msg_id, conversation_id, "user", user_message, now],
        )?;

        // Update conversation timestamp
        conn.execute(
            "UPDATE conversations SET updated_at = ?1 WHERE id = ?2",
            rusqlite::params![now, conversation_id],
        )?;

        // Audit log for user message
        let _ = audit::log_audit(&conn, AuditAction::ChatMessage, Some("conversation"), Some(&conversation_id), &serde_json::json!({"role": "user"}));

        // Track chat message metric
        state.inner().metrics.increment(crate::metrics::MetricCounter::ChatMessagesSent);

        let host: String = conn.query_row(
            "SELECT value FROM settings WHERE key = 'ollama_host'", [], |row: &rusqlite::Row| row.get(0),
        )?;
        let port: String = conn.query_row(
            "SELECT value FROM settings WHERE key = 'ollama_port'", [], |row: &rusqlite::Row| row.get(0),
        )?;
        let embedding_model: String = conn.query_row(
            "SELECT value FROM settings WHERE key = 'embedding_model'", [], |row: &rusqlite::Row| row.get(0),
        )?;
        let chat_model: String = conn.query_row(
            "SELECT value FROM settings WHERE key = 'chat_model'", [], |row: &rusqlite::Row| row.get(0),
        )?;
        let context_chunks: String = conn.query_row(
            "SELECT value FROM settings WHERE key = 'context_chunks'", [], |row: &rusqlite::Row| row.get(0),
        ).unwrap_or_else(|_| "5".to_string());
        let rrf_k: String = conn.query_row(
            "SELECT value FROM settings WHERE key = 'rrf_k'", [], |row: &rusqlite::Row| row.get(0),
        ).unwrap_or_else(|_| "60".to_string());
        let vector_top_k: String = conn.query_row(
            "SELECT value FROM settings WHERE key = 'vector_top_k'", [], |row: &rusqlite::Row| row.get(0),
        ).unwrap_or_else(|_| "20".to_string());
        let keyword_top_k: String = conn.query_row(
            "SELECT value FROM settings WHERE key = 'keyword_top_k'", [], |row: &rusqlite::Row| row.get(0),
        ).unwrap_or_else(|_| "20".to_string());

        (host, port, embedding_model, chat_model, context_chunks, rrf_k, vector_top_k, keyword_top_k)
    };

    let context_k: usize = context_chunks.parse().unwrap_or(5);
    let rrf_k_val: f64 = rrf_k.parse().unwrap_or(60.0);
    let vec_top_k: usize = vector_top_k.parse().unwrap_or(20);
    let kw_top_k: usize = keyword_top_k.parse().unwrap_or(20);

    // 2. Run hybrid search — needs embedding (async), then DB queries
    //    We need the lock again for the DB portion of search, but generate_embedding is async
    let query_embedding =
        ollama::generate_embedding(&host, &port, &embedding_model, &user_message).await?;

    let context_results: Vec<SearchResult> = {
        let conn = get_conn(state.inner())?;
        // Use the internal search that works on a Connection directly
        // We inline the search here since hybrid_search_internal is async but we only
        // need the DB parts (we already have the embedding)
        let vr = crate::commands::search::vector_search_in_db_with_embedding(
            &conn, &collection_id, &query_embedding, vec_top_k,
        )?;
        let kr = crate::commands::search::keyword_search_in_db(
            &conn, &collection_id, &user_message, kw_top_k,
        )?;
        crate::commands::search::reciprocal_rank_fusion_pub(vr, kr, rrf_k_val, context_k)
    };

    // 3. Build system prompt with context
    let system_prompt = build_system_prompt(&context_results);

    // 4. Load conversation history (last 10 messages)
    let history: Vec<ChatMessage> = {
        let conn = get_conn(state.inner())?;
        load_conversation_history(&conn, &conversation_id, 10)?
    };

    // 5. Build full messages array
    let mut messages: Vec<ChatMessage> = Vec::new();
    messages.push(ChatMessage {
        role: "system".to_string(),
        content: system_prompt,
    });
    messages.extend(history);
    messages.push(ChatMessage {
        role: "user".to_string(),
        content: user_message.clone(),
    });

    // 5b. Use model override if provided
    let active_model = model_override.unwrap_or(chat_model.clone());

    // 6. Create cancellation token and stream response
    let cancel_token = CancellationToken::new();
    {
        let mut tokens = CANCEL_TOKENS.lock().map_err(|e| AppError::LockFailed(e.to_string()))?;
        tokens.insert(conversation_id.clone(), cancel_token.clone());
    }

    let full_response = ollama::chat_stream(&host, &port, &active_model, &messages, &app_handle, Some(&cancel_token)).await?;

    // Remove cancel token
    {
        let mut tokens = CANCEL_TOKENS.lock().map_err(|e| AppError::LockFailed(e.to_string()))?;
        tokens.remove(&conversation_id);
    }

    // 7. Save assistant message + citations to DB
    let assistant_msg_id = uuid::Uuid::new_v4().to_string();
    let msg_now = chrono::Utc::now().to_rfc3339();

    let citations: Vec<Citation> = {
        let conn = get_conn(state.inner())?;

        conn.execute(
            "INSERT INTO messages (id, conversation_id, role, content, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![assistant_msg_id, conversation_id, "assistant", full_response, msg_now],
        )?;

        // Save citations (one per context chunk)
        let mut saved_citations = Vec::new();
        for result in &context_results {
            let citation_id = uuid::Uuid::new_v4().to_string();
            // Truncate snippet to first 200 chars
            let snippet: String = result.content.chars().take(200).collect();

            conn.execute(
                "INSERT INTO citations (id, message_id, chunk_id, document_id, document_title, section_title, page_number, relevance_score, snippet)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                rusqlite::params![
                    citation_id,
                    assistant_msg_id,
                    result.chunk_id,
                    result.document_id,
                    result.document_title,
                    result.section_title,
                    result.page_number,
                    result.score,
                    snippet,
                ],
            )?;

            saved_citations.push(Citation {
                id: citation_id,
                message_id: assistant_msg_id.clone(),
                chunk_id: result.chunk_id.clone(),
                document_id: result.document_id.clone(),
                document_title: result.document_title.clone(),
                section_title: result.section_title.clone(),
                page_number: result.page_number,
                relevance_score: result.score,
                snippet,
            });
        }

        saved_citations
    };

    // 8. Emit chat-complete event with full message object
    let assistant_message = Message {
        id: assistant_msg_id,
        conversation_id: conversation_id.clone(),
        role: "assistant".to_string(),
        content: full_response.clone(),
        created_at: msg_now,
    };

    let _ = app_handle.emit(
        "chat-complete",
        serde_json::json!({
            "message": assistant_message,
            "citations": citations,
        }),
    );

    // 9. Auto-title: if this is the first exchange (2 messages), generate a title
    let msg_count: i64 = {
        let conn = get_conn(state.inner())?;
        conn.query_row(
            "SELECT COUNT(*) FROM messages WHERE conversation_id = ?1",
            rusqlite::params![conversation_id],
            |row| row.get(0),
        )?
    };

    if msg_count == 2 {
        let snippet: String = full_response.chars().take(500).collect();
        let conv_id = conversation_id.clone();
        let host_c = host.clone();
        let port_c = port.clone();
        let model_c = chat_model;
        let app = app_handle.clone();

        // Fire-and-forget: don't block on title generation
        tauri::async_runtime::spawn(async move {
            let title_messages = vec![
                ChatMessage {
                    role: "user".to_string(),
                    content: user_message.clone(),
                },
                ChatMessage {
                    role: "assistant".to_string(),
                    content: snippet,
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: "Summarize this conversation in 3-5 words for a title. Reply with ONLY the title, nothing else.".to_string(),
                },
            ];

            if let Ok(title) = ollama::chat_once(&host_c, &port_c, &model_c, &title_messages).await {
                let title = title.trim().trim_matches('"').to_string();
                if !title.is_empty() {
                    let now = chrono::Utc::now().to_rfc3339();
                    let state: State<'_, AppState> = app.state();
                    if let Ok(conn) = crate::state::get_conn(state.inner()) {
                        let _ = conn.execute(
                            "UPDATE conversations SET title = ?1, updated_at = ?2 WHERE id = ?3",
                            rusqlite::params![title, now, conv_id],
                        );
                        let _ = app.emit(
                            "conversation-title-updated",
                            serde_json::json!({"conversationId": conv_id, "title": title}),
                        );
                    }
                }
            }
        });
    }

    Ok(())
}

/// Build a system prompt injecting retrieved context chunks.
fn build_system_prompt(context: &[SearchResult]) -> String {
    let mut prompt = String::from(
        "You are a helpful research assistant. Answer based ONLY on the provided context.\n\
         If the context doesn't contain enough information, say so clearly.\n\
         Cite sources by referencing document name and section.\n\n\
         Context:\n",
    );

    for result in context {
        let section = result
            .section_title
            .as_deref()
            .unwrap_or("(no section)");
        prompt.push_str(&format!(
            "[Source: {}, Section: {}] {}\n\n",
            result.document_title, section, result.content
        ));
    }

    prompt
}

/// Load the last N messages from a conversation as ChatMessages.
/// Uses a subquery to fetch the last N by created_at DESC, then re-orders ASC.
fn load_conversation_history(
    conn: &rusqlite::Connection,
    conversation_id: &str,
    limit: usize,
) -> Result<Vec<ChatMessage>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT role, content FROM (
            SELECT role, content, created_at FROM messages
            WHERE conversation_id = ?1
            ORDER BY created_at DESC
            LIMIT ?2
        ) sub ORDER BY created_at ASC",
    )?;

    let messages: Vec<ChatMessage> = stmt
        .query_map(rusqlite::params![conversation_id, limit as i64], |row| {
            Ok(ChatMessage {
                role: row.get(0)?,
                content: row.get(1)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(messages)
}

/// Cancel an active chat generation for a conversation.
#[tauri::command]
pub fn cancel_chat_generation(conversation_id: String) -> Result<(), AppError> {
    let tokens = CANCEL_TOKENS
        .lock()
        .map_err(|e| AppError::LockFailed(e.to_string()))?;
    if let Some(token) = tokens.get(&conversation_id) {
        token.cancel();
    }
    Ok(())
}

/// Delete the last assistant message from a conversation and return the last user message.
#[tauri::command]
pub fn delete_last_assistant_message(
    state: State<'_, AppState>,
    conversation_id: String,
) -> Result<String, AppError> {
    let conn = get_conn(state.inner())?;

    // Find last assistant message
    let last_assistant_id: String = conn.query_row(
        "SELECT id FROM messages WHERE conversation_id = ?1 AND role = 'assistant' ORDER BY created_at DESC LIMIT 1",
        rusqlite::params![conversation_id],
        |row| row.get(0),
    ).map_err(|_| AppError::NotFound("No assistant message found".to_string()))?;

    // Delete its citations then the message
    conn.execute(
        "DELETE FROM citations WHERE message_id = ?1",
        rusqlite::params![last_assistant_id],
    )?;
    conn.execute(
        "DELETE FROM messages WHERE id = ?1",
        rusqlite::params![last_assistant_id],
    )?;

    // Return last user message content
    let last_user_content: String = conn.query_row(
        "SELECT content FROM messages WHERE conversation_id = ?1 AND role = 'user' ORDER BY created_at DESC LIMIT 1",
        rusqlite::params![conversation_id],
        |row| row.get(0),
    ).map_err(|_| AppError::NotFound("No user message found".to_string()))?;

    Ok(last_user_content)
}

#[tauri::command]
pub async fn create_conversation(
    state: State<'_, AppState>,
    collection_id: String,
    title: String,
) -> Result<Conversation, AppError> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let conn = get_conn(state.inner())?;
    conn.execute(
        "INSERT INTO conversations (id, collection_id, title, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![id, collection_id, title, now, now],
    )?;

    // Audit log for conversation creation
    let _ = audit::log_audit(&conn, AuditAction::ConversationCreate, Some("conversation"), Some(&id), &serde_json::json!({"title": title}));

    Ok(Conversation {
        id,
        collection_id,
        title,
        created_at: now.clone(),
        updated_at: now,
    })
}

#[tauri::command]
pub fn list_conversations(
    state: State<'_, AppState>,
    collection_id: String,
    page: Option<usize>,
    page_size: Option<usize>,
) -> Result<PaginatedResponse<Conversation>, AppError> {
    let conn = get_conn(state.inner())?;
    let page = page.unwrap_or(1).max(1);
    let page_size = page_size.unwrap_or(30).max(1);
    let offset = (page - 1) * page_size;

    let total: i64 = conn.query_row(
        "SELECT COUNT(*) FROM conversations WHERE collection_id = ?1",
        rusqlite::params![collection_id],
        |row| row.get(0),
    )?;

    let mut stmt = conn.prepare(
        "SELECT id, collection_id, title, created_at, updated_at
         FROM conversations
         WHERE collection_id = ?1
         ORDER BY updated_at DESC
         LIMIT ?2 OFFSET ?3",
    )?;

    let conversations = stmt
        .query_map(rusqlite::params![collection_id, page_size as i64, offset as i64], |row| {
            Ok(Conversation {
                id: row.get(0)?,
                collection_id: row.get(1)?,
                title: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let has_more = (offset + conversations.len()) < total as usize;

    Ok(PaginatedResponse {
        items: conversations,
        total,
        page,
        page_size,
        has_more,
    })
}

#[tauri::command]
pub fn get_conversation_messages(
    state: State<'_, AppState>,
    conversation_id: String,
    page: Option<usize>,
    page_size: Option<usize>,
) -> Result<PaginatedResponse<Message>, AppError> {
    let conn = get_conn(state.inner())?;
    let page = page.unwrap_or(1).max(1);
    let page_size = page_size.unwrap_or(50).max(1);
    let offset = (page - 1) * page_size;

    let total: i64 = conn.query_row(
        "SELECT COUNT(*) FROM messages WHERE conversation_id = ?1",
        rusqlite::params![conversation_id],
        |row| row.get(0),
    )?;

    let mut stmt = conn.prepare(
        "SELECT id, conversation_id, role, content, created_at
         FROM messages
         WHERE conversation_id = ?1
         ORDER BY created_at ASC
         LIMIT ?2 OFFSET ?3",
    )?;

    let messages = stmt
        .query_map(rusqlite::params![conversation_id, page_size as i64, offset as i64], |row| {
            Ok(Message {
                id: row.get(0)?,
                conversation_id: row.get(1)?,
                role: row.get(2)?,
                content: row.get(3)?,
                created_at: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let has_more = (offset + messages.len()) < total as usize;

    Ok(PaginatedResponse {
        items: messages,
        total,
        page,
        page_size,
        has_more,
    })
}

#[tauri::command]
pub fn get_message_citations(
    state: State<'_, AppState>,
    message_id: String,
) -> Result<Vec<Citation>, AppError> {
    let conn = get_conn(state.inner())?;

    let mut stmt = conn.prepare(
        "SELECT id, message_id, chunk_id, document_id, document_title, section_title, page_number, relevance_score, snippet
         FROM citations
         WHERE message_id = ?1
         ORDER BY relevance_score DESC",
    )?;

    let citations = stmt
        .query_map(rusqlite::params![message_id], |row| {
            Ok(Citation {
                id: row.get(0)?,
                message_id: row.get(1)?,
                chunk_id: row.get(2)?,
                document_id: row.get(3)?,
                document_title: row.get(4)?,
                section_title: row.get(5)?,
                page_number: row.get(6)?,
                relevance_score: row.get(7)?,
                snippet: row.get(8)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(citations)
}

#[tauri::command]
pub fn delete_conversation(
    state: State<'_, AppState>,
    conversation_id: String,
) -> Result<(), AppError> {
    let conn = get_conn(state.inner())?;

    // Audit log before deletion
    let _ = audit::log_audit(&conn, AuditAction::ConversationDelete, Some("conversation"), Some(&conversation_id), &serde_json::json!({}));

    let rows = conn.execute(
        "DELETE FROM conversations WHERE id = ?1",
        rusqlite::params![conversation_id],
    )?;

    if rows == 0 {
        return Err(AppError::NotFound(format!(
            "Conversation '{}' not found",
            conversation_id
        )));
    }

    Ok(())
}

#[tauri::command]
pub fn rename_conversation(
    state: State<'_, AppState>,
    conversation_id: String,
    title: String,
) -> Result<(), AppError> {
    let title = title.trim().to_string();
    if title.is_empty() {
        return Err(AppError::Validation(
            "Conversation title cannot be empty".into(),
        ));
    }

    let now = chrono::Utc::now().to_rfc3339();
    let conn = get_conn(state.inner())?;

    // Audit log for conversation rename
    let _ = audit::log_audit(&conn, AuditAction::ConversationRename, Some("conversation"), Some(&conversation_id), &serde_json::json!({"title": title}));

    let rows = conn.execute(
        "UPDATE conversations SET title = ?1, updated_at = ?2 WHERE id = ?3",
        rusqlite::params![title, now, conversation_id],
    )?;

    if rows == 0 {
        return Err(AppError::NotFound(format!(
            "Conversation '{}' not found",
            conversation_id
        )));
    }

    Ok(())
}

#[tauri::command]
pub fn export_conversation_markdown(
    state: State<'_, AppState>,
    conversation_id: String,
) -> Result<String, AppError> {
    let conn = get_conn(state.inner())?;

    // Load conversation title
    let title: String = conn
        .query_row(
            "SELECT title FROM conversations WHERE id = ?1",
            rusqlite::params![conversation_id],
            |row: &rusqlite::Row| row.get(0),
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => {
                AppError::NotFound(format!("Conversation '{}' not found", conversation_id))
            }
            other => AppError::Database(other),
        })?;

    // Load all messages ordered by created_at ASC
    let mut msg_stmt = conn.prepare(
        "SELECT id, role, content FROM messages WHERE conversation_id = ?1 ORDER BY created_at ASC",
    )?;

    let messages: Vec<(String, String, String)> = msg_stmt
        .query_map(rusqlite::params![conversation_id], |row: &rusqlite::Row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    // Prepare citation query
    let mut citation_stmt = conn.prepare(
        "SELECT document_title, section_title FROM citations WHERE message_id = ?1 ORDER BY relevance_score DESC",
    )?;

    let mut md = format!("# {}\n", title);

    for (msg_id, role, content) in &messages {
        md.push('\n');

        match role.as_str() {
            "user" => {
                md.push_str(&format!("**User**: {}\n", content));
            }
            "assistant" => {
                md.push_str(&format!("**Assistant**: {}\n", content));

                // Load citations for this assistant message
                let citations: Vec<(String, Option<String>)> = citation_stmt
                    .query_map(rusqlite::params![msg_id], |row: &rusqlite::Row| {
                        Ok((
                            row.get::<_, String>(0)?,
                            row.get::<_, Option<String>>(1)?,
                        ))
                    })?
                    .collect::<Result<Vec<_>, _>>()?;

                if !citations.is_empty() {
                    let sources: Vec<String> = citations
                        .iter()
                        .map(|(doc_title, section)| {
                            match section {
                                Some(s) if !s.is_empty() => format!("{} ({})", doc_title, s),
                                _ => doc_title.clone(),
                            }
                        })
                        .collect();
                    md.push_str(&format!("\n> Sources: {}\n", sources.join(", ")));
                }
            }
            _ => {
                md.push_str(&format!("**{}**: {}\n", role, content));
            }
        }

        md.push_str("\n---\n");
    }

    Ok(md)
}
