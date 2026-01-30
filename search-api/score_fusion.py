#!/usr/bin/env python3
"""
Score Fusion Module for Hybrid Search
Combines BM25 (keyword) and HNSW (vector) scores into unified ranking
Implements: Reciprocal Rank Fusion (RRF) + weighted combination
"""

import math
from typing import List, Dict, Tuple


class ScoreFusion:
    """Hybrid search score fusion strategies"""

    @staticmethod
    def reciprocal_rank_fusion(
        bm25_results: List[Tuple[str, float]],
        vector_results: List[Tuple[str, float]],
        k: int = 60,
    ) -> List[Tuple[str, float]]:
        """
        Reciprocal Rank Fusion (RRF)
        Score(d) = sum 1/(k + rank(d))

        Benefits:
        - Balanced weight to both ranking methods
        - Robust to outliers
        - No parameter tuning needed

        Reference: https://arxiv.org/abs/1809.01852
        """
        scores = {}

        for rank, (article_id, _) in enumerate(bm25_results, 1):
            rrf_score = 1.0 / (k + rank)
            scores[article_id] = scores.get(article_id, 0) + rrf_score

        for rank, (article_id, _) in enumerate(vector_results, 1):
            rrf_score = 1.0 / (k + rank)
            scores[article_id] = scores.get(article_id, 0) + rrf_score

        ranked = sorted(scores.items(), key=lambda x: x[1], reverse=True)
        return ranked

    @staticmethod
    def weighted_combination(
        bm25_results: List[Tuple[str, float]],
        vector_results: List[Tuple[str, float]],
        bm25_weight: float = 0.3,
        vector_weight: float = 0.6,
    ) -> List[Tuple[str, float]]:
        """
        Weighted Combination of BM25 and vector scores.
        Score(d) = bm25_weight * norm(bm25) + vector_weight * norm(vector)

        All scores normalized to [0, 1]
        """
        if not bm25_results:
            bm25_max = 1.0
            bm25_dict = {}
        else:
            bm25_max = max(s for _, s in bm25_results) if bm25_results else 1.0
            bm25_dict = {
                article_id: min(1.0, score / max(bm25_max, 0.01))
                for article_id, score in bm25_results
            }

        if not vector_results:
            vector_dict = {}
        else:
            vector_dict = {
                article_id: min(1.0, max(0.0, score))
                for article_id, score in vector_results
            }

        all_ids = set(bm25_dict.keys()) | set(vector_dict.keys())
        combined = {}

        for article_id in all_ids:
            bm25_norm = bm25_dict.get(article_id, 0.0)
            vector_norm = vector_dict.get(article_id, 0.0)

            score = (bm25_weight * bm25_norm) + (vector_weight * vector_norm)
            combined[article_id] = score

        ranked = sorted(combined.items(), key=lambda x: x[1], reverse=True)
        return ranked

    @staticmethod
    def adaptive_fusion(
        bm25_results: List[Tuple[str, float]],
        vector_results: List[Tuple[str, float]],
        query_type: str,
    ) -> List[Tuple[str, float]]:
        """
        Adaptive Fusion Based on Query Intent

        Adjust weights based on detected query type:
        - Policy: favor vector (semantic understanding of "am I allowed" queries)
        - Procedure: balance both (steps need semantic + keyword matching)
        - Reference: boost vector (semantic understanding critical)
        - Unknown: balanced with slight vector preference
        """
        weights = {
            "policy": {"bm25": 0.35, "vector": 0.65},
            "procedure": {"bm25": 0.40, "vector": 0.60},
            "reference": {"bm25": 0.20, "vector": 0.80},
            "unknown": {"bm25": 0.30, "vector": 0.70},
        }

        w = weights.get(query_type, weights["unknown"])

        return ScoreFusion.weighted_combination(
            bm25_results,
            vector_results,
            bm25_weight=w["bm25"],
            vector_weight=w["vector"],
        )


if __name__ == "__main__":
    print("Testing Score Fusion Algorithms\n")

    bm25_results = [
        ("article_1", 2.5),
        ("article_2", 2.1),
        ("article_3", 1.8),
    ]

    vector_results = [
        ("article_2", 0.85),
        ("article_1", 0.82),
        ("article_4", 0.79),
    ]

    print("Reciprocal Rank Fusion:")
    rrf_ranked = ScoreFusion.reciprocal_rank_fusion(bm25_results, vector_results)
    for i, (article_id, score) in enumerate(rrf_ranked[:5], 1):
        print(f"  {i}. {article_id}: {score:.4f}")

    print("\nWeighted Combination:")
    weighted_ranked = ScoreFusion.weighted_combination(bm25_results, vector_results)
    for i, (article_id, score) in enumerate(weighted_ranked[:5], 1):
        print(f"  {i}. {article_id}: {score:.4f}")

    print("\nAdaptive (Policy Intent):")
    adaptive_ranked = ScoreFusion.adaptive_fusion(
        bm25_results, vector_results, "policy"
    )
    for i, (article_id, score) in enumerate(adaptive_ranked[:5], 1):
        print(f"  {i}. {article_id}: {score:.4f}")

    print("\nAll fusion algorithms working")
