# State Model (Revamp)

Status: Drafted for Phase 2 Gate  
Date: 2026-02-08

## 1) Global State Domains
1. Session/UI state
2. Ticket/work queue state
3. Response workspace state
4. Integration runtime state
5. Diagnostics and audit state

## 2) Integration Runtime State Machine
### States
1. `disabled` - feature intentionally off
2. `checking` - preflight/validation in progress
3. `ready` - enrichment available
4. `degraded` - service reachable with contract/response caveat
5. `offline` - service unavailable/timeout/network fault

### Transition Rules
1. Startup enters `checking` only in background; UI remains usable.
2. `checking` -> `ready` on preflight success.
3. `checking` -> `degraded` on recoverable mismatch/malformed envelopes.
4. `checking` -> `offline` on connection/timeout failures.
5. Any non-ready state must preserve deterministic fallback behavior.

## 3) Workspace State Buckets
1. `ticketContext`: immutable source-of-truth ticket metadata per selected item.
2. `draftContext`: mutable response inputs and generation parameters.
3. `enrichmentContext`: optional data from MemoryKernel adapter.
4. `actionContext`: pending transitions/escalation/post actions.

## 4) Persistence Model
1. Draft/session persistence stored in local encrypted DB.
2. Integration config stored as validated JSON object only.
3. Runtime telemetry and audit persisted via existing audit pipeline.

## 5) Anti-Patterns (Disallowed)
1. Direct mutation of persisted state from UI components.
2. Cross-domain state writes without service abstraction.
3. Reaching into adapter internals from UI hooks.
