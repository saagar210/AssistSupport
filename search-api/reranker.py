#!/usr/bin/env python3
"""
Cross-encoder re-ranker for search result refinement.
Uses cross-encoder/ms-marco-MiniLM-L-6-v2 to re-score top candidates.

Blends cross-encoder score with fusion score (70/30) to avoid noisy
content overriding good retrieval signals.
"""

import re
from sentence_transformers import CrossEncoder
from typing import List


class Reranker:
    """Re-rank search results using a cross-encoder model."""

    # Blend weights: fusion-dominant, cross-encoder as tiebreaker
    RERANK_WEIGHT = 0.15
    FUSION_WEIGHT = 0.85

    def __init__(self, model_name="cross-encoder/ms-marco-MiniLM-L-6-v2"):
        self.model = CrossEncoder(model_name)
        print(f"Reranker initialized: {model_name}")

    def _clean_passage(self, text: str) -> str:
        """Remove noisy content from passage text before scoring."""
        # Remove "Attachments:" sections and similar noise
        text = re.sub(r"Attachments?:.*$", "", text, flags=re.DOTALL)
        text = re.sub(r"Related [Aa]rticles?:.*$", "", text, flags=re.DOTALL)
        text = re.sub(r"\s+", " ", text).strip()
        return text[:512]  # Cross-encoder context window

    def rerank(
        self,
        query: str,
        candidates: List[dict],
        top_k: int = 10,
    ) -> List[dict]:
        """
        Re-rank candidates using blended cross-encoder + fusion scores.

        Uses normalized cross-encoder scores blended with original fusion
        scores to prevent noisy content from overriding good retrieval.
        """
        if not candidates or len(candidates) <= 1:
            return candidates[:top_k]

        # Build clean (query, passage) pairs
        pairs = []
        for c in candidates:
            title = c.get("title", "")
            preview = c.get("content_preview", "")
            passage = self._clean_passage(f"{title}. {preview}")
            pairs.append((query, passage))

        # Score all pairs
        raw_scores = self.model.predict(pairs)

        # Normalize cross-encoder scores to [0, 1]
        min_s = min(raw_scores)
        max_s = max(raw_scores)
        score_range = max_s - min_s if max_s != min_s else 1.0
        norm_scores = [(s - min_s) / score_range for s in raw_scores]

        # Normalize fusion scores to [0, 1]
        fusion_scores = [c.get("fusion_score", 0) for c in candidates]
        f_min = min(fusion_scores) if fusion_scores else 0
        f_max = max(fusion_scores) if fusion_scores else 1
        f_range = f_max - f_min if f_max != f_min else 1.0
        norm_fusion = [(s - f_min) / f_range for s in fusion_scores]

        # Blend scores
        for c, ce_norm, fu_norm, raw in zip(candidates, norm_scores, norm_fusion, raw_scores):
            blended = self.RERANK_WEIGHT * ce_norm + self.FUSION_WEIGHT * fu_norm
            c["rerank_score"] = float(raw)
            c["fusion_score"] = float(blended)  # Replace fusion with blended

        reranked = sorted(candidates, key=lambda x: x["fusion_score"], reverse=True)
        return reranked[:top_k]


if __name__ == "__main__":
    print("Testing Reranker\n")

    reranker = Reranker()

    query = "Can I use a flash drive?"
    candidates = [
        {"title": "Adobe Runbook", "content_preview": "How to provision Adobe licenses for new users.", "fusion_score": 0.3},
        {"title": "Flash Drive and USB Storage Policy", "content_preview": "USB flash drives and removable storage devices are forbidden on company networks.", "fusion_score": 0.8},
        {"title": "Cloud Storage Policy", "content_preview": "Approved cloud storage solutions include Box and Google Drive.", "fusion_score": 0.5},
    ]

    results = reranker.rerank(query, candidates)
    for i, r in enumerate(results, 1):
        print(f"  {i}. [{r['fusion_score']:.4f}] {r['title']} (rerank={r['rerank_score']:.4f})")

    print("\nReranker working")
