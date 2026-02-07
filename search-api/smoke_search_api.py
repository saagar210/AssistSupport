#!/usr/bin/env python3
"""Smoke test for Search API auth and health behavior under production config."""

from __future__ import annotations

import json
import os
import sys

from runtime_config import load_runtime_config, ensure_valid_runtime_config


def main() -> int:
    config = load_runtime_config()
    ensure_valid_runtime_config(config, check_backends=True)

    # Import after runtime validation so config problems fail clearly first.
    import search_api

    search_api.API_KEY = config.api_key
    search_api.app.config["TESTING"] = True

    with search_api.app.test_client() as client:
        health = client.get("/health")
        if health.status_code != 200:
            print("Health endpoint failed", file=sys.stderr)
            return 1

        no_auth = client.post("/search", json={"query": "test"})
        if no_auth.status_code != 401:
            print(
                f"Expected 401 for unauthenticated /search in production, got {no_auth.status_code}",
                file=sys.stderr,
            )
            return 1

        body = {
            "health_status": health.status_code,
            "search_without_auth_status": no_auth.status_code,
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
