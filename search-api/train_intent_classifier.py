#!/usr/bin/env python3
"""
Step 10: Train a lightweight intent classifier to replace keyword-based detection.
Uses TF-IDF features + Logistic Regression trained on synthetic + real query data.
Exports a joblib model for use in production.
"""

import sys
import os
import psycopg2
import json
import numpy as np

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from sklearn.feature_extraction.text import TfidfVectorizer
from sklearn.linear_model import LogisticRegression
from sklearn.model_selection import cross_val_score
from sklearn.pipeline import Pipeline
import joblib


# Synthetic training data covering IT support query patterns
TRAINING_DATA = [
    # POLICY queries — "am I allowed", rules, governance, restrictions
    ("Can I use a flash drive?", "policy"),
    ("Am I allowed to use USB devices?", "policy"),
    ("Is it okay to install personal software?", "policy"),
    ("What is the remote work policy?", "policy"),
    ("Are we allowed to use personal devices?", "policy"),
    ("USB device policy", "policy"),
    ("Cloud storage allowed", "policy"),
    ("What are the password requirements?", "policy"),
    ("Is it permitted to share credentials?", "policy"),
    ("Can I work from home?", "policy"),
    ("What is the BYOD policy?", "policy"),
    ("Are flash drives forbidden?", "policy"),
    ("Is personal email allowed on work laptop?", "policy"),
    ("What data can I store on external drives?", "policy"),
    ("Email security guidelines", "policy"),
    ("Firewall rules for remote workers", "policy"),
    ("VPN usage requirements", "policy"),
    ("What software is approved for use?", "policy"),
    ("Can I access company data from home?", "policy"),
    ("Is screen sharing allowed in client meetings?", "policy"),
    ("Data retention policy", "policy"),
    ("Mobile device management policy", "policy"),
    ("Acceptable use policy", "policy"),
    ("Security compliance requirements", "policy"),
    ("What encryption is required for laptops?", "policy"),
    ("Are we required to use MFA?", "policy"),
    ("Can I connect to public wifi?", "policy"),
    ("Is Dropbox allowed?", "policy"),
    ("Social media policy at work", "policy"),
    ("Travel security policy", "policy"),
    ("Removable media restrictions", "policy"),
    ("Data classification rules", "policy"),
    ("Guest network access policy", "policy"),
    ("Can contractors use their own laptops?", "policy"),
    ("What happens if I violate security policy?", "policy"),
    ("Password expiration rules", "policy"),
    ("Am I allowed to use ChatGPT at work?", "policy"),
    ("Is it okay to forward work emails to personal account?", "policy"),
    ("Clean desk policy", "policy"),
    ("What are the rules for storing PII?", "policy"),
    ("Company policy on encrypted drives", "policy"),
    ("Dual monitor policy", "policy"),
    ("Are personal phones allowed in secure areas?", "policy"),
    ("Internet usage policy", "policy"),
    ("Can I use my own keyboard and mouse?", "policy"),
    ("What is the Okta access policy?", "policy"),
    ("Duo enforcement policy exemptions", "policy"),

    # PROCEDURE queries — "how do I", setup, step-by-step, troubleshooting
    ("How do I reset my password?", "procedure"),
    ("How to set up VPN?", "procedure"),
    ("Steps to request a new laptop", "procedure"),
    ("How to connect to the printer", "procedure"),
    ("How do I submit a software request?", "procedure"),
    ("Laptop approval process", "procedure"),
    ("Software license approval", "procedure"),
    ("How to set up a printer", "procedure"),
    ("Steps to configure email on phone", "procedure"),
    ("How to request access to a shared drive", "procedure"),
    ("How to create a Jira ticket", "procedure"),
    ("Process for onboarding new employees", "procedure"),
    ("How to set up dual monitors", "procedure"),
    ("How to connect to VPN using GlobalProtect", "procedure"),
    ("Steps to set up a new hire's laptop", "procedure"),
    ("How do I request a hardware refresh?", "procedure"),
    ("How to troubleshoot slow internet", "procedure"),
    ("Process for submitting an IT ticket", "procedure"),
    ("How to install approved software", "procedure"),
    ("How to set up Slack notifications", "procedure"),
    ("New hire onboarding checklist", "procedure"),
    ("How to configure Outlook", "procedure"),
    ("Process for offboarding an employee", "procedure"),
    ("Steps to escalate a ticket", "procedure"),
    ("How to transfer files between computers", "procedure"),
    ("How to join a Zoom meeting", "procedure"),
    ("Steps to reset Okta MFA", "procedure"),
    ("How to update my laptop operating system", "procedure"),
    ("How to set up Google Groups", "procedure"),
    ("Process for laptop return", "procedure"),
    ("Steps to request admin access", "procedure"),
    ("How to back up my data", "procedure"),
    ("How to configure wifi on Mac", "procedure"),
    ("Steps for provisioning Adobe license", "procedure"),
    ("How to whitelist an email sender", "procedure"),
    ("Process for changing my email alias", "procedure"),
    ("How to clear browser cache", "procedure"),
    ("Steps to set up time tracking", "procedure"),
    ("How to submit an expense report", "procedure"),
    ("Process for requesting a monitor", "procedure"),
    ("How to update Chrome", "procedure"),
    ("How to reset my Slack password", "procedure"),
    ("Steps to install antivirus", "procedure"),
    ("How to connect bluetooth headphones", "procedure"),
    ("Monitor setup dual screen", "procedure"),
    ("Slack channel request process", "procedure"),

    # REFERENCE queries — "what is", definitions, overviews, information
    ("What is Okta?", "reference"),
    ("What cloud storage options are available?", "reference"),
    ("List of approved hardware", "reference"),
    ("Google workspace admin overview", "reference"),
    ("Zoom meeting best practices", "reference"),
    ("What is the IT help desk phone number?", "reference"),
    ("IT support contact information", "reference"),
    ("List of available software licenses", "reference"),
    ("What monitors are supported?", "reference"),
    ("Office locations and IT support hours", "reference"),
    ("What VPN clients are available?", "reference"),
    ("Explain the IT support tiers", "reference"),
    ("Available laptop models for engineers", "reference"),
    ("What is ServiceDesk?", "reference"),
    ("Overview of the remote support model", "reference"),
    ("Tell me about the password manager", "reference"),
    ("What collaboration tools do we use?", "reference"),
    ("Information about Jira", "reference"),
    ("What is the approved video conferencing tool?", "reference"),
    ("Details about Box storage limits", "reference"),
    ("What operating systems are supported?", "reference"),
    ("Available printer locations", "reference"),
    ("IT team structure and contacts", "reference"),
    ("Network diagram overview", "reference"),
    ("List of IT services", "reference"),
    ("SLA for ticket response times", "reference"),
    ("What is the difference between Jira and ServiceDesk?", "reference"),
    ("Supported browsers for internal tools", "reference"),
    ("What devices are in the hardware catalog?", "reference"),
    ("Available SSO integrations", "reference"),
    ("What is LucidChart?", "reference"),
    ("Data backup requirements", "reference"),
    ("What antivirus do we use?", "reference"),
    ("Available email distribution lists", "reference"),
    ("What is Corp Vault?", "reference"),
    ("Explain the disaster recovery plan", "reference"),
    ("What AWS services are available?", "reference"),
    ("Jira project creation guide", "reference"),
    ("Wifi troubleshooting reference", "reference"),

    # UNKNOWN queries — ambiguous, general, or off-topic
    ("help", "unknown"),
    ("error", "unknown"),
    ("not working", "unknown"),
    ("my computer is slow", "unknown"),
    ("something is broken", "unknown"),
    ("urgent issue", "unknown"),
    ("test", "unknown"),
    ("hello", "unknown"),
    ("where is the kitchen?", "unknown"),
    ("lunch menu", "unknown"),
    ("need help", "unknown"),
    ("system down", "unknown"),
    ("can't login", "unknown"),
    ("blue screen", "unknown"),
    ("frozen screen", "unknown"),
    ("wifi not connecting", "unknown"),
    ("it doesn't work", "unknown"),
    ("please help me", "unknown"),
    ("problem with my machine", "unknown"),
    ("stuff is broken", "unknown"),
]


def load_real_queries():
    """Load and deduplicate real queries from the database."""
    conn = psycopg2.connect(
        host="localhost",
        user="assistsupport_dev",
        password="dev_password_123",
        database="assistsupport_dev",
    )
    cur = conn.cursor()
    cur.execute("""
        SELECT DISTINCT query_text, category_filter, intent_confidence
        FROM query_performance
        WHERE intent_confidence >= 0.4
        ORDER BY intent_confidence DESC
    """)
    rows = cur.fetchall()
    cur.close()
    conn.close()

    # Only use high-confidence labels
    real_data = []
    for query, intent, conf in rows:
        if intent and conf >= 0.4:
            real_data.append((query, intent))
    return real_data


def main():
    print("=" * 70)
    print("  Step 10: Train Intent Classifier")
    print("=" * 70)
    print()

    # Combine synthetic + real data
    real_queries = load_real_queries()
    print(f"Synthetic training examples: {len(TRAINING_DATA)}")
    print(f"Real query examples (conf >= 0.4): {len(real_queries)}")

    all_data = TRAINING_DATA + real_queries
    # Deduplicate by lowercase query
    seen = set()
    deduped = []
    for query, label in all_data:
        key = query.lower().strip()
        if key not in seen:
            seen.add(key)
            deduped.append((query, label))

    texts = [d[0] for d in deduped]
    labels = [d[1] for d in deduped]

    print(f"Total unique training examples: {len(deduped)}")

    # Count per class
    from collections import Counter
    dist = Counter(labels)
    for intent, count in sorted(dist.items()):
        print(f"  {intent}: {count}")
    print()

    # Build pipeline
    pipeline = Pipeline([
        ("tfidf", TfidfVectorizer(
            ngram_range=(1, 2),
            max_features=5000,
            sublinear_tf=True,
            min_df=1,
        )),
        ("clf", LogisticRegression(
            C=10.0,
            max_iter=1000,
            class_weight="balanced",
            solver="lbfgs",
            multi_class="multinomial",
        )),
    ])

    # Cross-validate
    scores = cross_val_score(pipeline, texts, labels, cv=5, scoring="accuracy")
    print(f"5-fold CV accuracy: {scores.mean():.3f} (+/- {scores.std():.3f})")
    print(f"  Per-fold: {[f'{s:.3f}' for s in scores]}")
    print()

    # Train on full data
    pipeline.fit(texts, labels)

    # Test on our standard query set
    test_queries = [
        "Can I use a flash drive?",
        "How do I reset my password?",
        "What is the remote work policy?",
        "VPN setup instructions",
        "Cloud storage allowed",
        "Laptop approval process",
        "Email security guidelines",
        "Data backup requirements",
        "Software license approval",
        "USB device policy",
        "Okta access policy",
        "Wifi not connecting",
        "Google workspace admin",
        "Zoom meeting best practices",
        "Jira project creation",
        "New hire onboarding checklist",
        "Firewall rules for remote",
        "Slack channel request",
        "How to set up a printer",
        "Monitor setup dual screen",
    ]

    print("Test predictions:")
    print("-" * 60)
    for query in test_queries:
        pred = pipeline.predict([query])[0]
        proba = pipeline.predict_proba([query])[0]
        classes = pipeline.classes_
        conf = max(proba)
        print(f"  {query:45s} → {pred:10s} ({conf:.2f})")

    # Save model
    model_path = os.path.join(os.path.dirname(os.path.abspath(__file__)), "intent_model.joblib")
    joblib.dump(pipeline, model_path)
    print(f"\nModel saved to: {model_path}")
    print(f"Model size: {os.path.getsize(model_path) / 1024:.1f} KB")

    # Save class names for reference
    meta = {
        "classes": list(pipeline.classes_),
        "training_size": len(deduped),
        "cv_accuracy": float(scores.mean()),
        "model_file": "intent_model.joblib",
    }
    meta_path = os.path.join(os.path.dirname(os.path.abspath(__file__)), "intent_model_meta.json")
    with open(meta_path, "w") as f:
        json.dump(meta, f, indent=2)

    print(f"\n{'=' * 70}")
    print("  INTENT CLASSIFIER TRAINING COMPLETE")
    print(f"{'=' * 70}")


if __name__ == "__main__":
    main()
