#!/usr/bin/env python3
"""
Production Search API for AssistSupport
Hybrid search endpoint with authentication, rate limiting, and monitoring
"""

import sys
import os
import logging
import threading

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from flask import Flask, request, jsonify
from flask_limiter import Limiter
from flask_limiter.util import get_remote_address
from functools import wraps
from datetime import datetime, timezone
from runtime_config import (
    DEFAULT_API_KEY,
    DEFAULT_RATE_LIMIT_STORAGE_URI,
    load_runtime_config,
    ensure_valid_runtime_config,
    validate_runtime_config,
    RuntimeConfigError,
)
from db_config import check_db_connection
from managed_embedding_model import get_model_status

# Initialize Flask app
app = Flask(__name__)
app.config["JSON_SORT_KEYS"] = False
# Fail closed on oversized request bodies (prevents accidental log/alloc abuse).
# Search requests are tiny; keep the cap conservative.
app.config["MAX_CONTENT_LENGTH"] = int(os.environ.get("ASSISTSUPPORT_SEARCH_API_MAX_BODY_BYTES", "262144"))

# Configuration
_RUNTIME_CONFIG = load_runtime_config()
API_KEY = _RUNTIME_CONFIG.api_key
API_PORT = _RUNTIME_CONFIG.api_port
RATE_LIMIT_STORAGE_URI = _RUNTIME_CONFIG.rate_limit_storage_uri
AUTH_REQUIRED = _RUNTIME_CONFIG.require_auth

# Rate limiting: 100 requests per minute per IP
limiter = Limiter(
    app=app,
    key_func=get_remote_address,
    default_limits=["100 per minute"],
    storage_uri=RATE_LIMIT_STORAGE_URI,
)

# Global search engine instance (initialized on first request)
_engine = None
_engine_lock = threading.Lock()


def is_production() -> bool:
    return os.environ.get("ENVIRONMENT", "development").lower() == "production"


def _get_engine():
    """Lazy-initialize the search engine singleton"""
    global _engine
    if _engine is None:
        with _engine_lock:
            if _engine is None:
                from hybrid_search import HybridSearchEngine

                _engine = HybridSearchEngine(store_raw_query_text=_RUNTIME_CONFIG.store_raw_query_text)
                app.logger.info("Search engine initialized")
    return _engine


def _check_runtime_config():
    errors = validate_runtime_config(_RUNTIME_CONFIG, check_backends=False)
    if errors:
        return {"status": "error", "errors": errors}
    return {"status": "ok"}


def _check_database():
    try:
        if check_db_connection():
            return {"status": "ok"}
        return {"status": "error", "error": "Database connectivity check returned False"}
    except Exception as exc:
        return {"status": "error", "error": str(exc)}


def _check_rate_limit_backend():
    uri = _RUNTIME_CONFIG.rate_limit_storage_uri
    if _RUNTIME_CONFIG.is_production and uri == DEFAULT_RATE_LIMIT_STORAGE_URI:
        return {
            "status": "error",
            "backend": "memory",
            "error": "In-memory rate limiting is not allowed in production",
        }

    if not uri.startswith(("redis://", "rediss://")):
        return {
            "status": "ok",
            "backend": uri.split(":", 1)[0] or "memory",
            "details": "Reachability check skipped for non-Redis limiter backend",
        }

    try:
        import redis  # type: ignore

        client = redis.Redis.from_url(
            uri,
            socket_connect_timeout=2,
            socket_timeout=2,
        )
        client.ping()
        return {"status": "ok", "backend": "redis"}
    except Exception as exc:
        return {"status": "error", "backend": "redis", "error": str(exc)}


def _check_model_components():
    try:
        embedding_status = get_model_status()
        if not embedding_status.ready:
            return {
                "status": "error",
                "details": {
                    "embedder": {
                        "status": "error",
                        "model_name": embedding_status.model_name,
                        "revision": embedding_status.revision,
                        "local_path": embedding_status.local_path,
                        "error": embedding_status.error,
                    },
                    "reranker": {
                        "status": "optional",
                        "message": "Reranker loads only when rerank fusion is requested",
                    },
                },
                "error": embedding_status.error,
            }

        engine = _get_engine()
        engine_checks = (
            engine.get_component_status()
            if hasattr(engine, "get_component_status")
            else engine.get_readiness()
            if hasattr(engine, "get_readiness")
            else {}
        )
        return {
            "status": "ok",
            "details": engine_checks,
        }
    except Exception as exc:
        return {"status": "error", "error": str(exc)}


def _build_ready_payload():
    """Collect dependency-backed readiness details."""
    checks = {
        "runtime_config": _check_runtime_config(),
        "database": _check_database(),
        "rate_limit_backend": _check_rate_limit_backend(),
        "models": _check_model_components(),
    }

    ready = all(check["status"] == "ok" for check in checks.values())
    return {
        "status": "ok" if ready else "degraded",
        "service": "AssistSupport Hybrid Search API",
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "checks": checks,
    }, (200 if ready else 503)


def internal_error_response(error: Exception, *, context: str):
    """Return a safe error payload and avoid leaking internals in production."""
    # Never print raw exception messages in production logs; they can contain user input.
    if is_production():
        app.logger.error("%s: %s", context, error.__class__.__name__)
    else:
        app.logger.exception("%s", context)
    error_msg = str(error) if not is_production() else "Internal server error"
    return (
        jsonify(
            {
                "status": "error",
                "error": error_msg,
                "timestamp": datetime.now(timezone.utc).isoformat(),
            }
        ),
        500,
    )


def require_api_key(f):
    """Decorator for API key authentication"""

    @wraps(f)
    def decorated_function(*args, **kwargs):
        if AUTH_REQUIRED and API_KEY == DEFAULT_API_KEY:
            return (
                jsonify(
                    {
                        "error": "Server is misconfigured: ASSISTSUPPORT_API_KEY must be set when ASSISTSUPPORT_SEARCH_API_REQUIRE_AUTH is enabled"
                    }
                ),
                500,
            )

        if AUTH_REQUIRED:
            auth_header = request.headers.get("Authorization", "")
            if not auth_header.startswith("Bearer "):
                return (
                    jsonify({"error": "Missing or invalid Authorization header"}),
                    401,
                )

            token = auth_header.split(" ", 1)[1]
            if token != API_KEY:
                return jsonify({"error": "Invalid API key"}), 403

        return f(*args, **kwargs)

    return decorated_function


@app.route("/health", methods=["GET"])
def health():
    """Health check endpoint"""
    return (
        jsonify(
            {
                "status": "ok",
                "timestamp": datetime.now(timezone.utc).isoformat(),
                "service": "AssistSupport Hybrid Search API",
            }
        ),
        200,
    )


@app.route("/ready", methods=["GET"])
def ready():
    """Dependency-backed readiness endpoint."""
    payload, status_code = _build_ready_payload()
    return jsonify(payload), status_code


@app.route("/search", methods=["POST"])
@limiter.limit("100 per minute")
@require_api_key
def search():
    """
    Search endpoint

    Request body:
    {
        "query": "Can I use a flash drive?",
        "top_k": 10,
        "include_scores": true,
        "fusion_strategy": "adaptive"
    }
    """
    try:
        data = request.get_json(silent=True)

        if not data:
            return jsonify({"error": "Request body required"}), 400

        query_raw = data.get("query", "")
        if not isinstance(query_raw, str):
            return jsonify({"error": "Query must be a string"}), 400

        query = query_raw.strip()
        if not query:
            return jsonify({"error": "Query parameter required"}), 400

        top_k_raw = data.get("top_k", 10)
        if not isinstance(top_k_raw, int):
            return jsonify({"error": "top_k must be an integer"}), 400
        if top_k_raw < 1:
            return jsonify({"error": "top_k must be >= 1"}), 400
        top_k = min(top_k_raw, 50)

        include_scores = bool(data.get("include_scores", False))
        fusion_strategy_raw = data.get("fusion_strategy", "adaptive")
        if not isinstance(fusion_strategy_raw, str):
            return jsonify({"error": "fusion_strategy must be a string"}), 400
        fusion_strategy = fusion_strategy_raw.strip().lower()
        allowed_strategies = {"adaptive", "rrf", "weighted", "rerank"}
        if fusion_strategy not in allowed_strategies:
            return jsonify({"error": f"Invalid fusion_strategy: {fusion_strategy_raw}"}), 400

        engine = _get_engine()

        # Execute search (engine handles timing, logging, intent detection internally)
        result = engine.search(
            query,
            limit=top_k,
            use_deduplication=True,
            fusion_strategy=fusion_strategy,
        )

        # Format response using the dict-based result structure
        formatted_results = []
        for i, r in enumerate(result["results"], 1):
            res = {
                "rank": i,
                "article_id": r["article_id"],
                "title": r["title"],
                "category": r["category"],
                "preview": r["content_preview"],
                "source_document": r.get("source_document_id"),
                "section": r.get("heading_path"),
            }

            if include_scores:
                res["scores"] = {
                    "bm25": round(r["bm25_score"], 3),
                    "vector": round(r["vector_score"], 3),
                    "fused": round(r["fusion_score"], 3),
                }

            formatted_results.append(res)

        response = {
            "status": "success",
            "query": result["query"],
            "query_id": result.get("query_id"),
            "intent": result["intent"],
            "intent_confidence": round(result["intent_confidence"], 2),
            "results_count": len(formatted_results),
            "results": formatted_results,
            "metrics": {
                "latency_ms": round(result["metrics"]["total_time_ms"], 1),
                "embedding_time_ms": round(result["metrics"]["embedding_time_ms"], 1),
                "search_time_ms": round(result["metrics"]["search_time_ms"], 1),
                "rerank_time_ms": round(result["metrics"].get("rerank_time_ms", 0), 1),
                "result_count": len(formatted_results),
                "timestamp": datetime.now(timezone.utc).isoformat(),
            },
        }

        return jsonify(response), 200

    except Exception as e:
        return internal_error_response(e, context="Search error")


@app.route("/feedback", methods=["POST"])
@limiter.limit("100 per minute")
@require_api_key
def submit_feedback():
    """
    Feedback submission endpoint

    Request body:
    {
        "query_id": "uuid",
        "result_rank": 1,
        "rating": "helpful" | "not_helpful" | "incorrect",
        "comment": "optional user comment"
    }
    """
    try:
        data = request.get_json(silent=True)

        if not data:
            return jsonify({"error": "Request body required"}), 400

        query_id = data.get("query_id")
        result_rank = data.get("result_rank")
        rating = data.get("rating")
        comment = data.get("comment", "")
        article_id = data.get("article_id")

        if not all([query_id, result_rank is not None, rating]):
            return (
                jsonify({"error": "query_id, result_rank, and rating required"}),
                400,
            )

        if not isinstance(query_id, str) or not query_id.strip():
            return jsonify({"error": "query_id must be a non-empty string"}), 400

        if not isinstance(result_rank, int) or result_rank < 1:
            return jsonify({"error": "result_rank must be a positive integer"}), 400

        if not isinstance(rating, str):
            return jsonify({"error": "rating must be a string"}), 400

        if comment is None:
            comment = ""
        if not isinstance(comment, str):
            return jsonify({"error": "comment must be a string"}), 400

        if article_id is not None and not isinstance(article_id, str):
            return jsonify({"error": "article_id must be a string"}), 400

        if rating not in ("helpful", "not_helpful", "incorrect"):
            return jsonify({"error": f"Invalid rating: {rating}"}), 400

        engine = _get_engine()
        engine._log_feedback(query_id, result_rank, rating, comment, article_id)

        return (
            jsonify(
                {
                    "status": "success",
                    "message": "Feedback recorded",
                    "timestamp": datetime.now(timezone.utc).isoformat(),
                }
            ),
            200,
        )

    except Exception as e:
        return internal_error_response(e, context="Feedback error")


@app.route("/stats", methods=["GET"])
@require_api_key
def stats():
    """Stats endpoint for monitoring dashboard"""
    try:
        engine = _get_engine()
        data = engine._get_stats()

        return (
            jsonify(
                {
                    "status": "success",
                    "data": data,
                    "timestamp": datetime.now(timezone.utc).isoformat(),
                }
            ),
            200,
        )

    except Exception as e:
        return internal_error_response(e, context="Stats error")


@app.errorhandler(429)
def ratelimit_handler(e):
    return jsonify({"error": "Rate limit exceeded", "message": str(e.description)}), 429


@app.errorhandler(404)
def not_found(e):
    return jsonify({"error": "Endpoint not found", "path": request.path}), 404


def run_server():
    """Start the API server"""
    runtime_config = load_runtime_config()
    ensure_valid_runtime_config(
        runtime_config,
        check_backends=not runtime_config.is_production,
    )

    if runtime_config.is_production:
        raise RuntimeConfigError(
            "run_server() is for development only; use the WSGI entrypoint in production"
        )

    # Ensure we have a sane default logger in bare CLI mode.
    if not logging.getLogger().handlers:
        level = logging.INFO if is_production() else logging.DEBUG
        logging.basicConfig(level=level, format="%(asctime)s %(levelname)s %(name)s: %(message)s")

    app.logger.info("Starting AssistSupport Search API on port %s", runtime_config.api_port)
    app.logger.info("Environment: %s", runtime_config.environment)

    # Flask's debug mode implicitly enables the reloader, which forks a second process.
    # That is convenient for local iteration but confusing for "run the full stack"
    # workflows (multiple PIDs, double log lines, and easier to trip over port reuse).
    # Default: debug on (non-production), reloader off unless explicitly enabled.
    debug_enabled = runtime_config.environment != "production" and os.environ.get(
        "ASSISTSUPPORT_SEARCH_API_DEBUG", "1"
    ) in ("1", "true", "yes")
    use_reloader = debug_enabled and os.environ.get("ASSISTSUPPORT_SEARCH_API_RELOAD", "0") in (
        "1",
        "true",
        "yes",
    )

    app.run(
        host="localhost",
        port=runtime_config.api_port,
        debug=debug_enabled,
        use_reloader=use_reloader,
        threaded=True,
    )


if __name__ == "__main__":
    run_server()
