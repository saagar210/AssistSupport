use tauri::{AppHandle, State};
use tauri::Emitter;

use crate::error::AppError;
use crate::models::{Citation, Conversation, Message};
use crate::ollama::{self, ChatMessage};
use crate::state::AppState;

use super::search::SearchResult;

/// Helper to lock the DB mutex.
fn lock_db<'a>(
    state: &'a State<'a, AppState>,
) -> Result<std::sync::MutexGuard<'a, rusqlite::Connection>, AppError> {
    crate::state::lock_db(state.inner())
}

/// Send a chat message: run RAG pipeline (search + context + stream LLM response).
#[tauri::command]
pub async fn send_chat_message(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    conversation_id: String,
    collection_id: String,
    user_message: String,
) -> Result<(), AppError> {
    let now = chrono::Utc::now().to_rfc3339();
    let user_msg_id = uuid::Uuid::new_v4().to_string();

    // 1. Save user message to DB + read all settings we need, then drop lock
    let (host, port, embedding_model, chat_model, context_chunks, rrf_k, vector_top_k, keyword_top_k) = {
        let db = lock_db(&state)?;

        db.execute(
            "INSERT INTO messages (id, conversation_id, role, content, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![user_msg_id, conversation_id, "user", user_message, now],
        )?;

        // Update conversation timestamp
        db.execute(
            "UPDATE conversations SET updated_at = ?1 WHERE id = ?2",
            rusqlite::params![now, conversation_id],
        )?;

        let host: String = db.query_row(
            "SELECT value FROM settings WHERE key = 'ollama_host'", [], |row: &rusqlite::Row| row.get(0),
        )?;
        let port: String = db.query_row(
            "SELECT value FROM settings WHERE key = 'ollama_port'", [], |row: &rusqlite::Row| row.get(0),
        )?;
        let embedding_model: String = db.query_row(
            "SELECT value FROM settings WHERE key = 'embedding_model'", [], |row: &rusqlite::Row| row.get(0),
        )?;
        let chat_model: String = db.query_row(
            "SELECT value FROM settings WHERE key = 'chat_model'", [], |row: &rusqlite::Row| row.get(0),
        )?;
        let context_chunks: String = db.query_row(
            "SELECT value FROM settings WHERE key = 'context_chunks'", [], |row: &rusqlite::Row| row.get(0),
        ).unwrap_or_else(|_| "5".to_string());
        let rrf_k: String = db.query_row(
            "SELECT value FROM settings WHERE key = 'rrf_k'", [], |row: &rusqlite::Row| row.get(0),
        ).unwrap_or_else(|_| "60".to_string());
        let vector_top_k: String = db.query_row(
            "SELECT value FROM settings WHERE key = 'vector_top_k'", [], |row: &rusqlite::Row| row.get(0),
        ).unwrap_or_else(|_| "20".to_string());
        let keyword_top_k: String = db.query_row(
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
        let db = lock_db(&state)?;
        // Use the internal search that works on a Connection directly
        // We inline the search here since hybrid_search_internal is async but we only
        // need the DB parts (we already have the embedding)
        let vr = crate::commands::search::vector_search_in_db_with_embedding(
            &db, &collection_id, &query_embedding, vec_top_k,
        )?;
        let kr = crate::commands::search::keyword_search_in_db(
            &db, &collection_id, &user_message, kw_top_k,
        )?;
        crate::commands::search::reciprocal_rank_fusion_pub(vr, kr, rrf_k_val, context_k)
    };

    // 3. Build system prompt with context
    let system_prompt = build_system_prompt(&context_results);

    // 4. Load conversation history (last 10 messages)
    let history: Vec<ChatMessage> = {
        let db = lock_db(&state)?;
        load_conversation_history(&db, &conversation_id, 10)?
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
        content: user_message,
    });

    // 6. Stream response from Ollama
    let full_response = ollama::chat_stream(&host, &port, &chat_model, &messages, &app_handle).await?;

    // 7. Save assistant message + citations to DB
    let assistant_msg_id = uuid::Uuid::new_v4().to_string();
    let msg_now = chrono::Utc::now().to_rfc3339();

    let citations: Vec<Citation> = {
        let db = lock_db(&state)?;

        db.execute(
            "INSERT INTO messages (id, conversation_id, role, content, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![assistant_msg_id, conversation_id, "assistant", full_response, msg_now],
        )?;

        // Save citations (one per context chunk)
        let mut saved_citations = Vec::new();
        for result in &context_results {
            let citation_id = uuid::Uuid::new_v4().to_string();
            // Truncate snippet to first 200 chars
            let snippet: String = result.content.chars().take(200).collect();

            db.execute(
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
        content: full_response,
        created_at: msg_now,
    };

    let _ = app_handle.emit(
        "chat-complete",
        serde_json::json!({
            "message": assistant_message,
            "citations": citations,
        }),
    );

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

#[tauri::command]
pub async fn create_conversation(
    state: State<'_, AppState>,
    collection_id: String,
    title: String,
) -> Result<Conversation, AppError> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let db = lock_db(&state)?;
    db.execute(
        "INSERT INTO conversations (id, collection_id, title, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![id, collection_id, title, now, now],
    )?;

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
) -> Result<Vec<Conversation>, AppError> {
    let db = lock_db(&state)?;

    let mut stmt = db.prepare(
        "SELECT id, collection_id, title, created_at, updated_at
         FROM conversations
         WHERE collection_id = ?1
         ORDER BY updated_at DESC",
    )?;

    let conversations = stmt
        .query_map(rusqlite::params![collection_id], |row| {
            Ok(Conversation {
                id: row.get(0)?,
                collection_id: row.get(1)?,
                title: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(conversations)
}

#[tauri::command]
pub fn get_conversation_messages(
    state: State<'_, AppState>,
    conversation_id: String,
) -> Result<Vec<Message>, AppError> {
    let db = lock_db(&state)?;

    let mut stmt = db.prepare(
        "SELECT id, conversation_id, role, content, created_at
         FROM messages
         WHERE conversation_id = ?1
         ORDER BY created_at ASC",
    )?;

    let messages = stmt
        .query_map(rusqlite::params![conversation_id], |row| {
            Ok(Message {
                id: row.get(0)?,
                conversation_id: row.get(1)?,
                role: row.get(2)?,
                content: row.get(3)?,
                created_at: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(messages)
}

#[tauri::command]
pub fn get_message_citations(
    state: State<'_, AppState>,
    message_id: String,
) -> Result<Vec<Citation>, AppError> {
    let db = lock_db(&state)?;

    let mut stmt = db.prepare(
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
    let db = lock_db(&state)?;

    let rows = db.execute(
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
    let db = lock_db(&state)?;

    let rows = db.execute(
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
