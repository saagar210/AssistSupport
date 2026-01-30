# AssistSupport — Response Quality Improvement Roadmap

## Date: January 29, 2026

---

## Root Causes Identified

| # | Root Cause | Impact | Effort | Expected Gain |
|---|---|---|---|---|
| 1 | Curated KB not in PostgreSQL | CRITICAL | 1-2 hrs | +40-50% |
| 2 | DB full of low-quality fragments | HIGH | 2-3 hrs | +15-25% |
| 3 | Score fusion policy_boost bug | MEDIUM | 30 min | +10-15% |
| 4 | Intent detection doesn't filter results | LOW-MED | 1 hr | +5-10% |

Combined potential: **+70-90% quality improvement**

---

## Improvement Paths

### Option A: Quick Wins (3-4 hours, +60-70% quality)

Fix the most impactful issues without model changes or major refactoring.

**Steps:**

1. **Ingest curated KB into PostgreSQL** (1-2 hours)
   - Write Python script to read 27 markdown files from `knowledge_base/`
   - Parse title, category, content from each file
   - Generate embeddings using same model (all-MiniLM-L6-v2)
   - INSERT into `kb_articles` table
   - Verify with test queries

2. **Fix policy_boost bug in score_fusion.py** (30 min)
   - Current: `if bm25_norm > 0.5: score += policy_boost` (always triggers for top result)
   - Fix: Only apply when `query_type == "policy"` AND article `category == "POLICY"`
   - Pass category data through the fusion pipeline

3. **Remove junk articles** (30 min)
   - DELETE articles under 100 chars (293 articles of garbage)
   - Mark articles under 200 chars as `is_active = false` (additional ~200)
   - Re-test search quality

4. **Add category boost for intent matches** (1 hour)
   - When intent=policy with confidence > 0.4, boost POLICY results by +0.15
   - When intent=procedure with confidence > 0.4, boost PROCEDURE results by +0.10
   - Implement in adaptive_fusion function

**Expected outcome:**
- Flash drive query → Returns flash_drives_forbidden.md (correct)
- Password policy query → Returns password_policy.md (correct)
- VPN setup query → Returns vpn_setup.md (correct)
- Relevance rate: 60-70% → estimated 85-90%

**Risks:** Low. All changes are additive or bug fixes.

---

### Option B: Option A + Content Cleanup (6-8 hours, +75-85% quality)

Everything in Option A plus systematic content quality improvement.

**Additional steps:**

5. **Clean breadcrumb titles** (1 hour)
   - Extract final segment from titles like `"OS X > ASD tests > Installing"` → `"Installing ASD Tests on OS X"`
   - Use heading_path for breadcrumb context, keep title clean
   - Update ~829 articles with titles > 100 chars

6. **Identify and expand thin articles** (2-3 hours)
   - Query: top 50 searched terms from `query_performance`
   - Cross-reference with articles < 500 chars on those topics
   - Expand 20-30 key articles from 200 chars → 500+ chars
   - Add context, examples, and escalation paths

7. **Rebuild FTS and HNSW indexes** (30 min)
   - After content changes, regenerate `fts_content` tsvectors
   - Regenerate embeddings for modified articles
   - REINDEX FTS and HNSW indexes

**Expected outcome:**
- Relevance rate: estimated 90-95%
- Fewer false-positive BM25 matches from noisy titles
- Better semantic matches from enriched content

**Risks:** Medium. Content editing requires domain knowledge validation.

---

### Option C: Full Solution (12-16 hours, +85-95% quality)

Everything in Option B plus model upgrade and ML improvements.

**Additional steps:**

8. **Upgrade embedding model** (2-3 hours)
   - Current: all-MiniLM-L6-v2 (384 dims, general purpose)
   - Target: e5-base-v2 or jina-embeddings-v2-base (768 dims, IR-optimized)
   - Generate new embeddings for all articles
   - Rebuild HNSW index with new dimension
   - Benchmark latency impact (+20-35ms expected)

9. **Implement result re-ranking** (2-3 hours)
   - Add cross-encoder re-ranking on top-10 results
   - Use ms-marco-MiniLM-L-6-v2 cross-encoder
   - Re-score top candidates for better precision
   - Latency impact: +30-50ms for re-ranking

10. **Train intent classifier** (3-4 hours)
    - Collect 200+ labeled query→intent pairs from logs
    - Train lightweight classifier (logistic regression or small NN)
    - Replace keyword-based intent detection
    - Expected confidence: 0.4 avg → 0.8+ avg

11. **Implement feedback loop** (2-3 hours)
    - Use search_feedback data to adjust per-article quality scores
    - Down-rank articles with "not_helpful" feedback
    - Up-rank articles with "helpful" feedback
    - Minimum 50 feedback entries needed

**Expected outcome:**
- Relevance rate: 95%+
- Intent accuracy: 90%+
- Latency: ~130-150ms (acceptable trade-off)

**Risks:** Higher complexity. Model changes require testing. Cross-encoder adds latency.

---

## Decision Tree

```
START
  │
  ├─ Need immediate fix? (< 4 hours)
  │   └─ YES → Option A
  │       Result: +60-70% quality, no latency change
  │
  ├─ Have 1 day? Want 90%+ quality?
  │   └─ YES → Option B
  │       Result: +75-85% quality, no latency change
  │
  └─ Have 2-3 days? Want production-grade quality?
      └─ YES → Option C
          Result: +85-95% quality, +30-50ms latency
```

---

## Embedding Model Comparison

| Model | Size | Dims | Speed | Quality | Domain |
|---|---|---|---|---|---|
| all-MiniLM-L6-v2 (current) | 33M | 384 | 8ms | Medium | General |
| all-mpnet-base-v2 | 438M | 768 | 45-60ms | High | General |
| e5-base-v2 | 110M | 768 | 25-35ms | High | IR-tuned |
| jina-embeddings-v2-base | 150M | 768 | 20-30ms | Very High | IR + technical |
| nomic-embed-text (local app) | ~137M | 768 | 15-25ms | High | General |

**Recommendation:** Model upgrade is Option C territory. Fix content first (Options A/B) — the model can't find articles that don't exist in the database, regardless of how good the model is.

---

## Score Fusion Analysis

### Current Adaptive Weights

| Intent | BM25 Weight | Vector Weight | Policy Boost |
|---|---|---|---|
| policy | 0.50 | 0.40 | 0.10 |
| procedure | 0.35 | 0.55 | 0.10 |
| reference | 0.20 | 0.70 | 0.10 |
| unknown | 0.30 | 0.60 | 0.10 |

### Issues

1. **Policy boost is unconditional** — applied to any article with BM25 > 0.5, not just policy articles
2. **BM25 weight is too high for policy queries** — "Can I use a flash drive?" should favor semantic understanding, not keyword matching
3. **No category filtering** — intent detection result is only used for weight adjustment, not result filtering

### Recommended Weights (after fixing bug)

| Intent | BM25 Weight | Vector Weight | Category Boost |
|---|---|---|---|
| policy | 0.35 | 0.55 | +0.15 for POLICY docs |
| procedure | 0.35 | 0.55 | +0.10 for PROCEDURE docs |
| reference | 0.20 | 0.70 | +0.05 for REFERENCE docs |
| unknown | 0.30 | 0.60 | none |

---

## Success Metrics

Track these before and after each improvement phase:

| Metric | Current | After A | After B | After C |
|---|---|---|---|---|
| Top-1 Relevance | 20% | 70% | 85% | 95% |
| Top-3 Relevance | 40% | 85% | 90% | 98% |
| Intent Accuracy | 60% | 65% | 70% | 90% |
| Avg Confidence | 0.42 | 0.50 | 0.55 | 0.80 |
| p95 Latency | 86ms | 86ms | 86ms | 130ms |
| Feedback Helpful % | 75% (n=4) | TBD | TBD | TBD |

### Test Query Set (10 queries for before/after comparison)

```
1. "Can I use a flash drive?"              → Expected: flash_drives_forbidden (POLICY)
2. "How do I reset my password?"           → Expected: password_reset (PROCEDURE)
3. "What is the remote work policy?"       → Expected: remote_access_policy (POLICY)
4. "VPN setup instructions"                → Expected: vpn_setup (PROCEDURE)
5. "Cloud storage allowed"                 → Expected: cloud_storage_policy (POLICY)
6. "Laptop approval process"               → Expected: request_new_laptop (PROCEDURE)
7. "Email security guidelines"             → Expected: email_security_policy (POLICY)
8. "Data backup requirements"              → Expected: data_backup (PROCEDURE)
9. "Software license approval"             → Expected: software_request (PROCEDURE)
10. "USB device policy"                    → Expected: removable_media_policy (POLICY)
```

---

## Testing Procedure

After each improvement phase:

```bash
# Run all 10 test queries
for query in \
  "Can I use a flash drive?" \
  "How do I reset my password?" \
  "What is the remote work policy?" \
  "VPN setup instructions" \
  "Cloud storage allowed" \
  "Laptop approval process" \
  "Email security guidelines" \
  "Data backup requirements" \
  "Software license approval" \
  "USB device policy"; do

  echo "=== $query ==="
  curl -s -X POST http://localhost:3000/search \
    -H "Content-Type: application/json" \
    -d "{\"query\":\"$query\",\"top_k\":3,\"include_scores\":true}" \
    | python3 -c "
import sys, json
data = json.load(sys.stdin)
print(f'Intent: {data[\"intent\"]} ({data[\"intent_confidence\"]})')
for r in data['results'][:3]:
    s = r.get('scores', {})
    print(f'  [{r[\"category\"]}] {r[\"title\"][:60]}')
    print(f'    fused={s.get(\"fused\")}, bm25={s.get(\"bm25\")}, vector={s.get(\"vector\")}')
"
  echo ""
done
```

Compare top-1 and top-3 relevance against expected results above.
