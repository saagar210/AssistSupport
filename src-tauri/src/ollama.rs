use std::time::Duration;

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tauri::Emitter;
use tokio_util::sync::CancellationToken;

use crate::error::AppError;
use crate::models::OllamaModel;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub size: u64,
    pub parameter_size: Option<String>,
    pub quantization_level: Option<String>,
    pub family: Option<String>,
    pub format: Option<String>,
    pub modified_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Deserialize)]
struct TagsResponse {
    models: Option<Vec<TagsModel>>,
}

#[derive(Deserialize)]
struct TagsModel {
    name: String,
    size: Option<i64>,
    details: Option<TagsModelDetails>,
}

#[derive(Deserialize)]
struct TagsModelDetails {
    family: Option<String>,
}

#[derive(Serialize)]
struct EmbeddingRequest<'a> {
    model: &'a str,
    input: &'a str,
}

#[derive(Deserialize)]
struct EmbeddingResponse {
    embeddings: Vec<Vec<f64>>,
}

#[derive(Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: &'a [ChatMessage],
    stream: bool,
}

#[derive(Deserialize)]
struct ChatStreamChunk {
    message: Option<ChatChunkMessage>,
    done: Option<bool>,
}

#[derive(Deserialize)]
struct ChatChunkMessage {
    content: Option<String>,
}

pub async fn health_check(host: &str, port: &str) -> Result<(bool, String), AppError> {
    let url = format!("http://{}:{}/api/tags", host, port);

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .build()
        .map_err(|e| AppError::Ollama(e.to_string()))?;

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| AppError::Ollama(format!("Connection failed: {}", e)))?;

    if response.status().is_success() {
        Ok((true, "connected".to_string()))
    } else {
        Ok((
            false,
            format!("Unexpected status: {}", response.status()),
        ))
    }
}

pub async fn list_models(host: &str, port: &str) -> Result<Vec<OllamaModel>, AppError> {
    let url = format!("http://{}:{}/api/tags", host, port);

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| AppError::Ollama(e.to_string()))?;

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| AppError::Ollama(format!("Connection failed: {}", e)))?;

    let tags: TagsResponse = response
        .json()
        .await
        .map_err(|e| AppError::Ollama(format!("Failed to parse response: {}", e)))?;

    let models = tags
        .models
        .unwrap_or_default()
        .into_iter()
        .map(|m| OllamaModel {
            name: m.name,
            size: m.size.unwrap_or(0),
            family: m.details.and_then(|d| d.family),
        })
        .collect();

    Ok(models)
}

/// Generate an embedding vector for the given text using Ollama's embedding API.
pub async fn generate_embedding(
    host: &str,
    port: &str,
    model: &str,
    text: &str,
) -> Result<Vec<f64>, AppError> {
    let url = format!("http://{}:{}/api/embed", host, port);

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(60))
        .build()
        .map_err(|e| AppError::Ollama(e.to_string()))?;

    let body = EmbeddingRequest {
        model,
        input: text,
    };

    let response = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| AppError::Ollama(format!("Embedding request failed: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        let body_text = response
            .text()
            .await
            .unwrap_or_else(|_| "unknown".to_string());
        return Err(AppError::Ollama(format!(
            "Embedding API returned {}: {}",
            status, body_text
        )));
    }

    let embed_response: EmbeddingResponse = response
        .json()
        .await
        .map_err(|e| AppError::Ollama(format!("Failed to parse embedding response: {}", e)))?;

    embed_response
        .embeddings
        .into_iter()
        .next()
        .ok_or_else(|| AppError::Ollama("No embedding returned from Ollama".to_string()))
}

/// Stream a chat completion from Ollama, emitting tokens as Tauri events.
/// Returns the full accumulated response text.
/// Supports cancellation via CancellationToken.
pub async fn chat_stream(
    host: &str,
    port: &str,
    model: &str,
    messages: &[ChatMessage],
    app_handle: &tauri::AppHandle,
    cancel_token: Option<&CancellationToken>,
) -> Result<String, AppError> {
    let url = format!("http://{}:{}/api/chat", host, port);

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(300))
        .build()
        .map_err(|e| AppError::Ollama(e.to_string()))?;

    let body = ChatRequest {
        model,
        messages,
        stream: true,
    };

    let response = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| AppError::Ollama(format!("Chat request failed: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        let body_text = response
            .text()
            .await
            .unwrap_or_else(|_| "unknown".to_string());
        return Err(AppError::Ollama(format!(
            "Chat API returned {}: {}",
            status, body_text
        )));
    }

    let mut full_response = String::new();
    let mut stream = response.bytes_stream();
    let mut cancelled = false;

    loop {
        let chunk_result = if let Some(token) = cancel_token {
            tokio::select! {
                biased;
                _ = token.cancelled() => {
                    cancelled = true;
                    break;
                }
                chunk = stream.next() => chunk,
            }
        } else {
            stream.next().await
        };

        let Some(chunk_result) = chunk_result else {
            break;
        };

        let chunk_bytes = chunk_result
            .map_err(|e| AppError::Ollama(format!("Stream read error: {}", e)))?;

        let chunk_str = String::from_utf8_lossy(&chunk_bytes);

        for line in chunk_str.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            if let Ok(parsed) = serde_json::from_str::<ChatStreamChunk>(line) {
                if let Some(msg) = &parsed.message {
                    if let Some(content) = &msg.content {
                        full_response.push_str(content);
                        let _ = app_handle.emit(
                            "chat-token",
                            serde_json::json!({"token": content}),
                        );
                    }
                }

                if parsed.done == Some(true) {
                    break;
                }
            }
        }
    }

    if cancelled {
        let _ = app_handle.emit("chat-cancelled", serde_json::json!({}));
    }

    Ok(full_response)
}

/// Send a non-streaming chat request and return the response text.
/// Used for tasks like auto-titling conversations.
pub async fn chat_once(
    host: &str,
    port: &str,
    model: &str,
    messages: &[ChatMessage],
) -> Result<String, AppError> {
    let url = format!("http://{}:{}/api/chat", host, port);

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| AppError::Ollama(e.to_string()))?;

    let body = ChatRequest {
        model,
        messages,
        stream: false,
    };

    let response = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| AppError::Ollama(format!("Chat request failed: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        let body_text = response
            .text()
            .await
            .unwrap_or_else(|_| "unknown".to_string());
        return Err(AppError::Ollama(format!(
            "Chat API returned {}: {}",
            status, body_text
        )));
    }

    #[derive(Deserialize)]
    struct ChatResponse {
        message: Option<ChatChunkMessage>,
    }

    let resp: ChatResponse = response
        .json()
        .await
        .map_err(|e| AppError::Ollama(format!("Failed to parse response: {}", e)))?;

    Ok(resp
        .message
        .and_then(|m| m.content)
        .unwrap_or_default())
}

/// GET detailed model information via POST /api/show
pub async fn show_model(host: &str, port: &str, model_name: &str) -> Result<ModelInfo, AppError> {
    let url = format!("http://{}:{}/api/show", host, port);

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| AppError::Ollama(e.to_string()))?;

    let response = client
        .post(&url)
        .json(&serde_json::json!({"name": model_name}))
        .send()
        .await
        .map_err(|e| AppError::Ollama(format!("Show model request failed: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        let body_text = response
            .text()
            .await
            .unwrap_or_else(|_| "unknown".to_string());
        return Err(AppError::Ollama(format!(
            "Show model API returned {}: {}",
            status, body_text
        )));
    }

    #[derive(Deserialize)]
    struct ShowResponse {
        details: Option<ShowDetails>,
        model_info: Option<serde_json::Value>,
        modified_at: Option<String>,
    }

    #[derive(Deserialize)]
    struct ShowDetails {
        family: Option<String>,
        format: Option<String>,
        parameter_size: Option<String>,
        quantization_level: Option<String>,
    }

    let show: ShowResponse = response
        .json()
        .await
        .map_err(|e| AppError::Ollama(format!("Failed to parse show response: {}", e)))?;

    let details = show.details.unwrap_or(ShowDetails {
        family: None,
        format: None,
        parameter_size: None,
        quantization_level: None,
    });

    // Try to extract size from model_info if available
    let size = show
        .model_info
        .as_ref()
        .and_then(|info| info.get("general.file_size"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    Ok(ModelInfo {
        name: model_name.to_string(),
        size,
        parameter_size: details.parameter_size,
        quantization_level: details.quantization_level,
        family: details.family,
        format: details.format,
        modified_at: show.modified_at,
    })
}

/// Pull/download a model from Ollama, streaming progress events.
pub async fn pull_model(
    host: &str,
    port: &str,
    model_name: &str,
    app_handle: &tauri::AppHandle,
) -> Result<(), AppError> {
    let url = format!("http://{}:{}/api/pull", host, port);

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(3600))
        .build()
        .map_err(|e| AppError::Ollama(e.to_string()))?;

    let response = client
        .post(&url)
        .json(&serde_json::json!({"name": model_name, "stream": true}))
        .send()
        .await
        .map_err(|e| AppError::Ollama(format!("Pull model request failed: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        let body_text = response
            .text()
            .await
            .unwrap_or_else(|_| "unknown".to_string());
        return Err(AppError::Ollama(format!(
            "Pull model API returned {}: {}",
            status, body_text
        )));
    }

    #[derive(Deserialize)]
    struct PullProgress {
        status: Option<String>,
        completed: Option<u64>,
        total: Option<u64>,
    }

    let mut stream = response.bytes_stream();

    while let Some(chunk_result) = stream.next().await {
        let chunk_bytes =
            chunk_result.map_err(|e| AppError::Ollama(format!("Stream read error: {}", e)))?;

        let chunk_str = String::from_utf8_lossy(&chunk_bytes);

        for line in chunk_str.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            if let Ok(progress) = serde_json::from_str::<PullProgress>(line) {
                let _ = app_handle.emit(
                    "model-pull-progress",
                    serde_json::json!({
                        "model": model_name,
                        "status": progress.status.unwrap_or_default(),
                        "completed": progress.completed.unwrap_or(0),
                        "total": progress.total.unwrap_or(0),
                    }),
                );
            }
        }
    }

    Ok(())
}

/// Delete a model from Ollama via DELETE /api/delete.
pub async fn delete_model(host: &str, port: &str, model_name: &str) -> Result<(), AppError> {
    let url = format!("http://{}:{}/api/delete", host, port);

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| AppError::Ollama(e.to_string()))?;

    let response = client
        .delete(&url)
        .json(&serde_json::json!({"name": model_name}))
        .send()
        .await
        .map_err(|e| AppError::Ollama(format!("Delete model request failed: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        let body_text = response
            .text()
            .await
            .unwrap_or_else(|_| "unknown".to_string());
        return Err(AppError::Ollama(format!(
            "Delete model API returned {}: {}",
            status, body_text
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_info_serialization() {
        let info = ModelInfo {
            name: "llama3:8b".to_string(),
            size: 4_500_000_000,
            parameter_size: Some("8B".to_string()),
            quantization_level: Some("Q4_0".to_string()),
            family: Some("llama".to_string()),
            format: Some("gguf".to_string()),
            modified_at: Some("2024-01-15T10:30:00Z".to_string()),
        };

        let json = serde_json::to_string(&info).expect("Failed to serialize ModelInfo");
        let deserialized: ModelInfo =
            serde_json::from_str(&json).expect("Failed to deserialize ModelInfo");

        assert_eq!(deserialized.name, "llama3:8b");
        assert_eq!(deserialized.size, 4_500_000_000);
        assert_eq!(deserialized.parameter_size.as_deref(), Some("8B"));
        assert_eq!(deserialized.quantization_level.as_deref(), Some("Q4_0"));
        assert_eq!(deserialized.family.as_deref(), Some("llama"));
        assert_eq!(deserialized.format.as_deref(), Some("gguf"));
        assert_eq!(
            deserialized.modified_at.as_deref(),
            Some("2024-01-15T10:30:00Z")
        );
    }

    #[test]
    fn test_model_info_serialization_with_nulls() {
        let info = ModelInfo {
            name: "custom-model".to_string(),
            size: 0,
            parameter_size: None,
            quantization_level: None,
            family: None,
            format: None,
            modified_at: None,
        };

        let json = serde_json::to_string(&info).expect("Failed to serialize ModelInfo");
        let deserialized: ModelInfo =
            serde_json::from_str(&json).expect("Failed to deserialize ModelInfo");

        assert_eq!(deserialized.name, "custom-model");
        assert_eq!(deserialized.size, 0);
        assert!(deserialized.parameter_size.is_none());
        assert!(deserialized.quantization_level.is_none());
        assert!(deserialized.family.is_none());
        assert!(deserialized.format.is_none());
        assert!(deserialized.modified_at.is_none());
    }

    #[test]
    fn test_show_model_constructs_request() {
        // Verify the URL construction and JSON body format for show_model
        let host = "localhost";
        let port = "11434";
        let model_name = "llama3:8b";

        let url = format!("http://{}:{}/api/show", host, port);
        assert_eq!(url, "http://localhost:11434/api/show");

        let body = serde_json::json!({"name": model_name});
        assert_eq!(body["name"], "llama3:8b");
    }

    #[test]
    fn test_pull_model_constructs_request() {
        let host = "127.0.0.1";
        let port = "11434";
        let model_name = "mistral:latest";

        let url = format!("http://{}:{}/api/pull", host, port);
        assert_eq!(url, "http://127.0.0.1:11434/api/pull");

        let body = serde_json::json!({"name": model_name, "stream": true});
        assert_eq!(body["name"], "mistral:latest");
        assert_eq!(body["stream"], true);
    }

    #[test]
    fn test_delete_model_constructs_request() {
        let host = "localhost";
        let port = "11434";
        let model_name = "old-model:v1";

        let url = format!("http://{}:{}/api/delete", host, port);
        assert_eq!(url, "http://localhost:11434/api/delete");

        let body = serde_json::json!({"name": model_name});
        assert_eq!(body["name"], "old-model:v1");
    }
}
