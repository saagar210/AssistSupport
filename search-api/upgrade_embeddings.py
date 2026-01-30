#!/usr/bin/env python3
"""
Step 8: Regenerate all embeddings using e5-base-v2 (768 dims).
Replaces all-MiniLM-L6-v2 (384 dims) embeddings.
"""

import sys
import os
import psycopg2
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from embedding_service import EmbeddingService


def main():
    conn = psycopg2.connect(
        host="localhost",
        user="assistsupport_dev",
        password="dev_password_123",
        database="assistsupport_dev",
    )
    conn.autocommit = True
    cur = conn.cursor()

    embedder = EmbeddingService()  # Now defaults to e5-base-v2

    print("=" * 70)
    print("  Step 8: Regenerate Embeddings with e5-base-v2 (768 dims)")
    print("=" * 70)
    print()

    cur.execute("""
        SELECT id, title, content
        FROM kb_articles
        WHERE is_active = true
        ORDER BY id
    """)
    articles = cur.fetchall()
    total = len(articles)
    print(f"Generating 768-dim embeddings for {total} active articles...\n")

    batch_size = 32
    updated = 0
    start = time.time()

    for i in range(0, total, batch_size):
        batch = articles[i:i + batch_size]
        ids = [a[0] for a in batch]
        texts = [f"{a[1]}. {a[2]}" for a in batch]

        # Use passage embedding (not query) for document indexing
        embeddings = embedder.embed_batch(texts, is_query=False)

        for article_id, embedding in zip(ids, embeddings):
            embedding_str = "[" + ",".join(f"{x:.6f}" for x in embedding) + "]"
            cur.execute(
                "UPDATE kb_articles SET embedding = %s::vector WHERE id = %s",
                (embedding_str, article_id),
            )
            updated += 1

        elapsed = time.time() - start
        pct = (i + len(batch)) / total * 100
        print(f"  Progress: {i + len(batch)}/{total} ({pct:.0f}%) — {elapsed:.1f}s elapsed")

    elapsed = time.time() - start
    print(f"\n  Embeddings regenerated: {updated} articles in {elapsed:.1f}s")

    # Rebuild HNSW index
    print("\nCreating HNSW index (768 dims)...")
    start_idx = time.time()
    cur.execute("""
        CREATE INDEX idx_kb_articles_embedding_hnsw
        ON kb_articles
        USING hnsw (embedding vector_cosine_ops)
        WITH (m = 16, ef_construction = 64)
    """)
    idx_time = time.time() - start_idx
    print(f"  HNSW index created in {idx_time:.1f}s")

    # Verify
    cur.execute("""
        SELECT indexrelname, pg_size_pretty(pg_relation_size(indexrelid)) as size
        FROM pg_stat_user_indexes
        WHERE relname = 'kb_articles' AND indexrelname LIKE '%hnsw%'
    """)
    for row in cur.fetchall():
        print(f"  {row[0]}: {row[1]}")

    cur.execute("SELECT COUNT(*) FROM kb_articles WHERE is_active = true AND embedding IS NOT NULL")
    emb_count = cur.fetchone()[0]
    print(f"\n  Embedding coverage: {emb_count}/{total} articles")

    print(f"\n{'=' * 70}")
    print("  EMBEDDING UPGRADE COMPLETE (384 → 768 dims)")
    print(f"{'=' * 70}")

    cur.close()
    conn.close()


if __name__ == "__main__":
    main()
