use std::collections::HashMap;

use crate::commands::search::SearchResult;
use crate::error::AppError;
use crate::ollama::{self, ChatMessage};

/// Parse a numbered list (1. ... 2. ... 3. ...) from LLM response text.
/// Returns up to `max` parsed items, stripping numbering and whitespace.
pub(crate) fn parse_rewritten_queries(text: &str, max: usize) -> Vec<String> {
    let mut results = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        // Match lines starting with a digit followed by . or )
        let stripped = if let Some(rest) = trimmed
            .strip_prefix(|c: char| c.is_ascii_digit())
            .and_then(|s| s.strip_prefix('.').or_else(|| s.strip_prefix(')')))
        {
            rest.trim()
        } else {
            // Also handle multi-digit numbers like "10."
            let mut chars = trimmed.chars();
            let first = chars.next();
            let second = chars.next();
            let third = chars.next();
            match (first, second, third) {
                (Some(a), Some(b), Some(c))
                    if a.is_ascii_digit() && b.is_ascii_digit() && (c == '.' || c == ')') =>
                {
                    trimmed.get(3..).map(|s| s.trim()).unwrap_or("")
                }
                _ => continue,
            }
        };

        if !stripped.is_empty() {
            // Remove surrounding quotes if present
            let cleaned = stripped.trim_matches('"').trim_matches('\'').trim();
            if !cleaned.is_empty() {
                results.push(cleaned.to_string());
            }
        }
        if results.len() >= max {
            break;
        }
    }
    results
}

/// Generate alternative phrasings of a search query using the chat LLM.
/// Returns up to 3 alternative queries (not including the original).
pub async fn rewrite_query(
    host: &str,
    port: &str,
    model: &str,
    original_query: &str,
) -> Result<Vec<String>, AppError> {
    let system_msg = ChatMessage {
        role: "system".to_string(),
        content: "Generate exactly 3 alternative phrasings of the following search query. \
                  Return them as a numbered list (1. 2. 3.) with nothing else."
            .to_string(),
    };
    let user_msg = ChatMessage {
        role: "user".to_string(),
        content: original_query.to_string(),
    };

    let response = ollama::chat_once(host, port, model, &[system_msg, user_msg]).await?;
    let queries = parse_rewritten_queries(&response, 3);

    if queries.is_empty() {
        return Err(AppError::Ollama(
            "Failed to parse rewritten queries from LLM response".to_string(),
        ));
    }

    Ok(queries)
}

/// Hypothetical Document Embedding (HyDE): generate a hypothetical answer paragraph,
/// then embed it to create a query embedding that lives in "document space."
pub async fn generate_hyde_embedding(
    host: &str,
    port: &str,
    chat_model: &str,
    embed_model: &str,
    query: &str,
) -> Result<Vec<f64>, AppError> {
    let system_msg = ChatMessage {
        role: "system".to_string(),
        content: "Write a short paragraph (2-3 sentences) that would appear in a knowledge base \
                  document answering this question. Write only the paragraph, nothing else."
            .to_string(),
    };
    let user_msg = ChatMessage {
        role: "user".to_string(),
        content: query.to_string(),
    };

    let hypothetical_doc =
        ollama::chat_once(host, port, chat_model, &[system_msg, user_msg]).await?;

    if hypothetical_doc.trim().is_empty() {
        return Err(AppError::Ollama(
            "HyDE: LLM returned empty hypothetical document".to_string(),
        ));
    }

    ollama::generate_embedding(host, port, embed_model, &hypothetical_doc).await
}

/// Generate embeddings for the original query plus rewritten alternatives.
/// Returns (rewritten_queries, all_embeddings) where all_embeddings[0] is the original.
pub async fn generate_multi_query_embeddings(
    host: &str,
    port: &str,
    embed_model: &str,
    chat_model: &str,
    query: &str,
) -> Result<(Vec<String>, Vec<Vec<f64>>), AppError> {
    let rewritten = rewrite_query(host, port, chat_model, query).await?;

    let mut all_queries = vec![query.to_string()];
    all_queries.extend(rewritten.clone());

    let mut embeddings = Vec::with_capacity(all_queries.len());
    for q in &all_queries {
        let emb = ollama::generate_embedding(host, port, embed_model, q).await?;
        embeddings.push(emb);
    }

    Ok((rewritten, embeddings))
}

/// Multi-list Reciprocal Rank Fusion: merge N ranked lists into one.
pub fn multi_rrf(result_sets: Vec<Vec<SearchResult>>, k: f64, top_k: usize) -> Vec<SearchResult> {
    let mut scores: HashMap<String, (f64, SearchResult)> = HashMap::new();

    for result_set in result_sets {
        for (rank, result) in result_set.into_iter().enumerate() {
            let rrf_score = 1.0 / (k + (rank as f64) + 1.0);
            scores
                .entry(result.chunk_id.clone())
                .and_modify(|(s, _)| *s += rrf_score)
                .or_insert((rrf_score, result));
        }
    }

    let mut fused: Vec<(f64, SearchResult)> = scores.into_values().collect();
    fused.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    fused.truncate(top_k);

    fused
        .into_iter()
        .map(|(score, mut result)| {
            result.score = score;
            result
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rewritten_queries() {
        // Standard numbered list
        let text =
            "1. What is machine learning?\n2. How does ML work?\n3. Explain machine learning concepts";
        let result = parse_rewritten_queries(text, 3);
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], "What is machine learning?");
        assert_eq!(result[1], "How does ML work?");
        assert_eq!(result[2], "Explain machine learning concepts");

        // With extra whitespace and blank lines
        let text = "\n  1. First query  \n\n  2. Second query \n  3. Third query  \n";
        let result = parse_rewritten_queries(text, 3);
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], "First query");
        assert_eq!(result[1], "Second query");
        assert_eq!(result[2], "Third query");

        // With parenthesis style numbering
        let text = "1) Alpha query\n2) Beta query\n3) Gamma query";
        let result = parse_rewritten_queries(text, 3);
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], "Alpha query");

        // With quoted items
        let text = "1. \"What is Rust?\"\n2. 'How to learn Rust?'\n3. Rust programming language";
        let result = parse_rewritten_queries(text, 3);
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], "What is Rust?");
        assert_eq!(result[1], "How to learn Rust?");

        // Empty input
        let result = parse_rewritten_queries("", 3);
        assert!(result.is_empty());

        // Only 2 items when max is 3
        let text = "1. First\n2. Second";
        let result = parse_rewritten_queries(text, 3);
        assert_eq!(result.len(), 2);

        // Respects max limit
        let text = "1. A\n2. B\n3. C\n4. D";
        let result = parse_rewritten_queries(text, 2);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_multi_rrf_merges_correctly() {
        let set_a = vec![
            SearchResult {
                chunk_id: "c1".to_string(),
                document_id: "d1".to_string(),
                document_title: "Doc 1".to_string(),
                section_title: None,
                page_number: None,
                content: "Content 1".to_string(),
                score: 0.9,
            },
            SearchResult {
                chunk_id: "c2".to_string(),
                document_id: "d1".to_string(),
                document_title: "Doc 1".to_string(),
                section_title: None,
                page_number: None,
                content: "Content 2".to_string(),
                score: 0.8,
            },
        ];
        let set_b = vec![
            SearchResult {
                chunk_id: "c2".to_string(),
                document_id: "d1".to_string(),
                document_title: "Doc 1".to_string(),
                section_title: None,
                page_number: None,
                content: "Content 2".to_string(),
                score: 0.85,
            },
            SearchResult {
                chunk_id: "c3".to_string(),
                document_id: "d2".to_string(),
                document_title: "Doc 2".to_string(),
                section_title: None,
                page_number: None,
                content: "Content 3".to_string(),
                score: 0.7,
            },
        ];

        let fused = multi_rrf(vec![set_a, set_b], 60.0, 10);

        // c2 should be ranked highest because it appears in both sets
        assert_eq!(fused[0].chunk_id, "c2");
        // All 3 unique chunks should appear
        assert_eq!(fused.len(), 3);
        // c2's score should be sum of two RRF contributions
        let expected_c2_score = 1.0 / (60.0 + 1.0 + 1.0) + 1.0 / (60.0 + 0.0 + 1.0);
        assert!(
            (fused[0].score - expected_c2_score).abs() < 1e-10,
            "Expected c2 score ~{}, got {}",
            expected_c2_score,
            fused[0].score
        );
    }
}
