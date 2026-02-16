import string
import sys
from pathlib import Path

from hypothesis import given, strategies as st

SEARCH_API_DIR = Path(__file__).resolve().parents[1]
if str(SEARCH_API_DIR) not in sys.path:
    sys.path.insert(0, str(SEARCH_API_DIR))

from score_fusion import ScoreFusion  # noqa: E402


def _result_list_strategy():
    article_ids = st.text(alphabet=string.ascii_lowercase + string.digits, min_size=1, max_size=8)
    scores = st.floats(min_value=-5, max_value=200, allow_nan=False, allow_infinity=False)
    return st.lists(
        st.tuples(article_ids, scores),
        min_size=0,
        max_size=15,
        unique_by=lambda item: item[0],
    )


@given(bm25=_result_list_strategy(), vector=_result_list_strategy())
def test_rrf_contains_union_without_duplicates(bm25, vector):
    fused = ScoreFusion.reciprocal_rank_fusion(bm25, vector)
    ids = [article_id for article_id, _ in fused]

    assert len(ids) == len(set(ids))
    assert set(ids) == {article_id for article_id, _ in bm25} | {
        article_id for article_id, _ in vector
    }


@given(bm25=_result_list_strategy(), vector=_result_list_strategy())
def test_weighted_combination_scores_are_sorted_and_bounded(bm25, vector):
    fused = ScoreFusion.weighted_combination(bm25, vector)

    assert fused == sorted(fused, key=lambda pair: pair[1], reverse=True)
    for _, score in fused:
        assert 0.0 <= score <= 1.0


@given(bm25=_result_list_strategy(), vector=_result_list_strategy())
def test_adaptive_unknown_matches_unknown_weights(bm25, vector):
    adaptive = ScoreFusion.adaptive_fusion(bm25, vector, "definitely-unknown")
    weighted = ScoreFusion.weighted_combination(
        bm25,
        vector,
        bm25_weight=0.30,
        vector_weight=0.70,
    )
    assert adaptive == weighted


def test_weighted_combination_handles_negative_bm25_scores():
    bm25 = [("doc-a", -10.0), ("doc-b", -1.0)]
    vector = []
    fused = ScoreFusion.weighted_combination(bm25, vector)
    assert all(score >= 0.0 for _, score in fused)


def test_adaptive_fusion_returns_empty_when_inputs_empty():
    assert ScoreFusion.adaptive_fusion([], [], "policy") == []
