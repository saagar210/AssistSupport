use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::models::GraphEdge;

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

/// Decode a blob of bytes into a Vec<f64> (little-endian f64s).
fn bytes_to_f64_vec(bytes: &[u8]) -> Vec<f64> {
    bytes
        .chunks_exact(8)
        .map(|chunk| {
            let arr: [u8; 8] = chunk.try_into().unwrap_or([0u8; 8]);
            f64::from_le_bytes(arr)
        })
        .collect()
}

/// Cosine similarity between two vectors.
fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let mut dot = 0.0;
    let mut norm_a = 0.0;
    let mut norm_b = 0.0;

    for (x, y) in a.iter().zip(b.iter()) {
        dot += x * y;
        norm_a += x * x;
        norm_b += y * y;
    }

    let denom = norm_a.sqrt() * norm_b.sqrt();
    if denom == 0.0 {
        0.0
    } else {
        dot / denom
    }
}
