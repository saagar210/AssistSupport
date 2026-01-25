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

    /// Create the schema for chunks table
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
        ]))
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
        } else {
            // Create with initial empty data using from_iter_primitive
            let schema = self.create_schema();

            // Create empty arrays with proper types
            let id_array = StringArray::from(Vec::<String>::new());

            // Create an empty FixedSizeListArray using from_iter_primitive
            let vector_array = FixedSizeListArray::from_iter_primitive::<Float32Type, _, _>(
                std::iter::empty::<Option<Vec<Option<f32>>>>(),
                self.config.embedding_dim as i32,
            );

            let batch = RecordBatch::try_new(
                schema.clone(),
                vec![Arc::new(id_array), Arc::new(vector_array)],
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

    /// Insert embeddings into the vector store
    pub async fn insert_embeddings(
        &self,
        ids: &[String],
        embeddings: &[Vec<f32>],
    ) -> Result<(), VectorError> {
        if !self.enabled {
            return Err(VectorError::Disabled);
        }

        let table = self.table.as_ref().ok_or(VectorError::NotInitialized)?;

        if ids.len() != embeddings.len() {
            return Err(VectorError::LanceDb(
                "IDs and embeddings count mismatch".into(),
            ));
        }

        if ids.is_empty() {
            return Ok(());
        }

        // Build arrays
        let id_array = StringArray::from(ids.to_vec());

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
            vec![Arc::new(id_array), Arc::new(vector_array)],
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

    /// Search for similar vectors
    pub async fn search_similar(
        &self,
        query_embedding: &[f32],
        limit: usize,
    ) -> Result<Vec<VectorSearchResult>, VectorError> {
        if !self.enabled {
            return Err(VectorError::Disabled);
        }

        let table = self.table.as_ref().ok_or(VectorError::NotInitialized)?;

        let query = table
            .vector_search(query_embedding)
            .map_err(|e| VectorError::LanceDb(e.to_string()))?;

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

            for i in 0..batch.num_rows() {
                let id = ids.value(i).to_string();
                search_results.push(VectorSearchResult {
                    chunk_id: id,
                    distance: distances.value(i),
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

        // Build filter expression for deletion
        let quoted_ids: Vec<String> = ids.iter().map(|id| format!("'{}'", id)).collect();
        let filter = format!("id IN ({})", quoted_ids.join(", "));

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
}
