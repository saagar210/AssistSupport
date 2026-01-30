#!/usr/bin/env python3
"""
Intent Detection for Query Classification
Classifies queries as policy/procedure/reference and measures confidence
"""

import re
from typing import Tuple


class IntentDetector:
    """Detect query intent for better search result ranking"""

    # Strong signals get weight 2.0, regular keywords get 1.0
    # Priority signals are phrases that strongly indicate intent
    POLICY_PRIORITY = [
        "can i", "am i allowed", "am i permitted", "is it allowed",
        "is it okay", "are we allowed", "policy",
    ]

    POLICY_KEYWORDS = {
        "forbidden": [
            "forbidden",
            "not allowed",
            "banned",
            "prohibited",
            "restricted",
        ],
        "governance": ["rule", "must", "shall", "compliance"],
        "removable_media": [
            "usb",
            "flash drive",
            "portable",
            "removable",
            "sd card",
        ],
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
        Detect query intent
        Returns: (intent_type, confidence_0_to_1)
        """
        q_lower = query.lower()

        policy_score = IntentDetector._score_intent(
            q_lower,
            IntentDetector.POLICY_KEYWORDS,
            IntentDetector.POLICY_PRIORITY,
        )
        procedure_score = IntentDetector._score_intent(
            q_lower,
            IntentDetector.PROCEDURE_KEYWORDS,
            IntentDetector.PROCEDURE_PRIORITY,
        )
        reference_score = IntentDetector._score_intent(
            q_lower,
            IntentDetector.REFERENCE_KEYWORDS,
            IntentDetector.REFERENCE_PRIORITY,
        )

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
        """Score a query against priority phrases and keyword dictionary"""
        total_score = 0.0

        # Priority phrases get 2.0 weight
        for phrase in priority_phrases:
            if phrase in query:
                total_score += 2.0

        # Regular keywords get 1.0 weight
        for category, keywords in keywords_dict.items():
            for keyword in keywords:
                if keyword in query:
                    if re.search(rf"\b{re.escape(keyword)}\b", query):
                        total_score += 1.0
                    else:
                        total_score += 0.5

        # Normalize (max 5 keyword matches = confidence 1.0)
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
    ]

    for query in test_queries:
        intent, confidence = IntentDetector.detect(query)
        print(f"'{query}'")
        print(f"  -> {intent.upper()} (confidence: {confidence:.2f})\n")

    print("Intent detection working")
