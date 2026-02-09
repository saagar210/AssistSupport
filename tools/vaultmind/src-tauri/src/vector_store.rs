use rusqlite::Connection;

use crate::error::AppError;
use crate::utils::{bytes_to_f64_vec, cosine_similarity, f64_vec_to_bytes};

/// Initialize the chunk_embeddings table. Call this during DB setup.
#[allow(dead_code)]
pub fn create_table(conn: &Connection) -> Result<(), AppError> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS chunk_embeddings (
            chunk_id TEXT PRIMARY KEY,
            collection_id TEXT NOT NULL,
            document_id TEXT NOT NULL,
            embedding BLOB NOT NULL,
            content_preview TEXT
        );
        CREATE INDEX IF NOT EXISTS idx_chunk_embeddings_collection
            ON chunk_embeddings(collection_id);
        CREATE INDEX IF NOT EXISTS idx_chunk_embeddings_document
            ON chunk_embeddings(document_id);",
    )?;
    Ok(())
}

/// Store embedding vectors for a batch of chunks.
/// Each tuple: (chunk_id, collection_id, document_id, embedding_vec, content_preview)
pub fn store_embeddings(
    conn: &Connection,
    chunks: &[(String, String, String, Vec<f64>, String)],
) -> Result<(), AppError> {
    let mut stmt = conn.prepare(
        "INSERT OR REPLACE INTO chunk_embeddings (chunk_id, collection_id, document_id, embedding, content_preview)
         VALUES (?1, ?2, ?3, ?4, ?5)",
    )?;

    for (chunk_id, collection_id, document_id, embedding, preview) in chunks {
        let bytes = f64_vec_to_bytes(embedding);
        stmt.execute(rusqlite::params![chunk_id, collection_id, document_id, bytes, preview])?;
    }

    Ok(())
}

/// Search for the top_k most similar vectors in a collection.
/// Returns (chunk_id, cosine_similarity_score) pairs sorted by score descending.
#[allow(dead_code)]
pub fn search_vectors(
    conn: &Connection,
    collection_id: &str,
    query_vec: &[f64],
    top_k: usize,
) -> Result<Vec<(String, f64)>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT chunk_id, embedding FROM chunk_embeddings WHERE collection_id = ?1",
    )?;

    let rows = stmt.query_map(rusqlite::params![collection_id], |row| {
        let chunk_id: String = row.get(0)?;
        let blob: Vec<u8> = row.get(1)?;
        Ok((chunk_id, blob))
    })?;

    let mut scored: Vec<(String, f64)> = Vec::new();
    for row in rows {
        let (chunk_id, blob) = row?;
        let embedding = bytes_to_f64_vec(&blob);
        let score = cosine_similarity(query_vec, &embedding);
        scored.push((chunk_id, score));
    }

    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(top_k);

    Ok(scored)
}

/// Delete all embeddings for a specific document.
pub fn delete_document_vectors(conn: &Connection, document_id: &str) -> Result<(), AppError> {
    conn.execute(
        "DELETE FROM chunk_embeddings WHERE document_id = ?1",
        rusqlite::params![document_id],
    )?;
    Ok(())
}

/// Delete all embeddings for a specific collection.
#[allow(dead_code)]
pub fn delete_collection_vectors(conn: &Connection, collection_id: &str) -> Result<(), AppError> {
    conn.execute(
        "DELETE FROM chunk_embeddings WHERE collection_id = ?1",
        rusqlite::params![collection_id],
    )?;
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;

    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        create_table(&conn).unwrap();
        conn
    }

    #[test]
    fn test_create_table() {
        let conn = setup_db();
        let exists: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='chunk_embeddings'",
                [],
                |row| row.get::<_, i64>(0),
            )
            .map(|c| c > 0)
            .unwrap();
        assert!(exists);
    }

    #[test]
    fn test_store_and_retrieve_embeddings() {
        let conn = setup_db();
        let chunks = vec![(
            "chunk1".to_string(),
            "col1".to_string(),
            "doc1".to_string(),
            vec![1.0, 0.0, 0.0],
            "preview".to_string(),
        )];

        store_embeddings(&conn, &chunks).unwrap();

        let results = search_vectors(&conn, "col1", &[1.0, 0.0, 0.0], 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, "chunk1");
        assert!((results[0].1 - 1.0).abs() < 1e-9, "Identical vectors should have similarity ~1.0");
    }

    #[test]
    fn test_cosine_similarity_known_vectors() {
        // Identical vectors => 1.0
        assert!((cosine_similarity(&[1.0, 0.0], &[1.0, 0.0]) - 1.0).abs() < 1e-9);

        // Orthogonal vectors => 0.0
        assert!((cosine_similarity(&[1.0, 0.0], &[0.0, 1.0])).abs() < 1e-9);

        // Opposite vectors => -1.0
        assert!((cosine_similarity(&[1.0, 0.0], &[-1.0, 0.0]) - (-1.0)).abs() < 1e-9);

        // Empty vectors => 0.0
        assert_eq!(cosine_similarity(&[], &[]), 0.0);

        // Different lengths => 0.0
        assert_eq!(cosine_similarity(&[1.0], &[1.0, 2.0]), 0.0);
    }

    #[test]
    fn test_search_returns_correct_top_k_ordering() {
        let conn = setup_db();

        let chunks = vec![
            ("c1".into(), "col1".into(), "doc1".into(), vec![1.0, 0.0, 0.0], "p1".into()),
            ("c2".into(), "col1".into(), "doc1".into(), vec![0.9, 0.1, 0.0], "p2".into()),
            ("c3".into(), "col1".into(), "doc1".into(), vec![0.0, 1.0, 0.0], "p3".into()),
            ("c4".into(), "col1".into(), "doc1".into(), vec![0.5, 0.5, 0.0], "p4".into()),
        ];
        store_embeddings(&conn, &chunks).unwrap();

        let query = vec![1.0, 0.0, 0.0];
        let results = search_vectors(&conn, "col1", &query, 2).unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, "c1", "Most similar should be first");
        assert!(results[0].1 >= results[1].1, "Results should be sorted descending by score");
    }

    #[test]
    fn test_delete_by_document() {
        let conn = setup_db();
        let chunks = vec![
            ("c1".into(), "col1".into(), "doc1".into(), vec![1.0, 0.0], "p1".into()),
            ("c2".into(), "col1".into(), "doc2".into(), vec![0.0, 1.0], "p2".into()),
        ];
        store_embeddings(&conn, &chunks).unwrap();

        delete_document_vectors(&conn, "doc1").unwrap();

        let results = search_vectors(&conn, "col1", &[1.0, 0.0], 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, "c2");
    }

    #[test]
    fn test_delete_by_collection() {
        let conn = setup_db();
        let chunks = vec![
            ("c1".into(), "col1".into(), "doc1".into(), vec![1.0, 0.0], "p1".into()),
            ("c2".into(), "col2".into(), "doc2".into(), vec![0.0, 1.0], "p2".into()),
        ];
        store_embeddings(&conn, &chunks).unwrap();

        delete_collection_vectors(&conn, "col1").unwrap();

        let results = search_vectors(&conn, "col1", &[1.0, 0.0], 10).unwrap();
        assert!(results.is_empty());

        let results = search_vectors(&conn, "col2", &[0.0, 1.0], 10).unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_vec_bytes_roundtrip() {
        let original = vec![1.5, -2.3, 0.0, 42.0, std::f64::consts::PI];
        let bytes = f64_vec_to_bytes(&original);
        let restored = bytes_to_f64_vec(&bytes);
        assert_eq!(original, restored);
    }

    #[test]
    fn test_search_empty_collection() {
        let conn = setup_db();
        let results = search_vectors(&conn, "nonexistent", &[1.0, 0.0], 10).unwrap();
        assert!(results.is_empty());
    }
}
