//! LLM Engine for AssistSupport
//! Embedded llama.cpp inference with Metal GPU acceleration

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::num::NonZeroU32;
use tokio::sync::mpsc;
use thiserror::Error;
use parking_lot::RwLock;

use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{AddBos, LlamaModel};
use llama_cpp_2::sampling::LlamaSampler;

#[derive(Debug, Error)]
pub enum LlmError {
    #[error("LLM backend initialization failed: {0}")]
    BackendInit(String),
    #[error("Model load failed: {0}")]
    ModelLoad(String),
    #[error("Context creation failed: {0}")]
    ContextCreate(String),
    #[error("Tokenization failed: {0}")]
    Tokenize(String),
    #[error("Generation failed: {0}")]
    Generate(String),
    #[error("No model loaded")]
    NoModel,
    #[error("Model file not found: {0}")]
    ModelNotFound(String),
    #[error("Invalid model format: {0}")]
    InvalidFormat(String),
    #[error("Generation cancelled")]
    Cancelled,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Model information
#[derive(Debug, Clone, serde::Serialize)]
pub struct ModelInfo {
    pub id: String,
    pub path: PathBuf,
    pub name: String,
    pub size_bytes: u64,
    pub n_params: u64,
    pub n_ctx_train: u32,
    pub n_embd: u32,
    pub n_vocab: u32,
}

/// Generation parameters
#[derive(Debug, Clone)]
pub struct GenerationParams {
    pub max_tokens: u32,
    pub temperature: f32,
    pub top_p: f32,
    pub top_k: i32,
    pub repeat_penalty: f32,
    pub stop_sequences: Vec<String>,
    pub context_window: Option<u32>,
    /// Minimum milliseconds between token emissions for streaming stability
    /// Set to 0 to disable throttling, default is 16ms (~60fps)
    pub stream_throttle_ms: u32,
    /// Minimum characters to buffer before emitting a token event
    /// Helps reduce event frequency for UI responsiveness
    pub stream_min_chars: usize,
}

impl Default for GenerationParams {
    fn default() -> Self {
        Self {
            max_tokens: 512,
            temperature: 0.7,
            top_p: 0.9,
            top_k: 40,
            repeat_penalty: 1.1,
            stop_sequences: vec![],
            context_window: None, // Will use model default or 4096
            stream_throttle_ms: 16, // ~60fps default throttle
            stream_min_chars: 4, // Buffer at least 4 chars before emitting
        }
    }
}

/// Streaming generation event
#[derive(Debug, Clone)]
pub enum GenerationEvent {
    Token(String),
    Done { tokens_generated: u32, duration_ms: u64 },
    Error(String),
}

/// LLM Engine state
pub struct LlmState {
    backend: LlamaBackend,
    model: Option<LlamaModel>,
    model_info: Option<ModelInfo>,
}

/// Thread-safe LLM Engine
pub struct LlmEngine {
    pub state: Arc<RwLock<Option<LlmState>>>,
}

impl LlmEngine {
    /// Create a new LLM engine
    pub fn new() -> Result<Self, LlmError> {
        // Initialize the llama backend
        let backend = LlamaBackend::init()
            .map_err(|e| LlmError::BackendInit(e.to_string()))?;

        let state = LlmState {
            backend,
            model: None,
            model_info: None,
        };

        Ok(Self {
            state: Arc::new(RwLock::new(Some(state))),
        })
    }

    /// Check if a model is loaded
    pub fn is_model_loaded(&self) -> bool {
        self.state.read()
            .as_ref()
            .map(|s| s.model.is_some())
            .unwrap_or(false)
    }

    /// Get current model info
    pub fn model_info(&self) -> Option<ModelInfo> {
        self.state.read()
            .as_ref()
            .and_then(|s| s.model_info.clone())
    }

    /// Load a model from file
    pub fn load_model(&self, path: &Path, n_gpu_layers: u32, model_id: String) -> Result<ModelInfo, LlmError> {
        if !path.exists() {
            return Err(LlmError::ModelNotFound(path.display().to_string()));
        }

        // Get file size
        let metadata = std::fs::metadata(path)?;
        let size_bytes = metadata.len();

        // Validate it's a GGUF file
        let file_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        if !file_name.to_lowercase().ends_with(".gguf") {
            return Err(LlmError::InvalidFormat(
                "Model must be a .gguf file".into()
            ));
        }

        let mut state_guard = self.state.write();
        let state = state_guard.as_mut().ok_or(LlmError::BackendInit("Backend not initialized".into()))?;

        // Unload existing model
        state.model = None;
        state.model_info = None;

        // Configure model parameters with GPU offloading
        let model_params = LlamaModelParams::default()
            .with_n_gpu_layers(n_gpu_layers);

        // Load the model
        let model = LlamaModel::load_from_file(&state.backend, path, &model_params)
            .map_err(|e| LlmError::ModelLoad(e.to_string()))?;

        // Extract model info
        let info = ModelInfo {
            id: if model_id.trim().is_empty() { file_name.to_string() } else { model_id },
            path: path.to_path_buf(),
            name: file_name.to_string(),
            size_bytes,
            n_params: 0, // Not easily available from llama-cpp-2
            n_ctx_train: model.n_ctx_train(),
            n_embd: model.n_embd() as u32,
            n_vocab: model.n_vocab() as u32,
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

    /// Generate text with streaming output
    pub async fn generate_streaming(
        &self,
        prompt: &str,
        params: GenerationParams,
        tx: mpsc::Sender<GenerationEvent>,
        cancel_flag: Arc<std::sync::atomic::AtomicBool>,
    ) -> Result<(), LlmError> {
        let start_time = std::time::Instant::now();

        // Clone what we need for the blocking operation
        let prompt = prompt.to_string();
        let state = self.state.clone();

        // Run generation in blocking thread
        let result = tokio::task::spawn_blocking(move || {
            Self::generate_blocking(&state, &prompt, params, tx.clone(), cancel_flag, start_time)
        }).await;

        match result {
            Ok(inner) => inner,
            Err(e) => Err(LlmError::Generate(format!("Task join error: {}", e))),
        }
    }

    fn generate_blocking(
        state: &Arc<RwLock<Option<LlmState>>>,
        prompt: &str,
        params: GenerationParams,
        tx: mpsc::Sender<GenerationEvent>,
        cancel_flag: Arc<std::sync::atomic::AtomicBool>,
        start_time: std::time::Instant,
    ) -> Result<(), LlmError> {
        use std::sync::atomic::Ordering;

        let state_guard = state.read();
        let state = state_guard.as_ref().ok_or(LlmError::NoModel)?;
        let model = state.model.as_ref().ok_or(LlmError::NoModel)?;

        // Create context with configurable window size
        // Use provided context_window, or model's training context (capped at 8192), or 4096 as fallback
        let n_ctx = params.context_window
            .unwrap_or_else(|| model.n_ctx_train().clamp(2048, 8192));
        let ctx_params = LlamaContextParams::default()
            .with_n_ctx(NonZeroU32::new(n_ctx));

        let mut ctx = model.new_context(&state.backend, ctx_params)
            .map_err(|e| LlmError::ContextCreate(e.to_string()))?;

        // Tokenize prompt
        let tokens = model.str_to_token(prompt, AddBos::Always)
            .map_err(|e| LlmError::Tokenize(e.to_string()))?;

        if tokens.is_empty() {
            return Err(LlmError::Tokenize("Empty tokenization result".into()));
        }

        // Validate prompt fits within context window
        let min_generation_tokens = 64;
        if tokens.len() + min_generation_tokens > n_ctx as usize {
            return Err(LlmError::Generate(format!(
                "Prompt too long: {} tokens exceeds context window of {} (need {} reserved for generation)",
                tokens.len(), n_ctx, min_generation_tokens
            )));
        }

        // Create batch with dynamic size based on prompt length
        // Use prompt tokens + expected max output as initial capacity
        // Minimum 512, scale up for longer prompts, cap at n_ctx
        let batch_size = (tokens.len() + params.max_tokens as usize)
            .max(512)
            .min(n_ctx as usize);
        let mut batch = LlamaBatch::new(batch_size, 1);

        for (i, token) in tokens.iter().enumerate() {
            let is_last = i == tokens.len() - 1;
            batch.add(*token, i as i32, &[0], is_last)
                .map_err(|e| LlmError::Generate(format!("Batch add error: {}", e)))?;
        }

        // Decode prompt
        ctx.decode(&mut batch)
            .map_err(|e| LlmError::Generate(format!("Decode error: {}", e)))?;

        // Set up sampler
        let mut sampler = LlamaSampler::chain_simple([
            LlamaSampler::temp(params.temperature),
            LlamaSampler::top_k(params.top_k),
            LlamaSampler::top_p(params.top_p, 1),
            LlamaSampler::penalties(
                64,                         // penalty_last_n
                params.repeat_penalty,      // repeat penalty
                0.0,                        // frequency penalty
                0.0,                        // presence penalty
            ),
            LlamaSampler::dist(1234),
        ]);

        let mut n_cur = tokens.len();
        let mut tokens_generated = 0u32;
        let mut word_buffer = String::new();
        let mut full_output = String::new(); // Track full output for stop sequence checking
        let max_stop_seq_len = params.stop_sequences.iter()
            .map(|s| s.len())
            .max()
            .unwrap_or(0);

        // Throttling state for streaming stability
        let throttle_duration = std::time::Duration::from_millis(params.stream_throttle_ms as u64);
        let min_chars = params.stream_min_chars;
        let mut last_emit_time = std::time::Instant::now();

        // Generation loop
        while n_cur < (tokens.len() + params.max_tokens as usize) {
            // Check cancellation
            if cancel_flag.load(Ordering::Relaxed) {
                let _ = tx.blocking_send(GenerationEvent::Error("Cancelled".into()));
                return Err(LlmError::Cancelled);
            }

            // Sample next token
            let token = sampler.sample(&ctx, -1);
            sampler.accept(token);

            // Check for EOS
            if model.is_eog_token(token) {
                break;
            }

            // Decode token to text
            let piece = model.token_to_str(token, llama_cpp_2::model::Special::Tokenize)
                .map_err(|e| LlmError::Generate(format!("Token decode error: {}", e)))?;

            // Buffer and send complete words
            word_buffer.push_str(&piece);
            full_output.push_str(&piece);

            // Check for stop sequences
            let mut hit_stop_sequence = false;
            if !params.stop_sequences.is_empty() {
                // Only check recent portion of output (optimization)
                let check_start = full_output.len().saturating_sub(max_stop_seq_len + piece.len());
                let check_region = &full_output[check_start..];

                for stop_seq in &params.stop_sequences {
                    if check_region.ends_with(stop_seq) {
                        // Remove stop sequence from buffers
                        let trim_len = stop_seq.len();
                        full_output.truncate(full_output.len() - trim_len);

                        // Also trim from word_buffer if it contains the stop sequence
                        if word_buffer.ends_with(stop_seq) {
                            word_buffer.truncate(word_buffer.len() - trim_len);
                        }

                        hit_stop_sequence = true;
                        break;
                    }
                }
            }

            // Emit buffered text with throttling for UI stability
            // Conditions to emit:
            // 1. Word boundary (whitespace) with minimum chars accumulated
            // 2. Buffer exceeds 20 chars (force emit)
            // 3. Enough time has passed since last emit (throttle)
            let time_since_emit = last_emit_time.elapsed();
            let should_emit = !word_buffer.is_empty() && (
                // Large buffer: force emit
                word_buffer.len() > 20 ||
                // Word boundary with minimum chars and throttle passed
                (piece.contains(char::is_whitespace) &&
                 word_buffer.len() >= min_chars &&
                 time_since_emit >= throttle_duration)
            );

            if should_emit {
                let _ = tx.blocking_send(GenerationEvent::Token(word_buffer.clone()));
                word_buffer.clear();
                last_emit_time = std::time::Instant::now();
            }

            // Exit if we hit a stop sequence
            if hit_stop_sequence {
                break;
            }

            tokens_generated += 1;

            // Prepare for next token
            batch.clear();
            batch.add(token, n_cur as i32, &[0], true)
                .map_err(|e| LlmError::Generate(format!("Batch add error: {}", e)))?;

            ctx.decode(&mut batch)
                .map_err(|e| LlmError::Generate(format!("Decode error: {}", e)))?;

            n_cur += 1;
        }

        // Send any remaining buffered text (no throttle on final emit)
        if !word_buffer.is_empty() {
            let _ = tx.blocking_send(GenerationEvent::Token(word_buffer));
        }

        // Send completion event
        let duration_ms = start_time.elapsed().as_millis() as u64;
        let _ = tx.blocking_send(GenerationEvent::Done {
            tokens_generated,
            duration_ms,
        });

        Ok(())
    }

    /// Simple non-streaming generation (for testing)
    pub async fn generate(&self, prompt: &str, params: GenerationParams) -> Result<String, LlmError> {
        let (tx, mut rx) = mpsc::channel(100);
        let cancel_flag = Arc::new(std::sync::atomic::AtomicBool::new(false));

        // Start generation
        let gen_handle = {
            let prompt = prompt.to_string();
            let state = self.state.clone();
            let tx = tx.clone();
            let cancel = cancel_flag.clone();

            tokio::spawn(async move {
                let engine = LlmEngine { state };
                engine.generate_streaming(&prompt, params, tx, cancel).await
            })
        };

        // Collect output
        let mut output = String::new();
        while let Some(event) = rx.recv().await {
            match event {
                GenerationEvent::Token(text) => output.push_str(&text),
                GenerationEvent::Done { .. } => break,
                GenerationEvent::Error(e) => return Err(LlmError::Generate(e)),
            }
        }

        gen_handle.await.map_err(|e| LlmError::Generate(e.to_string()))??;

        Ok(output)
    }
}

// NOTE: We intentionally do NOT implement Default for LlmEngine.
// Initialization can fail (missing Metal support, memory issues, etc.)
// and we want to handle that gracefully with Result, not panic.
// Callers should use LlmEngine::new() and handle the Result appropriately.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let engine = LlmEngine::new();
        assert!(engine.is_ok(), "Engine should initialize");

        let engine = engine.unwrap();
        assert!(!engine.is_model_loaded(), "No model should be loaded initially");
    }

    #[test]
    fn test_generation_params_default() {
        let params = GenerationParams::default();
        assert_eq!(params.max_tokens, 512);
        assert!((params.temperature - 0.7).abs() < 0.01);
    }

    #[test]
    #[ignore] // Requires model download - run with: cargo test test_model_load_and_generate -- --ignored
    fn test_model_load_and_generate() {
        let model_path = dirs::data_dir()
            .unwrap()
            .join("AssistSupport/models/tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf");

        if !model_path.exists() {
            println!("Model not found at {:?}, skipping test", model_path);
            return;
        }

        // Create engine
        let engine = LlmEngine::new().expect("Failed to create engine");
        assert!(!engine.is_model_loaded());

        // Load model with GPU offload
        let model_id = model_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("custom-model")
            .to_string();
        let info = engine.load_model(&model_path, 1000, model_id).expect("Failed to load model");
        println!("Loaded model: {:?}", info);
        assert!(engine.is_model_loaded());
        assert!(info.n_vocab > 0);

        // Test generation (blocking for test simplicity)
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async {
            engine.generate(
                "Hello, my name is",
                GenerationParams {
                    max_tokens: 20,
                    temperature: 0.7,
                    ..Default::default()
                }
            ).await
        });

        let output = result.expect("Generation failed");
        println!("Generated: {}", output);
        assert!(!output.is_empty(), "Output should not be empty");

        // Cleanup
        engine.unload_model();
        assert!(!engine.is_model_loaded());
    }
}
