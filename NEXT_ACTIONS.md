# Next Actions — Response Quality Improvement

## Date: January 29, 2026

---

## Phase 1: Critical Fixes (3-4 hours)

### Step 1: Ingest Curated KB into PostgreSQL (1-2 hours)

- [ ] Write `ingest_curated_kb.py` script
  - Read all 27 markdown files from `knowledge_base/`
  - Parse frontmatter/headers for title and category
  - Category from directory: POLICIES → policy, PROCEDURES → procedure, REFERENCE → reference
  - Generate embeddings using same model as existing articles
  - INSERT into `kb_articles` with proper UUID, timestamps
  - Set `is_active = true`

- [ ] Run ingestion script
  ```bash
  cd ~/assistsupport-semantic-migration/week4
  source ../venv_week2/bin/activate
  python3 ingest_curated_kb.py
  ```

- [ ] Verify ingestion
  ```sql
  SELECT category, COUNT(*), AVG(LENGTH(content))
  FROM kb_articles
  WHERE source_document_id = 'curated_kb'
  GROUP BY category;
  ```

- [ ] Test key queries
  - "Can I use a flash drive?" → should now return flash_drives_forbidden
  - "USB device policy" → should return removable_media_policy
  - "VPN setup instructions" → should return vpn_setup

### Step 2: Fix Score Fusion Bug (30 minutes)

- [ ] Edit `~/assistsupport-semantic-migration/week4/score_fusion.py`

  **Current code (line ~86):**
  ```python
  # Policy boost: if BM25 score is high (>0.5), add boost
  if bm25_norm > 0.5:
      score += policy_boost
  ```

  **Fixed code:**
  ```python
  # Policy boost removed from weighted_combination
  # Category boosting handled in adaptive_fusion instead
  ```

- [ ] Add category-aware boosting to `adaptive_fusion` in score_fusion.py

- [ ] Restart Flask API
  ```bash
  # Kill existing process
  pkill -f search_api.py
  # Restart
  cd ~/assistsupport-semantic-migration/week4
  source ../venv_week2/bin/activate
  python3 search_api.py &
  ```

- [ ] Verify fix: "Can I use a flash drive?" should no longer return fused=0.6 for irrelevant articles

### Step 3: Remove Junk Articles (30 minutes)

- [ ] Remove articles under 100 chars
  ```sql
  -- Preview what will be removed
  SELECT COUNT(*) FROM kb_articles WHERE LENGTH(content) < 100;

  -- Soft delete (set inactive)
  UPDATE kb_articles SET is_active = false
  WHERE LENGTH(content) < 100;
  ```

- [ ] Verify remaining article count
  ```sql
  SELECT COUNT(*) as active_articles
  FROM kb_articles WHERE is_active = true;
  ```

- [ ] Re-test queries to confirm no degradation

### Step 4: Add Category Boost (1 hour)

- [ ] Modify `adaptive_fusion` in score_fusion.py to boost category-matching results

  ```python
  # After fusion, boost results matching detected intent category
  if query_type in ("policy", "procedure", "reference"):
      boosted = []
      for article_id, score in ranked:
          category = get_article_category(article_id)  # Need to pass this through
          if category and category.lower() == query_type:
              score *= 1.15  # 15% boost for category match
          boosted.append((article_id, score))
      ranked = sorted(boosted, key=lambda x: x[1], reverse=True)
  ```

- [ ] Pass article category data through the search/fusion pipeline
- [ ] Test with all 10 test queries

### Step 5: Validate Phase 1 (30 minutes)

- [ ] Run full 10-query test suite (see IMPROVEMENT_ROADMAP.md)
- [ ] Record top-1 and top-3 relevance
- [ ] Compare against baseline (20% top-1, 40% top-3)
- [ ] Target: 70% top-1, 85% top-3

---

## Phase 2: Content Cleanup (Optional, 3-4 hours)

Only proceed if Phase 1 achieves < 80% top-1 relevance.

### Step 6: Clean Breadcrumb Titles (1 hour)

- [ ] Write script to extract final segment from breadcrumb titles
  ```sql
  -- Find breadcrumb titles
  SELECT id, title FROM kb_articles
  WHERE title LIKE '%>%' AND LENGTH(title) > 100
  LIMIT 20;
  ```

- [ ] Update titles to use last meaningful segment
- [ ] Regenerate FTS tsvectors for updated articles
  ```sql
  UPDATE kb_articles
  SET fts_content = to_tsvector('english', title || ' ' || content)
  WHERE title WAS UPDATED;
  ```

### Step 7: Expand Thin Articles (2-3 hours)

- [ ] Identify top 50 searched topics
  ```sql
  SELECT query_text, COUNT(*) as search_count
  FROM query_performance
  GROUP BY query_text
  ORDER BY search_count DESC
  LIMIT 50;
  ```

- [ ] Find thin articles (< 500 chars) on popular topics
- [ ] Expand 20-30 articles with:
  - Additional context and explanation
  - Step-by-step instructions
  - Troubleshooting tips
  - Escalation paths

- [ ] Regenerate embeddings for modified articles
- [ ] Re-test search quality

### Step 8: Reindex (30 minutes)

- [ ] Regenerate FTS tsvectors for all modified articles
- [ ] Rebuild HNSW index
  ```sql
  REINDEX INDEX kb_articles_embedding_hnsw_idx;
  ```
- [ ] Verify index health

---

## Phase 3: Model & ML Upgrade (Optional, 6-8 hours)

Only proceed if Phase 1+2 achieves < 90% top-1 relevance.

### Step 9: Upgrade Embedding Model

- [ ] Evaluate e5-base-v2 vs jina-embeddings-v2-base
- [ ] Download target model
- [ ] Write embedding regeneration script
- [ ] Generate new embeddings for all articles
- [ ] Update pgvector dimension (384 → 768)
- [ ] Rebuild HNSW index
- [ ] Benchmark latency impact

### Step 10: Add Cross-Encoder Re-ranking

- [ ] Install ms-marco-MiniLM-L-6-v2
- [ ] Add re-ranking step after fusion
- [ ] Benchmark latency impact
- [ ] A/B test with and without re-ranking

### Step 11: Train Intent Classifier

- [ ] Extract labeled data from query_performance
- [ ] Train lightweight classifier
- [ ] Replace keyword-based detection
- [ ] Measure confidence improvement

---

## Tracking

### Baseline Metrics (January 29, 2026)

| Metric | Value |
|---|---|
| Total articles | 3,536 |
| Articles < 100 chars | 293 (8.3%) |
| Articles < 500 chars | 1,910 (54%) |
| Median article length | 445 chars |
| POLICY articles | 102 (2.9%) |
| Top-1 relevance (10 queries) | 20% (2/10) |
| Top-3 relevance (10 queries) | 40% (4/10) |
| Avg intent confidence | 0.42 |
| Unknown intent rate | 20% (37/183) |
| p95 latency | 86ms |
| Total feedback | 4 entries |

### After Phase 1 Target

| Metric | Target |
|---|---|
| Top-1 relevance | 70%+ |
| Top-3 relevance | 85%+ |
| Avg intent confidence | 0.50+ |
| p95 latency | 86ms (no change) |

### After Phase 2 Target

| Metric | Target |
|---|---|
| Top-1 relevance | 85%+ |
| Top-3 relevance | 95%+ |
| Active articles | ~3,200 (removed junk) |

### After Phase 3 Target

| Metric | Target |
|---|---|
| Top-1 relevance | 95%+ |
| Intent confidence | 0.80+ |
| p95 latency | < 150ms |
