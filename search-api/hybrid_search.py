#!/usr/bin/env python3
"""
AssistSupport Hybrid Search Engine
Combines BM25 (keyword) + HNSW (vector) + intent detection + logging
"""

import sys
import os
import time
from typing import List, Dict, Tuple

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from db_config import connect_db
from embedding_service import EmbeddingService
from score_fusion import ScoreFusion
from intent_detection import IntentDetector
from reranker import Reranker
from feedback_loop import get_quality_scores


class HybridSearchEngine:
    """Production hybrid search combining FTS + vector + intent detection"""

    def __init__(self):
        self.conn = connect_db()
        self.conn.autocommit = True
        self.cur = self.conn.cursor()
        # Set HNSW ef_search once at connection time
        self.cur.execute("SET hnsw.ef_search = 100")
        self.vector_search_enabled = self._detect_vector_capability()
        self.embedder = EmbeddingService()
        self.reranker = Reranker()
        print("Hybrid search engine initialized")

    def _detect_vector_capability(self) -> bool:
        """Detect whether pgvector is available before issuing vector SQL."""
        try:
            self.cur.execute("SELECT to_regtype('vector') IS NOT NULL")
            row = self.cur.fetchone()
            enabled = bool(row and row[0])
            if not enabled:
                print("Warning: pgvector extension not available; vector search disabled")
            return enabled
        except Exception as e:
            print(f"Warning: Failed to detect vector capability: {e}")
            return False

    def search(
        self,
        query: str,
        limit: int = 10,
        use_deduplication: bool = True,
        fusion_strategy: str = "adaptive",
    ) -> Dict:
        """
        Execute hybrid search

        Returns dict with query, intent, results, and metrics.
        """
        start_total = time.time()

        # Step 1: Detect intent
        intent, intent_conf = IntentDetector.detect(query)

        # Step 2: Generate query embedding
        start_embed = time.time()
        query_embedding = self.embedder.embed_query(query)
        embed_time = (time.time() - start_embed) * 1000

        # Step 3: Execute both searches
        start_search = time.time()
        bm25_results = self._bm25_search(query)
        vector_results = self._vector_search(query_embedding, limit * 2)
        search_time = (time.time() - start_search) * 1000

        # Step 4: Fuse scores
        start_fusion = time.time()
        if fusion_strategy == "rrf":
            fused = ScoreFusion.reciprocal_rank_fusion(bm25_results, vector_results)
        elif fusion_strategy == "weighted":
            fused = ScoreFusion.weighted_combination(bm25_results, vector_results)
        else:  # adaptive
            fused = ScoreFusion.adaptive_fusion(bm25_results, vector_results, intent)
        fusion_time = (time.time() - start_fusion) * 1000

        # Step 5: Category boost for intent-matching results
        if intent in ("policy", "procedure", "reference") and intent_conf >= 0.3:
            fused = self._apply_category_boost(fused, intent)

        # Step 5.5: Apply quality score from feedback loop
        fused = self._apply_quality_scores(fused)

        # Step 6: Deduplicate
        if use_deduplication:
            fused = self._deduplicate_results(fused)

        # Step 7: Fetch full articles
        results = self._fetch_and_format_results(
            fused, bm25_results, vector_results, limit
        )

        # Step 8: Cross-encoder re-ranking (optional, disabled by default)
        rerank_time = 0
        if fusion_strategy == "rerank":
            rerank_pool_results = self._fetch_and_format_results(
                fused, bm25_results, vector_results, min(limit * 2, 20)
            )
            start_rerank = time.time()
            results = self.reranker.rerank(query, rerank_pool_results, top_k=limit)
            rerank_time = (time.time() - start_rerank) * 1000

        # Step 9: Log query
        total_time = (time.time() - start_total) * 1000
        query_id = self._log_query(
            query,
            intent,
            intent_conf,
            len(bm25_results),
            len(vector_results),
            len(results),
            total_time,
            fusion_strategy,
        )

        return {
            "query": query,
            "query_id": query_id,
            "intent": intent,
            "intent_confidence": intent_conf,
            "results": results,
            "metrics": {
                "total_results": len(results),
                "total_time_ms": total_time,
                "embedding_time_ms": embed_time,
                "search_time_ms": search_time,
                "fusion_time_ms": fusion_time,
                "rerank_time_ms": rerank_time,
            },
        }

    def _bm25_search(self, query: str) -> List[Tuple[str, float]]:
        """Execute BM25 keyword search via FTS using plainto_tsquery"""
        try:
            self.cur.execute(
                """
                SELECT id, ts_rank(fts_content, query) as bm25_score
                FROM kb_articles, plainto_tsquery('english', %s) query
                WHERE fts_content @@ query AND is_active = true
                ORDER BY bm25_score DESC
                LIMIT 50
                """,
                (query,),
            )
            return [(str(row[0]), float(row[1])) for row in self.cur.fetchall()]
        except Exception as e:
            print(f"BM25 search error: {e}")
            return []

    def _apply_category_boost(
        self,
        fused_results: List[Tuple[str, float]],
        intent: str,
    ) -> List[Tuple[str, float]]:
        """Boost results whose category matches the detected intent."""
        article_ids = [r[0] for r in fused_results[:30]]  # Check top 30
        if not article_ids:
            return fused_results

        # Map intent to DB category
        intent_to_category = {
            "policy": "POLICY",
            "procedure": "PROCEDURE",
            "reference": "REFERENCE",
        }
        target_category = intent_to_category.get(intent)
        if not target_category:
            return fused_results

        placeholders = ",".join(["%s"] * len(article_ids))
        self.cur.execute(
            f"SELECT id, category FROM kb_articles WHERE id IN ({placeholders})",
            article_ids,
        )
        category_map = {str(row[0]): row[1] for row in self.cur.fetchall()}

        boosted = []
        for article_id, score in fused_results:
            cat = category_map.get(article_id, "")
            if cat == target_category:
                score *= 1.20  # 20% boost for category match
            boosted.append((article_id, score))

        return sorted(boosted, key=lambda x: x[1], reverse=True)

    def _apply_quality_scores(
        self, fused_results: List[Tuple[str, float]]
    ) -> List[Tuple[str, float]]:
        """Multiply fusion scores by per-article quality scores from feedback."""
        article_ids = [r[0] for r in fused_results[:30]]
        if not article_ids:
            return fused_results
        q_scores = get_quality_scores(self.conn, article_ids)
        adjusted = []
        for article_id, score in fused_results:
            qs = q_scores.get(article_id, 1.0)
            adjusted.append((article_id, score * qs))
        return sorted(adjusted, key=lambda x: x[1], reverse=True)

    def _vector_search(
        self, query_embedding, limit: int
    ) -> List[Tuple[str, float]]:
        """Execute HNSW vector semantic search"""
        if not self.vector_search_enabled:
            return []

        embedding_str = "[" + ",".join(f"{x:.6f}" for x in query_embedding) + "]"
        try:
            self.cur.execute(
                """
                SELECT id, 1 - (embedding <=> %s::vector) as cosine_similarity
                FROM kb_articles
                WHERE embedding IS NOT NULL AND is_active = true
                ORDER BY embedding <=> %s::vector
                LIMIT %s
                """,
                (embedding_str, embedding_str, limit),
            )
            return [(str(row[0]), float(row[1])) for row in self.cur.fetchall()]
        except Exception as e:
            print(f"Vector search unavailable, falling back to BM25 only: {e}")
            self.vector_search_enabled = False
            return []

    def _deduplicate_results(
        self, fused_results: List[Tuple[str, float]]
    ) -> List[Tuple[str, float]]:
        """Remove near-duplicate articles (same source document)"""
        article_ids = [r[0] for r in fused_results]
        if not article_ids:
            return fused_results

        placeholders = ",".join(["%s"] * len(article_ids))
        self.cur.execute(
            f"""
            SELECT id, source_document_id, chunk_index
            FROM kb_articles WHERE id IN ({placeholders})
            """,
            article_ids,
        )

        doc_map = {str(row[0]): (row[1], row[2]) for row in self.cur.fetchall()}

        seen_docs = {}
        deduplicated = []

        for article_id, fusion_score in fused_results:
            if article_id not in doc_map:
                deduplicated.append((article_id, fusion_score))
                continue

            doc_id, chunk_idx = doc_map[article_id]

            if doc_id is None or doc_id not in seen_docs:
                if doc_id is not None:
                    seen_docs[doc_id] = (article_id, fusion_score)
                deduplicated.append((article_id, fusion_score))

        return deduplicated

    def _fetch_and_format_results(
        self,
        fused: List[Tuple[str, float]],
        bm25_results: List[Tuple[str, float]],
        vector_results: List[Tuple[str, float]],
        limit: int,
    ) -> List[Dict]:
        """Fetch full article data and format for response"""
        bm25_map = {r[0]: r[1] for r in bm25_results}
        vector_map = {r[0]: r[1] for r in vector_results}

        article_ids = [r[0] for r in fused[:limit]]
        if not article_ids:
            return []

        placeholders = ",".join(["%s"] * len(article_ids))
        self.cur.execute(
            f"""
            SELECT id, title, content, category, source_document_id, chunk_index, heading_path
            FROM kb_articles WHERE id IN ({placeholders})
            """,
            article_ids,
        )

        articles = {str(row[0]): row for row in self.cur.fetchall()}

        results = []
        for article_id, fusion_score in fused[:limit]:
            if article_id not in articles:
                continue

            article = articles[article_id]
            content = article[2] or ""

            results.append(
                {
                    "article_id": str(article[0]),
                    "title": article[1],
                    "content_preview": content[:200] + ("..." if len(content) > 200 else ""),
                    "category": article[3],
                    "bm25_score": bm25_map.get(article_id, 0.0),
                    "vector_score": vector_map.get(article_id, 0.0),
                    "fusion_score": fusion_score,
                    "source_document_id": article[4],
                    "heading_path": article[6],
                }
            )

        return results

    def _log_query(
        self,
        query: str,
        intent: str,
        confidence: float,
        bm25_count: int,
        vector_count: int,
        results_count: int,
        time_ms: float,
        fusion_strategy: str,
    ):
        """Log search query to query_performance table"""
        try:
            self.cur.execute(
                """
                INSERT INTO query_performance
                    (query_text, ef_search_used, bm25_results_count,
                     vector_results_count, results_returned, response_time_ms,
                     recall_estimate, category_filter, intent_confidence, fusion_strategy)
                VALUES (%s, %s, %s, %s, %s, %s, %s, %s, %s, %s)
                RETURNING id
                """,
                (
                    query,
                    100,
                    bm25_count,
                    vector_count,
                    results_count,
                    time_ms,
                    confidence,
                    intent,
                    confidence,
                    fusion_strategy,
                ),
            )
            row = self.cur.fetchone()
            return str(row[0]) if row else None
        except Exception as e:
            print(f"Warning: Failed to log query: {e}")
            return None

    def _log_feedback(
        self, query_id: str, result_rank: int, rating: str, comment: str = "",
        article_id: str = None
    ):
        """Log user feedback on search results"""
        try:
            self.cur.execute(
                """
                INSERT INTO search_feedback (query_id, result_rank, rating, comment, article_id)
                VALUES (%s, %s, %s, %s, %s)
                """,
                (query_id, result_rank, rating, comment, article_id),
            )
        except Exception as e:
            print(f"Feedback logging error: {e}")

    def _get_stats(self) -> dict:
        """Get search statistics for monitoring"""
        self.cur.execute("SELECT COUNT(*) FROM query_performance")
        total_queries = self.cur.fetchone()[0]

        self.cur.execute(
            """
            SELECT COUNT(*) FROM query_performance
            WHERE created_at > NOW() - INTERVAL '24 hours'
            """
        )
        queries_24h = self.cur.fetchone()[0]

        self.cur.execute(
            """
            SELECT
                ROUND(AVG(response_time_ms)::numeric, 1) as avg_latency,
                ROUND(PERCENTILE_CONT(0.50) WITHIN GROUP (ORDER BY response_time_ms)::numeric, 1) as p50,
                ROUND(PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY response_time_ms)::numeric, 1) as p95,
                ROUND(PERCENTILE_CONT(0.99) WITHIN GROUP (ORDER BY response_time_ms)::numeric, 1) as p99
            FROM query_performance
            WHERE created_at > NOW() - INTERVAL '24 hours'
            """
        )
        lat = self.cur.fetchone()

        self.cur.execute(
            """
            SELECT category_filter, COUNT(*)
            FROM query_performance
            WHERE created_at > NOW() - INTERVAL '24 hours'
            AND category_filter IS NOT NULL
            GROUP BY category_filter
            """
        )
        intent_dist = {row[0]: row[1] for row in self.cur.fetchall()}

        self.cur.execute(
            """
            SELECT rating, COUNT(*) FROM search_feedback
            WHERE created_at > NOW() - INTERVAL '24 hours'
            GROUP BY rating
            """
        )
        feedback_stats = {row[0]: row[1] for row in self.cur.fetchall()}

        return {
            "queries_total": total_queries,
            "queries_24h": queries_24h,
            "latency_ms": {
                "avg": float(lat[0]) if lat[0] else 0,
                "p50": float(lat[1]) if lat[1] else 0,
                "p95": float(lat[2]) if lat[2] else 0,
                "p99": float(lat[3]) if lat[3] else 0,
            },
            "intent_distribution": intent_dist,
            "feedback_stats": feedback_stats,
        }

    def close(self):
        self.cur.close()
        self.conn.close()


if __name__ == "__main__":
    print("Testing Hybrid Search Engine\n")

    engine = HybridSearchEngine()

    test_queries = [
        "Can I use a flash drive?",
        "How do I reset my password?",
        "What cloud storage options are available?",
    ]

    for query in test_queries:
        print(f"Query: {query}")
        result = engine.search(query, limit=3)

        print(
            f"Intent: {result['intent']} (confidence: {result['intent_confidence']:.2f})"
        )
        print(f"Results: {result['metrics']['total_results']}")
        print(f"Time: {result['metrics']['total_time_ms']:.1f}ms\n")

        for i, res in enumerate(result["results"], 1):
            print(f"  {i}. {res['title'][:70]}")
            print(
                f"     Category: {res['category']} | BM25={res['bm25_score']:.3f}, "
                f"Vector={res['vector_score']:.3f}, Fusion={res['fusion_score']:.3f}"
            )

        print()

    engine.close()
    print("Hybrid search engine working")
