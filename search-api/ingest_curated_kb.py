#!/usr/bin/env python3
"""
Ingest curated knowledge_base/ markdown files into PostgreSQL.
Generates embeddings and inserts with proper category, title, and FTS.
"""

import os
import sys
import uuid
import re
import psycopg2
import numpy as np

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from embedding_service import EmbeddingService

KB_DIR = os.path.expanduser("~/AssistSupport/knowledge_base")

# Map directory names to categories
CATEGORY_MAP = {
    "POLICIES": "POLICY",
    "PROCEDURES": "PROCEDURE",
    "REFERENCE": "REFERENCE",
}

SKIP_FILES = {"INDEX.md"}


def extract_title(content: str, filename: str) -> str:
    """Extract title from first markdown heading or derive from filename."""
    for line in content.split("\n"):
        line = line.strip()
        match = re.match(r"^#+\s+(.+)$", line)
        if match:
            return match.group(1).strip()
    # Fallback: derive from filename
    name = os.path.splitext(filename)[0]
    return name.replace("_", " ").title()


def clean_content(content: str) -> str:
    """Remove frontmatter and clean content for indexing."""
    # Remove YAML frontmatter if present
    if content.startswith("---"):
        end = content.find("---", 3)
        if end != -1:
            content = content[end + 3:].strip()
    return content.strip()


def main():
    print("=" * 70)
    print("  Curated KB Ingestion → PostgreSQL")
    print("=" * 70)
    print()

    # Connect to database
    conn = psycopg2.connect(
        host="localhost",
        user="assistsupport_dev",
        password="dev_password_123",
        database="assistsupport_dev",
    )
    conn.autocommit = True
    cur = conn.cursor()

    # Initialize embedding service
    embedder = EmbeddingService()

    # Collect all files to ingest
    files_to_ingest = []
    for dir_name, category in CATEGORY_MAP.items():
        dir_path = os.path.join(KB_DIR, dir_name)
        if not os.path.isdir(dir_path):
            print(f"  WARNING: Directory not found: {dir_path}")
            continue
        for filename in sorted(os.listdir(dir_path)):
            if filename in SKIP_FILES or not filename.endswith(".md"):
                continue
            filepath = os.path.join(dir_path, filename)
            files_to_ingest.append((filepath, filename, category))

    print(f"Found {len(files_to_ingest)} files to ingest\n")

    # Check for existing curated articles and remove them (clean re-ingest)
    cur.execute(
        "DELETE FROM kb_articles WHERE source_document_id = 'curated_kb'"
    )
    deleted = cur.rowcount
    if deleted > 0:
        print(f"Removed {deleted} existing curated KB articles (clean re-ingest)\n")

    # Read all files
    articles = []
    for filepath, filename, category in files_to_ingest:
        with open(filepath, "r", encoding="utf-8") as f:
            raw_content = f.read()

        content = clean_content(raw_content)
        title = extract_title(content, filename)

        articles.append({
            "filepath": filepath,
            "filename": filename,
            "category": category,
            "title": title,
            "content": content,
        })
        print(f"  Read: [{category}] {title} ({len(content)} chars)")

    print(f"\nGenerating embeddings for {len(articles)} articles...")

    # Generate embeddings in batch
    texts_for_embedding = [f"{a['title']}. {a['content']}" for a in articles]
    embeddings = embedder.embed_batch(texts_for_embedding, show_progress=True)

    print(f"Generated {len(embeddings)} embeddings ({embedder.dimension} dims each)\n")

    # Insert into PostgreSQL
    inserted = 0
    for article, embedding in zip(articles, embeddings):
        article_id = str(uuid.uuid4())
        embedding_str = "[" + ",".join(f"{x:.6f}" for x in embedding) + "]"

        cur.execute(
            """
            INSERT INTO kb_articles
                (id, title, content, category, article_type, embedding,
                 embedding_model, embedding_version,
                 is_active, source_document_id, chunk_index, heading_path)
            VALUES
                (%s, %s, %s, %s, %s, %s::vector, %s, %s,
                 true, %s, %s, %s)
            """,
            (
                article_id,
                article["title"],
                article["content"],
                article["category"],
                article["category"].lower(),
                embedding_str,
                "all-MiniLM-L6-v2",
                1,
                "curated_kb",
                0,
                article["filename"],
            ),
        )
        inserted += 1
        print(f"  Inserted: [{article['category']}] {article['title']}")

    print(f"\n{'=' * 70}")
    print(f"  INGESTION COMPLETE: {inserted} articles inserted")
    print(f"{'=' * 70}")

    # Verify
    cur.execute(
        """
        SELECT category, COUNT(*), ROUND(AVG(LENGTH(content)))
        FROM kb_articles
        WHERE source_document_id = 'curated_kb'
        GROUP BY category
        ORDER BY category
        """
    )
    print("\nVerification:")
    for row in cur.fetchall():
        print(f"  {row[0]}: {row[1]} articles, avg {row[2]} chars")

    # Quick test: can we find the flash drive policy now?
    cur.execute(
        """
        SELECT id, title, category, LENGTH(content)
        FROM kb_articles
        WHERE source_document_id = 'curated_kb'
          AND LOWER(title) LIKE '%flash%'
        """
    )
    row = cur.fetchone()
    if row:
        print(f"\n  Flash drive policy found: [{row[2]}] {row[1]} ({row[3]} chars)")
    else:
        print("\n  WARNING: Flash drive policy NOT found after ingestion!")

    cur.close()
    conn.close()
    print("\nDone.")


if __name__ == "__main__":
    main()
