import json
import sys
import types
from pathlib import Path

import pytest


SEARCH_API_DIR = Path(__file__).resolve().parents[1]
REPO_ROOT = SEARCH_API_DIR.parent

if str(SEARCH_API_DIR) not in sys.path:
    sys.path.insert(0, str(SEARCH_API_DIR))


if "hybrid_search" not in sys.modules:
    hybrid_search_stub = types.ModuleType("hybrid_search")

    class _PlaceholderHybridSearchEngine:
        pass

    hybrid_search_stub.HybridSearchEngine = _PlaceholderHybridSearchEngine
    sys.modules["hybrid_search"] = hybrid_search_stub

try:
    # Prefer the real module so intent-detection unit tests are not polluted by a stub.
    import intent_detection  # noqa: F401
except Exception:
    intent_detection_stub = types.ModuleType("intent_detection")

    class _PlaceholderIntentDetector:
        pass

    intent_detection_stub.IntentDetector = _PlaceholderIntentDetector
    sys.modules["intent_detection"] = intent_detection_stub

import search_api  # noqa: E402


def _load_fixture(relative_path: str) -> dict:
    fixture_path = REPO_ROOT / relative_path
    return json.loads(fixture_path.read_text(encoding="utf-8"))


def _shape(value):
    if isinstance(value, dict):
        return {key: _shape(value[key]) for key in sorted(value.keys())}
    if isinstance(value, list):
        if not value:
            return []
        return [_shape(value[0])]
    return type(value).__name__


class _FakeEngine:
    def __init__(self):
        self.feedback_calls = []

    def search(self, query, limit=10, use_deduplication=True, fusion_strategy="adaptive"):
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
    fake = _FakeEngine()
    monkeypatch.setattr(search_api, "_engine", fake)
    monkeypatch.setattr(search_api, "_get_engine", lambda: fake)
    monkeypatch.setattr(search_api, "AUTH_REQUIRED", False)
    search_api.app.config["TESTING"] = True

    with search_api.app.test_client() as test_client:
        yield test_client


def test_search_response_shape_matches_fixture(client):
    expected = _load_fixture("contracts/search-api/v1/search_response.json")
    response = client.post(
        "/search",
        json={"query": "Can I use a flash drive?", "top_k": 5, "include_scores": True},
    )
    assert response.status_code == 200
    assert _shape(response.get_json()) == _shape(expected)


def test_feedback_response_shape_matches_fixture(client):
    expected = _load_fixture("contracts/search-api/v1/feedback_response.json")
    response = client.post(
        "/feedback",
        json={
            "query_id": "q1",
            "result_rank": 1,
            "rating": "helpful",
            "comment": "works",
            "article_id": "article-1",
        },
    )
    assert response.status_code == 200
    assert _shape(response.get_json()) == _shape(expected)


def test_stats_response_shape_matches_fixture(client):
    expected = _load_fixture("contracts/search-api/v1/stats_response.json")
    response = client.get("/stats")
    assert response.status_code == 200
    assert _shape(response.get_json()) == _shape(expected)


def test_health_response_shape_matches_fixture(client):
    expected = _load_fixture("contracts/search-api/v1/health_response.json")
    response = client.get("/health")
    assert response.status_code == 200
    assert _shape(response.get_json()) == _shape(expected)
