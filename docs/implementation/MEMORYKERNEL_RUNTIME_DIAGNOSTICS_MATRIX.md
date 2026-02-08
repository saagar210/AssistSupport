# MemoryKernel Runtime Diagnostics Matrix (AssistSupport)

Updated: 2026-02-08
Owner: AssistSupport

Purpose: make MemoryKernel lifecycle behavior operationally deterministic and easy to triage while preserving core Draft availability.

## Invariants
1. Draft flow must stay available under all MemoryKernel failure modes.
2. MemoryKernel enrichment is optional and non-blocking.
3. User-facing status must remain actionable and specific.

## State Mapping

| Runtime State | Trigger Condition | User-Facing Status | Enrichment | Operator Action |
|---|---|---|---|---|
| `ready` | Health + schema checks pass and versions match | Ready | Enabled | None |
| `disabled` | Feature flag off | Disabled by configuration | Disabled | Validate config intent |
| `offline` | Connection refused / DNS / timeout at startup | Service offline | Disabled | Start/verify service process and local port |
| `schema-unavailable` | `/v1/db/schema-version` non-success or unavailable | Schema unavailable | Disabled | Inspect producer DB migration state |
| `version-mismatch` | `service_contract_version` or `api_contract_version` mismatch | Version mismatch | Disabled | Repin or reconcile contract versions |
| `malformed-payload` | Invalid JSON/shape in health/schema/query response | Invalid producer payload | Disabled | Validate producer envelope/schema |
| `degraded` | Non-2xx query with typed error | Degraded (fallback active) | Disabled for request | Inspect error.code and producer logs |

## Typed Error Routing (service.v2)

| `error.code` | Recommended Consumer Label | Fallback Behavior |
|---|---|---|
| `invalid_json` | Invalid producer payload | Deterministic fallback |
| `validation_error` | Invalid request semantics | Deterministic fallback |
| `context_package_not_found` | Context package unavailable | Deterministic fallback |
| `write_conflict` | Producer write conflict | Deterministic fallback |
| `write_failed` | Producer write failure | Deterministic fallback |
| `schema_unavailable` | Schema unavailable | Deterministic fallback |
| `migration_failed` | Producer migration failure | Deterministic fallback |
| `query_failed` | Query execution failure | Deterministic fallback |
| `context_lookup_failed` | Context lookup failure | Deterministic fallback |
| `internal_error` | Internal producer error | Deterministic fallback |

## Escalation Rules
1. Three consecutive `offline` or `schema-unavailable` startup failures: create an incident ticket.
2. Any unknown `error.code`: treat as contract drift and block baseline promotion.
3. Any case where Draft is blocked: classify as release-blocking defect.

## Verification Routine
```bash
pnpm run check:memorykernel-pin
pnpm run test:memorykernel-contract
pnpm run test:ci
```

