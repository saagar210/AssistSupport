use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::models::GraphEdge;
use crate::utils::{bytes_to_f64_vec, cosine_similarity};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub file_type: String,
    pub chunk_count: i32,
    pub word_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphLink {
    pub source: String,
    pub target: String,
    pub weight: f64,
    pub relationship_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphData {
    pub nodes: Vec<GraphNode>,
    pub links: Vec<GraphLink>,
}

/// Build knowledge graph edges for a collection.
///
/// Creates two types of edges:
/// - "semantic": between chunks from DIFFERENT documents with cosine similarity > threshold
/// - "same_document": between sequential chunks in the same document (weight 1.0)
pub fn build_graph_edges(
    conn: &rusqlite::Connection,
    collection_id: &str,
    similarity_threshold: f64,
) -> Result<Vec<GraphEdge>, AppError> {
    // Check if chunk_embeddings table exists
    let table_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='chunk_embeddings'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .map(|c| c > 0)?;

    if !table_exists {
        return Ok(Vec::new());
    }

    // Load all chunk embeddings for this collection
    let mut stmt = conn.prepare(
        "SELECT ce.chunk_id, c.document_id, c.chunk_index, ce.embedding
         FROM chunk_embeddings ce
         JOIN chunks c ON c.id = ce.chunk_id
         WHERE c.collection_id = ?1
         ORDER BY c.document_id, c.chunk_index",
    )?;

    struct ChunkInfo {
        chunk_id: String,
        document_id: String,
        chunk_index: i32,
        embedding: Vec<f64>,
    }

    let chunks: Vec<ChunkInfo> = stmt
        .query_map(rusqlite::params![collection_id], |row| {
            let chunk_id: String = row.get(0)?;
            let document_id: String = row.get(1)?;
            let chunk_index: i32 = row.get(2)?;
            let embedding_blob: Vec<u8> = row.get(3)?;
            Ok((chunk_id, document_id, chunk_index, embedding_blob))
        })?
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .map(|(chunk_id, document_id, chunk_index, blob)| ChunkInfo {
            chunk_id,
            document_id,
            chunk_index,
            embedding: bytes_to_f64_vec(&blob),
        })
        .collect();

    let now = chrono::Utc::now().to_rfc3339();
    let mut edges: Vec<GraphEdge> = Vec::new();

    // 1. Semantic edges: between chunks from DIFFERENT documents
    for i in 0..chunks.len() {
        for j in (i + 1)..chunks.len() {
            if chunks[i].document_id == chunks[j].document_id {
                continue;
            }

            let sim = cosine_similarity(&chunks[i].embedding, &chunks[j].embedding);
            if sim > similarity_threshold {
                edges.push(GraphEdge {
                    id: uuid::Uuid::new_v4().to_string(),
                    source_chunk_id: chunks[i].chunk_id.clone(),
                    target_chunk_id: chunks[j].chunk_id.clone(),
                    collection_id: collection_id.to_string(),
                    weight: sim,
                    relationship_type: "semantic".to_string(),
                    created_at: now.clone(),
                });
            }
        }
    }

    // 2. Same-document edges: sequential chunks within each document
    for i in 0..chunks.len() {
        for j in (i + 1)..chunks.len() {
            if chunks[i].document_id != chunks[j].document_id {
                continue;
            }
            if chunks[j].chunk_index == chunks[i].chunk_index + 1 {
                edges.push(GraphEdge {
                    id: uuid::Uuid::new_v4().to_string(),
                    source_chunk_id: chunks[i].chunk_id.clone(),
                    target_chunk_id: chunks[j].chunk_id.clone(),
                    collection_id: collection_id.to_string(),
                    weight: 1.0,
                    relationship_type: "same_document".to_string(),
                    created_at: now.clone(),
                });
            }
        }
    }

    // Clear existing edges for this collection, then batch insert
    conn.execute(
        "DELETE FROM graph_edges WHERE collection_id = ?1",
        rusqlite::params![collection_id],
    )?;

    let mut insert_stmt = conn.prepare(
        "INSERT INTO graph_edges (id, source_chunk_id, target_chunk_id, collection_id, weight, relationship_type, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
    )?;

    for edge in &edges {
        insert_stmt.execute(rusqlite::params![
            edge.id,
            edge.source_chunk_id,
            edge.target_chunk_id,
            edge.collection_id,
            edge.weight,
            edge.relationship_type,
            edge.created_at,
        ])?;
    }

    Ok(edges)
}

/// Get graph visualization data: documents as nodes, aggregated semantic edges as links.
pub fn get_graph_data(
    conn: &rusqlite::Connection,
    collection_id: &str,
) -> Result<GraphData, AppError> {
    // Nodes = documents in collection with aggregated chunk stats
    let mut node_stmt = conn.prepare(
        "SELECT d.id, d.title, d.file_type, d.chunk_count, d.word_count
         FROM documents d
         WHERE d.collection_id = ?1
         ORDER BY d.title",
    )?;

    let nodes: Vec<GraphNode> = node_stmt
        .query_map(rusqlite::params![collection_id], |row| {
            Ok(GraphNode {
                id: row.get(0)?,
                label: row.get(1)?,
                file_type: row.get(2)?,
                chunk_count: row.get(3)?,
                word_count: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    // Links = aggregated graph_edges between documents (semantic only)
    // For each pair of documents, take the max weight of all chunk-level edges
    let mut link_stmt = conn.prepare(
        "SELECT
            c1.document_id AS source_doc,
            c2.document_id AS target_doc,
            MAX(ge.weight) AS max_weight
         FROM graph_edges ge
         JOIN chunks c1 ON c1.id = ge.source_chunk_id
         JOIN chunks c2 ON c2.id = ge.target_chunk_id
         WHERE ge.collection_id = ?1
           AND ge.relationship_type = 'semantic'
           AND c1.document_id != c2.document_id
         GROUP BY c1.document_id, c2.document_id",
    )?;

    let links: Vec<GraphLink> = link_stmt
        .query_map(rusqlite::params![collection_id], |row| {
            Ok(GraphLink {
                source: row.get(0)?,
                target: row.get(1)?,
                weight: row.get(2)?,
                relationship_type: "semantic".to_string(),
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(GraphData { nodes, links })
}


#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: create in-memory DB with all required tables and seed data
    fn setup_graph_db() -> rusqlite::Connection {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = OFF;").unwrap();

        conn.execute_batch(
            "
            CREATE TABLE collections (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                description TEXT DEFAULT '',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE TABLE documents (
                id TEXT PRIMARY KEY,
                collection_id TEXT NOT NULL,
                filename TEXT NOT NULL,
                file_path TEXT NOT NULL,
                file_type TEXT NOT NULL,
                file_size INTEGER NOT NULL,
                file_hash TEXT NOT NULL,
                title TEXT NOT NULL,
                author TEXT,
                page_count INTEGER,
                word_count INTEGER DEFAULT 0,
                chunk_count INTEGER DEFAULT 0,
                status TEXT DEFAULT 'done',
                error_message TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE TABLE chunks (
                id TEXT PRIMARY KEY,
                document_id TEXT NOT NULL,
                collection_id TEXT NOT NULL,
                content TEXT NOT NULL,
                chunk_index INTEGER NOT NULL,
                start_offset INTEGER DEFAULT 0,
                end_offset INTEGER DEFAULT 0,
                page_number INTEGER,
                section_title TEXT,
                token_count INTEGER DEFAULT 0,
                created_at TEXT NOT NULL
            );
            CREATE TABLE chunk_embeddings (
                chunk_id TEXT PRIMARY KEY,
                collection_id TEXT NOT NULL,
                document_id TEXT NOT NULL,
                embedding BLOB NOT NULL,
                content_preview TEXT
            );
            CREATE TABLE graph_edges (
                id TEXT PRIMARY KEY,
                source_chunk_id TEXT NOT NULL,
                target_chunk_id TEXT NOT NULL,
                collection_id TEXT NOT NULL,
                weight REAL DEFAULT 0.0,
                relationship_type TEXT DEFAULT 'semantic',
                created_at TEXT NOT NULL
            );
            ",
        )
        .unwrap();
        conn
    }

    fn f64_vec_to_bytes(v: &[f64]) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(v.len() * 8);
        for val in v {
            bytes.extend_from_slice(&val.to_le_bytes());
        }
        bytes
    }

    fn seed_two_docs_with_chunks(conn: &rusqlite::Connection) {
        let now = "2025-01-01T00:00:00Z";

        conn.execute(
            "INSERT INTO collections (id, name, description, created_at, updated_at) VALUES ('col1', 'Test', '', ?1, ?1)",
            rusqlite::params![now],
        ).unwrap();

        // Two documents
        for (doc_id, title) in &[("doc1", "Doc One"), ("doc2", "Doc Two")] {
            conn.execute(
                "INSERT INTO documents (id, collection_id, filename, file_path, file_type, file_size, file_hash, title, word_count, chunk_count, created_at, updated_at)
                 VALUES (?1, 'col1', 'f.txt', '/f.txt', 'txt', 100, 'hash', ?2, 50, 2, ?3, ?3)",
                rusqlite::params![doc_id, title, now],
            ).unwrap();
        }

        // Chunks for doc1: c1 (index 0), c2 (index 1)
        // Chunks for doc2: c3 (index 0), c4 (index 1)
        let chunks_data = vec![
            ("c1", "doc1", 0, vec![1.0, 0.0, 0.0]),
            ("c2", "doc1", 1, vec![0.9, 0.1, 0.0]),
            ("c3", "doc2", 0, vec![0.95, 0.05, 0.0]), // similar to doc1 chunks
            ("c4", "doc2", 1, vec![0.0, 0.0, 1.0]),   // very different
        ];

        for (chunk_id, doc_id, idx, embedding) in &chunks_data {
            conn.execute(
                "INSERT INTO chunks (id, document_id, collection_id, content, chunk_index, created_at) VALUES (?1, ?2, 'col1', 'text', ?3, ?4)",
                rusqlite::params![chunk_id, doc_id, idx, now],
            ).unwrap();

            let blob = f64_vec_to_bytes(embedding);
            conn.execute(
                "INSERT INTO chunk_embeddings (chunk_id, collection_id, document_id, embedding, content_preview) VALUES (?1, 'col1', ?2, ?3, 'preview')",
                rusqlite::params![chunk_id, doc_id, blob],
            ).unwrap();
        }
    }

    #[test]
    fn test_build_graph_edges_semantic() {
        let conn = setup_graph_db();
        seed_two_docs_with_chunks(&conn);

        let edges = build_graph_edges(&conn, "col1", 0.5).unwrap();

        let semantic: Vec<_> = edges.iter().filter(|e| e.relationship_type == "semantic").collect();
        // c1 (1,0,0) and c3 (0.95,0.05,0) are from different docs and very similar
        assert!(!semantic.is_empty(), "Should have semantic edges between similar cross-doc chunks");
    }

    #[test]
    fn test_build_graph_edges_same_document() {
        let conn = setup_graph_db();
        seed_two_docs_with_chunks(&conn);

        let edges = build_graph_edges(&conn, "col1", 0.5).unwrap();

        let same_doc: Vec<_> = edges.iter().filter(|e| e.relationship_type == "same_document").collect();
        // c1->c2 (doc1, index 0->1) and c3->c4 (doc2, index 0->1)
        assert_eq!(same_doc.len(), 2, "Should have 2 same_document edges (one per doc)");

        for edge in &same_doc {
            assert_eq!(edge.weight, 1.0);
        }
    }

    #[test]
    fn test_build_graph_edges_empty_collection() {
        let conn = setup_graph_db();
        conn.execute(
            "INSERT INTO collections (id, name, description, created_at, updated_at) VALUES ('empty', 'Empty', '', '2025-01-01', '2025-01-01')",
            [],
        ).unwrap();

        let edges = build_graph_edges(&conn, "empty", 0.5).unwrap();
        assert!(edges.is_empty());
    }

    #[test]
    fn test_get_graph_data_nodes_and_links() {
        let conn = setup_graph_db();
        seed_two_docs_with_chunks(&conn);

        // Build edges first
        build_graph_edges(&conn, "col1", 0.5).unwrap();

        let data = get_graph_data(&conn, "col1").unwrap();
        assert_eq!(data.nodes.len(), 2, "Should have 2 document nodes");

        let labels: Vec<&str> = data.nodes.iter().map(|n| n.label.as_str()).collect();
        assert!(labels.contains(&"Doc One"));
        assert!(labels.contains(&"Doc Two"));

        // Links are aggregated semantic edges between documents
        // There should be at least one since c1 and c3 are very similar
        assert!(!data.links.is_empty(), "Should have at least one semantic link between docs");
    }

    #[test]
    fn test_cosine_similarity_basic() {
        assert!((cosine_similarity(&[1.0, 0.0], &[1.0, 0.0]) - 1.0).abs() < 1e-9);
        assert!(cosine_similarity(&[1.0, 0.0], &[0.0, 1.0]).abs() < 1e-9);
        assert_eq!(cosine_similarity(&[], &[]), 0.0);
    }
}
