//! Embedding Engine for AssistSupport
//! Generates vector embeddings using llama-cpp-2

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::num::NonZeroU32;
use parking_lot::RwLock;
use thiserror::Error;

use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{AddBos, LlamaModel};

#[derive(Debug, Error)]
pub enum EmbeddingError {
    #[error("Backend initialization failed: {0}")]
    BackendInit(String),
    #[error("Model load failed: {0}")]
    ModelLoad(String),
    #[error("Context creation failed: {0}")]
    ContextCreate(String),
    #[error("Embedding generation failed: {0}")]
    Generate(String),
    #[error("No embedding model loaded")]
    NoModel,
    #[error("Model file not found: {0}")]
    ModelNotFound(String),
    #[error("Model does not support embeddings")]
    NotEmbeddingModel,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Embedding model info
#[derive(Debug, Clone, serde::Serialize)]
pub struct EmbeddingModelInfo {
    pub path: PathBuf,
    pub name: String,
    pub embedding_dim: usize,
    pub size_bytes: u64,
}

/// Internal state for embedding engine
struct EmbeddingState {
    backend: LlamaBackend,
    model: Option<LlamaModel>,
    model_info: Option<EmbeddingModelInfo>,
}

/// Embedding engine for generating vector embeddings
pub struct EmbeddingEngine {
    state: Arc<RwLock<Option<EmbeddingState>>>,
}

impl EmbeddingEngine {
    /// Create a new embedding engine
    pub fn new() -> Result<Self, EmbeddingError> {
        let backend = LlamaBackend::init()
            .map_err(|e| EmbeddingError::BackendInit(e.to_string()))?;

        let state = EmbeddingState {
            backend,
            model: None,
            model_info: None,
        };

        Ok(Self {
            state: Arc::new(RwLock::new(Some(state))),
        })
    }

    /// Check if an embedding model is loaded
    pub fn is_model_loaded(&self) -> bool {
        self.state.read()
            .as_ref()
            .map(|s| s.model.is_some())
            .unwrap_or(false)
    }

    /// Get current model info
    pub fn model_info(&self) -> Option<EmbeddingModelInfo> {
        self.state.read()
            .as_ref()
            .and_then(|s| s.model_info.clone())
    }

    /// Get embedding dimension
    pub fn embedding_dim(&self) -> Option<usize> {
        self.model_info().map(|m| m.embedding_dim)
    }

    /// Load an embedding model
    pub fn load_model(&self, path: &Path, n_gpu_layers: u32) -> Result<EmbeddingModelInfo, EmbeddingError> {
        if !path.exists() {
            return Err(EmbeddingError::ModelNotFound(path.display().to_string()));
        }

        let metadata = std::fs::metadata(path)?;
        let size_bytes = metadata.len();

        let file_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        let mut state_guard = self.state.write();
        let state = state_guard.as_mut()
            .ok_or(EmbeddingError::BackendInit("Backend not initialized".into()))?;

        // Unload existing model
        state.model = None;
        state.model_info = None;

        // Configure model parameters
        let model_params = LlamaModelParams::default()
            .with_n_gpu_layers(n_gpu_layers);

        // Load model
        let model = LlamaModel::load_from_file(&state.backend, path, &model_params)
            .map_err(|e| EmbeddingError::ModelLoad(e.to_string()))?;

        // Get embedding dimension from model
        let embedding_dim = model.n_embd() as usize;

        let info = EmbeddingModelInfo {
            path: path.to_path_buf(),
            name: file_name.to_string(),
            embedding_dim,
            size_bytes,
        };

        state.model = Some(model);
        state.model_info = Some(info.clone());

        Ok(info)
    }

    /// Unload the current model
    pub fn unload_model(&self) {
        if let Some(state) = self.state.write().as_mut() {
            state.model = None;
            state.model_info = None;
        }
    }

    /// Generate embedding for a single text
    pub fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        let embeddings = self.embed_batch(&[text.to_string()])?;
        embeddings.into_iter().next()
            .ok_or(EmbeddingError::Generate("No embedding generated".into()))
    }

    /// Generate embeddings for a batch of texts
    pub fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        if texts.is_empty() {
            return Ok(vec![]);
        }

        let state_guard = self.state.read();
        let state = state_guard.as_ref().ok_or(EmbeddingError::NoModel)?;
        let model = state.model.as_ref().ok_or(EmbeddingError::NoModel)?;

        // Create context for embeddings
        let ctx_params = LlamaContextParams::default()
            .with_n_ctx(NonZeroU32::new(512))
            .with_embeddings(true);

        let mut ctx = model.new_context(&state.backend, ctx_params)
            .map_err(|e| EmbeddingError::ContextCreate(e.to_string()))?;

        let embedding_dim = model.n_embd() as usize;
        let mut all_embeddings = Vec::with_capacity(texts.len());

        for text in texts {
            // Tokenize
            let tokens = model.str_to_token(text, AddBos::Always)
                .map_err(|e| EmbeddingError::Generate(format!("Tokenization failed: {}", e)))?;

            if tokens.is_empty() {
                // Return zero vector for empty text
                all_embeddings.push(vec![0.0; embedding_dim]);
                continue;
            }

            // Create batch
            let mut batch = LlamaBatch::new(512, 1);

            for (i, token) in tokens.iter().enumerate() {
                batch.add(*token, i as i32, &[0], i == tokens.len() - 1)
                    .map_err(|e| EmbeddingError::Generate(format!("Batch add error: {}", e)))?;
            }

            // Decode to generate embeddings
            ctx.decode(&mut batch)
                .map_err(|e| EmbeddingError::Generate(format!("Decode error: {}", e)))?;

            // Get embeddings - use sequence embeddings (averaged)
            let embeddings = ctx.embeddings_seq_ith(0)
                .map_err(|e| EmbeddingError::Generate(format!("Get embeddings error: {}", e)))?;

            // Normalize the embedding
            let normalized = Self::normalize_embedding(embeddings);
            all_embeddings.push(normalized);

            // Clear batch for next text
            batch.clear();
        }

        Ok(all_embeddings)
    }

    /// Normalize embedding to unit length
    fn normalize_embedding(embedding: &[f32]) -> Vec<f32> {
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            embedding.iter().map(|x| x / norm).collect()
        } else {
            embedding.to_vec()
        }
    }

    /// Compute cosine similarity between two embeddings
    pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }

        let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a > 0.0 && norm_b > 0.0 {
            dot / (norm_a * norm_b)
        } else {
            0.0
        }
    }
}

impl Default for EmbeddingEngine {
    fn default() -> Self {
        Self::new().expect("Failed to initialize embedding engine")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let engine = EmbeddingEngine::new();
        assert!(engine.is_ok());

        let engine = engine.unwrap();
        assert!(!engine.is_model_loaded());
    }

    #[test]
    fn test_normalize_embedding() {
        let embedding = vec![3.0, 4.0];
        let normalized = EmbeddingEngine::normalize_embedding(&embedding);

        // 3-4-5 triangle, normalized should be 0.6, 0.8
        assert!((normalized[0] - 0.6).abs() < 0.001);
        assert!((normalized[1] - 0.8).abs() < 0.001);

        // Check unit length
        let length: f32 = normalized.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((length - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0];
        let b = vec![1.0, 0.0];
        assert!((EmbeddingEngine::cosine_similarity(&a, &b) - 1.0).abs() < 0.001);

        let c = vec![0.0, 1.0];
        assert!(EmbeddingEngine::cosine_similarity(&a, &c).abs() < 0.001);

        let d = vec![-1.0, 0.0];
        assert!((EmbeddingEngine::cosine_similarity(&a, &d) + 1.0).abs() < 0.001);
    }
}
