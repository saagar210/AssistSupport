# Search API

## Runtime Setup

```bash
cd search-api
python3 -m venv venv
source venv/bin/activate
pip install -r requirements.txt
```

## Local Development

```bash
# Auth is enabled by default in all environments.
# Set a non-default bearer token before starting the service.
ENVIRONMENT=development \
ASSISTSUPPORT_API_KEY=local-dev-search-token \
python3 search_api.py
```

Query logging minimization (default-off raw query storage):

- By default, `query_performance.query_text` stores `sha256:<hex>` fingerprints, not raw query text.
- To opt in to raw query storage for local/dev analytics only, set:
  - `ASSISTSUPPORT_SEARCH_API_STORE_RAW_QUERY_TEXT=1`

Insecure local-only override (not recommended):

```bash
ENVIRONMENT=development \
ASSISTSUPPORT_SEARCH_API_REQUIRE_AUTH=0 \
python3 search_api.py
```

## Production-like Local Run

```bash
# Requires local redis server for rate-limit backend
redis-server --daemonize yes

cp .env.example .env.production
# Edit .env.production with a non-default API key and redis URI

set -a
source .env.production
set +a

python3 validate_runtime.py --check-backends
python3 search_api.py
```

The service exposes:

- `GET /health` for cheap liveness only
- `GET /ready` for dependency-backed readiness (DB, rate-limit backend when external, and model initialization)

## Production WSGI Serving

```bash
# Example using gunicorn from the repo root (install separately in deployment environment)
gunicorn --chdir search-api --bind 127.0.0.1:${ASSISTSUPPORT_API_PORT:-3000} wsgi:app

# Or from inside search-api/
cd search-api
gunicorn --bind 127.0.0.1:${ASSISTSUPPORT_API_PORT:-3000} wsgi:app
```

`wsgi.py` bootstraps its own import path and validates runtime configuration before exposing the app.

## Test and Smoke Checks

```bash
cd search-api
python3 -m venv venv
source venv/bin/activate
pip install -r requirements.txt -r requirements-test.txt
pytest -q
python -m pip_audit -r requirements.txt

# Production smoke check (expects /health, /ready, and auth enforcement)
ENVIRONMENT=production \
ASSISTSUPPORT_API_KEY=test-key \
ASSISTSUPPORT_RATE_LIMIT_STORAGE_URI=redis://127.0.0.1:6379/0 \
python3 smoke_search_api.py

# Regenerate the checked-in API contract
python3 generate_openapi.py > ../openapi/openapi.generated.json
```

Related operator docs:

- `docs/runbooks/search-api-local-deployment.md`
- `docs/runbooks/dependency-advisory-triage.md`
