#!/usr/bin/env python3
"""
Production Search API for AssistSupport
Hybrid search endpoint with authentication, rate limiting, and monitoring
"""

import sys
import os

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from flask import Flask, request, jsonify
from flask_limiter import Limiter
from flask_limiter.util import get_remote_address
from functools import wraps
from datetime import datetime
import time

from hybrid_search import HybridSearchEngine
from intent_detection import IntentDetector

# Initialize Flask app
app = Flask(__name__)
app.config["JSON_SORT_KEYS"] = False

# Rate limiting: 100 requests per minute per IP
limiter = Limiter(
    app=app,
    key_func=get_remote_address,
    default_limits=["100 per minute"],
)

# Configuration
API_KEY = os.environ.get("ASSISTSUPPORT_API_KEY", "dev-key-change-in-production")
API_PORT = int(os.environ.get("ASSISTSUPPORT_API_PORT", "3000"))

# Global search engine instance (initialized on first request)
_engine = None


def _get_engine():
    """Lazy-initialize the search engine singleton"""
    global _engine
    if _engine is None:
        _engine = HybridSearchEngine()
        print("Search engine initialized")
    return _engine


def require_api_key(f):
    """Decorator for API key authentication"""

    @wraps(f)
    def decorated_function(*args, **kwargs):
        if os.environ.get("ENVIRONMENT") == "production":
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
                "timestamp": datetime.utcnow().isoformat(),
                "service": "AssistSupport Hybrid Search API",
            }
        ),
        200,
    )


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
        data = request.get_json()

        if not data:
            return jsonify({"error": "Request body required"}), 400

        query = data.get("query", "").strip()
        if not query:
            return jsonify({"error": "Query parameter required"}), 400

        top_k = min(data.get("top_k", 10), 50)
        include_scores = data.get("include_scores", False)
        fusion_strategy = data.get("fusion_strategy", "adaptive")

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
                "timestamp": datetime.utcnow().isoformat(),
            },
        }

        return jsonify(response), 200

    except Exception as e:
        print(f"Search error: {e}")
        return (
            jsonify(
                {
                    "status": "error",
                    "error": str(e),
                    "timestamp": datetime.utcnow().isoformat(),
                }
            ),
            500,
        )


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
        data = request.get_json()

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

        if rating not in ("helpful", "not_helpful", "incorrect"):
            return jsonify({"error": f"Invalid rating: {rating}"}), 400

        engine = _get_engine()
        engine._log_feedback(query_id, result_rank, rating, comment, article_id)

        return (
            jsonify(
                {
                    "status": "success",
                    "message": "Feedback recorded",
                    "timestamp": datetime.utcnow().isoformat(),
                }
            ),
            200,
        )

    except Exception as e:
        print(f"Feedback error: {e}")
        return jsonify({"status": "error", "error": str(e)}), 500


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
                    "timestamp": datetime.utcnow().isoformat(),
                }
            ),
            200,
        )

    except Exception as e:
        return jsonify({"status": "error", "error": str(e)}), 500


@app.route("/config", methods=["GET"])
def config():
    """Configuration endpoint (no auth required)"""
    return (
        jsonify(
            {
                "api_url": f"http://localhost:{API_PORT}",
                "version": "1.0.0",
                "features": {
                    "hybrid_search": True,
                    "intent_detection": True,
                    "feedback_collection": True,
                },
            }
        ),
        200,
    )


@app.errorhandler(429)
def ratelimit_handler(e):
    return jsonify({"error": "Rate limit exceeded", "message": str(e.description)}), 429


@app.errorhandler(404)
def not_found(e):
    return jsonify({"error": "Endpoint not found", "path": request.path}), 404


def run_server():
    """Start the API server"""
    print(f"Starting AssistSupport Search API on port {API_PORT}")
    print(f"  Environment: {os.environ.get('ENVIRONMENT', 'development')}")

    app.run(
        host="localhost",
        port=API_PORT,
        debug=os.environ.get("ENVIRONMENT") != "production",
        threaded=True,
    )


if __name__ == "__main__":
    run_server()
