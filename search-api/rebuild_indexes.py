#!/usr/bin/env python3
"""
Rebuild FTS tsvectors and embeddings for modified articles.
Regenerates embeddings for articles whose content changed (merged/expanded).
"""

import sys
import os
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from db_config import connect_db
from embedding_service import EmbeddingService


def main():
    conn = connect_db()
    conn.autocommit = True
    cur = conn.cursor()

    embedder = EmbeddingService()

    print("=" * 70)
    print("  Step 7: Rebuild FTS & Embeddings for Modified Articles")
    print("=" * 70)
    print()

    # Find articles that need re-embedding
    # These are active articles whose content was modified (merged/expanded)
    # We'll re-embed ALL active articles to be safe (takes ~2-3 min for 2,597)
    cur.execute("""
        SELECT id, title, content
        FROM kb_articles
        WHERE is_active = true
        ORDER BY id
    """)
    articles = cur.fetchall()
    total = len(articles)
    print(f"Re-generating embeddings for {total} active articles...\n")

    # Process in batches
    batch_size = 64
    updated = 0
    start = time.time()

    for i in range(0, total, batch_size):
        batch = articles[i:i + batch_size]
        ids = [a[0] for a in batch]
        texts = [f"{a[1]}. {a[2]}" for a in batch]

        embeddings = embedder.embed_batch(texts)

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
    print("\nRebuilding HNSW index...")
    start_idx = time.time()
    cur.execute("REINDEX INDEX kb_articles_embedding_hnsw_idx")
    idx_time = time.time() - start_idx
    print(f"  HNSW index rebuilt in {idx_time:.1f}s")

    # Verify index health
    cur.execute("""
        SELECT indexrelname, idx_scan, idx_tup_read, idx_tup_fetch,
               pg_size_pretty(pg_relation_size(indexrelid)) as size
        FROM pg_stat_user_indexes
        WHERE relname = 'kb_articles'
        ORDER BY indexrelname
    """)
    print("\nIndex health:")
    for row in cur.fetchall():
        print(f"  {row[0]}: scans={row[1]}, size={row[4]}")

    # Verify FTS is working (generated column auto-updates)
    cur.execute("""
        SELECT COUNT(*) FROM kb_articles
        WHERE is_active = true AND fts_content IS NOT NULL
    """)
    fts_count = cur.fetchone()[0]
    print(f"\nFTS coverage: {fts_count}/{total} articles have tsvectors")

    print(f"\n{'=' * 70}")
    print("  INDEX REBUILD COMPLETE")
    print(f"{'=' * 70}")

    cur.close()
    conn.close()


if __name__ == "__main__":
    main()
