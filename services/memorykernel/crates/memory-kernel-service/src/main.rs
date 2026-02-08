use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use axum::extract::rejection::JsonRejection;
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
use serde_json::json;

const SERVICE_CONTRACT_VERSION: &str = "service.v3";
const OPENAPI_YAML: &str = include_str!("../../../openapi/openapi.yaml");

#[derive(Debug, Clone)]
struct ServiceState {
    api: MemoryKernelApi,
    operation_timeout: Duration,
    telemetry: Arc<ServiceTelemetry>,
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
    error: ServiceErrorPayload,
}

#[derive(Debug, Clone, Serialize)]
struct ServiceErrorPayload {
    code: &'static str,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<serde_json::Value>,
}

#[derive(Debug, Clone)]
struct ServiceFailure {
    status: StatusCode,
    code: &'static str,
    message: String,
    details: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
struct MigrateRequest {
    dry_run: bool,
}

#[derive(Debug, Clone, Serialize)]
struct HealthResponse {
    status: &'static str,
    timeout_ms: u64,
    telemetry: ServiceTelemetrySnapshot,
}

#[derive(Debug, Default)]
#[allow(clippy::struct_field_names)]
struct ServiceTelemetry {
    requests_total: AtomicU64,
    requests_success_total: AtomicU64,
    requests_failure_total: AtomicU64,
    timeout_total: AtomicU64,
    invalid_json_total: AtomicU64,
    validation_error_total: AtomicU64,
    context_not_found_total: AtomicU64,
    write_conflict_total: AtomicU64,
    schema_unavailable_total: AtomicU64,
    internal_error_total: AtomicU64,
    other_error_total: AtomicU64,
}

#[derive(Debug, Clone, Serialize)]
#[allow(clippy::struct_field_names)]
struct ServiceTelemetrySnapshot {
    requests_total: u64,
    requests_success_total: u64,
    requests_failure_total: u64,
    timeout_total: u64,
    invalid_json_total: u64,
    validation_error_total: u64,
    context_not_found_total: u64,
    write_conflict_total: u64,
    schema_unavailable_total: u64,
    internal_error_total: u64,
    other_error_total: u64,
}

#[derive(Debug, Clone, Serialize)]
struct ReadinessChecks {
    current_schema_version: i64,
    target_schema_version: i64,
    pending_migrations: usize,
    inferred_from_legacy: bool,
}

#[derive(Debug, Clone, Serialize)]
struct ReadinessResponse {
    status: &'static str,
    checks: ReadinessChecks,
}

#[derive(Debug, Parser)]
#[command(name = "memory-kernel-service")]
#[command(about = "Local HTTP service for Memory Kernel")]
struct Args {
    #[arg(long, default_value = "./memory_kernel.sqlite3")]
    db: PathBuf,
    #[arg(long, default_value = "127.0.0.1:4010")]
    bind: SocketAddr,
    #[arg(long, default_value_t = 2500)]
    operation_timeout_ms: u64,
}

impl IntoResponse for ServiceFailure {
    fn into_response(self) -> Response {
        let payload = ServiceError {
            service_contract_version: SERVICE_CONTRACT_VERSION,
            error: ServiceErrorPayload {
                code: self.code,
                message: self.message.clone(),
                details: self.details,
            },
        };
        (self.status, Json(payload)).into_response()
    }
}

impl ServiceState {
    fn failure(
        status: StatusCode,
        code: &'static str,
        message: impl Into<String>,
        details: Option<serde_json::Value>,
    ) -> ServiceFailure {
        ServiceFailure { status, code, message: message.into(), details }
    }

    fn invalid_json(rejection: &JsonRejection) -> ServiceFailure {
        Self::failure(
            rejection.status(),
            "invalid_json",
            rejection.body_text(),
            Some(json!({"rejection": rejection.to_string()})),
        )
    }

    fn invalid_json_with_telemetry(&self, rejection: &JsonRejection) -> ServiceFailure {
        self.telemetry.record_failure("invalid_json", false);
        Self::invalid_json(rejection)
    }

    fn classify_api_error(
        err: &anyhow::Error,
        default_status: StatusCode,
        default_code: &'static str,
    ) -> ServiceFailure {
        let message = err.to_string();
        let diagnostic = format!("{err:#}");
        let normalized = diagnostic.to_ascii_lowercase();

        if normalized.contains("context package not found") {
            return Self::failure(
                StatusCode::NOT_FOUND,
                "context_package_not_found",
                message,
                None,
            );
        }

        if normalized.contains("unique constraint failed")
            || normalized.contains("foreign key constraint failed")
            || normalized.contains("already exists")
        {
            return Self::failure(StatusCode::CONFLICT, "write_conflict", message, None);
        }

        if normalized.contains("validation failed")
            || normalized.contains("must be provided")
            || normalized.contains("cannot be empty")
            || normalized.contains("unknown record_type")
            || normalized.contains("unknown truth_status")
            || normalized.contains("unknown authority")
        {
            return Self::failure(StatusCode::BAD_REQUEST, "validation_error", message, None);
        }

        if normalized.contains("schema")
            || normalized.contains("sqlite")
            || normalized.contains("database")
        {
            return Self::failure(
                StatusCode::SERVICE_UNAVAILABLE,
                "schema_unavailable",
                message,
                None,
            );
        }

        Self::failure(default_status, default_code, message, None)
    }

    async fn run_blocking<T, F>(
        &self,
        default_status: StatusCode,
        default_code: &'static str,
        operation_label: &'static str,
        op: F,
    ) -> Result<T, ServiceFailure>
    where
        T: Send + 'static,
        F: FnOnce(MemoryKernelApi) -> anyhow::Result<T> + Send + 'static,
    {
        self.telemetry.requests_total.fetch_add(1, Ordering::Relaxed);
        let api = self.api.clone();
        let handle = tokio::task::spawn_blocking(move || op(api));
        let join_result =
            tokio::time::timeout(self.operation_timeout, handle).await.map_err(|_| {
                self.telemetry.record_failure(default_code, true);
                Self::failure(
                    default_status,
                    default_code,
                    format!(
                        "{operation_label} timed out after {} ms",
                        self.operation_timeout.as_millis()
                    ),
                    Some(json!({ "timeout_ms": self.operation_timeout.as_millis() })),
                )
            })?;

        let op_result = join_result.map_err(|err| {
            self.telemetry.record_failure("internal_error", false);
            Self::failure(
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                format!("{operation_label} join failure: {err}"),
                None,
            )
        })?;

        match op_result {
            Ok(value) => {
                self.telemetry.requests_success_total.fetch_add(1, Ordering::Relaxed);
                Ok(value)
            }
            Err(err) => {
                let failure = Self::classify_api_error(&err, default_status, default_code);
                self.telemetry.record_failure(failure.code, false);
                Err(failure)
            }
        }
    }
}

impl ServiceTelemetry {
    fn record_failure(&self, code: &str, timeout: bool) {
        self.requests_failure_total.fetch_add(1, Ordering::Relaxed);
        if timeout {
            self.timeout_total.fetch_add(1, Ordering::Relaxed);
        }
        match code {
            "invalid_json" => {
                self.invalid_json_total.fetch_add(1, Ordering::Relaxed);
            }
            "validation_error" => {
                self.validation_error_total.fetch_add(1, Ordering::Relaxed);
            }
            "context_package_not_found" => {
                self.context_not_found_total.fetch_add(1, Ordering::Relaxed);
            }
            "write_conflict" => {
                self.write_conflict_total.fetch_add(1, Ordering::Relaxed);
            }
            "schema_unavailable" => {
                self.schema_unavailable_total.fetch_add(1, Ordering::Relaxed);
            }
            "internal_error" => {
                self.internal_error_total.fetch_add(1, Ordering::Relaxed);
            }
            _ => {
                self.other_error_total.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    fn snapshot(&self) -> ServiceTelemetrySnapshot {
        ServiceTelemetrySnapshot {
            requests_total: self.requests_total.load(Ordering::Relaxed),
            requests_success_total: self.requests_success_total.load(Ordering::Relaxed),
            requests_failure_total: self.requests_failure_total.load(Ordering::Relaxed),
            timeout_total: self.timeout_total.load(Ordering::Relaxed),
            invalid_json_total: self.invalid_json_total.load(Ordering::Relaxed),
            validation_error_total: self.validation_error_total.load(Ordering::Relaxed),
            context_not_found_total: self.context_not_found_total.load(Ordering::Relaxed),
            write_conflict_total: self.write_conflict_total.load(Ordering::Relaxed),
            schema_unavailable_total: self.schema_unavailable_total.load(Ordering::Relaxed),
            internal_error_total: self.internal_error_total.load(Ordering::Relaxed),
            other_error_total: self.other_error_total.load(Ordering::Relaxed),
        }
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
        .route("/v1/ready", get(ready))
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
    let state = ServiceState {
        api: MemoryKernelApi::new(args.db),
        operation_timeout: Duration::from_millis(args.operation_timeout_ms),
        telemetry: Arc::new(ServiceTelemetry::default()),
    };
    let listener = tokio::net::TcpListener::bind(args.bind).await?;
    axum::serve(listener, app(state)).await?;
    Ok(())
}

async fn health(State(state): State<ServiceState>) -> Json<ServiceEnvelope<HealthResponse>> {
    let timeout_ms = u64::try_from(state.operation_timeout.as_millis()).unwrap_or(u64::MAX);
    Json(envelope(HealthResponse {
        status: "ok",
        timeout_ms,
        telemetry: state.telemetry.snapshot(),
    }))
}

async fn ready(
    State(state): State<ServiceState>,
) -> Result<Json<ServiceEnvelope<ReadinessResponse>>, ServiceFailure> {
    let schema_status = state
        .run_blocking(
            StatusCode::SERVICE_UNAVAILABLE,
            "schema_unavailable",
            "schema_status",
            |api| api.schema_status(),
        )
        .await?;

    let is_ready = schema_status.pending_versions.is_empty()
        && schema_status.current_version == schema_status.target_version;
    let checks = ReadinessChecks {
        current_schema_version: schema_status.current_version,
        target_schema_version: schema_status.target_version,
        pending_migrations: schema_status.pending_versions.len(),
        inferred_from_legacy: schema_status.inferred_from_legacy,
    };

    if is_ready {
        return Ok(Json(envelope(ReadinessResponse { status: "ready", checks })));
    }

    state.telemetry.record_failure("schema_unavailable", false);
    Err(ServiceState::failure(
        StatusCode::SERVICE_UNAVAILABLE,
        "schema_unavailable",
        "database schema is not ready; run /v1/db/migrate before serving traffic",
        Some(json!({
            "current_version": schema_status.current_version,
            "target_version": schema_status.target_version,
            "pending_versions": schema_status.pending_versions,
            "inferred_from_legacy": schema_status.inferred_from_legacy
        })),
    ))
}

async fn openapi() -> impl IntoResponse {
    (StatusCode::OK, [("content-type", "application/yaml; charset=utf-8")], OPENAPI_YAML)
}

async fn db_schema_version(
    State(state): State<ServiceState>,
) -> Result<Json<ServiceEnvelope<memory_kernel_store_sqlite::SchemaStatus>>, ServiceFailure> {
    let status = state
        .run_blocking(
            StatusCode::SERVICE_UNAVAILABLE,
            "schema_unavailable",
            "schema_status",
            |api| api.schema_status(),
        )
        .await?;
    Ok(Json(envelope(status)))
}

async fn db_migrate(
    State(state): State<ServiceState>,
    payload: Result<Json<MigrateRequest>, JsonRejection>,
) -> Result<Json<ServiceEnvelope<memory_kernel_api::MigrateResult>>, ServiceFailure> {
    let Json(request) =
        payload.map_err(|rejection| state.invalid_json_with_telemetry(&rejection))?;
    let result = state
        .run_blocking(
            StatusCode::INTERNAL_SERVER_ERROR,
            "migration_failed",
            "migrate",
            move |api| api.migrate(request.dry_run),
        )
        .await?;
    Ok(Json(envelope(result)))
}

async fn memory_add_constraint(
    State(state): State<ServiceState>,
    payload: Result<Json<AddConstraintRequest>, JsonRejection>,
) -> Result<Json<ServiceEnvelope<memory_kernel_core::MemoryRecord>>, ServiceFailure> {
    let Json(request) =
        payload.map_err(|rejection| state.invalid_json_with_telemetry(&rejection))?;
    let record = state
        .run_blocking(
            StatusCode::INTERNAL_SERVER_ERROR,
            "write_failed",
            "add_constraint",
            move |api| api.add_constraint(request),
        )
        .await?;
    Ok(Json(envelope(record)))
}

async fn memory_add_summary(
    State(state): State<ServiceState>,
    payload: Result<Json<AddSummaryRequest>, JsonRejection>,
) -> Result<Json<ServiceEnvelope<memory_kernel_core::MemoryRecord>>, ServiceFailure> {
    let Json(request) =
        payload.map_err(|rejection| state.invalid_json_with_telemetry(&rejection))?;
    let record = state
        .run_blocking(
            StatusCode::INTERNAL_SERVER_ERROR,
            "write_failed",
            "add_summary",
            move |api| api.add_summary(request),
        )
        .await?;
    Ok(Json(envelope(record)))
}

async fn memory_link(
    State(state): State<ServiceState>,
    payload: Result<Json<AddLinkRequest>, JsonRejection>,
) -> Result<Json<ServiceEnvelope<memory_kernel_api::AddLinkResult>>, ServiceFailure> {
    let Json(request) =
        payload.map_err(|rejection| state.invalid_json_with_telemetry(&rejection))?;
    let result = state
        .run_blocking(StatusCode::INTERNAL_SERVER_ERROR, "write_failed", "add_link", move |api| {
            api.add_link(request)
        })
        .await?;
    Ok(Json(envelope(result)))
}

async fn query_ask(
    State(state): State<ServiceState>,
    payload: Result<Json<AskRequest>, JsonRejection>,
) -> Result<Json<ServiceEnvelope<memory_kernel_core::ContextPackage>>, ServiceFailure> {
    let Json(request) =
        payload.map_err(|rejection| state.invalid_json_with_telemetry(&rejection))?;
    let package = state
        .run_blocking(StatusCode::INTERNAL_SERVER_ERROR, "query_failed", "query_ask", move |api| {
            api.query_ask(request)
        })
        .await?;
    Ok(Json(envelope(package)))
}

async fn query_recall(
    State(state): State<ServiceState>,
    payload: Result<Json<RecallRequest>, JsonRejection>,
) -> Result<Json<ServiceEnvelope<memory_kernel_core::ContextPackage>>, ServiceFailure> {
    let Json(request) =
        payload.map_err(|rejection| state.invalid_json_with_telemetry(&rejection))?;
    let package = state
        .run_blocking(
            StatusCode::INTERNAL_SERVER_ERROR,
            "query_failed",
            "query_recall",
            move |api| api.query_recall(request),
        )
        .await?;
    Ok(Json(envelope(package)))
}

async fn context_show(
    State(state): State<ServiceState>,
    Path(context_package_id): Path<String>,
) -> Result<Json<ServiceEnvelope<memory_kernel_core::ContextPackage>>, ServiceFailure> {
    let package = state
        .run_blocking(
            StatusCode::INTERNAL_SERVER_ERROR,
            "context_lookup_failed",
            "context_show",
            move |api| api.context_show(&context_package_id),
        )
        .await?;
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

    fn test_state(api: MemoryKernelApi, timeout_ms: u64) -> ServiceState {
        ServiceState {
            api,
            operation_timeout: Duration::from_millis(timeout_ms),
            telemetry: Arc::new(ServiceTelemetry::default()),
        }
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
        let state = test_state(MemoryKernelApi::new(unique_temp_db_path()), 2500);
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
        let state = test_state(MemoryKernelApi::new(unique_temp_db_path()), 2500);
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
        assert!(body.contains("version: service.v3"));
        assert!(body.contains("/v1/memory/add/summary"));
        assert!(body.contains("/v1/query/recall"));
        assert!(body.contains("/v1/ready"));
        assert!(body.contains("ServiceErrorEnvelope"));
    }

    // Test IDs: TSVC-010
    #[tokio::test]
    async fn ready_endpoint_reports_ready_when_schema_is_current() {
        let db_path = unique_temp_db_path();
        let api = MemoryKernelApi::new(db_path.clone());
        if let Err(err) = api.migrate(false) {
            panic!("failed to migrate schema before readiness test: {err:#}");
        }
        let router = app(test_state(api, 2500));

        let response = match router
            .oneshot(
                Request::builder()
                    .uri("/v1/ready")
                    .method("GET")
                    .body(axum::body::Body::empty())
                    .unwrap_or_else(|err| panic!("failed to build ready request: {err}")),
            )
            .await
        {
            Ok(response) => response,
            Err(err) => panic!("ready request failed: {err}"),
        };
        assert_eq!(response.status(), StatusCode::OK);

        let value = response_json(response).await;
        assert_eq!(
            value
                .get("data")
                .and_then(|data| data.get("status"))
                .and_then(serde_json::Value::as_str),
            Some("ready")
        );
        assert_eq!(
            value
                .get("data")
                .and_then(|data| data.get("checks"))
                .and_then(|checks| checks.get("pending_migrations"))
                .and_then(serde_json::Value::as_u64),
            Some(0)
        );

        let _ = std::fs::remove_file(&db_path);
    }

    // Test IDs: TSVC-011
    #[tokio::test]
    async fn ready_endpoint_returns_schema_unavailable_when_db_is_unreachable() {
        let db_path = std::env::temp_dir()
            .join(format!("memorykernel-service-missing-parent-{}/db.sqlite3", ulid::Ulid::new()));
        let state = test_state(MemoryKernelApi::new(db_path), 2500);
        let router = app(state);

        let response = match router
            .oneshot(
                Request::builder()
                    .uri("/v1/ready")
                    .method("GET")
                    .body(axum::body::Body::empty())
                    .unwrap_or_else(|err| panic!("failed to build ready request: {err}")),
            )
            .await
        {
            Ok(response) => response,
            Err(err) => panic!("ready request failed: {err}"),
        };
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);

        let value = response_json(response).await;
        assert_eq!(
            value
                .get("error")
                .and_then(|error| error.get("code"))
                .and_then(serde_json::Value::as_str),
            Some("schema_unavailable")
        );
        assert!(
            value.get("api_contract_version").is_none(),
            "error envelope must not include api_contract_version: {value}"
        );
    }

    // Test IDs: TSVC-012
    #[tokio::test]
    async fn run_blocking_returns_success_for_fast_operation() {
        let state = test_state(MemoryKernelApi::new(unique_temp_db_path()), 2500);

        let result = state
            .run_blocking(
                StatusCode::INTERNAL_SERVER_ERROR,
                "query_failed",
                "unit_fast_operation",
                |_api| Ok::<_, anyhow::Error>(42_u32),
            )
            .await;

        match result {
            Ok(value) => assert_eq!(value, 42),
            Err(err) => panic!("expected fast blocking operation to succeed: {err:?}"),
        }
    }

    // Test IDs: TSVC-013
    #[tokio::test]
    async fn run_blocking_times_out_with_mapped_error_status() {
        let state = test_state(MemoryKernelApi::new(unique_temp_db_path()), 1);

        let result = state
            .run_blocking(
                StatusCode::INTERNAL_SERVER_ERROR,
                "query_failed",
                "unit_timeout_operation",
                |_api| {
                    std::thread::sleep(Duration::from_millis(25));
                    Ok::<_, anyhow::Error>(())
                },
            )
            .await;

        match result {
            Ok(()) => panic!("expected timeout for slow blocking operation"),
            Err(err) => {
                assert_eq!(err.status, StatusCode::INTERNAL_SERVER_ERROR);
                assert_eq!(err.code, "query_failed");
                assert!(
                    err.message.contains("timed out"),
                    "timeout error message must mention timeout: {}",
                    err.message
                );
                assert!(err.details.is_some(), "timeout error should include details");
            }
        }
    }

    // Test IDs: TSVC-014
    #[tokio::test]
    async fn telemetry_counters_track_success_failure_and_timeout() {
        let state = test_state(MemoryKernelApi::new(unique_temp_db_path()), 1);

        let success = state
            .run_blocking(
                StatusCode::INTERNAL_SERVER_ERROR,
                "query_failed",
                "telemetry_success",
                |_api| Ok::<_, anyhow::Error>(1_u32),
            )
            .await;
        assert!(success.is_ok(), "expected success path for telemetry test");

        let timeout = state
            .run_blocking(
                StatusCode::INTERNAL_SERVER_ERROR,
                "query_failed",
                "telemetry_timeout",
                |_api| {
                    std::thread::sleep(Duration::from_millis(20));
                    Ok::<_, anyhow::Error>(0_u32)
                },
            )
            .await;
        assert!(timeout.is_err(), "expected timeout path for telemetry test");

        let snapshot = state.telemetry.snapshot();
        assert_eq!(snapshot.requests_total, 2);
        assert_eq!(snapshot.requests_success_total, 1);
        assert_eq!(snapshot.requests_failure_total, 1);
        assert_eq!(snapshot.timeout_total, 1);
    }

    // Test IDs: TSVC-002
    #[tokio::test]
    async fn service_add_query_and_context_flow_round_trip() {
        let db_path = unique_temp_db_path();
        let state = test_state(MemoryKernelApi::new(db_path.clone()), 2500);
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
        let state = test_state(MemoryKernelApi::new(db_path.clone()), 2500);
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

    // Test IDs: TSVC-005
    #[tokio::test]
    async fn context_show_missing_returns_not_found_machine_error() {
        let db_path = unique_temp_db_path();
        let state = test_state(MemoryKernelApi::new(db_path.clone()), 2500);
        let router = app(state);

        let response = match router
            .oneshot(
                Request::builder()
                    .uri("/v1/context/ctx_missing")
                    .method("GET")
                    .body(axum::body::Body::empty())
                    .unwrap_or_else(|err| panic!("failed to build context request: {err}")),
            )
            .await
        {
            Ok(response) => response,
            Err(err) => panic!("context request failed: {err}"),
        };
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let value = response_json(response).await;
        assert_eq!(
            value
                .get("error")
                .and_then(|error| error.get("code"))
                .and_then(serde_json::Value::as_str),
            Some("context_package_not_found")
        );
        assert!(
            value.get("legacy_error").is_none(),
            "legacy_error must not be present in service.v3: {value}"
        );

        let _ = std::fs::remove_file(&db_path);
    }

    // Test IDs: TSVC-006
    #[tokio::test]
    async fn add_constraint_validation_failure_returns_validation_error() {
        let db_path = unique_temp_db_path();
        let state = test_state(MemoryKernelApi::new(db_path.clone()), 2500);
        let router = app(state);

        let add_payload = serde_json::json!({
            "actor": "user",
            "action": "use",
            "resource": "usb_drive",
            "effect": "deny",
            "note": null,
            "memory_id": null,
            "version": 1,
            "writer": "",
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

        let response = match router
            .oneshot(
                Request::builder()
                    .uri("/v1/memory/add/constraint")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(add_payload.to_string()))
                    .unwrap_or_else(|err| panic!("failed to build request: {err}")),
            )
            .await
        {
            Ok(response) => response,
            Err(err) => panic!("request failed: {err}"),
        };

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let value = response_json(response).await;
        assert_eq!(
            value
                .get("error")
                .and_then(|error| error.get("code"))
                .and_then(serde_json::Value::as_str),
            Some("validation_error")
        );

        let _ = std::fs::remove_file(&db_path);
    }

    // Test IDs: TSVC-015
    #[tokio::test]
    async fn add_summary_validation_failure_returns_validation_error() {
        let db_path = unique_temp_db_path();
        let state = test_state(MemoryKernelApi::new(db_path.clone()), 2500);
        let router = app(state);

        let payload = serde_json::json!({
            "record_type": "decision",
            "summary": "summary without writer",
            "memory_id": null,
            "version": 1,
            "writer": "",
            "justification": "fixture",
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

        let response = match router
            .oneshot(
                Request::builder()
                    .uri("/v1/memory/add/summary")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(payload.to_string()))
                    .unwrap_or_else(|err| panic!("failed to build request: {err}")),
            )
            .await
        {
            Ok(response) => response,
            Err(err) => panic!("request failed: {err}"),
        };

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let value = response_json(response).await;
        assert_eq!(
            value
                .get("error")
                .and_then(|error| error.get("code"))
                .and_then(serde_json::Value::as_str),
            Some("validation_error")
        );

        let _ = std::fs::remove_file(&db_path);
    }

    // Test IDs: TSVC-007
    #[tokio::test]
    async fn invalid_json_payload_returns_invalid_json_error() {
        let db_path = unique_temp_db_path();
        let state = test_state(MemoryKernelApi::new(db_path.clone()), 2500);
        let router = app(state);

        let response = match router
            .oneshot(
                Request::builder()
                    .uri("/v1/query/ask")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from("{".to_string()))
                    .unwrap_or_else(|err| panic!("failed to build request: {err}")),
            )
            .await
        {
            Ok(response) => response,
            Err(err) => panic!("request failed: {err}"),
        };

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let value = response_json(response).await;
        assert_eq!(
            value
                .get("error")
                .and_then(|error| error.get("code"))
                .and_then(serde_json::Value::as_str),
            Some("invalid_json")
        );
        assert!(
            value
                .get("error")
                .and_then(|error| error.get("details"))
                .and_then(|details| details.get("rejection"))
                .and_then(serde_json::Value::as_str)
                .is_some(),
            "missing json rejection details: {value}"
        );

        let _ = std::fs::remove_file(&db_path);
    }

    // Test IDs: TSVC-016
    #[tokio::test]
    async fn memory_link_invalid_json_returns_invalid_json_error() {
        let db_path = unique_temp_db_path();
        let state = test_state(MemoryKernelApi::new(db_path.clone()), 2500);
        let router = app(state);

        let response = match router
            .oneshot(
                Request::builder()
                    .uri("/v1/memory/link")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from("{".to_string()))
                    .unwrap_or_else(|err| panic!("failed to build request: {err}")),
            )
            .await
        {
            Ok(response) => response,
            Err(err) => panic!("request failed: {err}"),
        };

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let value = response_json(response).await;
        assert_eq!(
            value
                .get("error")
                .and_then(|error| error.get("code"))
                .and_then(serde_json::Value::as_str),
            Some("invalid_json")
        );

        let _ = std::fs::remove_file(&db_path);
    }

    // Test IDs: TSVC-008
    #[tokio::test]
    async fn duplicate_identity_returns_write_conflict() {
        let db_path = unique_temp_db_path();
        let state = test_state(MemoryKernelApi::new(db_path.clone()), 2500);
        let router = app(state);

        let payload = serde_json::json!({
            "actor": "user",
            "action": "use",
            "resource": "usb_drive",
            "effect": "deny",
            "note": null,
            "memory_id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
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

        let first = match router
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/v1/memory/add/constraint")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(payload.to_string()))
                    .unwrap_or_else(|err| panic!("failed to build request: {err}")),
            )
            .await
        {
            Ok(response) => response,
            Err(err) => panic!("request failed: {err}"),
        };
        assert_eq!(first.status(), StatusCode::OK);

        let second = match router
            .oneshot(
                Request::builder()
                    .uri("/v1/memory/add/constraint")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(payload.to_string()))
                    .unwrap_or_else(|err| panic!("failed to build request: {err}")),
            )
            .await
        {
            Ok(response) => response,
            Err(err) => panic!("request failed: {err}"),
        };
        assert_eq!(second.status(), StatusCode::CONFLICT);
        let value = response_json(second).await;
        assert_eq!(
            value
                .get("error")
                .and_then(|error| error.get("code"))
                .and_then(serde_json::Value::as_str),
            Some("write_conflict")
        );

        let _ = std::fs::remove_file(&db_path);
    }

    // Test IDs: TSVC-009
    #[tokio::test]
    async fn non_2xx_error_envelope_keeps_service_v3_shape() {
        let db_path = unique_temp_db_path();
        let state = test_state(MemoryKernelApi::new(db_path.clone()), 2500);
        let router = app(state);

        let invalid_payload = serde_json::json!({
            "actor": "user",
            "action": "use",
            "resource": "usb_drive",
            "effect": "deny",
            "note": null,
            "memory_id": null,
            "version": 1,
            "writer": "",
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

        let response = match router
            .oneshot(
                Request::builder()
                    .uri("/v1/memory/add/constraint")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(invalid_payload.to_string()))
                    .unwrap_or_else(|err| panic!("failed to build request: {err}")),
            )
            .await
        {
            Ok(response) => response,
            Err(err) => panic!("request failed: {err}"),
        };
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let value = response_json(response).await;
        assert_eq!(
            value.get("service_contract_version").and_then(serde_json::Value::as_str),
            Some(SERVICE_CONTRACT_VERSION)
        );
        assert!(
            value.get("api_contract_version").is_none(),
            "error envelope must not include api_contract_version: {value}"
        );
        assert_eq!(
            value
                .get("error")
                .and_then(|error| error.get("code"))
                .and_then(serde_json::Value::as_str),
            Some("validation_error")
        );

        assert!(
            value.get("legacy_error").is_none(),
            "legacy_error must not be present in service.v3: {value}"
        );

        let _ = std::fs::remove_file(&db_path);
    }
}
