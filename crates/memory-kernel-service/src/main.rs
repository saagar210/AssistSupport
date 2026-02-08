use std::net::SocketAddr;
use std::path::PathBuf;

use anyhow::Result;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use clap::Parser;
use memory_kernel_api::{
    AddConstraintRequest, AddLinkRequest, AddSummaryRequest, AskRequest, MemoryKernelApi,
    RecallRequest, API_CONTRACT_VERSION,
};
use serde::{Deserialize, Serialize};

const SERVICE_CONTRACT_VERSION: &str = "service.v1";
const OPENAPI_YAML: &str = include_str!("../../../openapi/openapi.yaml");

#[derive(Debug, Clone)]
struct ServiceState {
    api: MemoryKernelApi,
}

#[derive(Debug, Clone, Serialize)]
struct ServiceEnvelope<T>
where
    T: Serialize,
{
    service_contract_version: &'static str,
    api_contract_version: &'static str,
    data: T,
}

#[derive(Debug, Clone, Serialize)]
struct ServiceError {
    service_contract_version: &'static str,
    error: String,
}

#[derive(Debug, Clone, Deserialize)]
struct MigrateRequest {
    dry_run: bool,
}

#[derive(Debug, Clone, Serialize)]
struct HealthResponse {
    status: &'static str,
}

#[derive(Debug, Parser)]
#[command(name = "memory-kernel-service")]
#[command(about = "Local HTTP service for Memory Kernel")]
struct Args {
    #[arg(long, default_value = "./memory_kernel.sqlite3")]
    db: PathBuf,
    #[arg(long, default_value = "127.0.0.1:4010")]
    bind: SocketAddr,
}

impl IntoResponse for ServiceError {
    fn into_response(self) -> Response {
        let status = StatusCode::BAD_REQUEST;
        (status, Json(self)).into_response()
    }
}

impl ServiceState {
    fn error(message: impl Into<String>) -> ServiceError {
        ServiceError { service_contract_version: SERVICE_CONTRACT_VERSION, error: message.into() }
    }
}

fn envelope<T>(data: T) -> ServiceEnvelope<T>
where
    T: Serialize,
{
    ServiceEnvelope {
        service_contract_version: SERVICE_CONTRACT_VERSION,
        api_contract_version: API_CONTRACT_VERSION,
        data,
    }
}

fn app(state: ServiceState) -> Router {
    Router::new()
        .route("/v1/health", get(health))
        .route("/v1/openapi", get(openapi))
        .route("/v1/db/schema-version", post(db_schema_version))
        .route("/v1/db/migrate", post(db_migrate))
        .route("/v1/memory/add/constraint", post(memory_add_constraint))
        .route("/v1/memory/add/summary", post(memory_add_summary))
        .route("/v1/memory/link", post(memory_link))
        .route("/v1/query/ask", post(query_ask))
        .route("/v1/query/recall", post(query_recall))
        .route("/v1/context/:context_package_id", get(context_show))
        .with_state(state)
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let state = ServiceState { api: MemoryKernelApi::new(args.db) };
    let listener = tokio::net::TcpListener::bind(args.bind).await?;
    axum::serve(listener, app(state)).await?;
    Ok(())
}

async fn health() -> Json<ServiceEnvelope<HealthResponse>> {
    Json(envelope(HealthResponse { status: "ok" }))
}

async fn openapi() -> impl IntoResponse {
    (StatusCode::OK, [("content-type", "application/yaml; charset=utf-8")], OPENAPI_YAML)
}

async fn db_schema_version(
    State(state): State<ServiceState>,
) -> Result<Json<ServiceEnvelope<memory_kernel_store_sqlite::SchemaStatus>>, ServiceError> {
    let status = state.api.schema_status().map_err(|err| ServiceState::error(err.to_string()))?;
    Ok(Json(envelope(status)))
}

async fn db_migrate(
    State(state): State<ServiceState>,
    Json(request): Json<MigrateRequest>,
) -> Result<Json<ServiceEnvelope<memory_kernel_api::MigrateResult>>, ServiceError> {
    let result =
        state.api.migrate(request.dry_run).map_err(|err| ServiceState::error(err.to_string()))?;
    Ok(Json(envelope(result)))
}

async fn memory_add_constraint(
    State(state): State<ServiceState>,
    Json(request): Json<AddConstraintRequest>,
) -> Result<Json<ServiceEnvelope<memory_kernel_core::MemoryRecord>>, ServiceError> {
    let record =
        state.api.add_constraint(request).map_err(|err| ServiceState::error(err.to_string()))?;
    Ok(Json(envelope(record)))
}

async fn memory_add_summary(
    State(state): State<ServiceState>,
    Json(request): Json<AddSummaryRequest>,
) -> Result<Json<ServiceEnvelope<memory_kernel_core::MemoryRecord>>, ServiceError> {
    let record =
        state.api.add_summary(request).map_err(|err| ServiceState::error(err.to_string()))?;
    Ok(Json(envelope(record)))
}

async fn memory_link(
    State(state): State<ServiceState>,
    Json(request): Json<AddLinkRequest>,
) -> Result<Json<ServiceEnvelope<memory_kernel_api::AddLinkResult>>, ServiceError> {
    let result = state.api.add_link(request).map_err(|err| ServiceState::error(err.to_string()))?;
    Ok(Json(envelope(result)))
}

async fn query_ask(
    State(state): State<ServiceState>,
    Json(request): Json<AskRequest>,
) -> Result<Json<ServiceEnvelope<memory_kernel_core::ContextPackage>>, ServiceError> {
    let package =
        state.api.query_ask(request).map_err(|err| ServiceState::error(err.to_string()))?;
    Ok(Json(envelope(package)))
}

async fn query_recall(
    State(state): State<ServiceState>,
    Json(request): Json<RecallRequest>,
) -> Result<Json<ServiceEnvelope<memory_kernel_core::ContextPackage>>, ServiceError> {
    let package =
        state.api.query_recall(request).map_err(|err| ServiceState::error(err.to_string()))?;
    Ok(Json(envelope(package)))
}

async fn context_show(
    State(state): State<ServiceState>,
    Path(context_package_id): Path<String>,
) -> Result<Json<ServiceEnvelope<memory_kernel_core::ContextPackage>>, ServiceError> {
    let package = state
        .api
        .context_show(&context_package_id)
        .map_err(|err| ServiceState::error(err.to_string()))?;
    Ok(Json(envelope(package)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use http::Request;
    use tower::ServiceExt;

    fn unique_temp_db_path() -> PathBuf {
        std::env::temp_dir().join(format!("memorykernel-service-{}.sqlite3", ulid::Ulid::new()))
    }

    async fn response_json(response: Response) -> serde_json::Value {
        let bytes = match to_bytes(response.into_body(), 1024 * 1024).await {
            Ok(bytes) => bytes,
            Err(err) => panic!("failed to read response body: {err}"),
        };
        let body = match String::from_utf8(bytes.to_vec()) {
            Ok(body) => body,
            Err(err) => panic!("response body is not UTF-8: {err}"),
        };
        match serde_json::from_str(&body) {
            Ok(value) => value,
            Err(err) => panic!("response body is not JSON: {err}; body={body}"),
        }
    }

    // Test IDs: TSVC-001
    #[tokio::test]
    async fn health_endpoint_reports_ok() {
        let state = ServiceState { api: MemoryKernelApi::new(unique_temp_db_path()) };
        let router = app(state);

        let response = match router
            .oneshot(
                Request::builder()
                    .uri("/v1/health")
                    .method("GET")
                    .body(axum::body::Body::empty())
                    .unwrap_or_else(|err| panic!("failed to build request: {err}")),
            )
            .await
        {
            Ok(response) => response,
            Err(err) => panic!("router request failed: {err}"),
        };
        assert_eq!(response.status(), StatusCode::OK);

        let value = response_json(response).await;
        assert_eq!(
            value.get("service_contract_version").and_then(serde_json::Value::as_str),
            Some(SERVICE_CONTRACT_VERSION)
        );
    }

    // Test IDs: TSVC-003
    #[tokio::test]
    async fn openapi_endpoint_returns_versioned_artifact() {
        let state = ServiceState { api: MemoryKernelApi::new(unique_temp_db_path()) };
        let router = app(state);

        let response = match router
            .oneshot(
                Request::builder()
                    .uri("/v1/openapi")
                    .method("GET")
                    .body(axum::body::Body::empty())
                    .unwrap_or_else(|err| panic!("failed to build request: {err}")),
            )
            .await
        {
            Ok(response) => response,
            Err(err) => panic!("router request failed: {err}"),
        };
        assert_eq!(response.status(), StatusCode::OK);

        let bytes = match to_bytes(response.into_body(), 1024 * 1024).await {
            Ok(bytes) => bytes,
            Err(err) => panic!("failed to read response body: {err}"),
        };
        let body = match String::from_utf8(bytes.to_vec()) {
            Ok(body) => body,
            Err(err) => panic!("response body is not UTF-8: {err}"),
        };
        assert!(body.contains("openapi: 3.1.0"));
        assert!(body.contains("version: service.v1"));
        assert!(body.contains("/v1/memory/add/summary"));
        assert!(body.contains("/v1/query/recall"));
    }

    // Test IDs: TSVC-002
    #[tokio::test]
    async fn service_add_query_and_context_flow_round_trip() {
        let db_path = unique_temp_db_path();
        let state = ServiceState { api: MemoryKernelApi::new(db_path.clone()) };
        let router = app(state);

        let add_payload = serde_json::json!({
            "actor": "user",
            "action": "use",
            "resource": "usb_drive",
            "effect": "deny",
            "note": null,
            "memory_id": null,
            "version": 1,
            "writer": "tester",
            "justification": "service fixture",
            "source_uri": "file:///policy.md",
            "source_hash": "sha256:abc123",
            "evidence": [],
            "confidence": 0.9,
            "truth_status": "asserted",
            "authority": "authoritative",
            "created_at": null,
            "effective_at": null,
            "supersedes": [],
            "contradicts": []
        });

        let add_response = match router
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/v1/memory/add/constraint")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(add_payload.to_string()))
                    .unwrap_or_else(|err| panic!("failed to build add request: {err}")),
            )
            .await
        {
            Ok(response) => response,
            Err(err) => panic!("add request failed: {err}"),
        };
        assert_eq!(add_response.status(), StatusCode::OK);

        let ask_payload = serde_json::json!({
            "text": "Am I allowed to use a USB drive?",
            "actor": "user",
            "action": "use",
            "resource": "usb_drive",
            "as_of": null
        });
        let ask_response = match router
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/v1/query/ask")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(ask_payload.to_string()))
                    .unwrap_or_else(|err| panic!("failed to build ask request: {err}")),
            )
            .await
        {
            Ok(response) => response,
            Err(err) => panic!("ask request failed: {err}"),
        };
        assert_eq!(ask_response.status(), StatusCode::OK);
        let ask_value = response_json(ask_response).await;
        let context_id = ask_value
            .get("data")
            .and_then(|data| data.get("context_package_id"))
            .and_then(serde_json::Value::as_str)
            .unwrap_or_else(|| panic!("missing data.context_package_id in response: {ask_value}"))
            .to_string();

        let context_response = match router
            .oneshot(
                Request::builder()
                    .uri(format!("/v1/context/{context_id}"))
                    .method("GET")
                    .body(axum::body::Body::empty())
                    .unwrap_or_else(|err| panic!("failed to build context request: {err}")),
            )
            .await
        {
            Ok(response) => response,
            Err(err) => panic!("context request failed: {err}"),
        };
        assert_eq!(context_response.status(), StatusCode::OK);
        let context_value = response_json(context_response).await;
        assert_eq!(
            context_value
                .get("data")
                .and_then(|data| data.get("context_package_id"))
                .and_then(serde_json::Value::as_str),
            Some(context_id.as_str())
        );

        let _ = std::fs::remove_file(&db_path);
    }

    // Test IDs: TSVC-004
    #[tokio::test]
    async fn service_summary_add_and_recall_flow_round_trip() {
        let db_path = unique_temp_db_path();
        let state = ServiceState { api: MemoryKernelApi::new(db_path.clone()) };
        let router = app(state);

        let add_summary_payload = serde_json::json!({
            "record_type": "decision",
            "summary": "Decision: USB devices require explicit approval",
            "memory_id": null,
            "version": 1,
            "writer": "tester",
            "justification": "service recall fixture",
            "source_uri": "file:///decision.md",
            "source_hash": "sha256:abc123",
            "evidence": [],
            "confidence": 0.8,
            "truth_status": "observed",
            "authority": "authoritative",
            "created_at": null,
            "effective_at": null,
            "supersedes": [],
            "contradicts": []
        });

        let add_response = match router
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/v1/memory/add/summary")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(add_summary_payload.to_string()))
                    .unwrap_or_else(|err| panic!("failed to build summary add request: {err}")),
            )
            .await
        {
            Ok(response) => response,
            Err(err) => panic!("summary add request failed: {err}"),
        };
        assert_eq!(add_response.status(), StatusCode::OK);

        let recall_payload = serde_json::json!({
            "text": "usb approval",
            "record_types": ["decision", "outcome"],
            "as_of": null
        });
        let recall_response = match router
            .oneshot(
                Request::builder()
                    .uri("/v1/query/recall")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(recall_payload.to_string()))
                    .unwrap_or_else(|err| panic!("failed to build recall request: {err}")),
            )
            .await
        {
            Ok(response) => response,
            Err(err) => panic!("recall request failed: {err}"),
        };
        assert_eq!(recall_response.status(), StatusCode::OK);

        let recall_value = response_json(recall_response).await;
        assert_eq!(
            recall_value
                .get("data")
                .and_then(|data| data.get("determinism"))
                .and_then(|determinism| determinism.get("ruleset_version"))
                .and_then(serde_json::Value::as_str),
            Some("recall-ordering.v1")
        );

        let _ = std::fs::remove_file(&db_path);
    }
}
