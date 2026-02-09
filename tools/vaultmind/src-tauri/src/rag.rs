use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiHopResult {
    pub chunk_id: String,
    pub document_id: String,
    pub document_title: String,
    pub content: String,
    pub score: f64,
    pub hop_distance: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreciseCitation {
    pub snippet: String,
    pub start_char: usize,
    pub end_char: usize,
    pub confidence: f64,
}

/// Follow graph edges from initial results to find related context.
///
/// Starting from the chunk_ids in `initial_results` (hop_distance = 0), traverses
/// graph_edges to discover neighboring chunks up to `max_hops` levels deep.
/// Returns at most `max_additional` extra results beyond the initial set.
pub fn multi_hop_retrieval(
    conn: &rusqlite::Connection,
    initial_results: &[SearchResult],
    collection_id: &str,
    max_hops: usize,
    max_additional: usize,
) -> Result<Vec<MultiHopResult>, AppError> {
    if initial_results.is_empty() || max_hops == 0 || max_additional == 0 {
        return Ok(Vec::new());
    }

    // Track all seen chunk_ids to avoid revisiting
    let mut seen: HashSet<String> = initial_results.iter().map(|r| r.chunk_id.clone()).collect();

    // Current frontier: chunk_ids we just discovered
    let mut frontier: Vec<String> = initial_results.iter().map(|r| r.chunk_id.clone()).collect();

    let mut additional_results: Vec<MultiHopResult> = Vec::new();

    for hop in 1..=max_hops {
        if frontier.is_empty() || additional_results.len() >= max_additional {
            break;
        }

        // Find all neighbors of the current frontier via graph_edges
        let placeholders: String = frontier.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let sql = format!(
            "SELECT source_chunk_id, target_chunk_id, weight
             FROM graph_edges
             WHERE collection_id = ?1
               AND (source_chunk_id IN ({placeholders}) OR target_chunk_id IN ({placeholders}))"
        );

        let mut stmt = conn.prepare(&sql)?;

        // Bind parameters: ?1 = collection_id, then frontier twice
        let param_count = 1 + frontier.len() * 2;
        let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::with_capacity(param_count);
        params.push(Box::new(collection_id.to_string()));
        for chunk_id in &frontier {
            params.push(Box::new(chunk_id.clone()));
        }
        for chunk_id in &frontier {
            params.push(Box::new(chunk_id.clone()));
        }

        let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();

        let mut neighbor_ids: Vec<(String, f64)> = Vec::new();
        let mut rows = stmt.query(param_refs.as_slice())?;
        while let Some(row) = rows.next()? {
            let src: String = row.get(0)?;
            let tgt: String = row.get(1)?;
            let weight: f64 = row.get(2)?;

            // The neighbor is whichever side is NOT in the frontier
            if frontier.contains(&src) && !seen.contains(&tgt) {
                neighbor_ids.push((tgt.clone(), weight));
            }
            if frontier.contains(&tgt) && !seen.contains(&src) {
                neighbor_ids.push((src, weight));
            }
        }

        // Deduplicate and sort by weight descending
        let mut unique_neighbors: HashMap<String, f64> = HashMap::new();
        for (id, weight) in neighbor_ids {
            unique_neighbors
                .entry(id)
                .and_modify(|w| {
                    if weight > *w {
                        *w = weight;
                    }
                })
                .or_insert(weight);
        }

        let mut sorted_neighbors: Vec<(String, f64)> = unique_neighbors.into_iter().collect();
        sorted_neighbors.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let mut next_frontier: Vec<String> = Vec::new();

        for (neighbor_id, weight) in sorted_neighbors {
            if additional_results.len() >= max_additional {
                break;
            }

            seen.insert(neighbor_id.clone());

            // Enrich with chunk content and document info
            let enrichment = conn.query_row(
                "SELECT c.content, c.document_id, d.title
                 FROM chunks c
                 JOIN documents d ON d.id = c.document_id
                 WHERE c.id = ?1",
                rusqlite::params![neighbor_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                    ))
                },
            );

            if let Ok((content, document_id, document_title)) = enrichment {
                additional_results.push(MultiHopResult {
                    chunk_id: neighbor_id.clone(),
                    document_id,
                    document_title,
                    content,
                    score: weight,
                    hop_distance: hop,
                });
                next_frontier.push(neighbor_id);
            }
        }

        frontier = next_frontier;
    }

    Ok(additional_results)
}

/// Extract the best matching sentence from chunk content for a citation.
///
/// Splits the chunk into sentences, computes word overlap with the query,
/// and returns the sentence with the highest overlap as a precise citation.
pub fn extract_precise_citation(chunk_content: &str, query: &str) -> PreciseCitation {
    if chunk_content.is_empty() || query.is_empty() {
        return PreciseCitation {
            snippet: String::new(),
            start_char: 0,
            end_char: 0,
            confidence: 0.0,
        };
    }

    let query_words: HashSet<String> = query
        .split_whitespace()
        .map(|w| w.to_lowercase().trim_matches(|c: char| !c.is_alphanumeric()).to_string())
        .filter(|w| !w.is_empty())
        .collect();

    if query_words.is_empty() {
        return PreciseCitation {
            snippet: chunk_content.to_string(),
            start_char: 0,
            end_char: chunk_content.len(),
            confidence: 0.0,
        };
    }

    // Split into sentences using common delimiters
    let mut sentences: Vec<(usize, usize, &str)> = Vec::new();
    let mut start = 0;
    for (i, c) in chunk_content.char_indices() {
        if c == '.' || c == '!' || c == '?' {
            let end = i + c.len_utf8();
            let sentence = chunk_content[start..end].trim();
            if !sentence.is_empty() {
                // Find the actual trimmed positions
                let trim_start = start + chunk_content[start..end].find(sentence).unwrap_or(0);
                let trim_end = trim_start + sentence.len();
                sentences.push((trim_start, trim_end, sentence));
            }
            start = end;
        }
    }
    // Handle trailing text without sentence-ending punctuation
    if start < chunk_content.len() {
        let remaining = chunk_content[start..].trim();
        if !remaining.is_empty() {
            let trim_start = start + chunk_content[start..].find(remaining).unwrap_or(0);
            let trim_end = trim_start + remaining.len();
            sentences.push((trim_start, trim_end, remaining));
        }
    }

    if sentences.is_empty() {
        return PreciseCitation {
            snippet: chunk_content.to_string(),
            start_char: 0,
            end_char: chunk_content.len(),
            confidence: 0.0,
        };
    }

    let mut best_score: f64 = -1.0;
    let mut best_idx: usize = 0;

    for (i, (_start, _end, sentence)) in sentences.iter().enumerate() {
        let sentence_words: HashSet<String> = sentence
            .split_whitespace()
            .map(|w| w.to_lowercase().trim_matches(|c: char| !c.is_alphanumeric()).to_string())
            .filter(|w| !w.is_empty())
            .collect();

        let overlap = query_words.intersection(&sentence_words).count() as f64;
        let score = if query_words.is_empty() {
            0.0
        } else {
            overlap / query_words.len() as f64
        };

        if score > best_score {
            best_score = score;
            best_idx = i;
        }
    }

    let (start_char, end_char, snippet) = sentences[best_idx];

    PreciseCitation {
        snippet: snippet.to_string(),
        start_char,
        end_char,
        confidence: best_score.clamp(0.0, 1.0),
    }
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

    #[test]
    fn test_multi_hop_retrieval_finds_neighbors() {
        let conn = rusqlite::Connection::open_in_memory().expect("open in-memory DB");
        conn.execute_batch("PRAGMA foreign_keys = OFF;").expect("set pragma");
        conn.execute_batch(
            "CREATE TABLE collections (id TEXT PRIMARY KEY, name TEXT, description TEXT, created_at TEXT, updated_at TEXT);
             CREATE TABLE documents (id TEXT PRIMARY KEY, collection_id TEXT, filename TEXT, file_path TEXT, file_type TEXT, file_size INTEGER, file_hash TEXT, title TEXT, author TEXT, page_count INTEGER, word_count INTEGER DEFAULT 0, chunk_count INTEGER DEFAULT 0, status TEXT DEFAULT 'done', error_message TEXT, created_at TEXT, updated_at TEXT);
             CREATE TABLE chunks (id TEXT PRIMARY KEY, document_id TEXT, collection_id TEXT, content TEXT, chunk_index INTEGER, start_offset INTEGER DEFAULT 0, end_offset INTEGER DEFAULT 0, page_number INTEGER, section_title TEXT, token_count INTEGER DEFAULT 0, created_at TEXT);
             CREATE TABLE graph_edges (id TEXT PRIMARY KEY, source_chunk_id TEXT, target_chunk_id TEXT, collection_id TEXT, weight REAL DEFAULT 0.0, relationship_type TEXT DEFAULT 'semantic', created_at TEXT);"
        ).expect("create tables");

        let now = "2025-01-01T00:00:00Z";
        conn.execute("INSERT INTO collections (id, name, description, created_at, updated_at) VALUES ('col1', 'Test', '', ?1, ?1)", rusqlite::params![now]).expect("insert col");
        conn.execute(
            "INSERT INTO documents (id, collection_id, filename, file_path, file_type, file_size, file_hash, title, word_count, chunk_count, created_at, updated_at) VALUES ('doc1', 'col1', 'f.txt', '/f.txt', 'txt', 100, 'h', 'Doc One', 50, 3, ?1, ?1)",
            rusqlite::params![now],
        ).expect("insert doc1");
        conn.execute(
            "INSERT INTO documents (id, collection_id, filename, file_path, file_type, file_size, file_hash, title, word_count, chunk_count, created_at, updated_at) VALUES ('doc2', 'col1', 'g.txt', '/g.txt', 'txt', 100, 'h2', 'Doc Two', 50, 1, ?1, ?1)",
            rusqlite::params![now],
        ).expect("insert doc2");

        for (cid, did, idx, content) in &[
            ("c1", "doc1", 0, "Machine learning basics and fundamentals."),
            ("c2", "doc1", 1, "Neural networks are a subset of machine learning."),
            ("c3", "doc1", 2, "Deep learning uses multiple neural network layers."),
            ("c4", "doc2", 0, "Reinforcement learning is another ML approach."),
        ] {
            conn.execute(
                "INSERT INTO chunks (id, document_id, collection_id, content, chunk_index, created_at) VALUES (?1, ?2, 'col1', ?3, ?4, ?5)",
                rusqlite::params![cid, did, content, idx, now],
            ).expect("insert chunk");
        }

        for (eid, src, tgt, w) in &[("e1", "c1", "c2", 0.9), ("e2", "c2", "c3", 0.85), ("e3", "c2", "c4", 0.7)] {
            conn.execute(
                "INSERT INTO graph_edges (id, source_chunk_id, target_chunk_id, collection_id, weight, relationship_type, created_at) VALUES (?1, ?2, ?3, 'col1', ?4, 'semantic', ?5)",
                rusqlite::params![eid, src, tgt, w, now],
            ).expect("insert edge");
        }

        let initial = vec![SearchResult {
            chunk_id: "c1".to_string(),
            document_id: "doc1".to_string(),
            document_title: "Doc One".to_string(),
            section_title: None,
            page_number: None,
            content: "Machine learning basics and fundamentals.".to_string(),
            score: 0.95,
        }];

        // 1 hop should discover c2
        let results = multi_hop_retrieval(&conn, &initial, "col1", 1, 10).expect("multi_hop 1");
        assert!(!results.is_empty(), "Should find at least one neighbor");
        assert!(results.iter().any(|r| r.chunk_id == "c2"), "Should discover c2 at hop 1");
        assert!(results.iter().all(|r| r.hop_distance == 1), "All should be hop_distance=1");

        // 2 hops should also discover c3 and c4
        let results_2hop = multi_hop_retrieval(&conn, &initial, "col1", 2, 10).expect("multi_hop 2");
        let chunk_ids: HashSet<String> = results_2hop.iter().map(|r| r.chunk_id.clone()).collect();
        assert!(chunk_ids.contains("c2"), "Should contain c2 at hop 1");
        assert!(chunk_ids.contains("c3"), "Should contain c3 at hop 2");
        assert!(chunk_ids.contains("c4"), "Should contain c4 at hop 2");

        for r in &results_2hop {
            if r.chunk_id == "c2" { assert_eq!(r.hop_distance, 1); }
            if r.chunk_id == "c3" || r.chunk_id == "c4" { assert_eq!(r.hop_distance, 2); }
        }

        // max_additional limit
        let limited = multi_hop_retrieval(&conn, &initial, "col1", 2, 1).expect("multi_hop limited");
        assert_eq!(limited.len(), 1, "Should respect max_additional=1");
    }

    #[test]
    fn test_extract_precise_citation_finds_best_sentence() {
        let content = "The weather today is sunny. Machine learning uses neural networks for pattern recognition. The cat sat on the mat.";
        let query = "machine learning neural networks";

        let citation = extract_precise_citation(content, query);
        assert!(citation.snippet.contains("Machine learning"), "Should pick ML sentence, got: '{}'", citation.snippet);
        assert!(citation.confidence > 0.0, "Confidence should be positive");
        assert!(citation.start_char < citation.end_char, "start < end");
        assert_eq!(&content[citation.start_char..citation.end_char], citation.snippet, "Offsets should match snippet");

        // Empty inputs
        let empty = extract_precise_citation("", "query");
        assert!(empty.snippet.is_empty());
        assert_eq!(empty.confidence, 0.0);

        let empty_query = extract_precise_citation("Some content here.", "");
        assert_eq!(empty_query.confidence, 0.0);

        // Single sentence without period
        let single = extract_precise_citation("Only one sentence here", "sentence here");
        assert_eq!(single.snippet, "Only one sentence here");
        assert!(single.confidence > 0.0);

        // Perfect match
        let perfect = extract_precise_citation("Alpha beta gamma.", "alpha beta gamma");
        assert_eq!(perfect.confidence, 1.0, "Perfect overlap should give confidence 1.0");
    }
}
