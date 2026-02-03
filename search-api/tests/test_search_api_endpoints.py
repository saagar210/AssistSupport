import sys
import types
from pathlib import Path

import pytest


SEARCH_API_DIR = Path(__file__).resolve().parents[1]
if str(SEARCH_API_DIR) not in sys.path:
    sys.path.insert(0, str(SEARCH_API_DIR))


# Stub heavy modules before importing the Flask app module.
if "hybrid_search" not in sys.modules:
    hybrid_search_stub = types.ModuleType("hybrid_search")

    class _PlaceholderHybridSearchEngine:
        pass

    hybrid_search_stub.HybridSearchEngine = _PlaceholderHybridSearchEngine
    sys.modules["hybrid_search"] = hybrid_search_stub

if "intent_detection" not in sys.modules:
    intent_detection_stub = types.ModuleType("intent_detection")

    class _PlaceholderIntentDetector:
        pass

    intent_detection_stub.IntentDetector = _PlaceholderIntentDetector
    sys.modules["intent_detection"] = intent_detection_stub

import search_api  # noqa: E402


class FakeEngine:
    def __init__(self):
        self.feedback_calls = []
        self.mode = "ok"

    def search(self, query, limit=10, use_deduplication=True, fusion_strategy="adaptive"):
        if self.mode == "error":
            raise RuntimeError("simulated backend failure")
        if self.mode == "partial":
            return {"query": query, "results": []}

        return {
            "query": query,
            "query_id": "query-123",
            "intent": "policy",
            "intent_confidence": 0.88,
            "results": [
                {
                    "article_id": "article-1",
                    "title": "USB Policy",
                    "category": "POLICY",
                    "content_preview": "USB devices are restricted",
                    "source_document_id": "doc-1",
                    "heading_path": "Policy > USB",
                    "bm25_score": 1.2,
                    "vector_score": 0.81,
                    "fusion_score": 0.94,
                }
            ],
            "metrics": {
                "total_time_ms": 16.2,
                "embedding_time_ms": 2.1,
                "search_time_ms": 6.7,
                "rerank_time_ms": 0.0,
            },
        }

    def _log_feedback(self, query_id, result_rank, rating, comment="", article_id=None):
        self.feedback_calls.append(
            {
                "query_id": query_id,
                "result_rank": result_rank,
                "rating": rating,
                "comment": comment,
                "article_id": article_id,
            }
        )

    def _get_stats(self):
        return {
            "queries_total": 10,
            "queries_24h": 3,
            "latency_ms": {"avg": 12.3, "p50": 11.0, "p95": 19.1, "p99": 22.0},
            "intent_distribution": {"policy": 2, "procedure": 1},
            "feedback_stats": {"helpful": 2, "incorrect": 1},
        }


@pytest.fixture
def client(monkeypatch):
    fake = FakeEngine()
    monkeypatch.setattr(search_api, "_engine", fake)
    monkeypatch.setattr(search_api, "_get_engine", lambda: fake)
    search_api.app.config["TESTING"] = True
    with search_api.app.test_client() as test_client:
        yield test_client, fake


def test_health_endpoint(client):
    test_client, _ = client
    response = test_client.get("/health")
    assert response.status_code == 200
    body = response.get_json()
    assert body["status"] == "ok"
    assert "timestamp" in body


def test_search_happy_path_with_scores(client, monkeypatch):
    test_client, fake_engine = client
    monkeypatch.setenv("ENVIRONMENT", "development")

    response = test_client.post(
        "/search",
        json={"query": "Can I use a flash drive?", "top_k": 5, "include_scores": True},
    )
    assert response.status_code == 200

    body = response.get_json()
    assert body["status"] == "success"
    assert body["query"] == "Can I use a flash drive?"
    assert body["results_count"] == 1
    assert body["results"][0]["scores"]["fused"] == pytest.approx(0.94, rel=1e-3)
    assert fake_engine.feedback_calls == []


def test_search_rejects_invalid_or_malformed_input(client):
    test_client, _ = client

    malformed = test_client.post(
        "/search",
        data="{not-json",
        content_type="application/json",
    )
    assert malformed.status_code == 400
    assert malformed.get_json()["error"] == "Request body required"

    non_string = test_client.post("/search", json={"query": ["bad"], "top_k": 3})
    assert non_string.status_code == 400
    assert non_string.get_json()["error"] == "Query must be a string"

    bad_top_k = test_client.post("/search", json={"query": "ok", "top_k": "ten"})
    assert bad_top_k.status_code == 400
    assert bad_top_k.get_json()["error"] == "top_k must be an integer"


def test_search_engine_failures_are_handled(client):
    test_client, fake_engine = client

    fake_engine.mode = "error"
    response = test_client.post("/search", json={"query": "vpn setup"})
    assert response.status_code == 500
    assert response.get_json()["status"] == "error"

    fake_engine.mode = "partial"
    response = test_client.post("/search", json={"query": "vpn setup"})
    assert response.status_code == 500
    assert response.get_json()["status"] == "error"


def test_feedback_endpoint_validation_and_success(client):
    test_client, fake_engine = client

    invalid_rank = test_client.post(
        "/feedback",
        json={"query_id": "q1", "result_rank": "first", "rating": "helpful"},
    )
    assert invalid_rank.status_code == 400
    assert invalid_rank.get_json()["error"] == "result_rank must be a positive integer"

    invalid_rating = test_client.post(
        "/feedback",
        json={"query_id": "q1", "result_rank": 1, "rating": "great"},
    )
    assert invalid_rating.status_code == 400
    assert "Invalid rating" in invalid_rating.get_json()["error"]

    success = test_client.post(
        "/feedback",
        json={
            "query_id": "q1",
            "result_rank": 1,
            "rating": "helpful",
            "comment": "worked",
            "article_id": "article-1",
        },
    )
    assert success.status_code == 200
    assert success.get_json()["status"] == "success"
    assert len(fake_engine.feedback_calls) == 1
    assert fake_engine.feedback_calls[0]["rating"] == "helpful"


def test_stats_and_not_found_endpoints(client):
    test_client, _ = client

    stats = test_client.get("/stats")
    assert stats.status_code == 200
    stats_body = stats.get_json()
    assert stats_body["status"] == "success"
    assert stats_body["data"]["queries_total"] == 10

    not_found = test_client.get("/missing-endpoint")
    assert not_found.status_code == 404
    assert not_found.get_json()["error"] == "Endpoint not found"


def test_authentication_checks_apply_in_production(client, monkeypatch):
    test_client, _ = client
    monkeypatch.setenv("ENVIRONMENT", "production")
    monkeypatch.setattr(search_api, "API_KEY", "secret-key")

    missing_auth = test_client.post("/search", json={"query": "policy"})
    assert missing_auth.status_code == 401

    wrong_auth = test_client.post(
        "/search",
        json={"query": "policy"},
        headers={"Authorization": "Bearer wrong"},
    )
    assert wrong_auth.status_code == 403

    correct_auth = test_client.post(
        "/search",
        json={"query": "policy"},
        headers={"Authorization": "Bearer secret-key"},
    )
    assert correct_auth.status_code == 200

    # Reset env for neighboring tests if run in shared process.
    monkeypatch.setenv("ENVIRONMENT", "development")
