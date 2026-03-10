# 0001: Remediation Hardening Foundations

- Status: Accepted
- Date: 2026-03-09

## Context

AssistSupport’s hardening audit identified three near-term trust-boundary problems that needed repository-wide direction before lower-level implementation could proceed safely:

1. Vector search state could not be trusted across older stores because metadata-free rows broke namespace isolation and delete guarantees.
2. The localhost Python search sidecar did not expose a clear production contract for readiness, connection ownership, or serving mode.
3. Startup security UX exposed passphrase setup as if it were fully supported for new users, while the reliable path that exists today is keychain/default storage plus a dedicated unlock path for existing passphrase users.

These were architectural decisions, not just local bug fixes, because they shape runtime behavior, release gating, and user-facing trust claims.

## Decision

### Vector-store trust restoration

- Treat pre-fix vector stores as untrusted until rebuilt.
- Version vector-store state in SQLite and quarantine stale stores automatically at startup.
- Use SQLite chunk metadata as the source of truth for vector rebuilds.
- Purge vector rows whenever documents or namespaces are deleted or cleared.

### Search API runtime contract

- Keep `GET /health` as a cheap liveness probe only.
- Add `GET /ready` as the dependency-backed operator signal for DB, runtime config, rate-limit backend, and model readiness.
- Keep authenticated access on `/search`, `/feedback`, and `/stats`.
- Treat `wsgi.py` as the production entrypoint and `run_server()` as development-only.
- Remove the unused `/config` endpoint rather than carry an undocumented semi-public surface.

### Startup and key-storage behavior

- Hide new-user passphrase setup during stabilization.
- Support existing passphrase workspaces through an explicit unlock flow.
- Keep frontend initialization contracts aligned to the Rust backend as the single source of truth.

## Consequences

### Positive

- The app now has a deterministic rebuild/quarantine story for vector integrity.
- Operators and CI can distinguish liveness from readiness.
- The user-facing encryption and startup story is narrower but truthful.

### Tradeoffs

- Users with older vector stores must rebuild before vector search is re-enabled.
- The Python service gains more explicit configuration and CI requirements.
- New-user passphrase setup is temporarily unavailable until a complete end-to-end onboarding path exists.

## Follow-up

- Add the remaining recovery-mode, documentation, and permission-zoning ADRs from the remediation plan.
- Extend CI to validate OpenAPI drift, Python dependency audits, and real runtime smoke checks against Postgres and Redis.
