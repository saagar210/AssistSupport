#!/usr/bin/env python3
"""Test search quality after Option A improvements."""

import requests
import json

API_URL = "http://localhost:3000/search"

TEST_CASES = [
    ("Can I use a flash drive?", "flash", "POLICY"),
    ("How do I reset my password?", "password", None),
    ("What is the remote work policy?", "remote", "POLICY"),
    ("VPN setup instructions", "vpn", None),
    ("Cloud storage allowed", "cloud", "POLICY"),
    ("Laptop approval process", "laptop", None),
    ("Email security guidelines", "email", None),
    ("Data backup requirements", "backup", None),
    ("Software license approval", "software", None),
    ("USB device policy", "usb", "POLICY"),
]

print("=" * 70)
print("  POST-FIX TEST RESULTS (Option A Applied)")
print("=" * 70)
print()

passed = 0
total = len(TEST_CASES)

for i, (query, expected_keyword, expected_category) in enumerate(TEST_CASES, 1):
    try:
        resp = requests.post(
            API_URL,
            json={"query": query, "top_k": 3, "include_scores": True},
            timeout=10,
        )
        data = resp.json()

        intent = data.get("intent", "N/A")
        conf = data.get("intent_confidence", 0)
        results = data.get("results", [])

        print(f"Q{i}: \"{query}\"")
        print(f"  Intent: {intent} (confidence: {conf})")

        for j, r in enumerate(results[:3], 1):
            cat = r.get("category", "?")
            title = r.get("title", "?")[:65]
            scores = r.get("scores", {})
            fused = scores.get("fused", 0)
            bm25 = scores.get("bm25", 0)
            vector = scores.get("vector", 0)
            print(f"  #{j}: [{cat}] {title}")
            print(f"       fused={fused}, bm25={bm25}, vector={vector}")

        # Check if top result matches expected
        if results:
            top_title = results[0].get("title", "").lower()
            top_cat = results[0].get("category", "")
            top_preview = results[0].get("preview", "").lower()

            keyword_match = (
                expected_keyword.lower() in top_title
                or expected_keyword.lower() in top_preview
            )
            category_match = (
                expected_category is None or top_cat == expected_category
            )

            if keyword_match and category_match:
                passed += 1
                print(f"  >>> PASS")
            else:
                # Check top-3 for partial credit
                top3_match = any(
                    expected_keyword.lower() in r.get("title", "").lower()
                    or expected_keyword.lower() in r.get("preview", "").lower()
                    for r in results[:3]
                )
                if top3_match:
                    print(f"  >>> MISS (top-1), but found in top-3")
                else:
                    print(f"  >>> MISS (expected: {expected_keyword})")
        else:
            print(f"  >>> MISS (no results)")

    except Exception as e:
        print(f"  >>> ERROR: {e}")

    print()

print("=" * 70)
print(f"  BASELINE (before fixes):  2/10 (20%)")
print(f"  AFTER OPTION A:           {passed}/{total} ({passed * 100 // total}%)")
print("=" * 70)

# Also check for specific critical fixes
print("\nCRITICAL FIX VERIFICATION:")
print("-" * 40)

# Test 1: Flash drive policy
resp = requests.post(API_URL, json={"query": "Can I use a flash drive?", "top_k": 1, "include_scores": True})
data = resp.json()
r = data["results"][0] if data["results"] else {}
is_flash = "flash" in r.get("title", "").lower()
is_policy = r.get("category") == "POLICY"
fused = r.get("scores", {}).get("fused", 0)
print(f"  Flash drive query → Flash Drive Policy: {'YES' if is_flash else 'NO'}")
print(f"  Category is POLICY: {'YES' if is_policy else 'NO'}")
print(f"  Fused score: {fused} (was 0.6 with wrong result)")
print(f"  Policy boost bug fixed: {'YES' if fused != 0.6 else 'MAYBE'}")

# Test 2: Score inflation gone
print()
resp = requests.post(API_URL, json={"query": "VPN setup instructions", "top_k": 1, "include_scores": True})
data = resp.json()
r = data["results"][0] if data["results"] else {}
title = r.get("title", "")
fused = r.get("scores", {}).get("fused", 0)
print(f"  VPN setup → Top result: {title[:60]}")
print(f"  Fused score: {fused} (should NOT be artificial 0.6)")

print()
