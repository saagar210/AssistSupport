# Service Contract (Normative)

## Version

- Service contract version: `service.v2`
- API contract version surfaced in envelopes: `api.v1`

## Transport

- Local HTTP service.
- Default bind: `127.0.0.1:4010`.
- JSON request/response bodies unless noted.

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
- `legacy_error` (transitional string mirror of `error.message`; required for `service.v2`)

## Error Envelope Policy (`service.v2`)

- Non-2xx responses intentionally do **not** include `api_contract_version`.
- Non-2xx responses MUST include:
  - `service_contract_version`
  - `error` object with `code` + `message` (and optional `details`)
  - `legacy_error` for transition-safe consumers
- `legacy_error` remains required for the full `service.v2` lifecycle and may only be removed in `service.v3` with explicit migration guidance.

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

- `openapi/openapi.yaml` is the versioned artifact for `service.v2`.
