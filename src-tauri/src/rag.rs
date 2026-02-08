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

/// Estimate token count for a string using words * 1.3 heuristic.
pub fn estimate_tokens(text: &str) -> usize {
    let word_count = text.split_whitespace().count();
    (word_count as f64 * 1.3).ceil() as usize
}

/// Dynamic relevance threshold: mean(scores) - 1*stddev, minimum 0.3.
/// Returns only results above the computed threshold.
pub fn filter_by_relevance(results: &[SearchResult]) -> Vec<SearchResult> {
    if results.is_empty() {
        return Vec::new();
    }

    let scores: Vec<f64> = results.iter().map(|r| r.score).collect();
    let n = scores.len() as f64;
    let mean = scores.iter().sum::<f64>() / n;

    let variance = scores.iter().map(|s| (s - mean).powi(2)).sum::<f64>() / n;
    let stddev = variance.sqrt();

    let threshold = (mean - stddev).max(0.3);

    results
        .iter()
        .filter(|r| r.score >= threshold)
        .cloned()
        .collect()
}

/// Select chunks fitting token budget (prioritizing high scores) and trim
/// conversation history to fit within its own budget.
///
/// Returns (selected_chunks, trimmed_history) where:
/// - selected_chunks: highest-scoring results that fit in `max_tokens`
/// - trimmed_history: most recent (role, content) pairs fitting `history_token_budget`
pub fn build_adaptive_context(
    results: &[SearchResult],
    max_tokens: usize,
    conversation_history: &[(String, String)],
    history_token_budget: usize,
) -> (Vec<SearchResult>, Vec<(String, String)>) {
    // 1. Filter by relevance first
    let mut relevant = filter_by_relevance(results);

    // 2. Sort by score descending (highest first)
    relevant.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

    // 3. Greedily add chunks until token budget exhausted
    let mut selected_chunks = Vec::new();
    let mut tokens_used: usize = 0;
    for result in &relevant {
        let chunk_tokens = estimate_tokens(&result.content);
        if tokens_used + chunk_tokens > max_tokens {
            break;
        }
        tokens_used += chunk_tokens;
        selected_chunks.push(result.clone());
    }

    // 4. Trim history: keep most recent messages that fit within budget
    let mut selected_history: Vec<(String, String)> = Vec::new();
    let mut history_tokens_used: usize = 0;
    // Iterate from most recent to oldest
    for (role, content) in conversation_history.iter().rev() {
        let msg_tokens = estimate_tokens(content) + estimate_tokens(role);
        if history_tokens_used + msg_tokens > history_token_budget {
            break;
        }
        history_tokens_used += msg_tokens;
        selected_history.push((role.clone(), content.clone()));
    }
    // Reverse to restore chronological order
    selected_history.reverse();

    (selected_chunks, selected_history)
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

    fn make_result(chunk_id: &str, content: &str, score: f64) -> SearchResult {
        SearchResult {
            chunk_id: chunk_id.to_string(),
            document_id: "d1".to_string(),
            document_title: "Doc".to_string(),
            section_title: None,
            page_number: None,
            content: content.to_string(),
            score,
        }
    }

    #[test]
    fn test_estimate_tokens() {
        // Empty string -> 0 tokens
        assert_eq!(estimate_tokens(""), 0);

        // Single word -> ceil(1 * 1.3) = 2
        assert_eq!(estimate_tokens("hello"), 2);

        // 10 words -> ceil(10 * 1.3) = 13
        let ten_words = "one two three four five six seven eight nine ten";
        assert_eq!(estimate_tokens(ten_words), 13);

        // 100 words -> ceil(100 * 1.3) = 130
        let hundred_words: String = (0..100).map(|i| format!("word{}", i)).collect::<Vec<_>>().join(" ");
        assert_eq!(estimate_tokens(&hundred_words), 130);
    }

    #[test]
    fn test_filter_by_relevance_removes_low_scores() {
        let results = vec![
            make_result("c1", "high score content", 0.9),
            make_result("c2", "medium score content", 0.7),
            make_result("c3", "low score content", 0.2),
            make_result("c4", "very low score content", 0.1),
        ];

        let filtered = filter_by_relevance(&results);

        // Low scores (0.2, 0.1) should be filtered out
        // mean = (0.9+0.7+0.2+0.1)/4 = 0.475
        // variance = ((0.425^2 + 0.225^2 + 0.275^2 + 0.375^2))/4 = 0.104375
        // stddev = ~0.323
        // threshold = max(0.475 - 0.323, 0.3) = max(0.152, 0.3) = 0.3
        // So 0.9 and 0.7 pass, 0.2 and 0.1 do not
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|r| r.score >= 0.3));
        assert!(filtered.iter().any(|r| r.chunk_id == "c1"));
        assert!(filtered.iter().any(|r| r.chunk_id == "c2"));
    }

    #[test]
    fn test_filter_by_relevance_minimum_threshold() {
        // All identical high scores: stddev=0, threshold = max(0.9 - 0, 0.3) = 0.9
        // All results equal the threshold so all pass (>= check)
        let results = vec![
            make_result("c1", "content a", 0.9),
            make_result("c2", "content b", 0.9),
            make_result("c3", "content c", 0.9),
        ];

        let filtered = filter_by_relevance(&results);
        assert_eq!(filtered.len(), 3);

        // With low uniform scores above 0.3, all pass
        let low_results = vec![
            make_result("c1", "content a", 0.35),
            make_result("c2", "content b", 0.35),
        ];
        let filtered_low = filter_by_relevance(&low_results);
        // mean=0.35, stddev=0, threshold=max(0.35, 0.3)=0.35, all pass
        assert_eq!(filtered_low.len(), 2);

        // When all scores are below 0.3, minimum threshold of 0.3 filters everything
        let very_low = vec![
            make_result("c1", "content a", 0.2),
            make_result("c2", "content b", 0.1),
        ];
        let filtered_vlow = filter_by_relevance(&very_low);
        assert!(filtered_vlow.is_empty());
    }

    #[test]
    fn test_build_adaptive_context_respects_budget() {
        // Each chunk ~13 tokens ("one two three four five six seven eight nine ten" = 10 words * 1.3)
        let ten_words = "one two three four five six seven eight nine ten";
        let results = vec![
            make_result("c1", ten_words, 0.9),
            make_result("c2", ten_words, 0.8),
            make_result("c3", ten_words, 0.7),
            make_result("c4", ten_words, 0.6),
        ];

        // Budget of 30 tokens should fit ~2 chunks (each ~13 tokens)
        let (chunks, _) = build_adaptive_context(&results, 30, &[], 0);

        let total_tokens: usize = chunks.iter().map(|c| estimate_tokens(&c.content)).sum();
        assert!(total_tokens <= 30, "Total tokens {} should be <= 30", total_tokens);
        assert_eq!(chunks.len(), 2);
    }

    #[test]
    fn test_build_adaptive_context_prioritizes_high_scores() {
        let results = vec![
            make_result("c_low", "some low scoring content here", 0.5),
            make_result("c_high", "some high scoring content here", 0.95),
            make_result("c_mid", "some medium scoring content here", 0.75),
        ];

        // Large budget so all relevant results fit
        let (chunks, _) = build_adaptive_context(&results, 10000, &[], 0);

        // The first chunk should be the highest scored
        if !chunks.is_empty() {
            assert_eq!(chunks[0].chunk_id, "c_high");
        }

        // If there are multiple, they should be in score-descending order
        for i in 1..chunks.len() {
            assert!(
                chunks[i - 1].score >= chunks[i].score,
                "Chunks should be ordered by score descending"
            );
        }
    }
}
