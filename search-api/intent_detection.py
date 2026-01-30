#!/usr/bin/env python3
"""
Intent Detection for Query Classification
Uses trained ML classifier (TF-IDF + Logistic Regression) with keyword fallback.
Classifies queries as policy/procedure/reference/unknown and measures confidence.
"""

import os
import re
from typing import Tuple

_MODEL = None
_MODEL_LOADED = False


def _load_model():
    """Load the trained intent classifier if available."""
    global _MODEL, _MODEL_LOADED
    if _MODEL_LOADED:
        return _MODEL
    _MODEL_LOADED = True
    model_path = os.path.join(os.path.dirname(os.path.abspath(__file__)), "intent_model.joblib")
    if os.path.exists(model_path):
        try:
            import joblib
            _MODEL = joblib.load(model_path)
            print(f"Intent classifier loaded from {model_path}")
        except Exception as e:
            print(f"Failed to load intent classifier: {e}")
            _MODEL = None
    return _MODEL


class IntentDetector:
    """Detect query intent using ML classifier with keyword fallback."""

    # Keyword-based fallback (used when model is unavailable)
    POLICY_PRIORITY = [
        "can i", "am i allowed", "am i permitted", "is it allowed",
        "is it okay", "are we allowed", "policy",
    ]
    POLICY_KEYWORDS = {
        "forbidden": ["forbidden", "not allowed", "banned", "prohibited", "restricted"],
        "governance": ["rule", "must", "shall", "compliance"],
        "removable_media": ["usb", "flash drive", "portable", "removable", "sd card"],
        "security": ["firewall", "vpn", "encryption", "mfa"],
        "data_handling": ["confidential", "pii", "encrypt"],
    }

    PROCEDURE_PRIORITY = [
        "how do i", "how to", "how do you", "how can i", "steps to",
    ]
    PROCEDURE_KEYWORDS = {
        "action": ["procedure", "process", "walkthrough", "guide"],
        "request": ["request", "apply for", "submit", "fill out", "approval"],
        "setup": ["setup", "install", "configure", "set up", "initialization"],
        "account": ["account", "login", "reset", "register"],
        "hardware": ["laptop", "computer", "phone", "monitor", "keyboard", "device"],
        "software": ["software", "application", "app", "tool", "license"],
    }

    REFERENCE_PRIORITY = [
        "what is", "what are", "what does", "tell me about",
    ]
    REFERENCE_KEYWORDS = {
        "definition": ["definition", "explain", "describe", "meaning"],
        "information": ["about", "information", "details", "overview", "summary"],
        "list": ["list", "options", "available", "approved", "allowed"],
        "requirements": ["requirement", "requirements"],
    }

    @staticmethod
    def detect(query: str) -> Tuple[str, float]:
        """
        Detect query intent using ML model (primary) or keywords (fallback).
        Returns: (intent_type, confidence_0_to_1)
        """
        model = _load_model()
        if model is not None:
            return IntentDetector._detect_ml(query, model)
        return IntentDetector._detect_keywords(query)

    @staticmethod
    def _detect_ml(query: str, model) -> Tuple[str, float]:
        """ML-based intent detection using trained classifier."""
        proba = model.predict_proba([query])[0]
        classes = model.classes_
        best_idx = proba.argmax()
        intent = classes[best_idx]
        confidence = float(proba[best_idx])

        # If confidence is below threshold, fall back to "unknown"
        if confidence < 0.4:
            intent = "unknown"
            confidence = float(1.0 - max(proba))

        return intent, round(confidence, 2)

    @staticmethod
    def _detect_keywords(query: str) -> Tuple[str, float]:
        """Keyword-based fallback intent detection."""
        q_lower = query.lower()

        policy_score = IntentDetector._score_intent(
            q_lower, IntentDetector.POLICY_KEYWORDS, IntentDetector.POLICY_PRIORITY)
        procedure_score = IntentDetector._score_intent(
            q_lower, IntentDetector.PROCEDURE_KEYWORDS, IntentDetector.PROCEDURE_PRIORITY)
        reference_score = IntentDetector._score_intent(
            q_lower, IntentDetector.REFERENCE_KEYWORDS, IntentDetector.REFERENCE_PRIORITY)

        scores = {
            "policy": policy_score,
            "procedure": procedure_score,
            "reference": reference_score,
        }

        intent = max(scores, key=scores.get)
        confidence = scores[intent]

        if confidence < 0.1:
            intent = "unknown"

        return intent, confidence

    @staticmethod
    def _score_intent(query: str, keywords_dict: dict, priority_phrases: list) -> float:
        """Score a query against priority phrases and keyword dictionary."""
        total_score = 0.0
        for phrase in priority_phrases:
            if phrase in query:
                total_score += 2.0
        for category, keywords in keywords_dict.items():
            for keyword in keywords:
                if keyword in query:
                    if re.search(rf"\b{re.escape(keyword)}\b", query):
                        total_score += 1.0
                    else:
                        total_score += 0.5
        return min(1.0, total_score / 5.0)


if __name__ == "__main__":
    print("Testing Intent Detection\n")

    test_queries = [
        "Can I use a flash drive?",
        "How do I reset my password?",
        "What cloud storage options are available?",
        "Am I allowed to work from home?",
        "What is the VPN setup procedure?",
        "List approved hardware for engineers",
        "I need to request a new laptop",
        "What's the policy on encrypted USB drives?",
        "Wifi not connecting",
        "Jira project creation",
        "Google workspace admin",
    ]

    for query in test_queries:
        intent, confidence = IntentDetector.detect(query)
        print(f"'{query}'")
        print(f"  -> {intent.upper()} (confidence: {confidence:.2f})\n")

    print("Intent detection working")
