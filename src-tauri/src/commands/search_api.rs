//! Tauri commands for PostgreSQL hybrid search API
//!
//! These commands proxy requests to the Python Flask API running on localhost:3000,
//! which performs hybrid BM25 + HNSW vector search against PostgreSQL.

use serde::{Deserialize, Serialize};

const SEARCH_API_BASE: &str = "http://localhost:3000";
const DEFAULT_TOP_K: usize = 10;
const MAX_TOP_K: usize = 50;
const MIN_TOP_K: usize = 1;

// ── Request / Response types ──────────────────────────────────────────────────

#[derive(Debug, Serialize)]
struct SearchApiRequest {
    query: String,
    top_k: usize,
    include_scores: bool,
    fusion_strategy: String,
}

#[derive(Debug, Serialize)]
struct FeedbackApiRequest {
    query_id: String,
    result_rank: usize,
    rating: String,
    comment: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridSearchScores {
    pub bm25: f64,
    pub vector: f64,
    pub fused: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridSearchResult {
    pub rank: usize,
    pub article_id: String,
    pub title: String,
    pub category: String,
    pub preview: String,
    pub source_document: Option<String>,
    pub section: Option<String>,
    pub scores: Option<HybridSearchScores>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridSearchMetrics {
    pub latency_ms: f64,
    pub embedding_time_ms: f64,
    pub search_time_ms: f64,
    pub result_count: usize,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridSearchResponse {
    pub status: String,
    pub query: String,
    pub query_id: Option<String>,
    pub intent: String,
    pub intent_confidence: f64,
    pub results_count: usize,
    pub results: Vec<HybridSearchResult>,
    pub metrics: HybridSearchMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchApiLatency {
    pub avg: f64,
    pub p50: f64,
    pub p95: f64,
    pub p99: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchApiFeedbackStats {
    #[serde(default)]
    pub helpful: u64,
    #[serde(default)]
    pub not_helpful: u64,
    #[serde(default)]
    pub incorrect: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchApiStatsData {
    pub queries_24h: u64,
    pub queries_total: u64,
    pub latency_ms: SearchApiLatency,
    pub feedback_stats: SearchApiFeedbackStats,
    pub intent_distribution: std::collections::HashMap<String, u64>,
}

#[derive(Debug, Deserialize)]
struct StatsApiResponse {
    #[allow(dead_code)]
    status: String,
    data: SearchApiStatsData,
    #[allow(dead_code)]
    timestamp: String,
}

#[derive(Debug, Deserialize)]
struct HealthApiResponse {
    status: String,
    #[allow(dead_code)]
    service: Option<String>,
    #[allow(dead_code)]
    timestamp: Option<String>,
}

fn sanitize_top_k(top_k: Option<usize>) -> usize {
    top_k.unwrap_or(DEFAULT_TOP_K).clamp(MIN_TOP_K, MAX_TOP_K)
}

fn is_valid_feedback_rating(rating: &str) -> bool {
    matches!(rating, "helpful" | "not_helpful" | "incorrect")
}

// ── Tauri commands ────────────────────────────────────────────────────────────

/// Execute a hybrid search against the PostgreSQL search API.
#[tauri::command]
pub async fn hybrid_search(
    query: String,
    top_k: Option<usize>,
) -> Result<HybridSearchResponse, String> {
    let client = reqwest::Client::new();

    let request = SearchApiRequest {
        query,
        top_k: sanitize_top_k(top_k),
        include_scores: true,
        fusion_strategy: "adaptive".to_string(),
    };

    let response = client
        .post(format!("{}/search", SEARCH_API_BASE))
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("Search API unavailable: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("Search API error ({}): {}", status, body));
    }

    response
        .json::<HybridSearchResponse>()
        .await
        .map_err(|e| format!("Failed to parse search response: {}", e))
}

/// Submit feedback on a search result (helpful / not_helpful / incorrect).
#[tauri::command]
pub async fn submit_search_feedback(
    query_id: String,
    result_rank: usize,
    rating: String,
    comment: Option<String>,
) -> Result<String, String> {
    if !is_valid_feedback_rating(&rating) {
        return Err(format!(
            "Invalid rating '{}': must be helpful, not_helpful, or incorrect",
            rating
        ));
    }

    let client = reqwest::Client::new();

    let feedback = FeedbackApiRequest {
        query_id,
        result_rank,
        rating,
        comment: comment.unwrap_or_default(),
    };

    let response = client
        .post(format!("{}/feedback", SEARCH_API_BASE))
        .json(&feedback)
        .send()
        .await
        .map_err(|e| format!("Feedback submission failed: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("Feedback API error ({}): {}", status, body));
    }

    Ok("Feedback submitted".to_string())
}

/// Get search monitoring statistics (last 24 hours).
#[tauri::command]
pub async fn get_search_api_stats() -> Result<SearchApiStatsData, String> {
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/stats", SEARCH_API_BASE))
        .send()
        .await
        .map_err(|e| format!("Stats API unavailable: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("Stats API error ({}): {}", status, body));
    }

    let stats: StatsApiResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse stats response: {}", e))?;

    Ok(stats.data)
}

/// Check if the search API is healthy.
#[tauri::command]
pub async fn check_search_api_health() -> Result<bool, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| e.to_string())?;

    match client
        .get(format!("{}/health", SEARCH_API_BASE))
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<HealthApiResponse>().await {
                    Ok(health) => Ok(health.status == "ok"),
                    Err(_) => Ok(false),
                }
            } else {
                Ok(false)
            }
        }
        Err(_) => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_top_k_applies_defaults_and_bounds() {
        assert_eq!(sanitize_top_k(None), DEFAULT_TOP_K);
        assert_eq!(sanitize_top_k(Some(0)), MIN_TOP_K);
        assert_eq!(sanitize_top_k(Some(1)), MIN_TOP_K);
        assert_eq!(sanitize_top_k(Some(25)), 25);
        assert_eq!(sanitize_top_k(Some(500)), MAX_TOP_K);
    }

    #[test]
    fn feedback_rating_validation_accepts_only_known_values() {
        assert!(is_valid_feedback_rating("helpful"));
        assert!(is_valid_feedback_rating("not_helpful"));
        assert!(is_valid_feedback_rating("incorrect"));
        assert!(!is_valid_feedback_rating("HELPFUL"));
        assert!(!is_valid_feedback_rating(" meh "));
    }

    #[test]
    fn stats_response_defaults_missing_feedback_fields_to_zero() {
        let payload = serde_json::json!({
            "status": "success",
            "data": {
                "queries_24h": 1,
                "queries_total": 2,
                "latency_ms": {
                    "avg": 1.0,
                    "p50": 1.0,
                    "p95": 2.0,
                    "p99": 3.0
                },
                "feedback_stats": {},
                "intent_distribution": {}
            },
            "timestamp": "2026-02-03T00:00:00Z"
        });

        let parsed: StatsApiResponse =
            serde_json::from_value(payload).expect("valid stats payload");
        assert_eq!(parsed.data.feedback_stats.helpful, 0);
        assert_eq!(parsed.data.feedback_stats.not_helpful, 0);
        assert_eq!(parsed.data.feedback_stats.incorrect, 0);
    }

    #[tokio::test]
    async fn submit_feedback_rejects_invalid_rating_before_network_call() {
        let result = submit_search_feedback(
            "query-123".to_string(),
            1,
            "invalid".to_string(),
            Some("bad".to_string()),
        )
        .await;

        assert!(result.is_err());
        assert!(result
            .expect_err("must reject invalid rating")
            .contains("Invalid rating"));
    }
}
