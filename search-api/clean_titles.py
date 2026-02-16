#!/usr/bin/env python3
"""
Clean breadcrumb titles in kb_articles.
Extracts meaningful title from breadcrumb paths like:
  "OKTA - Runbook > OKTA - Runbook > Access Policy" → "OKTA - Runbook: Access Policy"
Preserves original title in heading_path if not already set.
"""

import re
from db_config import connect_db


def clean_title(title: str, heading_path: str) -> tuple:
    """
    Clean a breadcrumb title into a concise, searchable title.
    Returns (new_title, new_heading_path).
    """
    if " > " not in title:
        return title, heading_path

    segments = [s.strip() for s in title.split(" > ")]
    segments = [s for s in segments if s]  # remove empties

    if not segments:
        return title, heading_path

    # Preserve original title in heading_path if heading_path is missing or same as title
    if not heading_path or heading_path == title:
        new_heading_path = title
    else:
        new_heading_path = heading_path

    # Remove duplicate adjacent segments (e.g., "OKTA - Runbook > OKTA - Runbook > ...")
    deduped = [segments[0]]
    for seg in segments[1:]:
        if seg.lower() != deduped[-1].lower():
            deduped.append(seg)
    segments = deduped

    # Clean markdown artifacts from segments
    cleaned = []
    for seg in segments:
        seg = re.sub(r"\*\*", "", seg)  # Remove bold markers
        seg = re.sub(r"^\*+\s*", "", seg)  # Remove leading asterisks
        seg = seg.strip()
        if seg:
            cleaned.append(seg)
    segments = cleaned

    if len(segments) == 1:
        new_title = segments[0]
    elif len(segments) == 2:
        # "Document > Section" → "Document: Section"
        new_title = f"{segments[0]}: {segments[1]}"
    else:
        # "Doc > Section > Subsection" → "Doc: Section — Subsection"
        # Take first as doc name, last meaningful segment as detail
        doc_name = segments[0]
        # Find the most meaningful non-doc segment (prefer shorter, specific ones)
        detail_segments = segments[1:]
        # Pick the segment that looks most like a section heading (shorter, not a sentence)
        best_detail = None
        for seg in detail_segments:
            if len(seg) < 80 and not seg.endswith("."):
                best_detail = seg
                break
        if best_detail is None:
            best_detail = detail_segments[0]

        if len(detail_segments) > 1 and detail_segments[-1] != best_detail and len(detail_segments[-1]) < 60:
            new_title = f"{doc_name}: {best_detail} — {detail_segments[-1]}"
        else:
            new_title = f"{doc_name}: {best_detail}"

    # Truncate to 150 chars max
    if len(new_title) > 150:
        new_title = new_title[:147] + "..."

    return new_title, new_heading_path


def main():
    conn = connect_db()
    conn.autocommit = True
    cur = conn.cursor()

    # Fetch all active articles with breadcrumb titles
    cur.execute("""
        SELECT id, title, heading_path
        FROM kb_articles
        WHERE is_active = true AND title LIKE '%>%'
        AND source_document_id != 'curated_kb'
    """)
    rows = cur.fetchall()
    print(f"Found {len(rows)} articles with breadcrumb titles\n")

    updated = 0
    samples = []

    for article_id, old_title, heading_path in rows:
        new_title, new_heading_path = clean_title(old_title, heading_path)

        if new_title != old_title:
            cur.execute(
                "UPDATE kb_articles SET title = %s, heading_path = %s WHERE id = %s",
                (new_title, new_heading_path, article_id),
            )
            updated += 1

            if len(samples) < 15:
                samples.append((old_title[:80], new_title[:80]))

    print(f"Updated {updated} titles\n")
    print("Sample transformations:")
    print("-" * 85)
    for old, new in samples:
        print(f"  OLD: {old}")
        print(f"  NEW: {new}")
        print()

    # Stats after cleanup
    cur.execute("""
        SELECT
            COUNT(*) as total,
            ROUND(AVG(LENGTH(title)), 1) as avg_len,
            COUNT(CASE WHEN LENGTH(title) > 100 THEN 1 END) as over_100,
            COUNT(CASE WHEN LENGTH(title) > 150 THEN 1 END) as over_150
        FROM kb_articles WHERE is_active = true
    """)
    row = cur.fetchone()
    print(f"After cleanup:")
    print(f"  Total active: {row[0]}")
    print(f"  Avg title length: {row[1]} chars (was 76.4)")
    print(f"  Titles > 100 chars: {row[2]} (was 770)")
    print(f"  Titles > 150 chars: {row[3]} (was 188)")

    cur.close()
    conn.close()


if __name__ == "__main__":
    main()
