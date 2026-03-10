# Search API Local Deployment Runbook

## Goal

Start the local search sidecar in a production-like way that proves readiness, auth, and dependency health.

## Prerequisites

- PostgreSQL is reachable with the configured AssistSupport DB credentials.
- Redis is running when the rate-limit backend is not in-memory.
- A non-default API key is available.
- Python virtual environment is created in `search-api/venv`.

## Setup

```bash
cd search-api
python3 -m venv venv
source venv/bin/activate
pip install -r requirements.txt
```

## Recommended production-like startup

```bash
redis-server --daemonize yes
cp .env.example .env.production

set -a
source .env.production
set +a

python3 validate_runtime.py --check-backends
gunicorn --chdir search-api --bind 127.0.0.1:${ASSISTSUPPORT_API_PORT:-3000} wsgi:app
```

## Required environment expectations

- `ASSISTSUPPORT_API_KEY` must be set.
- `ASSISTSUPPORT_SEARCH_API_REQUIRE_AUTH` should remain enabled.
- `ASSISTSUPPORT_RATE_LIMIT_STORAGE_URI` should point to Redis for production-like mode.
- DB credentials must resolve to the intended Postgres instance.

## Health model

- `GET /health` is liveness only.
- `GET /ready` is the deploy/readiness gate.

## Smoke checks

From another shell:

```bash
curl http://127.0.0.1:3000/health
curl http://127.0.0.1:3000/ready
ENVIRONMENT=production ASSISTSUPPORT_API_KEY=test-key ASSISTSUPPORT_RATE_LIMIT_STORAGE_URI=redis://127.0.0.1:6379/0 python3 smoke_search_api.py
```

## Expected results

- `/health` returns JSON with status information even before full readiness.
- `/ready` returns HTTP 200 only when DB, rate-limit backend, and model checks are healthy.
- Authenticated `/search` requests succeed.
- Unauthenticated `/search` requests fail.

## Common failure patterns

- `/health` works but `/ready` is degraded:
  - check Postgres connectivity,
  - check Redis availability,
  - check model/reranker initialization.
- Repo-root import errors:
  - use `wsgi.py` as the entrypoint, not ad hoc module bootstrapping.
- Auth failures:
  - verify the bearer token stored in AssistSupport matches `ASSISTSUPPORT_API_KEY`.

## Completion criteria

- Runtime validation passes.
- `/ready` is healthy.
- Smoke check succeeds.
- AssistSupport desktop shows the search sidecar as ready, not just alive.
