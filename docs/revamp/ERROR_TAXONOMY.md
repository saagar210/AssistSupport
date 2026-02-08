# Error Taxonomy (Revamp)

Status: Drafted for Phase 2 Gate  
Date: 2026-02-08

## 1) Categories
1. `USER_INPUT` - invalid user-provided data/configuration.
2. `INTEGRATION` - adapter or remote service failure.
3. `RUNTIME` - local generation/runtime system fault.
4. `DATA` - persistence/read/write/consistency fault.
5. `SECURITY` - policy, validation, or trust-boundary violation.
6. `UNKNOWN` - uncategorized fallback bucket (must be minimized).

## 2) Severity Mapping
1. `INFO` - expected operational signal.
2. `WARNING` - degraded but recoverable path.
3. `ERROR` - user-impacting failure with fallback path.
4. `CRITICAL` - potential data/security/system integrity issue.

## 3) UI Message Policy
1. User-facing messages must be actionable and non-ambiguous.
2. Raw stack traces never exposed in UI.
3. Integration failures must report state and next action.

## 4) Integration Error Handling Requirements
1. Route by normalized machine error code when available.
2. Preserve legacy compatibility parsing where contract requires.
3. Convert transport/protocol faults into deterministic fallback messages.
4. Exclude remote internal payload leakage from user-visible copy.

## 5) Logging Requirements
1. Log category + severity + operation + correlation ID.
2. Include safe context for debugging (no secrets/PII leakage).
3. Critical errors generate explicit Ops-visible event.

## 6) Test Requirements
1. At least one positive and one negative test per category path.
2. Integration tests for offline, timeout, malformed payload, version mismatch, non-2xx envelope.
