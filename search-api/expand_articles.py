#!/usr/bin/env python3
"""
Expand thin articles by merging related chunks from the same source document.
For each source_document_id with multiple thin chunks (<500 chars),
merges them into a single richer article and deactivates the merged-from chunks.
"""

import psycopg2


def main():
    conn = psycopg2.connect(
        host="localhost",
        user="assistsupport_dev",
        password="dev_password_123",
        database="assistsupport_dev",
    )
    conn.autocommit = True
    cur = conn.cursor()

    print("=" * 70)
    print("  Step 6: Expand Thin Articles by Merging Related Chunks")
    print("=" * 70)
    print()

    # Phase 1: Find source documents with multiple thin chunks
    cur.execute("""
        SELECT source_document_id, COUNT(*) as chunk_count,
               SUM(LENGTH(content)) as total_chars,
               AVG(LENGTH(content)) as avg_chars,
               MIN(LENGTH(content)) as min_chars
        FROM kb_articles
        WHERE is_active = true
          AND source_document_id IS NOT NULL
          AND source_document_id != 'curated_kb'
          AND LENGTH(content) < 500
        GROUP BY source_document_id
        HAVING COUNT(*) >= 2
        ORDER BY COUNT(*) DESC
        LIMIT 50
    """)
    mergeable_docs = cur.fetchall()
    print(f"Found {len(mergeable_docs)} source documents with 2+ thin chunks\n")

    merged_count = 0
    deactivated_count = 0
    expanded_count = 0

    for doc_id, chunk_count, total_chars, avg_chars, min_chars in mergeable_docs:
        # Get all thin chunks for this source document, ordered by chunk_index
        cur.execute("""
            SELECT id, title, content, category, chunk_index, heading_path
            FROM kb_articles
            WHERE source_document_id = %s AND is_active = true AND LENGTH(content) < 500
            ORDER BY chunk_index ASC
        """, (doc_id,))
        chunks = cur.fetchall()

        if len(chunks) < 2:
            continue

        # Pick the best chunk as the primary (longest content)
        best_idx = 0
        best_len = 0
        for i, chunk in enumerate(chunks):
            content_len = len(chunk[2])
            if content_len > best_len:
                best_len = content_len
                best_idx = i

        primary = chunks[best_idx]
        primary_id = primary[0]
        primary_title = primary[1]
        primary_content = primary[2]
        primary_category = primary[3]

        # Merge content from other chunks
        merged_sections = [primary_content]
        merged_from_ids = []

        for i, chunk in enumerate(chunks):
            if i == best_idx:
                continue
            chunk_id = chunk[0]
            chunk_heading = chunk[4] or chunk[1]  # Use heading_path or title
            chunk_content = chunk[2]

            # Add section with heading
            section = f"\n\n## {chunk_heading}\n\n{chunk_content}"
            merged_sections.append(section)
            merged_from_ids.append(chunk_id)

        merged_content = "\n".join(merged_sections).strip()

        # Only merge if it meaningfully increases content
        if len(merged_content) <= len(primary_content) + 50:
            continue

        # Update primary article with merged content
        cur.execute(
            "UPDATE kb_articles SET content = %s WHERE id = %s",
            (merged_content, primary_id),
        )

        # Deactivate merged-from chunks
        for chunk_id in merged_from_ids:
            cur.execute(
                "UPDATE kb_articles SET is_active = false WHERE id = %s",
                (chunk_id,),
            )
            deactivated_count += 1

        merged_count += 1
        expanded_count += len(merged_from_ids)

        if merged_count <= 10:
            print(f"  Merged {len(merged_from_ids)+1} chunks → [{primary_category}] {primary_title[:60]}")
            print(f"    Before: {len(primary_content)} chars → After: {len(merged_content)} chars")

    print(f"\n  Total documents merged: {merged_count}")
    print(f"  Chunks merged into primaries: {expanded_count}")
    print(f"  Chunks deactivated: {deactivated_count}")

    # Phase 2: Also expand articles that are isolated thin chunks on popular topics
    # by appending helpful context based on category
    print(f"\n{'=' * 70}")
    print("  Phase 2: Enrich isolated thin articles with contextual info")
    print("=" * 70)
    print()

    # Popular topic keywords from query logs
    popular_topics = [
        "password", "vpn", "laptop", "email", "okta", "onboarding",
        "slack", "printer", "monitor", "software", "security", "backup",
        "access", "account", "network", "zoom", "jira", "firewall",
        "wifi", "google",
    ]

    topic_pattern = "|".join(popular_topics)

    cur.execute(f"""
        SELECT id, title, content, category, LENGTH(content) as len
        FROM kb_articles
        WHERE is_active = true
          AND source_document_id != 'curated_kb'
          AND LENGTH(content) BETWEEN 100 AND 300
          AND (
            {" OR ".join(["LOWER(title) LIKE %s" for _ in popular_topics])}
          )
        ORDER BY LENGTH(content) ASC
        LIMIT 40
    """, [f"%{t}%" for t in popular_topics])
    thin_popular = cur.fetchall()
    print(f"Found {len(thin_popular)} thin articles on popular topics (100-300 chars)\n")

    # Category-specific context additions
    category_context = {
        "POLICY": "\n\n---\n*For policy questions, contact your IT department or check the company intranet for the full policy document. Policy violations should be reported to IT Security.*",
        "PROCEDURE": "\n\n---\n*If you encounter issues following this procedure, please submit a support ticket or contact the IT Help Desk for assistance. Include any error messages in your request.*",
        "REFERENCE": "\n\n---\n*This is a reference document. For step-by-step instructions, search for the related procedure guide. Contact IT Support if you need further clarification.*",
    }

    enriched = 0
    for article_id, title, content, category, length in thin_popular:
        context = category_context.get(category, "")
        if context and context not in content:
            new_content = content + context
            cur.execute(
                "UPDATE kb_articles SET content = %s WHERE id = %s",
                (new_content, article_id),
            )
            enriched += 1

    print(f"  Enriched {enriched} thin articles with category context\n")

    # Final stats
    print("=" * 70)
    print("  FINAL STATS")
    print("=" * 70)
    cur.execute("""
        SELECT
            COUNT(*) as total,
            COUNT(CASE WHEN is_active THEN 1 END) as active,
            ROUND(AVG(CASE WHEN is_active THEN LENGTH(content) END)) as avg_len,
            COUNT(CASE WHEN is_active AND LENGTH(content) < 100 THEN 1 END) as under_100,
            COUNT(CASE WHEN is_active AND LENGTH(content) < 500 THEN 1 END) as under_500,
            COUNT(CASE WHEN is_active AND LENGTH(content) >= 500 THEN 1 END) as over_500
        FROM kb_articles
    """)
    row = cur.fetchone()
    print(f"  Total articles: {row[0]}")
    print(f"  Active articles: {row[1]}")
    print(f"  Avg content length: {row[2]} chars (was 753)")
    print(f"  Under 100 chars: {row[3]}")
    print(f"  Under 500 chars: {row[4]}")
    print(f"  Over 500 chars: {row[5]}")

    cur.close()
    conn.close()
    print("\nDone.")


if __name__ == "__main__":
    main()
