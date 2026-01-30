#!/usr/bin/env python3
"""
AssistSupport Embedding Service
Generates 768-dimensional embeddings using intfloat/e5-base-v2 (IR-optimized)
Fallback: sentence-transformers/all-MiniLM-L6-v2 (384 dims)
"""

import os
import numpy as np
from sentence_transformers import SentenceTransformer


class EmbeddingService:
    # Models that require "query: "/"passage: " prefixes
    PREFIX_MODELS = {"intfloat/e5-base-v2", "intfloat/e5-small-v2", "intfloat/e5-large-v2"}

    def __init__(self, model_name="sentence-transformers/all-MiniLM-L6-v2"):
        """Initialize embedding service with local model"""
        model_dir = os.path.expanduser("~/assistsupport-semantic-migration/models")
        self.model = SentenceTransformer(model_name, cache_folder=model_dir)
        self.model_name = model_name
        self.dimension = self.model.get_sentence_embedding_dimension()
        self.uses_prefix = model_name in self.PREFIX_MODELS
        print(f"Embedding service initialized: {model_name}")
        print(f"  Dimension: {self.dimension}")
        print(f"  Device: {self.model.device}")
        print(f"  Uses prefix: {self.uses_prefix}")

    def embed_query(self, text: str) -> np.ndarray:
        """Generate embedding for a search query"""
        if not text or not isinstance(text, str):
            raise ValueError("Text must be non-empty string")
        prefixed = f"query: {text}" if self.uses_prefix else text
        embedding = self.model.encode(prefixed, normalize_embeddings=True)
        return embedding.astype(np.float32)

    def embed_text(self, text: str) -> np.ndarray:
        """Generate embedding for a document/passage"""
        if not text or not isinstance(text, str):
            raise ValueError("Text must be non-empty string")
        prefixed = f"passage: {text}" if self.uses_prefix else text
        embedding = self.model.encode(prefixed, normalize_embeddings=True)
        return embedding.astype(np.float32)

    def embed_batch(self, texts: list, batch_size=32, show_progress=False, is_query=False) -> list:
        """Generate embeddings for multiple texts"""
        if not texts:
            raise ValueError("Texts must be non-empty list")
        if self.uses_prefix:
            prefix = "query: " if is_query else "passage: "
            texts = [f"{prefix}{t}" for t in texts]
        embeddings = self.model.encode(
            texts,
            batch_size=batch_size,
            show_progress_bar=show_progress,
            normalize_embeddings=True,
        )
        return [e.astype(np.float32) for e in embeddings]

    def test(self):
        """Verify service is working"""
        queries = ["Can I use a flash drive?"]
        passages = [
            "USB drives and removable media are forbidden",
            "Cloud storage is approved for business use",
        ]

        print("\nTesting embedding service...")
        q_emb = self.embed_query(queries[0])
        p_embs = self.embed_batch(passages, is_query=False)

        print(f"  Query:   {queries[0]:50s} -> {len(q_emb):3d} dims")
        for text, embedding in zip(passages, p_embs):
            print(f"  Passage: {text:50s} -> {len(embedding):3d} dims")

        sim_01 = np.dot(q_emb, p_embs[0])
        sim_02 = np.dot(q_emb, p_embs[1])
        print(f"\n  Cosine similarity (Q 'flash drive' vs P 'USB forbidden'): {sim_01:.4f}")
        print(f"  Cosine similarity (Q 'flash drive' vs P 'cloud storage'):  {sim_02:.4f}")
        print(f"  Q is more similar to P1 than P2: {sim_01 > sim_02}")
        print("Service test passed\n")


if __name__ == "__main__":
    service = EmbeddingService()
    service.test()
    print("Ready for batch embedding generation")
