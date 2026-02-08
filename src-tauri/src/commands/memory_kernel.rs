//! Tauri commands for MemoryKernel service integration.
//!
//! This module is the single outbound integration boundary for MemoryKernel.
//! Frontend code must call these Tauri commands instead of calling the service directly.

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

const MEMORY_KERNEL_ENABLE_ENV: &str = "ASSISTSUPPORT_ENABLE_MEMORY_KERNEL";
const MEMORY_KERNEL_BASE_URL_ENV: &str = "ASSISTSUPPORT_MEMORY_KERNEL_BASE_URL";
const MEMORY_KERNEL_TIMEOUT_MS_ENV: &str = "ASSISTSUPPORT_MEMORY_KERNEL_TIMEOUT_MS";
const FALLBACK_REASON_FEATURE_DISABLED: &str = "feature-disabled";
const FALLBACK_REASON_OFFLINE: &str = "offline";
const FALLBACK_REASON_TIMEOUT: &str = "timeout";
const FALLBACK_REASON_MALFORMED_PAYLOAD: &str = "malformed-payload";
const FALLBACK_REASON_VERSION_MISMATCH: &str = "version-mismatch";
const FALLBACK_REASON_SCHEMA_UNAVAILABLE: &str = "schema-unavailable";
const FALLBACK_REASON_DEGRADED: &str = "degraded";
const FALLBACK_REASON_NON_2XX: &str = "non-2xx";
const FALLBACK_REASON_NETWORK_ERROR: &str = "network-error";
const FALLBACK_REASON_QUERY_ERROR: &str = "query-error";
const FALLBACK_REASON_EMPTY_CONTEXT: &str = "empty-context";
const FALLBACK_REASON_UNKNOWN: &str = "unknown";

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
    pub fallback_reason: Option<String>,
    pub machine_error_code: Option<String>,
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

#[derive(Debug, Deserialize)]
struct MachineReadableErrorEnvelope {
    error: MachineReadableError,
}

#[derive(Debug, Deserialize)]
struct MachineReadableError {
    code: String,
    #[allow(dead_code)]
    message: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LegacyErrorEnvelope {
    error: Option<serde_json::Value>,
    legacy_error: Option<serde_json::Value>,
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

fn preflight_fallback_reason(status: &str) -> &'static str {
    match status {
        "disabled" => FALLBACK_REASON_FEATURE_DISABLED,
        "offline" => FALLBACK_REASON_OFFLINE,
        "schema-unavailable" => FALLBACK_REASON_SCHEMA_UNAVAILABLE,
        "version-mismatch" => FALLBACK_REASON_VERSION_MISMATCH,
        "malformed-payload" => FALLBACK_REASON_MALFORMED_PAYLOAD,
        "degraded" => FALLBACK_REASON_DEGRADED,
        _ => FALLBACK_REASON_UNKNOWN,
    }
}

fn extract_machine_error(body: &str) -> Option<MachineReadableError> {
    serde_json::from_str::<MachineReadableErrorEnvelope>(body)
        .ok()
        .map(|payload| payload.error)
        .filter(|error| !error.code.trim().is_empty())
}

fn legacy_error_value_to_string(value: &serde_json::Value) -> Option<String> {
    if let Some(text) = value.as_str() {
        let trimmed = text.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }

    if let Some(object) = value.as_object() {
        for key in ["message", "error", "code"] {
            if let Some(text) = object.get(key).and_then(serde_json::Value::as_str) {
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    return Some(trimmed.to_string());
                }
            }
        }
    }

    None
}

fn extract_legacy_error_message(body: &str) -> Option<String> {
    serde_json::from_str::<LegacyErrorEnvelope>(body)
        .ok()
        .and_then(|payload| {
            payload
                .legacy_error
                .as_ref()
                .and_then(legacy_error_value_to_string)
                .or_else(|| payload.error.as_ref().and_then(legacy_error_value_to_string))
        })
}

fn normalize_machine_error_code(code: &str) -> &'static str {
    match code.trim().to_ascii_lowercase().as_str() {
        "invalid_json" | "validation_failed" | "validation_error" | "invalid_request" => {
            "validation-error"
        }
        "context_package_not_found" => "context-not-found",
        "write_conflict" => "write-conflict",
        "schema_unavailable" | "schema_mismatch" => FALLBACK_REASON_SCHEMA_UNAVAILABLE,
        "version_mismatch" | "contract_mismatch" => FALLBACK_REASON_VERSION_MISMATCH,
        "service_unavailable" | "upstream_unavailable" => FALLBACK_REASON_OFFLINE,
        "timeout" | "upstream_timeout" => FALLBACK_REASON_TIMEOUT,
        _ => FALLBACK_REASON_NON_2XX,
    }
}

fn fallback_reason_from_status_code(status_code: reqwest::StatusCode) -> &'static str {
    match status_code {
        reqwest::StatusCode::REQUEST_TIMEOUT | reqwest::StatusCode::GATEWAY_TIMEOUT => {
            FALLBACK_REASON_TIMEOUT
        }
        reqwest::StatusCode::BAD_GATEWAY
        | reqwest::StatusCode::SERVICE_UNAVAILABLE
        | reqwest::StatusCode::TOO_MANY_REQUESTS => FALLBACK_REASON_OFFLINE,
        reqwest::StatusCode::BAD_REQUEST | reqwest::StatusCode::UNPROCESSABLE_ENTITY => {
            "validation-error"
        }
        _ => FALLBACK_REASON_NON_2XX,
    }
}

fn fallback_reason_for_query_error(err: &reqwest::Error) -> &'static str {
    if err.is_timeout() {
        FALLBACK_REASON_TIMEOUT
    } else if err.is_connect() {
        FALLBACK_REASON_OFFLINE
    } else if err.is_request() || err.is_body() || err.is_decode() {
        FALLBACK_REASON_QUERY_ERROR
    } else {
        FALLBACK_REASON_NETWORK_ERROR
    }
}

fn fallback_result(
    preflight: MemoryKernelPreflightStatus,
    message: String,
    fallback_reason: &'static str,
    machine_error_code: Option<String>,
) -> MemoryKernelEnrichmentResult {
    MemoryKernelEnrichmentResult {
        applied: false,
        status: "fallback".to_string(),
        message,
        fallback_reason: Some(fallback_reason.to_string()),
        machine_error_code,
        context_package_id: None,
        enrichment_text: None,
        preflight,
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
        return Ok(fallback_result(
            preflight.clone(),
            preflight.message.clone(),
            preflight_fallback_reason(&preflight.status),
            None,
        ));
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
            return Ok(fallback_result(
                preflight,
                format!("MemoryKernel query ask failed: {}", err),
                fallback_reason_for_query_error(&err),
                None,
            ));
        }
    };

    if !response.status().is_success() {
        let code = response.status();
        let body = response.text().await.unwrap_or_default();
        let machine_error = extract_machine_error(&body);
        let machine_code = machine_error.as_ref().map(|error| error.code.clone());
        let legacy_error = extract_legacy_error_message(&body);
        let fallback_reason = machine_code
            .as_deref()
            .map(normalize_machine_error_code)
            .unwrap_or_else(|| fallback_reason_from_status_code(code));
        let message = match machine_code.as_deref() {
            Some(error_code) => match legacy_error.as_deref() {
                Some(legacy_message) => format!(
                    "MemoryKernel query ask returned HTTP {} [{}]: {} (legacy_error: {})",
                    code.as_u16(),
                    error_code,
                    body,
                    legacy_message
                ),
                None => format!(
                    "MemoryKernel query ask returned HTTP {} [{}]: {}",
                    code.as_u16(),
                    error_code,
                    body
                ),
            },
            None => match legacy_error.as_deref() {
                Some(legacy_message) => format!(
                    "MemoryKernel query ask returned HTTP {}: {} (legacy_error: {})",
                    code.as_u16(),
                    body,
                    legacy_message
                ),
                None => format!(
                    "MemoryKernel query ask returned HTTP {}: {}",
                    code.as_u16(),
                    body
                ),
            },
        };
        return Ok(fallback_result(
            preflight,
            message,
            fallback_reason,
            machine_code,
        ));
    }

    let body = response.text().await.unwrap_or_default();
    let envelope: ServiceEnvelope<serde_json::Value> = match serde_json::from_str(&body) {
        Ok(payload) => payload,
        Err(_) => {
            return Ok(fallback_result(
                preflight,
                "MemoryKernel query ask returned malformed JSON envelope".to_string(),
                FALLBACK_REASON_MALFORMED_PAYLOAD,
                None,
            ));
        }
    };

    if !contracts_match(
        &pin,
        &envelope.service_contract_version,
        &envelope.api_contract_version,
    ) {
        return Ok(fallback_result(
            preflight,
            format!(
                "MemoryKernel query ask contract mismatch (expected {}/{}, got {}/{})",
                pin.expected_service_contract_version,
                pin.expected_api_contract_version,
                envelope.service_contract_version,
                envelope.api_contract_version
            ),
            FALLBACK_REASON_VERSION_MISMATCH,
            None,
        ));
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
        fallback_reason: if enrichment_text.is_some() {
            None
        } else {
            Some(FALLBACK_REASON_EMPTY_CONTEXT.to_string())
        },
        machine_error_code: None,
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

    fn fixture_health_ok() -> String {
        format!(
            "{{\"service_contract_version\":\"{}\",\"api_contract_version\":\"{}\",\"data\":{{\"status\":\"ok\"}}}}",
            INTEGRATION_PIN.expected_service_contract_version,
            INTEGRATION_PIN.expected_api_contract_version
        )
    }

    fn fixture_schema_ok() -> String {
        format!(
            "{{\"service_contract_version\":\"{}\",\"api_contract_version\":\"{}\",\"data\":{{\"current_version\":1}}}}",
            INTEGRATION_PIN.expected_service_contract_version,
            INTEGRATION_PIN.expected_api_contract_version
        )
    }

    fn fixture_query_allow() -> String {
        format!(
            "{{\"service_contract_version\":\"{}\",\"api_contract_version\":\"{}\",\"data\":{{\"context_package_id\":\"ctx_123\",\"answer\":{{\"result\":\"allow\",\"why\":\"Policy allows managed USB under controls\"}},\"selected_items\":[{{\"record_type\":\"constraint\",\"memory_id\":\"usb-policy\",\"why\":{{\"reasons\":[\"authoritative policy\"]}}}}]}}}}",
            INTEGRATION_PIN.expected_service_contract_version,
            INTEGRATION_PIN.expected_api_contract_version
        )
    }

    fn fixture_transitional_legacy_error(code: &str, message: &str, legacy_message: &str) -> String {
        format!(
            "{{\"service_contract_version\":\"{}\",\"error\":{{\"code\":\"{}\",\"message\":\"{}\"}},\"legacy_error\":\"{}\"}}",
            INTEGRATION_PIN.expected_service_contract_version, code, message, legacy_message
        )
    }

    fn fixture_typed_error(code: &str, message: &str) -> String {
        if INTEGRATION_PIN.expected_service_contract_version == "service.v2" {
            format!(
                "{{\"service_contract_version\":\"{}\",\"error\":{{\"code\":\"{}\",\"message\":\"{}\"}},\"legacy_error\":\"{}\"}}",
                INTEGRATION_PIN.expected_service_contract_version, code, message, message
            )
        } else {
            format!(
                "{{\"service_contract_version\":\"{}\",\"error\":{{\"code\":\"{}\",\"message\":\"{}\"}}}}",
                INTEGRATION_PIN.expected_service_contract_version, code, message
            )
        }
    }

    fn mismatched_service_contract_version() -> &'static str {
        if INTEGRATION_PIN.expected_service_contract_version == "service.v3" {
            "service.v2"
        } else {
            "service.v3"
        }
    }

    fn assert_non_2xx_envelope_policy(body: &str, expected_code: &str, allow_legacy_error: bool) {
        let value: serde_json::Value =
            serde_json::from_str(body).expect("error fixture should be valid JSON");
        assert_eq!(
            value
                .get("service_contract_version")
                .and_then(serde_json::Value::as_str),
            Some(INTEGRATION_PIN.expected_service_contract_version.as_str())
        );
        assert!(
            value.get("api_contract_version").is_none(),
            "non-2xx envelope must not include api_contract_version"
        );
        let error = value
            .get("error")
            .and_then(serde_json::Value::as_object)
            .expect("error object missing");
        assert_eq!(
            error.get("code").and_then(serde_json::Value::as_str),
            Some(expected_code)
        );
        assert!(
            error
                .get("message")
                .and_then(serde_json::Value::as_str)
                .is_some(),
            "error.message must be present"
        );
        if INTEGRATION_PIN.expected_service_contract_version == "service.v2" || allow_legacy_error {
            assert!(
                value
                    .get("legacy_error")
                    .and_then(serde_json::Value::as_str)
                    .is_some(),
                "legacy_error must be present for service.v2 compatibility"
            );
        } else {
            assert!(
                value.get("legacy_error").is_none(),
                "legacy_error must be absent for service.v3 compatibility"
            );
        }
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
            body: fixture_health_ok(),
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
                body: format!(
                    "{{\"service_contract_version\":\"{}\",\"api_contract_version\":\"api.v1\",\"data\":{{\"status\":\"ok\"}}}}",
                    mismatched_service_contract_version()
                ),
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
        let health_body = fixture_health_ok();
        let schema_body = fixture_schema_ok();
        let query_body = fixture_query_allow();
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
        assert!(result.fallback_reason.is_none());
        assert!(result.machine_error_code.is_none());
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
                body: format!(
                    "{{\"service_contract_version\":\"{}\",\"api_contract_version\":\"api.v1\",\"data\":{{\"status\":\"ok\"}}}}",
                    mismatched_service_contract_version()
                ),
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
        assert_eq!(
            result.fallback_reason.as_deref(),
            Some(FALLBACK_REASON_VERSION_MISMATCH)
        );
        assert!(result.enrichment_text.is_none());

        handle.join().expect("server thread panicked");
        clear_test_env();
    }

    #[tokio::test]
    async fn query_ask_keeps_deterministic_fallback_on_non_2xx_response() {
        let _guard = ENV_LOCK.lock().expect("env lock poisoned");
        let health_body = fixture_health_ok();
        let schema_body = fixture_schema_ok();
        let error_body = fixture_typed_error("validation_error", "validation failed");
        assert_non_2xx_envelope_policy(&error_body, "validation_error", false);
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
                status: 400,
                body: error_body,
                content_type: "application/json",
                delay_ms: 0,
            },
        ]);
        set_test_env(&base_url, 750, true);

        let result = memory_kernel_query_ask("Need decision support".to_string())
            .await
            .expect("query ask command should not fail");
        assert!(!result.applied);
        assert_eq!(result.status, "fallback");
        assert_eq!(result.fallback_reason.as_deref(), Some("validation-error"));
        assert!(result.message.contains("HTTP 400"));
        assert!(result.enrichment_text.is_none());

        handle.join().expect("server thread panicked");
        clear_test_env();
    }

    #[tokio::test]
    async fn query_ask_includes_machine_readable_error_code_in_fallback_message() {
        let _guard = ENV_LOCK.lock().expect("env lock poisoned");
        let health_body = fixture_health_ok();
        let schema_body = fixture_schema_ok();
        let error_body = fixture_typed_error("validation_failed", "Invalid query");
        assert_non_2xx_envelope_policy(&error_body, "validation_failed", false);
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
                status: 400,
                body: error_body,
                content_type: "application/json",
                delay_ms: 0,
            },
        ]);
        set_test_env(&base_url, 750, true);

        let result = memory_kernel_query_ask("Need decision support".to_string())
            .await
            .expect("query ask command should not fail");
        assert!(!result.applied);
        assert_eq!(result.status, "fallback");
        assert_eq!(result.fallback_reason.as_deref(), Some("validation-error"));
        assert_eq!(
            result.machine_error_code.as_deref(),
            Some("validation_failed")
        );
        assert!(result.message.contains("[validation_failed]"));
        assert!(result.enrichment_text.is_none());

        handle.join().expect("server thread panicked");
        clear_test_env();
    }

    #[tokio::test]
    async fn query_ask_preserves_transitional_legacy_error_compatibility() {
        let _guard = ENV_LOCK.lock().expect("env lock poisoned");
        let health_body = fixture_health_ok();
        let schema_body = fixture_schema_ok();
        let error_body =
            fixture_transitional_legacy_error("validation_error", "Invalid query", "validation failed");
        assert_non_2xx_envelope_policy(&error_body, "validation_error", true);
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
                status: 400,
                body: error_body,
                content_type: "application/json",
                delay_ms: 0,
            },
        ]);
        set_test_env(&base_url, 750, true);

        let result = memory_kernel_query_ask("Need decision support".to_string())
            .await
            .expect("query ask command should not fail");
        assert_eq!(result.status, "fallback");
        assert_eq!(result.fallback_reason.as_deref(), Some("validation-error"));
        assert_eq!(result.machine_error_code.as_deref(), Some("validation_error"));
        assert!(result.message.contains("legacy_error: validation failed"));

        handle.join().expect("server thread panicked");
        clear_test_env();
    }

    #[tokio::test]
    async fn query_ask_can_map_machine_error_codes_to_specific_consumer_states() {
        let _guard = ENV_LOCK.lock().expect("env lock poisoned");
        let health_body = fixture_health_ok();
        let schema_body = fixture_schema_ok();
        let error_body = fixture_typed_error("validation_failed", "Invalid query");
        assert_non_2xx_envelope_policy(&error_body, "validation_failed", false);
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
                status: 400,
                body: error_body,
                content_type: "application/json",
                delay_ms: 0,
            },
        ]);
        set_test_env(&base_url, 750, true);

        let result = memory_kernel_query_ask("Need decision support".to_string())
            .await
            .expect("query ask command should not fail");
        assert_eq!(result.status, "fallback");
        assert_eq!(result.fallback_reason.as_deref(), Some("validation-error"));
        assert_eq!(
            result.machine_error_code.as_deref(),
            Some("validation_failed")
        );

        handle.join().expect("server thread panicked");
        clear_test_env();
    }

    #[test]
    fn normalize_machine_error_code_covers_service_v2_codes() {
        assert_eq!(
            normalize_machine_error_code("invalid_json"),
            "validation-error"
        );
        assert_eq!(
            normalize_machine_error_code("validation_error"),
            "validation-error"
        );
        assert_eq!(
            normalize_machine_error_code("context_package_not_found"),
            "context-not-found"
        );
        assert_eq!(
            normalize_machine_error_code("write_conflict"),
            "write-conflict"
        );
        assert_eq!(
            normalize_machine_error_code("schema_unavailable"),
            FALLBACK_REASON_SCHEMA_UNAVAILABLE
        );
        assert_eq!(
            normalize_machine_error_code("write_failed"),
            FALLBACK_REASON_NON_2XX
        );
        assert_eq!(
            normalize_machine_error_code("migration_failed"),
            FALLBACK_REASON_NON_2XX
        );
        assert_eq!(
            normalize_machine_error_code("query_failed"),
            FALLBACK_REASON_NON_2XX
        );
        assert_eq!(
            normalize_machine_error_code("context_lookup_failed"),
            FALLBACK_REASON_NON_2XX
        );
        assert_eq!(
            normalize_machine_error_code("internal_error"),
            FALLBACK_REASON_NON_2XX
        );
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
