use std::sync::Arc;

use tokio::sync::Semaphore;

use crate::chunker::ChunkData;
use crate::error::AppError;
use crate::ollama;

pub async fn embed_chunks(
    host: &str,
    port: &str,
    model: &str,
    chunks: &[ChunkData],
) -> Result<Vec<Vec<f64>>, AppError> {
    if chunks.is_empty() {
        return Ok(Vec::new());
    }

    let semaphore = Arc::new(Semaphore::new(4));
    let host = host.to_string();
    let port = port.to_string();
    let model = model.to_string();

    let mut handles = Vec::with_capacity(chunks.len());

    for (idx, chunk) in chunks.iter().enumerate() {
        let sem = Arc::clone(&semaphore);
        let h = host.clone();
        let p = port.clone();
        let m = model.clone();
        let text = chunk.content.clone();

        let handle = tokio::spawn(async move {
            let _permit = sem
                .acquire()
                .await
                .map_err(|e| AppError::Ollama(format!("Semaphore error: {}", e)))?;

            let mut last_err = None;
            for attempt in 0..3u32 {
                match ollama::generate_embedding(&h, &p, &m, &text).await {
                    Ok(vec) => return Ok((idx, vec)),
                    Err(e) => {
                        last_err = Some(e);
                        if attempt < 2 {
                            let delay = std::time::Duration::from_secs(1u64 << attempt);
                            tokio::time::sleep(delay).await;
                        }
                    }
                }
            }

            Err(last_err.unwrap_or_else(|| {
                AppError::Ollama("Embedding failed after 3 retries".to_string())
            }))
        });

        handles.push(handle);
    }

    // Collect results preserving order
    let mut results: Vec<(usize, Vec<f64>)> = Vec::with_capacity(chunks.len());
    for handle in handles {
        let result = handle
            .await
            .map_err(|e| AppError::Ollama(format!("Task join error: {}", e)))??;
        results.push(result);
    }

    results.sort_by_key(|(idx, _)| *idx);
    let embeddings = results.into_iter().map(|(_, vec)| vec).collect();

    Ok(embeddings)
}
