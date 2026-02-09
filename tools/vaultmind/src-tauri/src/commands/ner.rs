use tauri::State;

use crate::error::AppError;
use crate::models::{Entity, EntityGraph, EntityGraphEdge, EntityGraphNode, EntityMention, EntityRelationship};
use crate::ner;
use crate::state::{get_conn, AppState};

/// Extract named entities from all chunks in a document using LLM-based NER.
/// Returns the total number of entity mentions found.
#[tauri::command]
pub async fn extract_document_entities(
    state: State<'_, AppState>,
    document_id: String,
    collection_id: String,
) -> Result<usize, AppError> {
    // Phase 1: Read from DB (sync scope -- connection dropped before await)
    let (chunks, host, port, model) = {
        let conn = get_conn(state.inner())?;

        let host: String = conn
            .query_row(
                "SELECT value FROM settings WHERE key = 'ollama_host'",
                [],
                |row| row.get(0),
            )
            .unwrap_or_else(|_| "localhost".to_string());

        let port: String = conn
            .query_row(
                "SELECT value FROM settings WHERE key = 'ollama_port'",
                [],
                |row| row.get(0),
            )
            .unwrap_or_else(|_| "11434".to_string());

        let model: String = conn
            .query_row(
                "SELECT value FROM settings WHERE key = 'chat_model'",
                [],
                |row| row.get(0),
            )
            .unwrap_or_else(|_| "llama3.2".to_string());

        let mut stmt = conn.prepare(
            "SELECT id, content FROM chunks WHERE document_id = ?1 ORDER BY chunk_index",
        )?;

        let chunks: Vec<(String, String)> = stmt
            .query_map(rusqlite::params![&document_id], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        (chunks, host, port, model)
    }; // conn dropped here

    // Phase 2: Call LLM for entity extraction (async -- no DB held)
    let chunk_entities =
        ner::extract_entities_for_chunks(&host, &port, &model, &chunks).await?;

    // Phase 3: Write results back to DB (sync scope)
    let conn = get_conn(state.inner())?;
    ner::save_entities_to_db(&conn, &chunk_entities, &document_id, &collection_id)
}

/// List all entities in a collection.
#[tauri::command]
pub fn list_entities(
    state: State<'_, AppState>,
    collection_id: String,
) -> Result<Vec<Entity>, AppError> {
    let conn = get_conn(state.inner())?;

    let mut stmt = conn.prepare(
        "SELECT id, name, entity_type, collection_id, first_seen_at, mention_count, metadata
         FROM entities WHERE collection_id = ?1 ORDER BY mention_count DESC, name ASC",
    )?;

    let entities = stmt
        .query_map(rusqlite::params![collection_id], |row| {
            Ok(Entity {
                id: row.get(0)?,
                name: row.get(1)?,
                entity_type: row.get(2)?,
                collection_id: row.get(3)?,
                first_seen_at: row.get(4)?,
                mention_count: row.get(5)?,
                metadata: row.get(6)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(entities)
}

/// Get all mentions of a specific entity.
#[tauri::command]
pub fn get_entity_mentions(
    state: State<'_, AppState>,
    entity_id: String,
) -> Result<Vec<EntityMention>, AppError> {
    let conn = get_conn(state.inner())?;

    let mut stmt = conn.prepare(
        "SELECT id, entity_id, chunk_id, document_id, start_offset, end_offset, context, created_at
         FROM entity_mentions WHERE entity_id = ?1 ORDER BY created_at DESC",
    )?;

    let mentions = stmt
        .query_map(rusqlite::params![entity_id], |row| {
            Ok(EntityMention {
                id: row.get(0)?,
                entity_id: row.get(1)?,
                chunk_id: row.get(2)?,
                document_id: row.get(3)?,
                start_offset: row.get(4)?,
                end_offset: row.get(5)?,
                context: row.get(6)?,
                created_at: row.get(7)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(mentions)
}

/// Extract relationships between entities in a collection using LLM.
/// Returns the total number of new relationships stored.
#[tauri::command]
pub async fn extract_collection_relationships(
    state: State<'_, AppState>,
    collection_id: String,
) -> Result<usize, AppError> {
    // Phase 1: Read settings (sync scope -- connection dropped before await)
    let (host, port, model) = {
        let conn = get_conn(state.inner())?;

        let host: String = conn
            .query_row(
                "SELECT value FROM settings WHERE key = 'ollama_host'",
                [],
                |row| row.get(0),
            )
            .unwrap_or_else(|_| "localhost".to_string());

        let port: String = conn
            .query_row(
                "SELECT value FROM settings WHERE key = 'ollama_port'",
                [],
                |row| row.get(0),
            )
            .unwrap_or_else(|_| "11434".to_string());

        let model: String = conn
            .query_row(
                "SELECT value FROM settings WHERE key = 'chat_model'",
                [],
                |row| row.get(0),
            )
            .unwrap_or_else(|_| "llama3.2".to_string());

        (host, port, model)
    }; // conn dropped here

    // Phase 2: Load collection data from DB (sync scope -- connection dropped before await)
    let (entity_rows, chunks, entity_names) = {
        let conn = get_conn(state.inner())?;
        let (entity_rows, chunks) = ner::load_collection_data(&conn, &collection_id)?;
        let entity_names: Vec<String> = entity_rows.iter().map(|(_, name)| name.clone()).collect();
        (entity_rows, chunks, entity_names)
    }; // conn dropped here

    if entity_rows.is_empty() {
        return Ok(0);
    }

    // Phase 3: Call LLM for relationship extraction (async -- no DB held)
    let chunk_relationships =
        ner::extract_relationships_for_chunks(&host, &port, &model, &chunks, &entity_names).await?;

    // Phase 4: Write results back to DB (sync scope)
    let conn = get_conn(state.inner())?;
    ner::save_relationships_to_db(&conn, &chunk_relationships, &entity_rows, &collection_id)
}

/// Get all relationships in a collection.
#[tauri::command]
pub fn get_entity_relationships(
    state: State<'_, AppState>,
    collection_id: String,
) -> Result<Vec<EntityRelationship>, AppError> {
    let conn = get_conn(state.inner())?;

    let mut stmt = conn.prepare(
        "SELECT id, source_entity_id, target_entity_id, relationship_type, confidence, evidence_chunk_id, collection_id, created_at
         FROM entity_relationships WHERE collection_id = ?1 ORDER BY confidence DESC",
    )?;

    let relationships = stmt
        .query_map(rusqlite::params![collection_id], |row| {
            Ok(EntityRelationship {
                id: row.get(0)?,
                source_entity_id: row.get(1)?,
                target_entity_id: row.get(2)?,
                relationship_type: row.get(3)?,
                confidence: row.get(4)?,
                evidence_chunk_id: row.get(5)?,
                collection_id: row.get(6)?,
                created_at: row.get(7)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(relationships)
}

/// Get the entity graph (nodes + edges) for a collection, suitable for visualization.
#[tauri::command]
pub fn get_entity_graph(
    state: State<'_, AppState>,
    collection_id: String,
) -> Result<EntityGraph, AppError> {
    let conn = get_conn(state.inner())?;

    // Load nodes (entities)
    let mut entity_stmt = conn.prepare(
        "SELECT id, name, entity_type, mention_count FROM entities WHERE collection_id = ?1",
    )?;
    let nodes: Vec<EntityGraphNode> = entity_stmt
        .query_map(rusqlite::params![collection_id], |row| {
            Ok(EntityGraphNode {
                id: row.get(0)?,
                name: row.get(1)?,
                entity_type: row.get(2)?,
                mention_count: row.get(3)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    // Load edges (relationships)
    let mut rel_stmt = conn.prepare(
        "SELECT source_entity_id, target_entity_id, relationship_type, confidence
         FROM entity_relationships WHERE collection_id = ?1",
    )?;
    let edges: Vec<EntityGraphEdge> = rel_stmt
        .query_map(rusqlite::params![collection_id], |row| {
            Ok(EntityGraphEdge {
                source_entity_id: row.get(0)?,
                target_entity_id: row.get(1)?,
                relationship_type: row.get(2)?,
                confidence: row.get(3)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(EntityGraph { nodes, edges })
}

#[cfg(test)]
mod tests {
    use crate::db;

    fn setup_db() -> rusqlite::Connection {
        let dir = tempfile::tempdir().unwrap();
        let pool = db::create_pool(dir.path()).unwrap();
        let conn_pooled = pool.get().unwrap();
        let db_path = conn_pooled.path().unwrap().to_owned();
        std::mem::forget(dir);
        drop(conn_pooled);
        let c = rusqlite::Connection::open(db_path).unwrap();
        db::configure_connection(&c).unwrap();
        c
    }

    #[test]
    fn test_list_entities_returns_correct_results() {
        let conn = setup_db();

        let collection_id: String = conn
            .query_row("SELECT id FROM collections LIMIT 1", [], |row| row.get(0))
            .unwrap();

        let now = chrono::Utc::now().to_rfc3339();

        // Insert some entities
        conn.execute(
            "INSERT INTO entities (id, name, entity_type, collection_id, first_seen_at, mention_count, metadata)
             VALUES ('e1', 'Alice', 'person', ?1, ?2, 5, '{}')",
            rusqlite::params![collection_id, now],
        ).unwrap();

        conn.execute(
            "INSERT INTO entities (id, name, entity_type, collection_id, first_seen_at, mention_count, metadata)
             VALUES ('e2', 'Rust', 'technology', ?1, ?2, 3, '{}')",
            rusqlite::params![collection_id, now],
        ).unwrap();

        // Create a second collection for the "Other" entity to test filtering
        let other_coll_id = uuid::Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO collections (id, name, description, created_at, updated_at) VALUES (?1, 'Other Collection', '', ?2, ?2)",
            rusqlite::params![other_coll_id, now],
        ).unwrap();

        conn.execute(
            "INSERT INTO entities (id, name, entity_type, collection_id, first_seen_at, mention_count, metadata)
             VALUES ('e3', 'Other', 'person', ?1, ?2, 1, '{}')",
            rusqlite::params![other_coll_id, now],
        ).unwrap();

        // Query entities for the collection
        let mut stmt = conn
            .prepare(
                "SELECT id, name, entity_type, collection_id, first_seen_at, mention_count, metadata
                 FROM entities WHERE collection_id = ?1 ORDER BY mention_count DESC, name ASC",
            )
            .unwrap();

        let entities: Vec<crate::models::Entity> = stmt
            .query_map(rusqlite::params![collection_id], |row| {
                Ok(crate::models::Entity {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    entity_type: row.get(2)?,
                    collection_id: row.get(3)?,
                    first_seen_at: row.get(4)?,
                    mention_count: row.get(5)?,
                    metadata: row.get(6)?,
                })
            })
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(entities.len(), 2);
        // Sorted by mention_count DESC, so Alice (5) first, Rust (3) second
        assert_eq!(entities[0].name, "Alice");
        assert_eq!(entities[0].mention_count, 5);
        assert_eq!(entities[1].name, "Rust");
        assert_eq!(entities[1].entity_type, "technology");
    }
}
