use assistsupport_lib::backup::{ExportSummary, ImportPreview, ImportSummary};
use assistsupport_lib::commands;
use assistsupport_lib::commands::backup::ExportFormat;
use assistsupport_lib::commands::search_api::{
    HybridSearchMetrics, HybridSearchResponse, HybridSearchResult, HybridSearchScores,
};
use assistsupport_lib::jira::{JiraConfig, JiraTicket};
use serde_json::{json, Value};

#[test]
fn generate_with_context_result_json_contract() {
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
    };

    let value = serde_json::to_value(payload).expect("serialize generation payload");
    let obj = value.as_object().expect("json object");

    for key in [
        "text",
        "tokens_generated",
        "duration_ms",
        "source_chunk_ids",
        "sources",
        "metrics",
        "prompt_template_version",
    ] {
        assert!(obj.contains_key(key), "missing key: {key}");
    }

    assert_eq!(obj["metrics"]["sources_used"], 1);
    assert_eq!(obj["sources"][0]["chunk_id"], "chunk-1");
}

#[test]
fn backup_summary_json_contracts() {
    let export = ExportSummary {
        drafts_count: 2,
        templates_count: 3,
        variables_count: 4,
        trees_count: 1,
        path: "/tmp/backup.zip".to_string(),
        encrypted: true,
    };
    let import = ImportSummary {
        drafts_imported: 2,
        templates_imported: 3,
        variables_imported: 4,
        trees_imported: 1,
    };
    let preview = ImportPreview {
        version: "1".to_string(),
        drafts_count: 2,
        templates_count: 3,
        variables_count: 4,
        trees_count: 1,
        encrypted: false,
        path: Some("/tmp/backup.zip".to_string()),
    };

    let export_v = serde_json::to_value(export).expect("serialize export summary");
    let import_v = serde_json::to_value(import).expect("serialize import summary");
    let preview_v = serde_json::to_value(preview).expect("serialize import preview");

    assert_eq!(export_v["drafts_count"], 2);
    assert_eq!(export_v["encrypted"], true);
    assert_eq!(import_v["templates_imported"], 3);
    assert_eq!(preview_v["version"], "1");
    assert_eq!(preview_v["path"], "/tmp/backup.zip");
}

#[test]
fn jira_contract_types_serialize_with_expected_keys() {
    let ticket = JiraTicket {
        key: "IT-123".to_string(),
        summary: "VPN request".to_string(),
        description: Some("Need VPN access for travel".to_string()),
        status: "Open".to_string(),
        priority: Some("High".to_string()),
        assignee: Some("Alex".to_string()),
        reporter: "Jamie".to_string(),
        created: "2026-02-03T10:00:00Z".to_string(),
        updated: "2026-02-03T10:30:00Z".to_string(),
        issue_type: "Service Request".to_string(),
    };
    let config = JiraConfig {
        base_url: "https://example.atlassian.net".to_string(),
        email: "it@example.com".to_string(),
    };

    let ticket_v = serde_json::to_value(ticket).expect("serialize jira ticket");
    let config_v = serde_json::to_value(config).expect("serialize jira config");

    for key in [
        "key",
        "summary",
        "description",
        "status",
        "priority",
        "assignee",
        "reporter",
        "created",
        "updated",
        "issue_type",
    ] {
        assert!(ticket_v.get(key).is_some(), "missing ticket key: {key}");
    }

    assert_eq!(config_v["base_url"], "https://example.atlassian.net");
    assert_eq!(config_v["email"], "it@example.com");
}

#[test]
fn search_api_response_contract_roundtrip() {
    let response = HybridSearchResponse {
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

    let serialized = serde_json::to_string(&response).expect("serialize search response");
    let parsed: Value = serde_json::from_str(&serialized).expect("parse json");

    assert_eq!(parsed["status"], "success");
    assert_eq!(parsed["results_count"], 1);
    assert_eq!(parsed["results"][0]["scores"]["fused"], json!(0.9));

    let roundtrip: HybridSearchResponse =
        serde_json::from_str(&serialized).expect("deserialize search response");
    assert_eq!(roundtrip.intent, "POLICY");
    assert_eq!(roundtrip.results.len(), 1);
}

#[test]
fn export_format_deserialization_contract() {
    let markdown: ExportFormat = serde_json::from_value(json!("Markdown")).expect("markdown");
    let plain_text: ExportFormat = serde_json::from_value(json!("PlainText")).expect("plain");
    let html: ExportFormat = serde_json::from_value(json!("Html")).expect("html");

    assert!(matches!(markdown, ExportFormat::Markdown));
    assert!(matches!(plain_text, ExportFormat::PlainText));
    assert!(matches!(html, ExportFormat::Html));
}

#[tokio::test]
async fn submit_search_feedback_rejects_invalid_rating_before_network() {
    let err = commands::search_api::submit_search_feedback(
        "query-1".to_string(),
        1,
        "bad_rating".to_string(),
        Some("note".to_string()),
    )
    .await
    .expect_err("invalid rating should fail");

    assert!(err.contains("Invalid rating"));
}

#[test]
fn process_ocr_bytes_rejects_oversized_payload() {
    // MAX_OCR_BASE64_BYTES is 10MB in command implementation.
    let oversized = "A".repeat(10 * 1024 * 1024 + 1);
    match commands::process_ocr_bytes(oversized) {
        Ok(_) => panic!("oversized payload should fail"),
        Err(err) => assert!(err.contains("Image too large")),
    }
}
