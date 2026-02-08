use std::collections::HashMap;

use instant_distance::{Builder, HnswMap, Search};
use rusqlite::Connection;

use crate::error::AppError;
use crate::utils::bytes_to_f64_vec;

/// A point in the HNSW index. Wraps a f64 vector.
#[derive(Clone, Debug)]
pub struct Point(pub Vec<f64>);

impl instant_distance::Point for Point {
    fn distance(&self, other: &Self) -> f32 {
        // Cosine distance = 1 - cosine_similarity
        let dot: f64 = self.0.iter().zip(other.0.iter()).map(|(a, b)| a * b).sum();
        let mag_a: f64 = self.0.iter().map(|x| x * x).sum::<f64>().sqrt();
        let mag_b: f64 = other.0.iter().map(|x| x * x).sum::<f64>().sqrt();
        if mag_a == 0.0 || mag_b == 0.0 {
            return 1.0;
        }
        let cosine_sim = dot / (mag_a * mag_b);
        (1.0 - cosine_sim) as f32
    }
}

/// In-memory HNSW index, one per collection.
pub struct VectorIndex {
    indices: HashMap<String, CollectionIndex>,
}

struct CollectionIndex {
    hnsw: HnswMap<Point, String>, // maps points to chunk_ids
    #[allow(dead_code)]
    chunk_ids: Vec<String>,       // for tracking what's in the index
}

impl VectorIndex {
    pub fn new() -> Self {
        VectorIndex {
            indices: HashMap::new(),
        }
    }

    /// Build index for a collection from database embeddings.
    pub fn build_collection_index(
        &mut self,
        conn: &Connection,
        collection_id: &str,
    ) -> Result<(), AppError> {
        let mut stmt = conn.prepare(
            "SELECT ce.chunk_id, ce.embedding
             FROM chunk_embeddings ce
             WHERE ce.collection_id = ?1",
        )?;

        let rows: Vec<(String, Vec<u8>)> = stmt
            .query_map(rusqlite::params![collection_id], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, Vec<u8>>(1)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        if rows.is_empty() {
            self.indices.remove(collection_id);
            return Ok(());
        }

        let mut points = Vec::with_capacity(rows.len());
        let mut chunk_ids = Vec::with_capacity(rows.len());

        for (chunk_id, blob) in &rows {
            let embedding = bytes_to_f64_vec(blob);
            points.push(Point(embedding));
            chunk_ids.push(chunk_id.clone());
        }

        let hnsw = Builder::default()
            .ef_construction(200)
            .build(points, chunk_ids.clone());

        self.indices.insert(
            collection_id.to_string(),
            CollectionIndex { hnsw, chunk_ids },
        );

        Ok(())
    }

    /// Search the HNSW index for a collection. Returns (chunk_id, similarity_score) pairs.
    pub fn search(
        &self,
        collection_id: &str,
        query: &[f64],
        top_k: usize,
    ) -> Result<Vec<(String, f64)>, AppError> {
        let index = match self.indices.get(collection_id) {
            Some(idx) => idx,
            None => return Ok(Vec::new()),
        };

        let query_point = Point(query.to_vec());
        let mut search = Search::default();
        let results = index.hnsw.search(&query_point, &mut search);

        let mut output = Vec::new();
        for item in results.take(top_k) {
            let similarity = 1.0 - item.distance as f64; // Convert distance back to similarity
            output.push((item.value.clone(), similarity));
        }

        Ok(output)
    }

    /// Rebuild index for a collection (used after inserts/deletes).
    pub fn rebuild_collection_index(
        &mut self,
        conn: &Connection,
        collection_id: &str,
    ) -> Result<(), AppError> {
        self.build_collection_index(conn, collection_id)
    }

    /// Drop the index for a collection.
    #[allow(dead_code)]
    pub fn drop_collection(&mut self, collection_id: &str) {
        self.indices.remove(collection_id);
    }

    /// Check if a collection has an index built.
    pub fn has_index(&self, collection_id: &str) -> bool {
        self.indices.contains_key(collection_id)
    }

    /// Get the number of vectors in a collection's index.
    #[allow(dead_code)]
    pub fn collection_size(&self, collection_id: &str) -> usize {
        self.indices
            .get(collection_id)
            .map(|i| i.chunk_ids.len())
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_point(vals: &[f64]) -> Point {
        Point(vals.to_vec())
    }

    fn setup_db_with_embeddings() -> (rusqlite::Connection, String) {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE chunk_embeddings (
                chunk_id TEXT PRIMARY KEY,
                collection_id TEXT NOT NULL,
                document_id TEXT NOT NULL,
                embedding BLOB NOT NULL,
                content_preview TEXT
            );",
        )
        .unwrap();

        let collection_id = "col1".to_string();
        // Insert 100 random-ish embeddings
        for i in 0..100 {
            let chunk_id = format!("chunk_{}", i);
            let mut embedding = vec![0.0f64; 10];
            embedding[i % 10] = 1.0; // One-hot-ish
            embedding[(i + 1) % 10] = 0.5;
            let bytes = crate::utils::f64_vec_to_bytes(&embedding);
            conn.execute(
                "INSERT INTO chunk_embeddings (chunk_id, collection_id, document_id, embedding, content_preview)
                 VALUES (?1, ?2, 'doc1', ?3, 'preview')",
                rusqlite::params![chunk_id, collection_id, bytes],
            )
            .unwrap();
        }

        (conn, collection_id)
    }

    #[test]
    fn test_vector_index_build_and_search() {
        let (conn, cid) = setup_db_with_embeddings();
        let mut index = VectorIndex::new();
        index.build_collection_index(&conn, &cid).unwrap();

        assert!(index.has_index(&cid));
        assert_eq!(index.collection_size(&cid), 100);

        // Search for a vector close to chunk_0 (1.0 at idx 0, 0.5 at idx 1)
        // Note: chunks 0, 10, 20, ..., 90 all share the same embedding pattern
        let query = vec![1.0, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let results = index.search(&cid, &query, 5).unwrap();
        assert_eq!(results.len(), 5);
        // Top result should have very high similarity (exact or near-exact match)
        assert!(
            results[0].1 > 0.99,
            "Expected top result similarity > 0.99, got {}",
            results[0].1
        );
        // Results should be sorted by similarity descending
        for w in results.windows(2) {
            assert!(w[0].1 >= w[1].1, "Results should be sorted descending by similarity");
        }
    }

    #[test]
    fn test_vector_index_empty_collection() {
        let index = VectorIndex::new();
        let results = index.search("nonexistent", &[1.0, 0.0], 5).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_vector_index_drop_collection() {
        let (conn, cid) = setup_db_with_embeddings();
        let mut index = VectorIndex::new();
        index.build_collection_index(&conn, &cid).unwrap();
        assert!(index.has_index(&cid));

        index.drop_collection(&cid);
        assert!(!index.has_index(&cid));
    }

    #[test]
    fn test_vector_index_rebuild() {
        let (conn, cid) = setup_db_with_embeddings();
        let mut index = VectorIndex::new();
        index.build_collection_index(&conn, &cid).unwrap();
        assert_eq!(index.collection_size(&cid), 100);

        // Add more embeddings
        for i in 100..110 {
            let chunk_id = format!("chunk_{}", i);
            let embedding = vec![0.1f64; 10];
            let bytes = crate::utils::f64_vec_to_bytes(&embedding);
            conn.execute(
                "INSERT INTO chunk_embeddings (chunk_id, collection_id, document_id, embedding, content_preview)
                 VALUES (?1, ?2, 'doc2', ?3, 'preview')",
                rusqlite::params![chunk_id, cid, bytes],
            )
            .unwrap();
        }

        index.rebuild_collection_index(&conn, &cid).unwrap();
        assert_eq!(index.collection_size(&cid), 110);
    }

    #[test]
    fn test_point_distance_identical() {
        use instant_distance::Point as PointTrait;
        let a = make_point(&[1.0, 0.0, 0.0]);
        let b = make_point(&[1.0, 0.0, 0.0]);
        assert!(a.distance(&b) < 0.001);
    }

    #[test]
    fn test_point_distance_orthogonal() {
        use instant_distance::Point as PointTrait;
        let a = make_point(&[1.0, 0.0]);
        let b = make_point(&[0.0, 1.0]);
        assert!((a.distance(&b) - 1.0).abs() < 0.01);
    }
}
