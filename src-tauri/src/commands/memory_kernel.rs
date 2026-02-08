//! Tauri commands for MemoryKernel service integration.
//!
//! This module is the single outbound integration boundary for MemoryKernel.
//! Frontend code must call these Tauri commands instead of calling the service directly.

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

const MEMORY_KERNEL_ENABLE_ENV: &str = "ASSISTSUPPORT_ENABLE_MEMORY_KERNEL";
const MEMORY_KERNEL_BASE_URL_ENV: &str = "ASSISTSUPPORT_MEMORY_KERNEL_BASE_URL";
const MEMORY_KERNEL_TIMEOUT_MS_ENV: &str = "ASSISTSUPPORT_MEMORY_KERNEL_TIMEOUT_MS";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryKernelIntegrationPin {
    pub memorykernel_repo: String,
    pub release_tag: String,
    pub commit_sha: String,
    pub expected_service_contract_version: String,
    pub expected_api_contract_version: String,
    pub expected_integration_baseline: String,
    pub default_base_url: String,
    pub default_timeout_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryKernelPreflightStatus {
    pub enabled: bool,
    pub ready: bool,
    pub enrichment_enabled: bool,
    pub status: String,
    pub message: String,
    pub base_url: String,
    pub service_contract_version: Option<String>,
    pub api_contract_version: Option<String>,
    pub expected_service_contract_version: String,
    pub expected_api_contract_version: String,
    pub integration_baseline: String,
    pub release_tag: String,
    pub commit_sha: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryKernelEnrichmentResult {
    pub applied: bool,
    pub status: String,
    pub message: String,
    pub context_package_id: Option<String>,
    pub enrichment_text: Option<String>,
    pub preflight: MemoryKernelPreflightStatus,
}

#[derive(Debug, Serialize)]
struct QueryAskRequest {
    text: String,
    actor: String,
    action: String,
    resource: String,
}

#[derive(Debug, Deserialize)]
struct ServiceEnvelope<T> {
    service_contract_version: String,
    api_contract_version: String,
    data: T,
}

#[derive(Debug, Deserialize)]
struct HealthData {
    status: String,
}

static INTEGRATION_PIN: Lazy<MemoryKernelIntegrationPin> = Lazy::new(|| {
    serde_json::from_str(include_str!(
        "../../../config/memorykernel-integration-pin.json"
    ))
    .expect("invalid memorykernel integration pin manifest")
});

fn parse_bool_env(var: &str, default: bool) -> bool {
    std::env::var(var)
        .ok()
        .map(|value| value.trim().to_ascii_lowercase())
        .map(|value| matches!(value.as_str(), "1" | "true" | "yes" | "on"))
        .unwrap_or(default)
}

fn integration_enabled() -> bool {
    parse_bool_env(MEMORY_KERNEL_ENABLE_ENV, true)
}

fn integration_base_url(pin: &MemoryKernelIntegrationPin) -> String {
    std::env::var(MEMORY_KERNEL_BASE_URL_ENV)
        .ok()
        .map(|v| v.trim().trim_end_matches('/').to_string())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| pin.default_base_url.clone())
}

fn integration_timeout_ms(pin: &MemoryKernelIntegrationPin) -> u64 {
    std::env::var(MEMORY_KERNEL_TIMEOUT_MS_ENV)
        .ok()
        .and_then(|v| v.trim().parse::<u64>().ok())
        .map(|v| v.clamp(100, 30_000))
        .unwrap_or(pin.default_timeout_ms)
}

fn preflight_status_template(
    pin: &MemoryKernelIntegrationPin,
    enabled: bool,
    base_url: String,
) -> MemoryKernelPreflightStatus {
    MemoryKernelPreflightStatus {
        enabled,
        ready: false,
        enrichment_enabled: false,
        status: "disabled".to_string(),
        message: "MemoryKernel enrichment is disabled by configuration".to_string(),
        base_url,
        service_contract_version: None,
        api_contract_version: None,
        expected_service_contract_version: pin.expected_service_contract_version.clone(),
        expected_api_contract_version: pin.expected_api_contract_version.clone(),
        integration_baseline: pin.expected_integration_baseline.clone(),
        release_tag: pin.release_tag.clone(),
        commit_sha: pin.commit_sha.clone(),
    }
}

fn contracts_match(
    pin: &MemoryKernelIntegrationPin,
    service_contract_version: &str,
    api_contract_version: &str,
) -> bool {
    service_contract_version == pin.expected_service_contract_version
        && api_contract_version == pin.expected_api_contract_version
}

fn version_mismatch_status(
    mut status: MemoryKernelPreflightStatus,
    service_contract_version: String,
    api_contract_version: String,
) -> MemoryKernelPreflightStatus {
    status.status = "version-mismatch".to_string();
    status.message = format!(
        "MemoryKernel contract mismatch. Expected {}/{} but got {}/{}.",
        status.expected_service_contract_version,
        status.expected_api_contract_version,
        service_contract_version,
        api_contract_version
    );
    status.service_contract_version = Some(service_contract_version);
    status.api_contract_version = Some(api_contract_version);
    status
}

async fn run_preflight_internal(
    client: &reqwest::Client,
    pin: &MemoryKernelIntegrationPin,
    enabled: bool,
    base_url: &str,
) -> MemoryKernelPreflightStatus {
    let mut status = preflight_status_template(pin, enabled, base_url.to_string());
    if !enabled {
        return status;
    }

    status.status = "checking".to_string();
    status.message = "Running MemoryKernel preflight checks".to_string();

    let health_response = match client.get(format!("{base_url}/v1/health")).send().await {
        Ok(resp) => resp,
        Err(err) => {
            status.status = "offline".to_string();
            status.message = format!(
                "MemoryKernel service is unavailable at {}: {}",
                base_url, err
            );
            return status;
        }
    };

    let health_status = health_response.status();
    let health_body = health_response.text().await.unwrap_or_default();
    if !health_status.is_success() {
        status.status = "offline".to_string();
        status.message = format!(
            "MemoryKernel health check failed with HTTP {} at {}/v1/health",
            health_status.as_u16(),
            base_url
        );
        return status;
    }

    let health_envelope: ServiceEnvelope<HealthData> = match serde_json::from_str(&health_body) {
        Ok(payload) => payload,
        Err(_) => {
            status.status = "malformed-payload".to_string();
            status.message = format!(
                "MemoryKernel health payload is not valid JSON envelope at {}/v1/health",
                base_url
            );
            return status;
        }
    };

    if !contracts_match(
        pin,
        &health_envelope.service_contract_version,
        &health_envelope.api_contract_version,
    ) {
        return version_mismatch_status(
            status,
            health_envelope.service_contract_version,
            health_envelope.api_contract_version,
        );
    }

    status.service_contract_version = Some(health_envelope.service_contract_version.clone());
    status.api_contract_version = Some(health_envelope.api_contract_version.clone());

    if health_envelope.data.status.to_ascii_lowercase() != "ok" {
        status.status = "degraded".to_string();
        status.message = "MemoryKernel health endpoint did not report status=ok".to_string();
        return status;
    }

    let schema_response = match client
        .post(format!("{base_url}/v1/db/schema-version"))
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(err) => {
            status.status = "schema-unavailable".to_string();
            status.message = format!(
                "MemoryKernel schema check failed at {}/v1/db/schema-version: {}",
                base_url, err
            );
            return status;
        }
    };

    if !schema_response.status().is_success() {
        status.status = "schema-unavailable".to_string();
        status.message = format!(
            "MemoryKernel schema check failed with HTTP {} at {}/v1/db/schema-version",
            schema_response.status().as_u16(),
            base_url
        );
        return status;
    }

    let schema_body = schema_response.text().await.unwrap_or_default();
    let schema_envelope: ServiceEnvelope<serde_json::Value> =
        match serde_json::from_str(&schema_body) {
            Ok(payload) => payload,
            Err(_) => {
                status.status = "malformed-payload".to_string();
                status.message = format!(
                "MemoryKernel schema payload is not valid JSON envelope at {}/v1/db/schema-version",
                base_url
            );
                return status;
            }
        };

    if !contracts_match(
        pin,
        &schema_envelope.service_contract_version,
        &schema_envelope.api_contract_version,
    ) {
        return version_mismatch_status(
            status,
            schema_envelope.service_contract_version,
            schema_envelope.api_contract_version,
        );
    }

    status.ready = true;
    status.enrichment_enabled = true;
    status.status = "ready".to_string();
    status.message = "MemoryKernel preflight passed (health + schema checks)".to_string();
    status
}

fn build_enrichment_text(context_package: &serde_json::Value) -> Option<String> {
    let mut lines: Vec<String> = Vec::new();

    if let Some(result) = context_package
        .pointer("/answer/result")
        .and_then(serde_json::Value::as_str)
    {
        lines.push(format!("Policy decision: {}", result));
    }

    if let Some(why) = context_package
        .pointer("/answer/why")
        .and_then(serde_json::Value::as_str)
    {
        lines.push(format!("Why: {}", why));
    }

    if let Some(items) = context_package
        .get("selected_items")
        .and_then(serde_json::Value::as_array)
    {
        if !items.is_empty() {
            lines.push("Selected memory evidence:".to_string());
            for item in items.iter().take(3) {
                let record_type = item
                    .get("record_type")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("unknown");
                let memory_id = item
                    .get("memory_id")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("unknown");
                let reason = item
                    .pointer("/why/reasons")
                    .and_then(serde_json::Value::as_array)
                    .and_then(|reasons| reasons.first())
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("No reason provided");
                lines.push(format!("- [{record_type}] {memory_id}: {reason}"));
            }
        }
    }

    if lines.is_empty() {
        None
    } else {
        Some(lines.join("\n"))
    }
}

#[tauri::command]
pub async fn get_memory_kernel_integration_pin() -> Result<MemoryKernelIntegrationPin, String> {
    Ok(INTEGRATION_PIN.clone())
}

#[tauri::command]
pub async fn get_memory_kernel_preflight_status() -> Result<MemoryKernelPreflightStatus, String> {
    let pin = INTEGRATION_PIN.clone();
    let enabled = integration_enabled();
    let base_url = integration_base_url(&pin);
    let timeout_ms = integration_timeout_ms(&pin);
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_millis(timeout_ms))
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))?;
    Ok(run_preflight_internal(&client, &pin, enabled, &base_url).await)
}

#[tauri::command]
pub async fn memory_kernel_query_ask(
    user_input: String,
) -> Result<MemoryKernelEnrichmentResult, String> {
    let trimmed = user_input.trim();
    if trimmed.is_empty() {
        return Err("user_input cannot be empty".to_string());
    }

    let pin = INTEGRATION_PIN.clone();
    let enabled = integration_enabled();
    let base_url = integration_base_url(&pin);
    let timeout_ms = integration_timeout_ms(&pin);
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_millis(timeout_ms))
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))?;
    let preflight = run_preflight_internal(&client, &pin, enabled, &base_url).await;

    if !preflight.enrichment_enabled {
        return Ok(MemoryKernelEnrichmentResult {
            applied: false,
            status: "fallback".to_string(),
            message: preflight.message.clone(),
            context_package_id: None,
            enrichment_text: None,
            preflight,
        });
    }

    let request = QueryAskRequest {
        text: trimmed.to_string(),
        actor: "support_agent".to_string(),
        action: "resolve".to_string(),
        resource: "support_ticket".to_string(),
    };

    let response = match client
        .post(format!("{base_url}/v1/query/ask"))
        .json(&request)
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(err) => {
            return Ok(MemoryKernelEnrichmentResult {
                applied: false,
                status: "fallback".to_string(),
                message: format!("MemoryKernel query ask failed: {}", err),
                context_package_id: None,
                enrichment_text: None,
                preflight,
            });
        }
    };

    if !response.status().is_success() {
        let code = response.status();
        let body = response.text().await.unwrap_or_default();
        return Ok(MemoryKernelEnrichmentResult {
            applied: false,
            status: "fallback".to_string(),
            message: format!(
                "MemoryKernel query ask returned HTTP {}: {}",
                code.as_u16(),
                body
            ),
            context_package_id: None,
            enrichment_text: None,
            preflight,
        });
    }

    let body = response.text().await.unwrap_or_default();
    let envelope: ServiceEnvelope<serde_json::Value> = match serde_json::from_str(&body) {
        Ok(payload) => payload,
        Err(_) => {
            return Ok(MemoryKernelEnrichmentResult {
                applied: false,
                status: "fallback".to_string(),
                message: "MemoryKernel query ask returned malformed JSON envelope".to_string(),
                context_package_id: None,
                enrichment_text: None,
                preflight,
            });
        }
    };

    if !contracts_match(
        &pin,
        &envelope.service_contract_version,
        &envelope.api_contract_version,
    ) {
        return Ok(MemoryKernelEnrichmentResult {
            applied: false,
            status: "fallback".to_string(),
            message: format!(
                "MemoryKernel query ask contract mismatch (expected {}/{}, got {}/{})",
                pin.expected_service_contract_version,
                pin.expected_api_contract_version,
                envelope.service_contract_version,
                envelope.api_contract_version
            ),
            context_package_id: None,
            enrichment_text: None,
            preflight,
        });
    }

    let context_package_id = envelope
        .data
        .get("context_package_id")
        .and_then(serde_json::Value::as_str)
        .map(ToString::to_string);
    let enrichment_text = build_enrichment_text(&envelope.data);

    Ok(MemoryKernelEnrichmentResult {
        applied: enrichment_text.is_some(),
        status: if enrichment_text.is_some() {
            "applied".to_string()
        } else {
            "fallback".to_string()
        },
        message: if enrichment_text.is_some() {
            "MemoryKernel enrichment applied".to_string()
        } else {
            "MemoryKernel returned no actionable context items".to_string()
        },
        context_package_id,
        enrichment_text,
        preflight,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};
    use std::sync::Mutex;
    use std::thread;
    use std::time::Duration;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[derive(Clone)]
    struct MockResponse {
        method: &'static str,
        path: &'static str,
        status: u16,
        body: String,
        content_type: &'static str,
        delay_ms: u64,
    }

    fn reason_phrase(status: u16) -> &'static str {
        match status {
            200 => "OK",
            400 => "Bad Request",
            404 => "Not Found",
            500 => "Internal Server Error",
            _ => "OK",
        }
    }

    fn read_request(stream: &mut TcpStream) -> String {
        let mut buffer = [0_u8; 8192];
        let bytes = stream.read(&mut buffer).expect("failed to read request");
        String::from_utf8_lossy(&buffer[..bytes]).to_string()
    }

    fn spawn_mock_server(responses: Vec<MockResponse>) -> (String, thread::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("failed to bind test server");
        let addr = listener.local_addr().expect("failed to read local addr");
        let handle = thread::spawn(move || {
            for response in responses {
                let (mut stream, _) = listener.accept().expect("failed to accept connection");
                let request = read_request(&mut stream);
                let mut parts = request
                    .lines()
                    .next()
                    .unwrap_or_default()
                    .split_whitespace();
                let method = parts.next().unwrap_or_default();
                let path = parts.next().unwrap_or_default();
                assert_eq!(method, response.method);
                assert_eq!(path, response.path);

                if response.delay_ms > 0 {
                    thread::sleep(Duration::from_millis(response.delay_ms));
                }

                let payload = format!(
                    "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    response.status,
                    reason_phrase(response.status),
                    response.content_type,
                    response.body.len(),
                    response.body
                );
                stream
                    .write_all(payload.as_bytes())
                    .expect("failed to write response");
            }
        });
        (format!("http://{}", addr), handle)
    }

    fn set_test_env(base_url: &str, timeout_ms: u64, enabled: bool) {
        std::env::set_var(MEMORY_KERNEL_BASE_URL_ENV, base_url);
        std::env::set_var(MEMORY_KERNEL_TIMEOUT_MS_ENV, timeout_ms.to_string());
        std::env::set_var(
            MEMORY_KERNEL_ENABLE_ENV,
            if enabled { "true" } else { "false" },
        );
    }

    fn clear_test_env() {
        std::env::remove_var(MEMORY_KERNEL_BASE_URL_ENV);
        std::env::remove_var(MEMORY_KERNEL_TIMEOUT_MS_ENV);
        std::env::remove_var(MEMORY_KERNEL_ENABLE_ENV);
    }

    #[tokio::test]
    async fn preflight_reports_offline_when_service_down() {
        let _guard = ENV_LOCK.lock().expect("env lock poisoned");
        set_test_env("http://127.0.0.1:9", 200, true);

        let status = get_memory_kernel_preflight_status()
            .await
            .expect("preflight command should not fail");
        assert!(!status.ready);
        assert_eq!(status.status, "offline");

        clear_test_env();
    }

    #[tokio::test]
    async fn preflight_reports_timeout_as_offline() {
        let _guard = ENV_LOCK.lock().expect("env lock poisoned");
        let (base_url, handle) = spawn_mock_server(vec![MockResponse {
            method: "GET",
            path: "/v1/health",
            status: 200,
            body: format!(
                "{{\"service_contract_version\":\"{}\",\"api_contract_version\":\"{}\",\"data\":{{\"status\":\"ok\"}}}}",
                INTEGRATION_PIN.expected_service_contract_version,
                INTEGRATION_PIN.expected_api_contract_version
            ),
            content_type: "application/json",
            delay_ms: 200,
        }]);
        set_test_env(&base_url, 50, true);

        let status = get_memory_kernel_preflight_status()
            .await
            .expect("preflight command should not fail");
        assert!(!status.ready);
        assert_eq!(status.status, "offline");

        handle.join().expect("server thread panicked");
        clear_test_env();
    }

    #[tokio::test]
    async fn preflight_detects_version_mismatch() {
        let _guard = ENV_LOCK.lock().expect("env lock poisoned");
        let (base_url, handle) = spawn_mock_server(vec![MockResponse {
            method: "GET",
            path: "/v1/health",
            status: 200,
            body: "{\"service_contract_version\":\"service.v2\",\"api_contract_version\":\"api.v1\",\"data\":{\"status\":\"ok\"}}".to_string(),
            content_type: "application/json",
            delay_ms: 0,
        }]);
        set_test_env(&base_url, 500, true);

        let status = get_memory_kernel_preflight_status()
            .await
            .expect("preflight command should not fail");
        assert!(!status.ready);
        assert_eq!(status.status, "version-mismatch");

        handle.join().expect("server thread panicked");
        clear_test_env();
    }

    #[tokio::test]
    async fn preflight_detects_malformed_payload() {
        let _guard = ENV_LOCK.lock().expect("env lock poisoned");
        let (base_url, handle) = spawn_mock_server(vec![MockResponse {
            method: "GET",
            path: "/v1/health",
            status: 200,
            body: "<html>not-json</html>".to_string(),
            content_type: "text/html",
            delay_ms: 0,
        }]);
        set_test_env(&base_url, 500, true);

        let status = get_memory_kernel_preflight_status()
            .await
            .expect("preflight command should not fail");
        assert!(!status.ready);
        assert_eq!(status.status, "malformed-payload");

        handle.join().expect("server thread panicked");
        clear_test_env();
    }

    #[tokio::test]
    async fn query_ask_happy_path_applies_enrichment() {
        let _guard = ENV_LOCK.lock().expect("env lock poisoned");
        let health_body = format!(
            "{{\"service_contract_version\":\"{}\",\"api_contract_version\":\"{}\",\"data\":{{\"status\":\"ok\"}}}}",
            INTEGRATION_PIN.expected_service_contract_version,
            INTEGRATION_PIN.expected_api_contract_version
        );
        let schema_body = format!(
            "{{\"service_contract_version\":\"{}\",\"api_contract_version\":\"{}\",\"data\":{{\"current_version\":1}}}}",
            INTEGRATION_PIN.expected_service_contract_version,
            INTEGRATION_PIN.expected_api_contract_version
        );
        let query_body = format!(
            "{{\"service_contract_version\":\"{}\",\"api_contract_version\":\"{}\",\"data\":{{\"context_package_id\":\"ctx_123\",\"answer\":{{\"result\":\"allow\",\"why\":\"Policy allows managed USB under controls\"}},\"selected_items\":[{{\"record_type\":\"constraint\",\"memory_id\":\"usb-policy\",\"why\":{{\"reasons\":[\"authoritative policy\"]}}}}]}}}}",
            INTEGRATION_PIN.expected_service_contract_version,
            INTEGRATION_PIN.expected_api_contract_version
        );
        let (base_url, handle) = spawn_mock_server(vec![
            MockResponse {
                method: "GET",
                path: "/v1/health",
                status: 200,
                body: health_body,
                content_type: "application/json",
                delay_ms: 0,
            },
            MockResponse {
                method: "POST",
                path: "/v1/db/schema-version",
                status: 200,
                body: schema_body,
                content_type: "application/json",
                delay_ms: 0,
            },
            MockResponse {
                method: "POST",
                path: "/v1/query/ask",
                status: 200,
                body: query_body,
                content_type: "application/json",
                delay_ms: 0,
            },
        ]);
        set_test_env(&base_url, 750, true);

        let result = memory_kernel_query_ask("Can I use a USB drive?".to_string())
            .await
            .expect("query ask command should not fail");
        assert!(result.applied);
        assert_eq!(result.status, "applied");
        assert_eq!(result.context_package_id.as_deref(), Some("ctx_123"));
        assert!(result
            .enrichment_text
            .as_deref()
            .unwrap_or_default()
            .contains("Policy decision"));
        assert!(result.preflight.ready);

        handle.join().expect("server thread panicked");
        clear_test_env();
    }

    #[tokio::test]
    async fn query_ask_uses_deterministic_fallback_when_preflight_fails() {
        let _guard = ENV_LOCK.lock().expect("env lock poisoned");
        let (base_url, handle) = spawn_mock_server(vec![MockResponse {
            method: "GET",
            path: "/v1/health",
            status: 200,
            body: "{\"service_contract_version\":\"service.v2\",\"api_contract_version\":\"api.v1\",\"data\":{\"status\":\"ok\"}}".to_string(),
            content_type: "application/json",
            delay_ms: 0,
        }]);
        set_test_env(&base_url, 500, true);

        let result = memory_kernel_query_ask("Need policy guidance".to_string())
            .await
            .expect("query ask command should not fail");
        assert!(!result.applied);
        assert_eq!(result.status, "fallback");
        assert_eq!(result.preflight.status, "version-mismatch");
        assert!(result.enrichment_text.is_none());

        handle.join().expect("server thread panicked");
        clear_test_env();
    }

    #[test]
    fn build_enrichment_text_includes_answer_and_selected_items() {
        let value = serde_json::json!({
            "answer": {
                "result": "deny",
                "why": "Policy explicitly denies unencrypted removable media"
            },
            "selected_items": [
                {
                    "record_type": "constraint",
                    "memory_id": "removable-media-policy",
                    "why": {
                        "reasons": ["Policy scope matched actor/action/resource"]
                    }
                }
            ]
        });

        let text = build_enrichment_text(&value).expect("enrichment text should be present");
        assert!(text.contains("Policy decision: deny"));
        assert!(text.contains("removable-media-policy"));
    }

    #[test]
    fn disabled_feature_returns_disabled_preflight_template() {
        let pin = INTEGRATION_PIN.clone();
        let status = preflight_status_template(&pin, false, pin.default_base_url.clone());
        assert!(!status.enabled);
        assert!(!status.ready);
        assert_eq!(status.status, "disabled");
    }
}
