# Service Contract (Normative)

## Version

- Service contract version: `service.v3`
- API contract version surfaced in envelopes: `api.v1`

## Transport

- Local HTTP service.
- Default bind: `127.0.0.1:4010`.
- JSON request/response bodies unless noted.
- Service handlers execute API operations on blocking worker threads and enforce a bounded timeout (`--operation-timeout-ms`, default `2500`).

## Envelope

Successful responses MUST include:

- `service_contract_version`
- `api_contract_version`
- `data`

Error responses MUST include:

- `service_contract_version`
- `error.code`
- `error.message`
- `error.details` (optional object)

Error responses MUST NOT include:

- `api_contract_version`
- `legacy_error`

## Error Envelope Policy (`service.v3`)

- Non-2xx responses intentionally do **not** include `api_contract_version`.
- Non-2xx responses MUST include:
  - `service_contract_version`
  - `error` object with `code` + `message` (and optional `details`)
- `legacy_error` is removed in `service.v3`. Consumers should route by `error.code`.

## Error Status and Code Mapping

The service emits machine-readable error codes with stable semantics:

- `invalid_json` -> `400`
- `validation_error` -> `400`
- `context_package_not_found` -> `404`
- `write_conflict` -> `409`
- `schema_unavailable` -> `503`
- `migration_failed` -> `500`
- `write_failed` -> `500`
- `query_failed` -> `500`
- `context_lookup_failed` -> `500`
- `internal_error` -> `500`

## Endpoints

- `GET /v1/health`
- `GET /v1/ready`
- `GET /v1/openapi`
- `POST /v1/db/schema-version`
- `POST /v1/db/migrate`
- `POST /v1/memory/add/constraint`
- `POST /v1/memory/add/summary`
- `POST /v1/memory/link`
- `POST /v1/query/ask`
- `POST /v1/query/recall`
- `GET /v1/context/{context_package_id}`

## OpenAPI Source of Truth

- `openapi/openapi.yaml` is the versioned artifact for `service.v3`.

## Health vs Readiness

- `GET /v1/health` is a liveness probe only (`status: ok`) and includes an in-process telemetry snapshot:
  - `timeout_ms`
  - `telemetry.requests_total`
  - `telemetry.requests_success_total`
  - `telemetry.requests_failure_total`
  - `telemetry.timeout_total`
  - per-class failure counters (`invalid_json`, `validation_error`, `context_package_not_found`, `write_conflict`, `schema_unavailable`, `internal_error`, `other_error`)
- `GET /v1/ready` is a schema readiness probe:
  - returns `200` with `data.status=ready` only when schema is current and no migrations are pending.
  - returns `503 schema_unavailable` when database/schema is unavailable or pending migrations prevent readiness.
