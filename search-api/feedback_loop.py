#!/usr/bin/env python3
"""
Feedback Loop — computes per-article quality scores from user feedback.
Updates kb_articles.quality_score based on aggregated ratings.

Score formula:
  quality_score = 1.0 + (helpful_ratio - 0.5) * weight
  where:
    helpful_ratio = helpful_count / total_count
    weight = min(0.3, total_count * 0.02)  # caps at 0.3 boost/penalty

Minimum 3 feedback entries required before adjusting an article's score.
"""

import psycopg2


MIN_FEEDBACK = 3
MAX_WEIGHT = 0.3
WEIGHT_PER_FEEDBACK = 0.02

RATING_VALUES = {
    "helpful": 1.0,
    "not_helpful": 0.0,
    "incorrect": -0.5,
}


def compute_quality_scores(conn):
    """Compute and update quality_score for all articles with sufficient feedback."""
    cur = conn.cursor()

    # Aggregate feedback per article
    cur.execute("""
        SELECT sf.article_id, sf.rating, COUNT(*) as cnt
        FROM search_feedback sf
        WHERE sf.article_id IS NOT NULL
        GROUP BY sf.article_id, sf.rating
    """)

    article_feedback = {}
    for article_id, rating, count in cur.fetchall():
        aid = str(article_id)
        if aid not in article_feedback:
            article_feedback[aid] = {"helpful": 0, "not_helpful": 0, "incorrect": 0, "total": 0}
        article_feedback[aid][rating] = count
        article_feedback[aid]["total"] += count

    updated = 0
    for article_id, stats in article_feedback.items():
        total = stats["total"]
        if total < MIN_FEEDBACK:
            continue

        # Weighted score: helpful=1.0, not_helpful=0.0, incorrect=-0.5
        score_sum = (
            stats["helpful"] * RATING_VALUES["helpful"]
            + stats["not_helpful"] * RATING_VALUES["not_helpful"]
            + stats["incorrect"] * RATING_VALUES["incorrect"]
        )
        helpful_ratio = max(0.0, score_sum / total)

        # Weight increases with more feedback, capped at MAX_WEIGHT
        weight = min(MAX_WEIGHT, total * WEIGHT_PER_FEEDBACK)

        quality_score = 1.0 + (helpful_ratio - 0.5) * weight
        quality_score = max(0.5, min(1.5, quality_score))  # Clamp to [0.5, 1.5]

        cur.execute(
            "UPDATE kb_articles SET quality_score = %s WHERE id = %s",
            (quality_score, article_id),
        )
        updated += 1

    conn.commit()
    cur.close()
    return updated


def get_quality_scores(conn, article_ids):
    """Fetch quality scores for a set of article IDs."""
    if not article_ids:
        return {}
    cur = conn.cursor()
    placeholders = ",".join(["%s"] * len(article_ids))
    cur.execute(
        f"SELECT id, quality_score FROM kb_articles WHERE id IN ({placeholders})",
        article_ids,
    )
    scores = {str(row[0]): float(row[1] or 1.0) for row in cur.fetchall()}
    cur.close()
    return scores


if __name__ == "__main__":
    conn = psycopg2.connect(
        host="localhost",
        user="assistsupport_dev",
        password="dev_password_123",
        database="assistsupport_dev",
    )
    conn.autocommit = True

    print("Computing quality scores from feedback...")
    updated = compute_quality_scores(conn)
    print(f"Updated {updated} article quality scores")

    # Show stats
    cur = conn.cursor()
    cur.execute("""
        SELECT COUNT(*) as total_feedback,
               COUNT(DISTINCT article_id) as articles_with_feedback,
               COUNT(CASE WHEN rating = 'helpful' THEN 1 END) as helpful,
               COUNT(CASE WHEN rating = 'not_helpful' THEN 1 END) as not_helpful,
               COUNT(CASE WHEN rating = 'incorrect' THEN 1 END) as incorrect
        FROM search_feedback
    """)
    row = cur.fetchone()
    print(f"\nFeedback stats:")
    print(f"  Total entries: {row[0]}")
    print(f"  Articles with feedback: {row[1]}")
    print(f"  Helpful: {row[2]}")
    print(f"  Not helpful: {row[3]}")
    print(f"  Incorrect: {row[4]}")
    print(f"\nMinimum {MIN_FEEDBACK} entries needed per article before score adjustment")

    cur.close()
    conn.close()
