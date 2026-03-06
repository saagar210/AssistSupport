use assistsupport_lib::commands;
use assistsupport_lib::commands::search_api::{
    HybridSearchMetrics, HybridSearchResponse, HybridSearchResult, HybridSearchScores,
    SearchApiHealthStatus,
};
use serde_json::Value;
use std::path::PathBuf;

fn load_fixture(relative_path: &str) -> Value {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("src-tauri should be nested under repo root")
        .to_path_buf();
    let fixture_path = repo_root.join(relative_path);
    let raw = std::fs::read_to_string(&fixture_path)
        .unwrap_or_else(|err| panic!("failed to read fixture {}: {}", fixture_path.display(), err));
    serde_json::from_str(&raw).unwrap_or_else(|err| {
        panic!(
            "fixture is not valid JSON {}: {}",
            fixture_path.display(),
            err
        )
    })
}

#[test]
fn generate_with_context_result_fixture_matches_contract() {
    let payload = commands::GenerateWithContextResult {
        text: "Use the approved VPN client and MFA.".to_string(),
        tokens_generated: 42,
        duration_ms: 1200,
        source_chunk_ids: vec!["chunk-1".to_string()],
        sources: vec![commands::ContextSource {
            chunk_id: "chunk-1".to_string(),
            document_id: "doc-1".to_string(),
            file_path: "/kb/policies/remote-work.md".to_string(),
            title: Some("Remote Work Policy".to_string()),
            heading_path: Some("Policy > VPN".to_string()),
            score: 0.97,
            search_method: Some("hybrid".to_string()),
            source_type: Some("file".to_string()),
        }],
        metrics: commands::GenerationMetrics {
            tokens_per_second: 35.0,
            sources_used: 1,
            word_count: 14,
            length_target_met: true,
            context_utilization: 0.31,
        },
        prompt_template_version: "v1.2.0".to_string(),
        confidence: commands::ConfidenceAssessment {
            mode: commands::ConfidenceMode::Answer,
            score: 0.92,
            rationale: "Grounded in one high-confidence source".to_string(),
        },
        grounding: vec![commands::GroundedClaim {
            claim: "Use the approved VPN client and MFA.".to_string(),
            source_indexes: vec![0],
            support_level: "supported".to_string(),
        }],
    };

    let actual = serde_json::to_value(payload).expect("serialize generation contract payload");
    let expected = load_fixture("contracts/tauri/v1/generate_with_context_result.json");
    assert_eq!(
        actual, expected,
        "generate_with_context_result fixture drifted; update contracts/tauri/v1/generate_with_context_result.json intentionally"
    );
}

#[test]
fn hybrid_search_response_fixture_matches_contract() {
    let payload = HybridSearchResponse {
        status: "success".to_string(),
        query: "Can I use a flash drive?".to_string(),
        query_id: Some("query-1".to_string()),
        intent: "POLICY".to_string(),
        intent_confidence: 0.92,
        results_count: 1,
        results: vec![HybridSearchResult {
            rank: 1,
            article_id: "article-1".to_string(),
            title: "Removable Media Policy".to_string(),
            category: "policy".to_string(),
            preview: "USB drives are restricted...".to_string(),
            source_document: Some("doc-1".to_string()),
            section: Some("Section 4.2".to_string()),
            scores: Some(HybridSearchScores {
                bm25: 0.91,
                vector: 0.88,
                fused: 0.90,
            }),
        }],
        metrics: HybridSearchMetrics {
            latency_ms: 22.1,
            embedding_time_ms: 3.5,
            search_time_ms: 8.2,
            result_count: 1,
            timestamp: "2026-02-03T10:00:00Z".to_string(),
        },
    };

    let actual = serde_json::to_value(payload).expect("serialize search response contract");
    let expected = load_fixture("contracts/tauri/v1/hybrid_search_response.json");
    assert_eq!(
        actual, expected,
        "hybrid_search_response fixture drifted; update contracts/tauri/v1/hybrid_search_response.json intentionally"
    );
}

#[test]
fn search_api_health_status_fixture_matches_contract() {
    let payload = SearchApiHealthStatus {
        healthy: true,
        status: "ok".to_string(),
        message: "Connected".to_string(),
        base_url: "http://localhost:3000".to_string(),
    };

    let actual = serde_json::to_value(payload).expect("serialize health status contract");
    let expected = load_fixture("contracts/tauri/v1/search_api_health_status.json");
    assert_eq!(
        actual, expected,
        "search_api_health_status fixture drifted; update contracts/tauri/v1/search_api_health_status.json intentionally"
    );
}
