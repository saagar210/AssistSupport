#!/usr/bin/env python3
"""Smoke test for Search API liveness, readiness, and auth behavior."""

from __future__ import annotations

import json
import os
import sys

from runtime_config import load_runtime_config, ensure_valid_runtime_config


class _SmokeEngine:
    def search(self, query, limit=10, use_deduplication=True, fusion_strategy="adaptive"):
        return {
            "query": query,
            "query_id": "smoke-query-1",
            "intent": "policy",
            "intent_confidence": 0.91,
            "results": [
                {
                    "article_id": "article-1",
                    "title": "Smoke Policy",
                    "category": "POLICY",
                    "content_preview": "Smoke result preview",
                    "source_document_id": "doc-1",
                    "heading_path": "Policy > Smoke",
                    "bm25_score": 1.0,
                    "vector_score": 0.5,
                    "fusion_score": 0.75,
                }
            ],
            "metrics": {
                "total_time_ms": 12.5,
                "embedding_time_ms": 1.5,
                "search_time_ms": 4.0,
                "rerank_time_ms": 0.0,
            },
        }

    def get_readiness(self):
        return {
            "vector_search_enabled": True,
            "embedder_ready": True,
            "reranker_ready": True,
        }


def main() -> int:
    config = load_runtime_config()
    ensure_valid_runtime_config(config, check_backends=True)

    # Import after runtime validation so config problems fail clearly first.
    import search_api

    search_api.API_KEY = config.api_key
    search_api.app.config["TESTING"] = True
    smoke_engine = _SmokeEngine()
    search_api._engine = smoke_engine
    search_api._get_engine = lambda: smoke_engine

    with search_api.app.test_client() as client:
        health = client.get("/health")
        if health.status_code != 200:
            print("Health endpoint failed", file=sys.stderr)
            return 1

        auth_header = {"Authorization": f"Bearer {config.api_key}"}

        ready = client.get("/ready", headers=auth_header)
        if ready.status_code != 200:
            print(
                f"Ready endpoint failed with status {ready.status_code}",
                file=sys.stderr,
            )
            return 1

        no_auth = client.post("/search", json={"query": "test"})
        if no_auth.status_code != 401:
            print(
                f"Expected 401 for unauthenticated /search in production, got {no_auth.status_code}",
                file=sys.stderr,
            )
            return 1

        authed_search = client.post(
            "/search",
            json={"query": "test", "top_k": 3},
            headers=auth_header,
        )
        if authed_search.status_code != 200:
            print(
                f"Expected 200 for authenticated /search in production, got {authed_search.status_code}",
                file=sys.stderr,
            )
            return 1

        body = {
            "health_status": health.status_code,
            "ready_status": ready.status_code,
            "search_without_auth_status": no_auth.status_code,
            "search_with_auth_status": authed_search.status_code,
            "environment": config.environment,
            "rate_limit_storage_uri": config.rate_limit_storage_uri,
        }
        print(json.dumps(body, indent=2, sort_keys=True))

    return 0


if __name__ == "__main__":
    if os.environ.get("ENVIRONMENT", "development").lower() != "production":
        print("smoke_search_api.py expects ENVIRONMENT=production", file=sys.stderr)
        raise SystemExit(1)
    raise SystemExit(main())
