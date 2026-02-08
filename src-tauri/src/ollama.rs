use std::time::Duration;

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tauri::Emitter;
use tokio_util::sync::CancellationToken;

use crate::error::AppError;
use crate::models::OllamaModel;

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
