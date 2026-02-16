import string
import sys
from pathlib import Path

from hypothesis import given, strategies as st

SEARCH_API_DIR = Path(__file__).resolve().parents[1]
if str(SEARCH_API_DIR) not in sys.path:
    sys.path.insert(0, str(SEARCH_API_DIR))

import intent_detection  # noqa: E402
from intent_detection import IntentDetector  # noqa: E402


def test_keyword_detection_expected_examples(monkeypatch):
    monkeypatch.setattr(intent_detection, "_MODEL", None)
    monkeypatch.setattr(intent_detection, "_MODEL_LOADED", True)

    assert IntentDetector.detect("Can I use a flash drive?")[0] == "policy"
    assert IntentDetector.detect("How do I reset my password?")[0] == "procedure"
    assert IntentDetector.detect("What is VPN?")[0] in {"reference", "unknown"}


@given(
    st.text(
        alphabet=string.ascii_letters + string.digits + " -_",
        min_size=0,
        max_size=200,
    )
)
def test_keyword_detector_bounds_and_labels(query):
    intent, confidence = IntentDetector._detect_keywords(query)
    assert intent in {"policy", "procedure", "reference", "unknown"}
    assert 0.0 <= confidence <= 1.0
