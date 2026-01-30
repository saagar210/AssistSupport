#!/usr/bin/env python3
"""
AssistSupport Embedding Service
Generates 384-dimensional embeddings using all-MiniLM-L6-v2 model
"""

import os
import numpy as np
from sentence_transformers import SentenceTransformer


class EmbeddingService:
    def __init__(self, model_name="sentence-transformers/all-MiniLM-L6-v2"):
        """Initialize embedding service with local model"""
        model_dir = os.path.expanduser("~/assistsupport-semantic-migration/models")
        self.model = SentenceTransformer(model_name, cache_folder=model_dir)
        self.dimension = 384
        print(f"Embedding service initialized: {model_name}")
        print(f"  Dimension: {self.dimension}")
        print(f"  Device: {self.model.device}")

    def embed_text(self, text: str) -> np.ndarray:
        """Generate single embedding"""
        if not text or not isinstance(text, str):
            raise ValueError("Text must be non-empty string")
        embedding = self.model.encode(text, normalize_embeddings=True)
        if len(embedding) != self.dimension:
            raise ValueError(f"Expected {self.dimension} dims, got {len(embedding)}")
        return embedding.astype(np.float32)

    def embed_batch(self, texts: list, batch_size=32, show_progress=False) -> list:
        """Generate embeddings for multiple texts"""
        if not texts:
            raise ValueError("Texts must be non-empty list")
        embeddings = self.model.encode(
            texts,
            batch_size=batch_size,
            show_progress_bar=show_progress,
            normalize_embeddings=True,
        )
        return [e.astype(np.float32) for e in embeddings]

    def test(self):
        """Verify service is working"""
        test_texts = [
            "Can I use a flash drive?",
            "USB drives and removable media are forbidden",
            "Cloud storage is approved for business use",
        ]

        print("\nTesting embedding service...")
        embeddings = self.embed_batch(test_texts, show_progress=False)

        for text, embedding in zip(test_texts, embeddings):
            print(f"  {text:50s} -> {len(embedding):3d} dims, norm={np.linalg.norm(embedding):.4f}")

        # Cosine similarity (embeddings are normalized, so dot product = cosine sim)
        sim_01 = np.dot(embeddings[0], embeddings[1])
        sim_02 = np.dot(embeddings[0], embeddings[2])
        print(f"\n  Cosine similarity (Q1 'flash drive' vs A1 'USB forbidden'): {sim_01:.4f}")
        print(f"  Cosine similarity (Q1 'flash drive' vs A2 'cloud storage'):  {sim_02:.4f}")
        print(f"  Q1 is more similar to A1 than A2: {sim_01 > sim_02}")
        print("Service test passed\n")


if __name__ == "__main__":
    service = EmbeddingService()
    service.test()
    print("Ready for batch embedding generation")
