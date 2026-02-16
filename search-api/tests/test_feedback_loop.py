import sys
from pathlib import Path

import pytest

SEARCH_API_DIR = Path(__file__).resolve().parents[1]
if str(SEARCH_API_DIR) not in sys.path:
    sys.path.insert(0, str(SEARCH_API_DIR))

from feedback_loop import compute_quality_scores, get_quality_scores  # noqa: E402


class FakeCursor:
    def __init__(self, fetchall_batches):
        self.fetchall_batches = list(fetchall_batches)
        self.executed = []
        self.closed = False

    def execute(self, sql, params=None):
        self.executed.append((" ".join(sql.split()), params))

    def fetchall(self):
        if not self.fetchall_batches:
            return []
        return self.fetchall_batches.pop(0)

    def close(self):
        self.closed = True


class FakeConnection:
    def __init__(self, cursor):
        self._cursor = cursor
        self.committed = False

    def cursor(self):
        return self._cursor

    def commit(self):
        self.committed = True


def test_compute_quality_scores_updates_only_articles_with_enough_feedback():
    # article-1 has 4 ratings (eligible), article-2 has 2 ratings (ignored)
    cursor = FakeCursor(
        [
            [
                ("article-1", "helpful", 3),
                ("article-1", "incorrect", 1),
                ("article-2", "helpful", 2),
            ]
        ]
    )
    conn = FakeConnection(cursor)

    updated = compute_quality_scores(conn)

    assert updated == 1
    assert conn.committed is True
    updates = [entry for entry in cursor.executed if entry[0].startswith("UPDATE kb_articles")]
    assert len(updates) == 1
    score, article_id = updates[0][1]
    assert article_id == "article-1"
    assert score == pytest.approx(1.01, abs=1e-6)
    assert cursor.closed is True


def test_get_quality_scores_defaults_null_values_to_one():
    cursor = FakeCursor(
        [
            [
                ("article-1", 1.3),
                ("article-2", None),
            ]
        ]
    )
    conn = FakeConnection(cursor)

    scores = get_quality_scores(conn, ["article-1", "article-2"])
    assert scores["article-1"] == pytest.approx(1.3)
    assert scores["article-2"] == pytest.approx(1.0)
    assert cursor.closed is True


def test_get_quality_scores_returns_empty_for_empty_input():
    cursor = FakeCursor([])
    conn = FakeConnection(cursor)
    assert get_quality_scores(conn, []) == {}
