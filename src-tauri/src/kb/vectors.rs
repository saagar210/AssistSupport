//! Vector storage module for AssistSupport
//! LanceDB-based vector search with encryption awareness

use std::path::{Path, PathBuf};
use std::sync::Arc;
use thiserror::Error;

use arrow_array::{Float32Array, RecordBatch, RecordBatchIterator, StringArray, FixedSizeListArray};
use arrow_array::types::Float32Type;
use arrow_schema::{DataType, Field, Schema};
use lancedb::query::{ExecutableQuery, QueryBase};
use lancedb::{connect, Connection, Table};

/// Sanitize a string value for use in LanceDB filter expressions.
/// This prevents filter injection attacks by escaping/rejecting malicious input.
///
/// # Security
/// Escapes single quotes and validates that the input doesn't contain
/// SQL injection patterns. Returns None if the input appears malicious.
fn sanitize_filter_value(value: &str) -> Option<String> {
    // Reject obviously malicious patterns
    let lower = value.to_lowercase();
    let suspicious_patterns = [
        "' or ", "' and ", "';", "'--", "/*", "*/",
        "union", "select", "insert", "update", "delete",
        "drop", "truncate", "exec", "execute",
    ];

    for pattern in &suspicious_patterns {
        if lower.contains(pattern) {
            return None;
        }
    }

    // Escape single quotes by doubling them
    Some(value.replace('\'', "''"))
}

/// Sanitize a chunk/document ID for use in filter expressions.
/// IDs should only contain alphanumeric characters, hyphens, and underscores.
fn sanitize_id(id: &str) -> Option<String> {
    // UUIDs and similar IDs should only have alphanumeric, hyphens, underscores
    if id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
        Some(id.to_string())
    } else {
        // Fallback: escape the value
        sanitize_filter_value(id)
    }
}

#[derive(Debug, Error)]
pub enum VectorError {
    #[error("LanceDB error: {0}")]
    LanceDb(String),
    #[error("Vector store not initialized")]
    NotInitialized,
    #[error("Encryption not supported - user consent required")]
    EncryptionNotSupported,
    #[error("Vector search disabled")]
    Disabled,
    #[error("Table not found: {0}")]
    TableNotFound(String),
    #[error("Arrow error: {0}")]
    Arrow(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Vector store configuration
#[derive(Debug, Clone)]
pub struct VectorStoreConfig {
    pub path: PathBuf,
    pub embedding_dim: usize,
    pub encryption_enabled: bool,
}

impl Default for VectorStoreConfig {
    fn default() -> Self {
        Self {
            path: PathBuf::from("./vectors"),
            embedding_dim: 768, // nomic-embed-text default
            encryption_enabled: false,
        }
    }
}

/// Vector search result
#[derive(Debug, Clone)]
pub struct VectorSearchResult {
    pub chunk_id: String,
    pub distance: f32,
    pub namespace_id: Option<String>,
    pub document_id: Option<String>,
}

/// Metadata for vector insertion
#[derive(Debug, Clone, Default)]
pub struct VectorMetadata {
    pub namespace_id: String,
    pub document_id: String,
}

/// Vector store manager
pub struct VectorStore {
    config: VectorStoreConfig,
    connection: Option<Connection>,
    table: Option<Table>,
    enabled: bool,
    encryption_supported: bool,
}

impl VectorStore {
    /// Create a new vector store (does not initialize until `init` is called)
    pub fn new(config: VectorStoreConfig) -> Self {
        Self {
            config,
            connection: None,
            table: None,
            enabled: false,
            encryption_supported: false,
        }
    }

    /// Check if LanceDB supports encryption
    ///
    /// As of LanceDB 0.17, native encryption is not yet supported.
    /// This function documents the status and returns false.
    pub fn check_encryption_support() -> bool {
        // LanceDB 0.17 does not support native encryption
        false
    }

    /// Get encryption support status
    pub fn encryption_supported(&self) -> bool {
        self.encryption_supported
    }

    /// Check if vector store is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Enable vector store (requires explicit consent if unencrypted)
    pub fn enable(&mut self, user_consented: bool) -> Result<(), VectorError> {
        if !self.encryption_supported && !user_consented {
            return Err(VectorError::EncryptionNotSupported);
        }
        self.enabled = true;
        Ok(())
    }

    /// Disable vector store
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Initialize the vector store
    pub async fn init(&mut self) -> Result<(), VectorError> {
        self.encryption_supported = Self::check_encryption_support();

        // Create directory if needed
        std::fs::create_dir_all(&self.config.path)?;

        // Connect to LanceDB
        let db_path = self.config.path.to_string_lossy().to_string();
        let conn = connect(&db_path)
            .execute()
            .await
            .map_err(|e| VectorError::LanceDb(e.to_string()))?;

        self.connection = Some(conn);

        // Vector store starts disabled by default
        self.enabled = false;

        Ok(())
    }

    /// Create the schema for chunks table (v2 with namespace and document_id)
    fn create_schema(&self) -> Arc<Schema> {
        Arc::new(Schema::new(vec![
            Field::new("id", DataType::Utf8, false),
            Field::new(
                "vector",
                DataType::FixedSizeList(
                    Arc::new(Field::new("item", DataType::Float32, true)),
                    self.config.embedding_dim as i32,
                ),
                false,
            ),
            Field::new("namespace_id", DataType::Utf8, false),
            Field::new("document_id", DataType::Utf8, false),
        ]))
    }

    /// Create the legacy schema (v1 without namespace/document_id) for migration detection
    #[allow(dead_code)]
    fn create_legacy_schema(&self) -> Arc<Schema> {
        Arc::new(Schema::new(vec![
            Field::new("id", DataType::Utf8, false),
            Field::new(
                "vector",
                DataType::FixedSizeList(
                    Arc::new(Field::new("item", DataType::Float32, true)),
                    self.config.embedding_dim as i32,
                ),
                false,
            ),
        ]))
    }

    /// Check if the table has the new schema with namespace_id
    async fn table_has_namespace(&self) -> Result<bool, VectorError> {
        let table = self.table.as_ref().ok_or(VectorError::NotInitialized)?;
        let schema = table.schema().await.map_err(|e| VectorError::LanceDb(e.to_string()))?;
        Ok(schema.field_with_name("namespace_id").is_ok())
    }

    /// Create or open the chunks table
    pub async fn create_table(&mut self) -> Result<(), VectorError> {
        let conn = self.connection.as_ref().ok_or(VectorError::NotInitialized)?;

        // Check if table exists
        let table_names = conn
            .table_names()
            .execute()
            .await
            .map_err(|e| VectorError::LanceDb(e.to_string()))?;

        if table_names.contains(&"chunks".to_string()) {
            // Open existing table
            let table = conn
                .open_table("chunks")
                .execute()
                .await
                .map_err(|e| VectorError::LanceDb(e.to_string()))?;
            self.table = Some(table);

            // Check if migration is needed (table doesn't have namespace_id)
            if !self.table_has_namespace().await? {
                // For now, we'll just log a warning. In production, you might want to
                // migrate the data or recreate the table.
                tracing::warn!("Vector table is using legacy schema without namespace_id. Consider rebuilding vectors.");
            }
        } else {
            // Create with initial empty data using the new schema
            let schema = self.create_schema();

            // Create empty arrays with proper types
            let id_array = StringArray::from(Vec::<String>::new());
            let namespace_array = StringArray::from(Vec::<String>::new());
            let document_array = StringArray::from(Vec::<String>::new());

            // Create an empty FixedSizeListArray using from_iter_primitive
            let vector_array = FixedSizeListArray::from_iter_primitive::<Float32Type, _, _>(
                std::iter::empty::<Option<Vec<Option<f32>>>>(),
                self.config.embedding_dim as i32,
            );

            let batch = RecordBatch::try_new(
                schema.clone(),
                vec![
                    Arc::new(id_array),
                    Arc::new(vector_array),
                    Arc::new(namespace_array),
                    Arc::new(document_array),
                ],
            )
            .map_err(|e| VectorError::Arrow(e.to_string()))?;

            let batches = RecordBatchIterator::new(vec![Ok(batch)], schema);

            let table = conn
                .create_table("chunks", Box::new(batches))
                .execute()
                .await
                .map_err(|e| VectorError::LanceDb(e.to_string()))?;

            self.table = Some(table);
        }

        Ok(())
    }

    /// Insert embeddings into the vector store with metadata
    pub async fn insert_embeddings_with_metadata(
        &self,
        ids: &[String],
        embeddings: &[Vec<f32>],
        metadata: &[VectorMetadata],
    ) -> Result<(), VectorError> {
        if !self.enabled {
            return Err(VectorError::Disabled);
        }

        let table = self.table.as_ref().ok_or(VectorError::NotInitialized)?;

        if ids.len() != embeddings.len() || ids.len() != metadata.len() {
            return Err(VectorError::LanceDb(
                "IDs, embeddings, and metadata count mismatch".into(),
            ));
        }

        if ids.is_empty() {
            return Ok(());
        }

        // Build arrays
        let id_array = StringArray::from(ids.to_vec());
        let namespace_array = StringArray::from(
            metadata.iter().map(|m| m.namespace_id.clone()).collect::<Vec<_>>()
        );
        let document_array = StringArray::from(
            metadata.iter().map(|m| m.document_id.clone()).collect::<Vec<_>>()
        );

        // Create FixedSizeListArray from embeddings
        let embedding_dim = self.config.embedding_dim as i32;
        let vector_iter = embeddings.iter().map(|emb| {
            Some(emb.iter().map(|&v| Some(v)).collect::<Vec<_>>())
        });
        let vector_array = FixedSizeListArray::from_iter_primitive::<Float32Type, _, _>(
            vector_iter,
            embedding_dim,
        );

        let schema = self.create_schema();

        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(id_array),
                Arc::new(vector_array),
                Arc::new(namespace_array),
                Arc::new(document_array),
            ],
        )
        .map_err(|e| VectorError::Arrow(e.to_string()))?;

        let batches = RecordBatchIterator::new(vec![Ok(batch)], schema);

        table
            .add(Box::new(batches))
            .execute()
            .await
            .map_err(|e| VectorError::LanceDb(e.to_string()))?;

        Ok(())
    }

    /// Insert embeddings into the vector store (legacy method without metadata)
    pub async fn insert_embeddings(
        &self,
        ids: &[String],
        embeddings: &[Vec<f32>],
    ) -> Result<(), VectorError> {
        // Create default metadata for backward compatibility
        let metadata: Vec<VectorMetadata> = ids.iter().map(|_| VectorMetadata {
            namespace_id: "default".to_string(),
            document_id: String::new(),
        }).collect();

        self.insert_embeddings_with_metadata(ids, embeddings, &metadata).await
    }

    /// Search for similar vectors
    pub async fn search_similar(
        &self,
        query_embedding: &[f32],
        limit: usize,
    ) -> Result<Vec<VectorSearchResult>, VectorError> {
        self.search_similar_in_namespace(query_embedding, None, limit).await
    }

    /// Search for similar vectors within a specific namespace
    pub async fn search_similar_in_namespace(
        &self,
        query_embedding: &[f32],
        namespace_id: Option<&str>,
        limit: usize,
    ) -> Result<Vec<VectorSearchResult>, VectorError> {
        if !self.enabled {
            return Err(VectorError::Disabled);
        }

        let table = self.table.as_ref().ok_or(VectorError::NotInitialized)?;

        let mut query = table
            .vector_search(query_embedding)
            .map_err(|e| VectorError::LanceDb(e.to_string()))?;

        // Apply namespace filter if specified (with injection protection)
        if let Some(ns) = namespace_id {
            let safe_ns = sanitize_filter_value(ns)
                .ok_or_else(|| VectorError::LanceDb("Invalid namespace ID".into()))?;
            query = query.only_if(format!("namespace_id = '{}'", safe_ns));
        }

        let results = query
            .limit(limit)
            .execute()
            .await
            .map_err(|e: lancedb::Error| VectorError::LanceDb(e.to_string()))?;

        let mut search_results = Vec::new();

        // Convert results to our format
        use futures::StreamExt;
        let batches: Vec<Result<RecordBatch, lancedb::Error>> = results.collect().await;

        for batch_result in batches {
            let batch = batch_result.map_err(|e: lancedb::Error| VectorError::LanceDb(e.to_string()))?;

            let id_col = batch
                .column_by_name("id")
                .ok_or_else(|| VectorError::LanceDb("Missing id column".into()))?;

            let distance_col = batch
                .column_by_name("_distance")
                .ok_or_else(|| VectorError::LanceDb("Missing _distance column".into()))?;

            let ids = id_col
                .as_any()
                .downcast_ref::<StringArray>()
                .ok_or_else(|| VectorError::Arrow("Invalid id column type".into()))?;

            let distances = distance_col
                .as_any()
                .downcast_ref::<Float32Array>()
                .ok_or_else(|| VectorError::Arrow("Invalid distance column type".into()))?;

            // Try to get namespace and document columns (may not exist in legacy tables)
            let namespace_col = batch.column_by_name("namespace_id");
            let document_col = batch.column_by_name("document_id");

            let namespaces = namespace_col.and_then(|c| c.as_any().downcast_ref::<StringArray>());
            let documents = document_col.and_then(|c| c.as_any().downcast_ref::<StringArray>());

            for i in 0..batch.num_rows() {
                let id = ids.value(i).to_string();
                search_results.push(VectorSearchResult {
                    chunk_id: id,
                    distance: distances.value(i),
                    namespace_id: namespaces.map(|n| n.value(i).to_string()),
                    document_id: documents.map(|d| d.value(i).to_string()),
                });
            }
        }

        Ok(search_results)
    }

    /// Delete vectors by chunk IDs
    pub async fn delete_by_ids(&self, ids: &[String]) -> Result<(), VectorError> {
        if !self.enabled {
            return Err(VectorError::Disabled);
        }

        let table = self.table.as_ref().ok_or(VectorError::NotInitialized)?;

        if ids.is_empty() {
            return Ok(());
        }

        // Build filter expression for deletion (with injection protection)
        let quoted_ids: Vec<String> = ids
            .iter()
            .filter_map(|id| sanitize_id(id).map(|safe_id| format!("'{}'", safe_id)))
            .collect();

        if quoted_ids.is_empty() {
            return Ok(());
        }

        let filter = format!("id IN ({})", quoted_ids.join(", "));

        table
            .delete(&filter)
            .await
            .map_err(|e| VectorError::LanceDb(e.to_string()))?;

        Ok(())
    }

    /// Delete all vectors for a specific document
    pub async fn delete_by_document(&self, document_id: &str) -> Result<(), VectorError> {
        if !self.enabled {
            return Err(VectorError::Disabled);
        }

        let table = self.table.as_ref().ok_or(VectorError::NotInitialized)?;

        // Sanitize document_id to prevent injection
        let safe_doc_id = sanitize_id(document_id)
            .ok_or_else(|| VectorError::LanceDb("Invalid document ID".into()))?;
        let filter = format!("document_id = '{}'", safe_doc_id);

        table
            .delete(&filter)
            .await
            .map_err(|e| VectorError::LanceDb(e.to_string()))?;

        Ok(())
    }

    /// Delete all vectors for a specific namespace
    pub async fn delete_by_namespace(&self, namespace_id: &str) -> Result<(), VectorError> {
        if !self.enabled {
            return Err(VectorError::Disabled);
        }

        let table = self.table.as_ref().ok_or(VectorError::NotInitialized)?;

        // Sanitize namespace_id to prevent injection
        let safe_ns_id = sanitize_filter_value(namespace_id)
            .ok_or_else(|| VectorError::LanceDb("Invalid namespace ID".into()))?;
        let filter = format!("namespace_id = '{}'", safe_ns_id);

        table
            .delete(&filter)
            .await
            .map_err(|e| VectorError::LanceDb(e.to_string()))?;

        Ok(())
    }

    /// Get the vector store path
    pub fn path(&self) -> &Path {
        &self.config.path
    }

    /// Get embedding dimension
    pub fn embedding_dim(&self) -> usize {
        self.config.embedding_dim
    }

    /// Get count of vectors in the store
    pub async fn count(&self) -> Result<usize, VectorError> {
        let table = self.table.as_ref().ok_or(VectorError::NotInitialized)?;

        let count = table
            .count_rows(None)
            .await
            .map_err(|e| VectorError::LanceDb(e.to_string()))?;

        Ok(count)
    }
}

/// Information about LanceDB encryption status
#[derive(Debug, Clone, serde::Serialize)]
pub struct EncryptionStatus {
    pub supported: bool,
    pub reason: String,
    pub recommendation: String,
}

impl EncryptionStatus {
    pub fn current() -> Self {
        Self {
            supported: VectorStore::check_encryption_support(),
            reason: "LanceDB 0.17 does not yet support native encryption for data at rest".into(),
            recommendation: "Vector search stores embeddings unencrypted. Enable only if you understand the security implications.".into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_not_supported() {
        // LanceDB 0.17 does not support encryption
        assert!(!VectorStore::check_encryption_support());
    }

    #[test]
    fn test_vector_store_disabled_by_default() {
        let config = VectorStoreConfig::default();
        let store = VectorStore::new(config);
        assert!(!store.is_enabled());
    }

    #[test]
    fn test_enable_requires_consent() {
        let config = VectorStoreConfig::default();
        let mut store = VectorStore::new(config);

        // Should fail without consent
        let result = store.enable(false);
        assert!(result.is_err());

        // Should succeed with consent
        let result = store.enable(true);
        assert!(result.is_ok());
        assert!(store.is_enabled());
    }

    #[test]
    fn test_encryption_status() {
        let status = EncryptionStatus::current();
        assert!(!status.supported);
        assert!(!status.reason.is_empty());
        assert!(!status.recommendation.is_empty());
    }

    #[test]
    fn test_sanitize_filter_value_valid() {
        // Normal values should pass
        assert_eq!(sanitize_filter_value("my-namespace"), Some("my-namespace".to_string()));
        assert_eq!(sanitize_filter_value("default"), Some("default".to_string()));
        assert_eq!(sanitize_filter_value("namespace-123"), Some("namespace-123".to_string()));
    }

    #[test]
    fn test_sanitize_filter_value_escapes_quotes() {
        // Single quotes should be escaped
        assert_eq!(sanitize_filter_value("it's"), Some("it''s".to_string()));
        assert_eq!(sanitize_filter_value("test'value"), Some("test''value".to_string()));
    }

    #[test]
    fn test_sanitize_filter_value_blocks_injection() {
        // SQL injection attempts should be rejected
        assert_eq!(sanitize_filter_value("' OR 1=1 --"), None);
        assert_eq!(sanitize_filter_value("'; DROP TABLE"), None);
        assert_eq!(sanitize_filter_value("test' AND 1=1"), None);
        assert_eq!(sanitize_filter_value("union select"), None);
        assert_eq!(sanitize_filter_value("/* comment */"), None);
    }

    #[test]
    fn test_sanitize_id_valid() {
        // UUIDs and simple IDs should pass
        assert_eq!(sanitize_id("550e8400-e29b-41d4-a716-446655440000"), Some("550e8400-e29b-41d4-a716-446655440000".to_string()));
        assert_eq!(sanitize_id("chunk_123"), Some("chunk_123".to_string()));
        assert_eq!(sanitize_id("abc123"), Some("abc123".to_string()));
    }

    #[test]
    fn test_sanitize_id_with_special_chars() {
        // IDs with special chars should be sanitized
        let result = sanitize_id("test'id");
        assert!(result.is_some());
        assert!(result.unwrap().contains("''"));
    }

    #[test]
    fn test_sanitize_id_blocks_injection() {
        // Injection attempts in IDs should be rejected
        assert_eq!(sanitize_id("'; DROP TABLE --"), None);
    }
}
