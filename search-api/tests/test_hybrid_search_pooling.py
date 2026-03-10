import sys
import types
from contextlib import contextmanager
from pathlib import Path

import pytest


SEARCH_API_DIR = Path(__file__).resolve().parents[1]
if str(SEARCH_API_DIR) not in sys.path:
    sys.path.insert(0, str(SEARCH_API_DIR))


ORIGINAL_EMBEDDING_SERVICE = sys.modules.get("embedding_service")
ORIGINAL_RERANKER = sys.modules.get("reranker")
ORIGINAL_INTENT_DETECTION = sys.modules.get("intent_detection")
ORIGINAL_SCORE_FUSION = sys.modules.get("score_fusion")
ORIGINAL_FEEDBACK_LOOP = sys.modules.get("feedback_loop")


embedding_service_stub = types.ModuleType("embedding_service")


class _EmbeddingService:
    def __init__(self):
        self.model_name = "fake-embedder"
        self.dimension = 2

    def embed_query(self, _query):
        return [1.0, 0.0]


embedding_service_stub.EmbeddingService = _EmbeddingService
sys.modules["embedding_service"] = embedding_service_stub

reranker_stub = types.ModuleType("reranker")


class _Reranker:
    def __init__(self):
        self.model_name = "fake-reranker"

    def rerank(self, _query, candidates, top_k=10):
        return candidates[:top_k]


reranker_stub.Reranker = _Reranker
sys.modules["reranker"] = reranker_stub

intent_detection_stub = types.ModuleType("intent_detection")


class _IntentDetector:
    @staticmethod
    def detect(_query):
        return "policy", 0.9


intent_detection_stub.IntentDetector = _IntentDetector
sys.modules["intent_detection"] = intent_detection_stub

score_fusion_stub = types.ModuleType("score_fusion")


class _ScoreFusion:
    @staticmethod
    def adaptive_fusion(bm25_results, _vector_results, _intent):
        return bm25_results

    @staticmethod
    def reciprocal_rank_fusion(bm25_results, _vector_results):
        return bm25_results

    @staticmethod
    def weighted_combination(bm25_results, _vector_results):
        return bm25_results


score_fusion_stub.ScoreFusion = _ScoreFusion
sys.modules["score_fusion"] = score_fusion_stub

feedback_loop_stub = types.ModuleType("feedback_loop")
feedback_loop_stub.get_quality_scores = lambda _conn, _ids: {}
sys.modules["feedback_loop"] = feedback_loop_stub

import hybrid_search  # noqa: E402


def _restore_module(name, original):
    if original is not None:
        sys.modules[name] = original
    else:
        sys.modules.pop(name, None)


_restore_module("embedding_service", ORIGINAL_EMBEDDING_SERVICE)
_restore_module("reranker", ORIGINAL_RERANKER)
_restore_module("intent_detection", ORIGINAL_INTENT_DETECTION)
_restore_module("score_fusion", ORIGINAL_SCORE_FUSION)
_restore_module("feedback_loop", ORIGINAL_FEEDBACK_LOOP)


class FakeCursor:
    def __init__(self):
        self.last_query = ""

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc, tb):
        return False

    def close(self):
        return None

    def execute(self, query, _params=None):
        self.last_query = " ".join(query.split())

    def fetchone(self):
        if "to_regtype('vector')" in self.last_query:
            return (True,)
        if "INSERT INTO query_performance" in self.last_query:
            return ("query-123",)
        if "SELECT COUNT(*) FROM query_performance WHERE created_at > NOW() - INTERVAL '24 hours'" in self.last_query:
            return (2,)
        if "SELECT COUNT(*) FROM query_performance" in self.last_query:
            return (5,)
        if "AVG(response_time_ms)" in self.last_query:
            return (10.0, 9.0, 15.0, 20.0)
        return (1,)

    def fetchall(self):
        if "FROM kb_articles, plainto_tsquery" in self.last_query:
            return [("article-1", 1.0)]
        if "ORDER BY embedding <=>" in self.last_query:
            return [("article-1", 0.9)]
        if "SELECT id, category FROM kb_articles" in self.last_query:
            return [("article-1", "POLICY")]
        if "SELECT id, source_document_id, chunk_index FROM kb_articles" in self.last_query:
            return [("article-1", "doc-1", 0)]
        if "SELECT id, title, content, category, source_document_id, chunk_index, heading_path FROM kb_articles" in self.last_query:
            return [
                (
                    "article-1",
                    "USB Policy",
                    "USB devices are restricted",
                    "POLICY",
                    "doc-1",
                    0,
                    "Policy > USB",
                )
            ]
        if "GROUP BY category_filter" in self.last_query:
            return [("policy", 2)]
        if "GROUP BY rating" in self.last_query:
            return [("helpful", 1)]
        return []


class FakeConnection:
    def cursor(self):
        return FakeCursor()


def test_search_uses_fresh_pooled_session_per_request(monkeypatch):
    sessions = []

    @contextmanager
    def fake_pooled_connection():
        conn = FakeConnection()
        sessions.append(conn)
        yield conn

    monkeypatch.setattr(hybrid_search, "pooled_connection", fake_pooled_connection)

    engine = hybrid_search.HybridSearchEngine()
    baseline = len(sessions)

    first = engine.search("flash drives", limit=1)
    second = engine.search("password reset", limit=1)

    assert first["query_id"] == "query-123"
    assert second["query_id"] == "query-123"
    assert len(sessions) == baseline + 2
    assert len({id(conn) for conn in sessions[baseline:]}) == 2


def test_feedback_logging_uses_fresh_pooled_connections(monkeypatch):
    sessions = []

    @contextmanager
    def fake_pooled_connection():
        conn = FakeConnection()
        sessions.append(conn)
        yield conn

    monkeypatch.setattr(hybrid_search, "pooled_connection", fake_pooled_connection)

    engine = hybrid_search.HybridSearchEngine()
    baseline = len(sessions)

    engine._log_feedback("q1", 1, "helpful", "worked", "article-1")
    engine._log_feedback("q2", 2, "incorrect", "", "article-2")

    assert len(sessions) == baseline + 2
    assert len({id(conn) for conn in sessions[baseline:]}) == 2


def test_feedback_logging_surfaces_pooled_connection_failures(monkeypatch):
    @contextmanager
    def failing_pooled_connection():
        raise RuntimeError("pool exhausted")
        yield  # pragma: no cover

    monkeypatch.setattr(hybrid_search, "pooled_connection", failing_pooled_connection)

    engine = hybrid_search.HybridSearchEngine()

    with pytest.raises(RuntimeError, match="pool exhausted"):
        engine._log_feedback("q1", 1, "helpful", "worked", "article-1")


def test_stats_uses_pooled_session(monkeypatch):
    sessions = []

    @contextmanager
    def fake_pooled_connection():
        conn = FakeConnection()
        sessions.append(conn)
        yield conn

    monkeypatch.setattr(hybrid_search, "pooled_connection", fake_pooled_connection)

    engine = hybrid_search.HybridSearchEngine()
    baseline = len(sessions)

    stats = engine._get_stats()

    assert len(sessions) == baseline + 1
    assert stats["queries_total"] == 5
    assert stats["queries_24h"] == 2
    assert stats["intent_distribution"]["policy"] == 2
