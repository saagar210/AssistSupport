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
# Development mode (no auth enforcement)
ENVIRONMENT=development python3 search_api.py
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

## Production WSGI Serving

```bash
# Example using gunicorn (install separately in deployment environment)
gunicorn --bind 127.0.0.1:${ASSISTSUPPORT_API_PORT:-3000} wsgi:app
```

`wsgi.py` validates runtime configuration before exposing the app.

## Test and Smoke Checks

```bash
cd search-api
python3 -m venv venv
source venv/bin/activate
pip install -r requirements-test.txt
pytest -q

# Production smoke check
ENVIRONMENT=production \
ASSISTSUPPORT_API_KEY=test-key \
ASSISTSUPPORT_RATE_LIMIT_STORAGE_URI=redis://127.0.0.1:6379/0 \
python3 smoke_search_api.py
```
