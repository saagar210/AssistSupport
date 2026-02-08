use std::collections::{HashMap, HashSet, VecDeque};

use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::models::GraphEdge;
use crate::utils::{bytes_to_f64_vec, cosine_similarity};
use crate::vector_index::VectorIndex;

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

/// Build graph edges incrementally for newly added chunks using the HNSW index.
///
/// For each new chunk, queries the HNSW index for top-k neighbors, filters by
/// similarity threshold and cross-document constraint, then creates semantic edges.
/// Also creates same_document edges for sequential chunks within the same document.
/// This is O(new_chunks * k log n) instead of O(n^2).
pub fn build_graph_edges_incremental(
    conn: &rusqlite::Connection,
    index: &VectorIndex,
    collection_id: &str,
    new_chunk_ids: &[String],
    similarity_threshold: f64,
) -> Result<Vec<GraphEdge>, AppError> {
    if new_chunk_ids.is_empty() {
        return Ok(Vec::new());
    }

    let new_set: HashSet<&str> = new_chunk_ids.iter().map(|s| s.as_str()).collect();
    let now = chrono::Utc::now().to_rfc3339();
    let mut edges: Vec<GraphEdge> = Vec::new();

    // Track which edge pairs we've already created to avoid duplicates
    let mut seen_pairs: HashSet<(String, String)> = HashSet::new();

    // Load document_id and chunk_index for new chunks
    struct ChunkMeta {
        chunk_id: String,
        document_id: String,
        chunk_index: i32,
    }

    // Build a lookup of chunk_id -> (document_id, chunk_index) for neighbor filtering
    // We need this for both new chunks and their potential neighbors
    let mut all_chunks_stmt = conn.prepare(
        "SELECT c.id, c.document_id, c.chunk_index FROM chunks c WHERE c.collection_id = ?1",
    )?;
    let all_chunk_metas: Vec<ChunkMeta> = all_chunks_stmt
        .query_map(rusqlite::params![collection_id], |row| {
            Ok(ChunkMeta {
                chunk_id: row.get(0)?,
                document_id: row.get(1)?,
                chunk_index: row.get(2)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let chunk_meta_map: std::collections::HashMap<&str, (&str, i32)> = all_chunk_metas
        .iter()
        .map(|m| (m.chunk_id.as_str(), (m.document_id.as_str(), m.chunk_index)))
        .collect();

    // 1. Semantic edges: for each new chunk, query HNSW for top-50 neighbors
    for chunk_id in new_chunk_ids {
        // Load the embedding for this chunk from DB
        let embedding_blob: Vec<u8> = match conn.query_row(
            "SELECT embedding FROM chunk_embeddings WHERE chunk_id = ?1",
            rusqlite::params![chunk_id],
            |row| row.get(0),
        ) {
            Ok(blob) => blob,
            Err(rusqlite::Error::QueryReturnedNoRows) => continue,
            Err(e) => return Err(AppError::Database(e)),
        };
        let embedding = bytes_to_f64_vec(&embedding_blob);

        let (source_doc_id, _) = match chunk_meta_map.get(chunk_id.as_str()) {
            Some(meta) => *meta,
            None => continue,
        };

        // Query HNSW index for top-50 neighbors
        let neighbors = index.search(collection_id, &embedding, 50)?;

        for (neighbor_id, similarity) in neighbors {
            // Skip self
            if neighbor_id == *chunk_id {
                continue;
            }

            // Skip if below threshold
            if similarity <= similarity_threshold {
                continue;
            }

            // Skip same-document neighbors (semantic edges are cross-document only)
            let (neighbor_doc_id, _) = match chunk_meta_map.get(neighbor_id.as_str()) {
                Some(meta) => *meta,
                None => continue,
            };
            if neighbor_doc_id == source_doc_id {
                continue;
            }

            // Normalize edge direction to avoid duplicates (smaller id first)
            let (a, b) = if *chunk_id < neighbor_id {
                (chunk_id.clone(), neighbor_id.clone())
            } else {
                (neighbor_id.clone(), chunk_id.clone())
            };

            if seen_pairs.contains(&(a.clone(), b.clone())) {
                continue;
            }
            seen_pairs.insert((a.clone(), b.clone()));

            edges.push(GraphEdge {
                id: uuid::Uuid::new_v4().to_string(),
                source_chunk_id: a,
                target_chunk_id: b,
                collection_id: collection_id.to_string(),
                weight: similarity,
                relationship_type: "semantic".to_string(),
                created_at: now.clone(),
            });
        }
    }

    // 2. Same-document edges: sequential chunks within each document that has new chunks
    // Find which documents the new chunks belong to
    let new_doc_ids: HashSet<&str> = new_chunk_ids
        .iter()
        .filter_map(|cid| chunk_meta_map.get(cid.as_str()).map(|(doc_id, _)| *doc_id))
        .collect();

    for doc_id in new_doc_ids {
        // Get all chunks for this document, ordered by chunk_index
        let mut doc_chunks: Vec<(&str, i32)> = all_chunk_metas
            .iter()
            .filter(|m| m.document_id == doc_id)
            .map(|m| (m.chunk_id.as_str(), m.chunk_index))
            .collect();
        doc_chunks.sort_by_key(|(_, idx)| *idx);

        for window in doc_chunks.windows(2) {
            let (id_a, idx_a) = window[0];
            let (id_b, idx_b) = window[1];

            // Only create edge if sequential and at least one chunk is new
            if idx_b == idx_a + 1 && (new_set.contains(id_a) || new_set.contains(id_b)) {
                let pair = (id_a.to_string(), id_b.to_string());
                if !seen_pairs.contains(&pair) {
                    seen_pairs.insert(pair);
                    edges.push(GraphEdge {
                        id: uuid::Uuid::new_v4().to_string(),
                        source_chunk_id: id_a.to_string(),
                        target_chunk_id: id_b.to_string(),
                        collection_id: collection_id.to_string(),
                        weight: 1.0,
                        relationship_type: "same_document".to_string(),
                        created_at: now.clone(),
                    });
                }
            }
        }
    }

    // Insert new edges (do NOT delete existing ones — this is incremental)
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

/// Rebuild all graph edges for a collection using the HNSW index.
///
/// Deletes all existing edges and rebuilds them using HNSW neighbor search.
/// This is O(n * k log n) instead of O(n^2) from the original `build_graph_edges`.
pub fn rebuild_graph_edges(
    conn: &rusqlite::Connection,
    index: &VectorIndex,
    collection_id: &str,
    similarity_threshold: f64,
) -> Result<usize, AppError> {
    // Delete all existing edges for this collection
    conn.execute(
        "DELETE FROM graph_edges WHERE collection_id = ?1",
        rusqlite::params![collection_id],
    )?;

    // Get all chunk IDs for this collection
    let mut stmt = conn.prepare(
        "SELECT id FROM chunks WHERE collection_id = ?1",
    )?;
    let all_chunk_ids: Vec<String> = stmt
        .query_map(rusqlite::params![collection_id], |row| row.get(0))?
        .collect::<Result<Vec<_>, _>>()?;

    if all_chunk_ids.is_empty() {
        return Ok(0);
    }

    let edges = build_graph_edges_incremental(
        conn,
        index,
        collection_id,
        &all_chunk_ids,
        similarity_threshold,
    )?;

    Ok(edges.len())
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphTraversalNode {
    pub chunk_id: String,
    pub document_id: String,
    pub document_title: String,
    pub depth: usize,
    pub path_weight: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Community {
    pub id: usize,
    pub members: Vec<String>,
    pub size: usize,
}

/// Load the adjacency list for a collection from graph_edges, filtering by min_weight.
/// Returns a map: chunk_id -> Vec<(neighbor_chunk_id, weight)>.
/// Edges are treated as undirected.
fn load_adjacency(
    conn: &rusqlite::Connection,
    collection_id: &str,
    min_weight: f64,
) -> Result<HashMap<String, Vec<(String, f64)>>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT source_chunk_id, target_chunk_id, weight
         FROM graph_edges
         WHERE collection_id = ?1 AND weight >= ?2",
    )?;

    let mut adj: HashMap<String, Vec<(String, f64)>> = HashMap::new();

    let rows = stmt.query_map(rusqlite::params![collection_id, min_weight], |row| {
        let src: String = row.get(0)?;
        let tgt: String = row.get(1)?;
        let w: f64 = row.get(2)?;
        Ok((src, tgt, w))
    })?;

    for row in rows {
        let (src, tgt, w) = row?;
        adj.entry(src.clone())
            .or_default()
            .push((tgt.clone(), w));
        adj.entry(tgt).or_default().push((src, w));
    }

    Ok(adj)
}

/// BFS traversal from a start node, up to max_depth, filtering edges by min_weight.
pub fn traverse_graph(
    conn: &rusqlite::Connection,
    collection_id: &str,
    start_chunk_id: &str,
    max_depth: usize,
    min_weight: f64,
) -> Result<Vec<GraphTraversalNode>, AppError> {
    let adj = load_adjacency(conn, collection_id, min_weight)?;

    // Build a lookup of chunk_id -> (document_id, document_title)
    let mut meta_stmt = conn.prepare(
        "SELECT c.id, c.document_id, d.title
         FROM chunks c
         JOIN documents d ON d.id = c.document_id
         WHERE c.collection_id = ?1",
    )?;
    let chunk_meta: HashMap<String, (String, String)> = meta_stmt
        .query_map(rusqlite::params![collection_id], |row| {
            let cid: String = row.get(0)?;
            let did: String = row.get(1)?;
            let title: String = row.get(2)?;
            Ok((cid, did, title))
        })?
        .filter_map(|r| r.ok())
        .map(|(cid, did, title)| (cid, (did, title)))
        .collect();

    let mut visited: HashSet<String> = HashSet::new();
    let mut queue: VecDeque<(String, usize, f64)> = VecDeque::new();
    let mut results: Vec<GraphTraversalNode> = Vec::new();

    visited.insert(start_chunk_id.to_string());
    queue.push_back((start_chunk_id.to_string(), 0, 1.0));

    while let Some((current, depth, path_weight)) = queue.pop_front() {
        let (doc_id, doc_title) = chunk_meta
            .get(&current)
            .cloned()
            .unwrap_or_else(|| ("unknown".to_string(), "Unknown".to_string()));

        results.push(GraphTraversalNode {
            chunk_id: current.clone(),
            document_id: doc_id,
            document_title: doc_title,
            depth,
            path_weight,
        });

        if depth >= max_depth {
            continue;
        }

        if let Some(neighbors) = adj.get(&current) {
            for (neighbor_id, weight) in neighbors {
                if !visited.contains(neighbor_id) {
                    visited.insert(neighbor_id.clone());
                    queue.push_back((
                        neighbor_id.clone(),
                        depth + 1,
                        path_weight * weight,
                    ));
                }
            }
        }
    }

    Ok(results)
}

/// Find shortest path between two chunks using BFS.
/// Returns the list of chunk_ids from source to target, or empty if no path exists.
pub fn find_path(
    conn: &rusqlite::Connection,
    collection_id: &str,
    from_chunk_id: &str,
    to_chunk_id: &str,
) -> Result<Vec<String>, AppError> {
    // Load all edges (min_weight=0 to consider all connections)
    let adj = load_adjacency(conn, collection_id, 0.0)?;

    let mut visited: HashSet<String> = HashSet::new();
    // Store (node, parent) for path reconstruction
    let mut queue: VecDeque<String> = VecDeque::new();
    let mut parent: HashMap<String, String> = HashMap::new();

    visited.insert(from_chunk_id.to_string());
    queue.push_back(from_chunk_id.to_string());

    let mut found = false;

    while let Some(current) = queue.pop_front() {
        if current == to_chunk_id {
            found = true;
            break;
        }

        if let Some(neighbors) = adj.get(&current) {
            for (neighbor_id, _) in neighbors {
                if !visited.contains(neighbor_id) {
                    visited.insert(neighbor_id.clone());
                    parent.insert(neighbor_id.clone(), current.clone());
                    queue.push_back(neighbor_id.clone());
                }
            }
        }
    }

    if !found {
        return Ok(Vec::new());
    }

    // Reconstruct path from to_chunk_id back to from_chunk_id
    let mut path = Vec::new();
    let mut current = to_chunk_id.to_string();
    path.push(current.clone());

    while current != from_chunk_id {
        match parent.get(&current) {
            Some(p) => {
                current = p.clone();
                path.push(current.clone());
            }
            None => return Ok(Vec::new()),
        }
    }

    path.reverse();
    Ok(path)
}

/// Detect communities using label propagation algorithm.
///
/// 1. Assign each node its own label
/// 2. In each iteration, each node adopts the most frequent label among its neighbors (weighted)
/// 3. Repeat until convergence (max 100 iterations)
/// 4. Group nodes by final label
pub fn detect_communities(
    conn: &rusqlite::Connection,
    collection_id: &str,
    min_weight: f64,
) -> Result<Vec<Community>, AppError> {
    let adj = load_adjacency(conn, collection_id, min_weight)?;

    let nodes: Vec<String> = adj.keys().cloned().collect();
    if nodes.is_empty() {
        return Ok(Vec::new());
    }

    // Step 1: each node gets a unique label (its index)
    let node_to_idx: HashMap<&str, usize> = nodes
        .iter()
        .enumerate()
        .map(|(i, n)| (n.as_str(), i))
        .collect();

    let mut labels: Vec<usize> = (0..nodes.len()).collect();

    // Step 2-3: iterate label propagation
    let max_iterations = 100;
    for _ in 0..max_iterations {
        let mut changed = false;

        for (i, node) in nodes.iter().enumerate() {
            if let Some(neighbors) = adj.get(node) {
                if neighbors.is_empty() {
                    continue;
                }

                // Accumulate weighted votes for each neighbor label
                let mut label_weights: HashMap<usize, f64> = HashMap::new();
                for (neighbor_id, weight) in neighbors {
                    if let Some(&idx) = node_to_idx.get(neighbor_id.as_str()) {
                        *label_weights.entry(labels[idx]).or_insert(0.0) += weight;
                    }
                }

                // Pick the label with the highest total weight
                if let Some((&best_label, _)) = label_weights
                    .iter()
                    .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
                {
                    if labels[i] != best_label {
                        labels[i] = best_label;
                        changed = true;
                    }
                }
            }
        }

        if !changed {
            break;
        }
    }

    // Step 4: group nodes by final label
    let mut groups: HashMap<usize, Vec<String>> = HashMap::new();
    for (i, node) in nodes.iter().enumerate() {
        groups.entry(labels[i]).or_default().push(node.clone());
    }

    let mut communities: Vec<Community> = groups
        .into_iter()
        .enumerate()
        .map(|(id, (_label, members))| {
            let size = members.len();
            Community { id, members, size }
        })
        .collect();

    // Sort by size descending for consistent output
    communities.sort_by(|a, b| b.size.cmp(&a.size));

    // Re-assign IDs after sorting
    for (i, community) in communities.iter_mut().enumerate() {
        community.id = i;
    }

    Ok(communities)
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

    /// Helper: build HNSW index from seeded data
    fn build_test_index(conn: &rusqlite::Connection, collection_id: &str) -> VectorIndex {
        let mut index = VectorIndex::new();
        index.build_collection_index(conn, collection_id).unwrap();
        index
    }

    #[test]
    fn test_incremental_graph_adds_edges_for_new_chunks() {
        let conn = setup_graph_db();
        seed_two_docs_with_chunks(&conn);
        let index = build_test_index(&conn, "col1");

        // Simulate: doc2 chunks (c3, c4) are "new"
        let new_ids = vec!["c3".to_string(), "c4".to_string()];
        let edges = build_graph_edges_incremental(&conn, &index, "col1", &new_ids, 0.5).unwrap();

        // Should have semantic edges (c3 is similar to c1/c2 across docs)
        let semantic: Vec<_> = edges.iter().filter(|e| e.relationship_type == "semantic").collect();
        assert!(!semantic.is_empty(), "Should create semantic edges for new chunks");

        // Should have same_document edge for c3->c4
        let same_doc: Vec<_> = edges.iter().filter(|e| e.relationship_type == "same_document").collect();
        assert!(
            same_doc.iter().any(|e| {
                (e.source_chunk_id == "c3" && e.target_chunk_id == "c4")
                    || (e.source_chunk_id == "c4" && e.target_chunk_id == "c3")
            }),
            "Should create same_document edge for sequential new chunks"
        );

        // Verify edges were persisted to DB
        let db_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM graph_edges WHERE collection_id = 'col1'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(db_count, edges.len() as i64, "All edges should be persisted to DB");
    }

    #[test]
    fn test_incremental_graph_preserves_existing_edges() {
        let conn = setup_graph_db();
        seed_two_docs_with_chunks(&conn);

        // First, build full graph edges (the original O(n^2) way)
        let _original_edges = build_graph_edges(&conn, "col1", 0.5).unwrap();
        let original_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM graph_edges WHERE collection_id = 'col1'", [], |r| r.get(0))
            .unwrap();
        assert!(original_count > 0, "Should have original edges");

        // Now add a new document with a new chunk
        let now = "2025-01-01T00:00:00Z";
        conn.execute(
            "INSERT INTO documents (id, collection_id, filename, file_path, file_type, file_size, file_hash, title, word_count, chunk_count, created_at, updated_at)
             VALUES ('doc3', 'col1', 'f3.txt', '/f3.txt', 'txt', 100, 'hash3', 'Doc Three', 50, 1, ?1, ?1)",
            rusqlite::params![now],
        ).unwrap();

        let new_embedding = vec![0.98, 0.02, 0.0]; // similar to c1
        conn.execute(
            "INSERT INTO chunks (id, document_id, collection_id, content, chunk_index, created_at) VALUES ('c5', 'doc3', 'col1', 'text', 0, ?1)",
            rusqlite::params![now],
        ).unwrap();
        let blob = f64_vec_to_bytes(&new_embedding);
        conn.execute(
            "INSERT INTO chunk_embeddings (chunk_id, collection_id, document_id, embedding, content_preview) VALUES ('c5', 'col1', 'doc3', ?1, 'preview')",
            rusqlite::params![blob],
        ).unwrap();

        // Rebuild HNSW index to include the new chunk
        let index = build_test_index(&conn, "col1");

        // Run incremental build for only the new chunk
        let new_edges = build_graph_edges_incremental(&conn, &index, "col1", &["c5".to_string()], 0.5).unwrap();
        assert!(!new_edges.is_empty(), "Should create edges for new chunk");

        // Verify original edges are still present
        let total_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM graph_edges WHERE collection_id = 'col1'", [], |r| r.get(0))
            .unwrap();
        assert!(
            total_count >= original_count + new_edges.len() as i64,
            "Total edges ({}) should be at least original ({}) + new ({})",
            total_count,
            original_count,
            new_edges.len()
        );

        // Verify at least one original edge still exists by checking for a known pattern
        let has_original: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM graph_edges WHERE collection_id = 'col1' AND id IN (SELECT id FROM graph_edges WHERE source_chunk_id IN ('c1','c2','c3','c4') AND target_chunk_id IN ('c1','c2','c3','c4'))",
                [],
                |r| r.get::<_, i64>(0),
            )
            .map(|c| c > 0)
            .unwrap();
        assert!(has_original, "Original edges should be preserved");
    }

    #[test]
    fn test_rebuild_graph_edges_uses_index() {
        let conn = setup_graph_db();
        seed_two_docs_with_chunks(&conn);

        // Build HNSW index
        let index = build_test_index(&conn, "col1");

        // Use rebuild_graph_edges (HNSW-based full rebuild)
        let edge_count = rebuild_graph_edges(&conn, &index, "col1", 0.5).unwrap();

        // Should have edges: semantic cross-doc + same_document sequential
        assert!(edge_count > 0, "rebuild_graph_edges should create edges");

        // Verify semantic edges exist between similar cross-doc chunks
        let semantic_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM graph_edges WHERE collection_id = 'col1' AND relationship_type = 'semantic'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert!(semantic_count > 0, "Should have semantic edges from HNSW-based rebuild");

        // Verify same_document edges
        let same_doc_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM graph_edges WHERE collection_id = 'col1' AND relationship_type = 'same_document'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(same_doc_count, 2, "Should have 2 same_document edges (one per doc)");

        // Total should match what rebuild returned
        let total: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM graph_edges WHERE collection_id = 'col1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(total, edge_count as i64, "DB edge count should match returned count");
    }

    /// Helper: seed a graph with known structure for algorithm tests.
    /// Creates two clusters:
    ///   Cluster A: a1 -- a2 -- a3  (all weight 0.9)
    ///   Cluster B: b1 -- b2        (weight 0.8)
    ///   Cross-link: a3 -- b1       (weight 0.4)
    fn seed_graph_for_algorithms(conn: &rusqlite::Connection) {
        let now = "2025-01-01T00:00:00Z";
        conn.execute(
            "INSERT INTO collections (id, name, description, created_at, updated_at) VALUES ('col1', 'Test', '', ?1, ?1)",
            rusqlite::params![now],
        ).unwrap();

        // Two documents
        for (doc_id, title) in &[("docA", "Doc A"), ("docB", "Doc B")] {
            conn.execute(
                "INSERT INTO documents (id, collection_id, filename, file_path, file_type, file_size, file_hash, title, word_count, chunk_count, created_at, updated_at)
                 VALUES (?1, 'col1', 'f.txt', '/f.txt', 'txt', 100, 'hash', ?2, 50, 3, ?3, ?3)",
                rusqlite::params![doc_id, title, now],
            ).unwrap();
        }

        // Chunks
        let chunks = vec![
            ("a1", "docA", 0), ("a2", "docA", 1), ("a3", "docA", 2),
            ("b1", "docB", 0), ("b2", "docB", 1),
        ];
        for (cid, did, idx) in &chunks {
            conn.execute(
                "INSERT INTO chunks (id, document_id, collection_id, content, chunk_index, created_at) VALUES (?1, ?2, 'col1', 'text', ?3, ?4)",
                rusqlite::params![cid, did, idx, now],
            ).unwrap();
        }

        // Manually insert graph edges
        let edges = vec![
            ("e1", "a1", "a2", 0.9),
            ("e2", "a2", "a3", 0.9),
            ("e3", "a3", "b1", 0.4),
            ("e4", "b1", "b2", 0.8),
        ];
        for (eid, src, tgt, weight) in &edges {
            conn.execute(
                "INSERT INTO graph_edges (id, source_chunk_id, target_chunk_id, collection_id, weight, relationship_type, created_at)
                 VALUES (?1, ?2, ?3, 'col1', ?4, 'semantic', ?5)",
                rusqlite::params![eid, src, tgt, weight, now],
            ).unwrap();
        }
    }

    #[test]
    fn test_traverse_graph_bfs() {
        let conn = setup_graph_db();
        seed_graph_for_algorithms(&conn);

        // Traverse from a1 with max_depth=2, min_weight=0.5 (excludes a3->b1 edge at 0.4)
        let result = traverse_graph(&conn, "col1", "a1", 2, 0.5).unwrap();

        // Should find a1 (depth 0), a2 (depth 1), a3 (depth 2)
        assert_eq!(result.len(), 3, "BFS with depth=2 from a1 should find 3 nodes");
        assert_eq!(result[0].chunk_id, "a1");
        assert_eq!(result[0].depth, 0);
        assert_eq!(result[1].chunk_id, "a2");
        assert_eq!(result[1].depth, 1);
        assert_eq!(result[2].chunk_id, "a3");
        assert_eq!(result[2].depth, 2);

        // b1 and b2 should NOT be included (edge a3->b1 is 0.4, below min_weight 0.5)
        let chunk_ids: Vec<&str> = result.iter().map(|n| n.chunk_id.as_str()).collect();
        assert!(!chunk_ids.contains(&"b1"), "b1 should be excluded by min_weight filter");

        // Verify document metadata is populated
        assert_eq!(result[0].document_id, "docA");
        assert_eq!(result[0].document_title, "Doc A");
    }

    #[test]
    fn test_find_path_direct_connection() {
        let conn = setup_graph_db();
        seed_graph_for_algorithms(&conn);

        // Direct neighbors a1 -> a2
        let path = find_path(&conn, "col1", "a1", "a2").unwrap();
        assert_eq!(path, vec!["a1", "a2"]);

        // Multi-hop a1 -> a2 -> a3 -> b1 -> b2
        let path = find_path(&conn, "col1", "a1", "b2").unwrap();
        assert_eq!(path, vec!["a1", "a2", "a3", "b1", "b2"]);
    }

    #[test]
    fn test_find_path_no_connection() {
        let conn = setup_graph_db();
        seed_graph_for_algorithms(&conn);

        // Add an isolated chunk
        let now = "2025-01-01T00:00:00Z";
        conn.execute(
            "INSERT INTO chunks (id, document_id, collection_id, content, chunk_index, created_at) VALUES ('isolated', 'docA', 'col1', 'text', 99, ?1)",
            rusqlite::params![now],
        ).unwrap();

        let path = find_path(&conn, "col1", "a1", "isolated").unwrap();
        assert!(path.is_empty(), "Path to disconnected chunk should be empty");
    }

    #[test]
    fn test_detect_communities_simple() {
        let conn = setup_graph_db();
        seed_graph_for_algorithms(&conn);

        // Use min_weight=0.5 to cut the weak a3->b1 edge (0.4)
        let communities = detect_communities(&conn, "col1", 0.5).unwrap();

        assert_eq!(communities.len(), 2, "Should detect 2 communities with min_weight=0.5");

        // Largest community should have 3 members (a1, a2, a3)
        assert_eq!(communities[0].size, 3);
        let cluster_a: HashSet<&str> = communities[0].members.iter().map(|s| s.as_str()).collect();
        assert!(cluster_a.contains("a1"));
        assert!(cluster_a.contains("a2"));
        assert!(cluster_a.contains("a3"));

        // Smaller community should have 2 members (b1, b2)
        assert_eq!(communities[1].size, 2);
        let cluster_b: HashSet<&str> = communities[1].members.iter().map(|s| s.as_str()).collect();
        assert!(cluster_b.contains("b1"));
        assert!(cluster_b.contains("b2"));
    }
}
