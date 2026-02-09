use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::ollama::{self, ChatMessage};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedEntity {
    pub name: String,
    pub entity_type: String,
    #[serde(default)]
    pub start_offset: usize,
    #[serde(default)]
    pub end_offset: usize,
    #[serde(default)]
    pub context: String,
}

/// Result of extracting entities from a single chunk.
pub struct ChunkEntities {
    pub chunk_id: String,
    pub entities: Vec<ExtractedEntity>,
}

const NER_PROMPT: &str = "Extract named entities from the following text. Return a JSON array of objects with fields: name, entity_type (one of: person, organization, location, concept, technology, date, event), context (short quote from text). Return ONLY the JSON array, no other text.";

/// LLM-based NER -- sends chunk text to Ollama with structured extraction prompt.
pub async fn extract_entities(
    host: &str,
    port: &str,
    model: &str,
    text: &str,
) -> Result<Vec<ExtractedEntity>, AppError> {
    let messages = vec![
        ChatMessage {
            role: "system".to_string(),
            content: NER_PROMPT.to_string(),
        },
        ChatMessage {
            role: "user".to_string(),
            content: text.to_string(),
        },
    ];

    let response = ollama::chat_once(host, port, model, &messages).await?;
    Ok(parse_entity_response(&response))
}

/// Parse the LLM response into a list of extracted entities.
/// If parsing fails, returns an empty vec rather than erroring.
pub fn parse_entity_response(response: &str) -> Vec<ExtractedEntity> {
    let trimmed = response.trim();

    // Try parsing the full response first
    if let Ok(entities) = serde_json::from_str::<Vec<ExtractedEntity>>(trimmed) {
        return entities;
    }

    // LLMs sometimes wrap JSON in markdown code fences -- strip them
    let stripped = trimmed
        .strip_prefix("```json")
        .or_else(|| trimmed.strip_prefix("```"))
        .unwrap_or(trimmed);
    let stripped = stripped
        .strip_suffix("```")
        .unwrap_or(stripped)
        .trim();

    if let Ok(entities) = serde_json::from_str::<Vec<ExtractedEntity>>(stripped) {
        return entities;
    }

    // Try to find a JSON array within the response
    if let Some(start) = stripped.find('[') {
        if let Some(end) = stripped.rfind(']') {
            let json_slice = &stripped[start..=end];
            if let Ok(entities) = serde_json::from_str::<Vec<ExtractedEntity>>(json_slice) {
                return entities;
            }
        }
    }

    tracing::warn!("Failed to parse NER response as JSON, returning empty: {}", trimmed);
    Vec::new()
}

/// Extract entities from a list of chunks via LLM. Pure async, no DB access.
pub async fn extract_entities_for_chunks(
    host: &str,
    port: &str,
    model: &str,
    chunks: &[(String, String)], // (chunk_id, content)
) -> Result<Vec<ChunkEntities>, AppError> {
    let mut results = Vec::new();

    for (chunk_id, content) in chunks {
        let entities = extract_entities(host, port, model, content).await?;
        results.push(ChunkEntities {
            chunk_id: chunk_id.clone(),
            entities,
        });
    }

    Ok(results)
}

/// Save extracted entities to the database, deduplicating by name+type within collection.
/// Returns total mention count.
pub fn save_entities_to_db(
    conn: &rusqlite::Connection,
    chunk_entities: &[ChunkEntities],
    document_id: &str,
    collection_id: &str,
) -> Result<usize, AppError> {
    let now = chrono::Utc::now().to_rfc3339();
    let mut total_mentions: usize = 0;

    for chunk_result in chunk_entities {
        for entity in &chunk_result.entities {
            // Look for existing entity with same name+type in collection (case-insensitive)
            let existing: Option<(String, i32)> = conn
                .query_row(
                    "SELECT id, mention_count FROM entities
                     WHERE LOWER(name) = LOWER(?1) AND entity_type = ?2 AND collection_id = ?3",
                    rusqlite::params![entity.name, entity.entity_type, collection_id],
                    |row| Ok((row.get::<_, String>(0)?, row.get::<_, i32>(1)?)),
                )
                .ok();

            let entity_id = if let Some((existing_id, current_count)) = existing {
                // Increment mention count
                conn.execute(
                    "UPDATE entities SET mention_count = ?1 WHERE id = ?2",
                    rusqlite::params![current_count + 1, existing_id],
                )?;
                existing_id
            } else {
                // Insert new entity
                let new_id = uuid::Uuid::new_v4().to_string();
                conn.execute(
                    "INSERT INTO entities (id, name, entity_type, collection_id, first_seen_at, mention_count, metadata)
                     VALUES (?1, ?2, ?3, ?4, ?5, 1, '{}')",
                    rusqlite::params![new_id, entity.name, entity.entity_type, collection_id, now],
                )?;
                new_id
            };

            // Create entity mention record
            let mention_id = uuid::Uuid::new_v4().to_string();
            conn.execute(
                "INSERT INTO entity_mentions (id, entity_id, chunk_id, document_id, start_offset, end_offset, context, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                rusqlite::params![
                    mention_id,
                    entity_id,
                    chunk_result.chunk_id,
                    document_id,
                    entity.start_offset as i32,
                    entity.end_offset as i32,
                    entity.context,
                    now,
                ],
            )?;

            total_mentions += 1;
        }
    }

    Ok(total_mentions)
}

// --- Relationship Extraction ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedRelationship {
    pub source_entity: String,
    pub target_entity: String,
    pub relationship_type: String,
    #[serde(default)]
    pub confidence: f64,
}

const RELATIONSHIP_PROMPT: &str = "Given these known entities: [{entities}]. Extract relationships between them from the following text. Return a JSON array of objects with: source_entity, target_entity, relationship_type (one of: works_at, located_in, authored, related_to, part_of, manages, uses, created_by), confidence (0.0-1.0). Return ONLY the JSON array.";

/// LLM-based relationship extraction between known entities.
pub async fn extract_relationships(
    host: &str,
    port: &str,
    model: &str,
    text: &str,
    known_entities: &[String],
) -> Result<Vec<ExtractedRelationship>, AppError> {
    if known_entities.is_empty() || text.is_empty() {
        return Ok(Vec::new());
    }

    let entity_list = known_entities.join(", ");
    let system_prompt = RELATIONSHIP_PROMPT.replace("{entities}", &entity_list);

    let messages = vec![
        ChatMessage {
            role: "system".to_string(),
            content: system_prompt,
        },
        ChatMessage {
            role: "user".to_string(),
            content: text.to_string(),
        },
    ];

    let response = ollama::chat_once(host, port, model, &messages).await?;
    Ok(parse_relationship_response(&response))
}

/// Parse the LLM response into a list of extracted relationships.
/// If parsing fails, returns an empty vec rather than erroring.
pub fn parse_relationship_response(response: &str) -> Vec<ExtractedRelationship> {
    let trimmed = response.trim();

    // Try parsing the full response first
    if let Ok(rels) = serde_json::from_str::<Vec<ExtractedRelationship>>(trimmed) {
        return rels;
    }

    // Strip markdown code fences
    let stripped = trimmed
        .strip_prefix("```json")
        .or_else(|| trimmed.strip_prefix("```"))
        .unwrap_or(trimmed);
    let stripped = stripped
        .strip_suffix("```")
        .unwrap_or(stripped)
        .trim();

    if let Ok(rels) = serde_json::from_str::<Vec<ExtractedRelationship>>(stripped) {
        return rels;
    }

    // Try to find a JSON array within the response
    if let Some(start) = stripped.find('[') {
        if let Some(end) = stripped.rfind(']') {
            let json_slice = &stripped[start..=end];
            if let Ok(rels) = serde_json::from_str::<Vec<ExtractedRelationship>>(json_slice) {
                return rels;
            }
        }
    }

    tracing::warn!(
        "Failed to parse relationship extraction response as JSON, returning empty: {}",
        trimmed
    );
    Vec::new()
}

/// Chunk-level relationship extraction result.
pub struct ChunkRelationships {
    pub chunk_id: String,
    pub relationships: Vec<ExtractedRelationship>,
}

/// Load entities and chunks for a collection from the database.
/// Returns (entity_rows as (id, name), chunks as (id, content)).
#[allow(clippy::type_complexity)]
pub fn load_collection_data(
    conn: &rusqlite::Connection,
    collection_id: &str,
) -> Result<(Vec<(String, String)>, Vec<(String, String)>), AppError> {
    let mut entity_stmt = conn.prepare(
        "SELECT id, name FROM entities WHERE collection_id = ?1",
    )?;
    let entity_rows: Vec<(String, String)> = entity_stmt
        .query_map(rusqlite::params![collection_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    if entity_rows.is_empty() {
        return Ok((Vec::new(), Vec::new()));
    }

    let mut chunk_stmt = conn.prepare(
        "SELECT id, content FROM chunks WHERE collection_id = ?1 ORDER BY chunk_index",
    )?;
    let chunks: Vec<(String, String)> = chunk_stmt
        .query_map(rusqlite::params![collection_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok((entity_rows, chunks))
}

/// Extract relationships for all chunks via LLM. Pure async, no DB access.
pub async fn extract_relationships_for_chunks(
    host: &str,
    port: &str,
    model: &str,
    chunks: &[(String, String)],
    entity_names: &[String],
) -> Result<Vec<ChunkRelationships>, AppError> {
    let mut results = Vec::new();

    for (chunk_id, content) in chunks {
        let relationships =
            extract_relationships(host, port, model, content, entity_names).await?;
        results.push(ChunkRelationships {
            chunk_id: chunk_id.clone(),
            relationships,
        });
    }

    Ok(results)
}

/// Save extracted relationships to the database, deduplicating by source+target+type.
/// Returns total number of new relationships stored.
pub fn save_relationships_to_db(
    conn: &rusqlite::Connection,
    chunk_relationships: &[ChunkRelationships],
    entity_rows: &[(String, String)],
    collection_id: &str,
) -> Result<usize, AppError> {
    // Build name -> id lookup (case-insensitive)
    let mut name_to_id: HashMap<String, String> = HashMap::new();
    for (id, name) in entity_rows {
        name_to_id.insert(name.to_lowercase(), id.clone());
    }

    // Track existing relationships for deduplication
    let mut existing: HashMap<(String, String, String), (String, f64)> = HashMap::new();
    {
        let mut rel_stmt = conn.prepare(
            "SELECT id, source_entity_id, target_entity_id, relationship_type, confidence
             FROM entity_relationships WHERE collection_id = ?1",
        )?;
        let rows = rel_stmt
            .query_map(rusqlite::params![collection_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, f64>(4)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        for (id, src, tgt, rtype, conf) in rows {
            existing.insert((src, tgt, rtype), (id, conf));
        }
    }

    let now = chrono::Utc::now().to_rfc3339();
    let mut total_stored: usize = 0;

    for chunk_result in chunk_relationships {
        for rel in &chunk_result.relationships {
            let source_id = match name_to_id.get(&rel.source_entity.to_lowercase()) {
                Some(id) => id.clone(),
                None => continue,
            };
            let target_id = match name_to_id.get(&rel.target_entity.to_lowercase()) {
                Some(id) => id.clone(),
                None => continue,
            };

            let key = (
                source_id.clone(),
                target_id.clone(),
                rel.relationship_type.clone(),
            );

            if let Some((existing_id, existing_conf)) = existing.get(&key) {
                if rel.confidence > *existing_conf {
                    conn.execute(
                        "UPDATE entity_relationships SET confidence = ?1, evidence_chunk_id = ?2 WHERE id = ?3",
                        rusqlite::params![rel.confidence, chunk_result.chunk_id, existing_id],
                    )?;
                    existing.insert(key, (existing_id.clone(), rel.confidence));
                }
            } else {
                let rel_id = uuid::Uuid::new_v4().to_string();
                conn.execute(
                    "INSERT INTO entity_relationships (id, source_entity_id, target_entity_id, relationship_type, confidence, evidence_chunk_id, collection_id, created_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                    rusqlite::params![
                        rel_id,
                        source_id,
                        target_id,
                        rel.relationship_type,
                        rel.confidence,
                        chunk_result.chunk_id,
                        collection_id,
                        now,
                    ],
                )?;
                existing.insert(key, (rel_id, rel.confidence));
                total_stored += 1;
            }
        }
    }

    Ok(total_stored)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_entity_extraction_response() {
        let json = r#"[
            {"name": "John Smith", "entity_type": "person", "context": "John Smith said"},
            {"name": "Acme Corp", "entity_type": "organization", "context": "works at Acme Corp"}
        ]"#;

        let entities = parse_entity_response(json);
        assert_eq!(entities.len(), 2);
        assert_eq!(entities[0].name, "John Smith");
        assert_eq!(entities[0].entity_type, "person");
        assert_eq!(entities[1].name, "Acme Corp");
        assert_eq!(entities[1].entity_type, "organization");
    }

    #[test]
    fn test_parse_entity_response_with_code_fences() {
        let json = "```json\n[{\"name\": \"Paris\", \"entity_type\": \"location\", \"context\": \"in Paris\"}]\n```";
        let entities = parse_entity_response(json);
        assert_eq!(entities.len(), 1);
        assert_eq!(entities[0].name, "Paris");
        assert_eq!(entities[0].entity_type, "location");
    }

    #[test]
    fn test_parse_entity_response_returns_empty_on_invalid() {
        let invalid = "This is not JSON at all";
        let entities = parse_entity_response(invalid);
        assert!(entities.is_empty());
    }

    #[test]
    fn test_parse_entity_response_extracts_embedded_json() {
        let response = "Here are the entities:\n[{\"name\": \"Rust\", \"entity_type\": \"technology\", \"context\": \"written in Rust\"}]\nDone.";
        let entities = parse_entity_response(response);
        assert_eq!(entities.len(), 1);
        assert_eq!(entities[0].name, "Rust");
    }

    #[test]
    fn test_entity_deduplication_logic() {
        let dir = tempfile::tempdir().unwrap();
        let pool = crate::db::create_pool(dir.path()).unwrap();
        let conn_pooled = pool.get().unwrap();
        let db_path = conn_pooled.path().unwrap().to_owned();
        drop(conn_pooled);

        let conn = rusqlite::Connection::open(db_path).unwrap();
        crate::db::configure_connection(&conn).unwrap();

        // Get a collection id
        let collection_id: String = conn
            .query_row("SELECT id FROM collections LIMIT 1", [], |row| row.get(0))
            .unwrap();

        let now = chrono::Utc::now().to_rfc3339();

        // Insert a document for foreign key constraints
        let doc_id = uuid::Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO documents (id, collection_id, filename, file_path, file_type, file_size, file_hash, title, word_count, chunk_count, status, created_at, updated_at)
             VALUES (?1, ?2, 'test.txt', '/tmp/test.txt', 'txt', 100, 'abc123', 'Test', 50, 1, 'ready', ?3, ?3)",
            rusqlite::params![doc_id, collection_id, now],
        ).unwrap();

        // Insert a chunk for foreign key constraints
        let chunk_id = uuid::Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO chunks (id, document_id, collection_id, content, chunk_index, token_count, created_at)
             VALUES (?1, ?2, ?3, 'test content', 0, 10, ?4)",
            rusqlite::params![chunk_id, doc_id, collection_id, now],
        ).unwrap();

        // Insert first entity
        let entity_id = uuid::Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO entities (id, name, entity_type, collection_id, first_seen_at, mention_count, metadata)
             VALUES (?1, 'John Smith', 'person', ?2, ?3, 1, '{}')",
            rusqlite::params![entity_id, collection_id, now],
        ).unwrap();

        // Simulate deduplication: find existing and increment
        let (existing_id, count): (String, i32) = conn
            .query_row(
                "SELECT id, mention_count FROM entities WHERE LOWER(name) = LOWER('john smith') AND entity_type = 'person' AND collection_id = ?1",
                rusqlite::params![collection_id],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, i32>(1)?)),
            )
            .unwrap();

        assert_eq!(existing_id, entity_id);
        assert_eq!(count, 1);

        conn.execute(
            "UPDATE entities SET mention_count = ?1 WHERE id = ?2",
            rusqlite::params![count + 1, existing_id],
        ).unwrap();

        // Verify updated count
        let new_count: i32 = conn
            .query_row(
                "SELECT mention_count FROM entities WHERE id = ?1",
                rusqlite::params![entity_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(new_count, 2);

        // Verify case-insensitive match
        let found: Option<String> = conn
            .query_row(
                "SELECT id FROM entities WHERE LOWER(name) = LOWER('JOHN SMITH') AND entity_type = 'person' AND collection_id = ?1",
                rusqlite::params![collection_id],
                |row| row.get(0),
            )
            .ok();
        assert_eq!(found, Some(entity_id));
    }

    #[test]
    fn test_parse_relationship_extraction_response() {
        let json = r#"[
            {"source_entity": "John Smith", "target_entity": "Acme Corp", "relationship_type": "works_at", "confidence": 0.95},
            {"source_entity": "Acme Corp", "target_entity": "New York", "relationship_type": "located_in", "confidence": 0.8}
        ]"#;

        let rels = parse_relationship_response(json);
        assert_eq!(rels.len(), 2);
        assert_eq!(rels[0].source_entity, "John Smith");
        assert_eq!(rels[0].target_entity, "Acme Corp");
        assert_eq!(rels[0].relationship_type, "works_at");
        assert!((rels[0].confidence - 0.95).abs() < f64::EPSILON);
        assert_eq!(rels[1].source_entity, "Acme Corp");
        assert_eq!(rels[1].target_entity, "New York");
        assert_eq!(rels[1].relationship_type, "located_in");
    }

    #[test]
    fn test_parse_relationship_response_with_code_fences() {
        let json = "```json\n[{\"source_entity\": \"Alice\", \"target_entity\": \"Bob\", \"relationship_type\": \"manages\", \"confidence\": 0.7}]\n```";
        let rels = parse_relationship_response(json);
        assert_eq!(rels.len(), 1);
        assert_eq!(rels[0].relationship_type, "manages");
    }

    #[test]
    fn test_parse_relationship_response_returns_empty_on_invalid() {
        let invalid = "No relationships found in this text.";
        let rels = parse_relationship_response(invalid);
        assert!(rels.is_empty());
    }

    #[test]
    fn test_parse_relationship_response_default_confidence() {
        // confidence is missing -- serde default should give 0.0
        let json = r#"[{"source_entity": "A", "target_entity": "B", "relationship_type": "related_to"}]"#;
        let rels = parse_relationship_response(json);
        assert_eq!(rels.len(), 1);
        assert!((rels[0].confidence - 0.0).abs() < f64::EPSILON);
    }
}
