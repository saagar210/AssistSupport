# Phase 4: KB Enrichment - Detailed Runbook

**Duration**: 1 week
**Prerequisites**: Phase 3 complete (router V2 validated)
**Success Criteria**: 85%+ top-1 relevance, 95%+ top-3 relevance

---

## Phase 4 Overview

Expand thin knowledge base articles to meet quality standards. Focus on:
- Expanding articles < 500 chars to 800+ chars
- Adding step-by-step instructions
- Including troubleshooting information
- Prioritizing articles with low pilot accuracy

---

## Step 1: Identify Thin Articles (30 min)

**Purpose**: Find articles needing expansion

**Script**: `scripts/identify_thin_articles.py`

```bash
cd src-tauri

# Identify thin articles
python3 ../scripts/identify_thin_articles.py

# Expected output:
# ┌────────────────────────────────────────┐
# │ KB Quality Analysis                     │
# ├────────────────────────────────────────┤
# │ Thin Articles (< 500 chars):    47     │
# │ Sparse Articles (500-800 chars): 89    │
# │ Full Articles (800+ chars):     314    │
# │                                        │
# │ Total articles:                 450    │
# │ Priority for expansion:         47     │
# └────────────────────────────────────────┘
```

**Output File**: `thin_articles_priority.json`

```json
{
  "thin_articles": [
    {
      "id": 1,
      "title": "Reset Password",
      "current_length": 142,
      "target_length": 800,
      "pilot_accuracy": 0.70,
      "mentions_in_feedback": 5,
      "priority": "high"
    },
    {
      "id": 2,
      "title": "Enable MFA",
      "current_length": 238,
      "target_length": 800,
      "pilot_accuracy": 0.65,
      "mentions_in_feedback": 3,
      "priority": "high"
    },
    ...
  ]
}
```

---

## Step 2: Expand Thin Articles (3-4 days)

**Purpose**: Write comprehensive articles for 47 thin articles

**Expansion Template**:

```markdown
# [Article Title]

## Overview
[1-2 sentence summary of what the article covers]

## Prerequisites
- [Requirement 1]
- [Requirement 2]

## Step-by-Step Instructions

### Step 1: [First step title]
[Detailed description and actions]

### Step 2: [Second step title]
[Detailed description and actions]

### Step 3: [Third step title]
[Detailed description and actions]

## Troubleshooting

### Issue: [Common problem 1]
**Symptom**: [How user recognizes this problem]
**Solution**: [Steps to fix]

### Issue: [Common problem 2]
**Symptom**: [How user recognizes this problem]
**Solution**: [Steps to fix]

## Related Articles
- [Link to related topic 1]
- [Link to related topic 2]

## FAQ

**Q: [Common question 1]**
A: [Answer]

**Q: [Common question 2]**
A: [Answer]
```

**Example Expansion** (Reset Password - before/after):

**Before** (142 chars):
```
To reset your password:
1. Go to the login page
2. Click "Forgot Password"
3. Follow the email link
4. Set a new password
```

**After** (800+ chars):
```
# Reset Your Active Directory Password

## Overview
This guide walks you through resetting your Active Directory password when you forget it or
need to change it for security reasons.

## Prerequisites
- Email access (for password reset link)
- Active Directory account with you
- Internet connection to access portal

## Step-by-Step Instructions

### Step 1: Access the Password Reset Portal
1. Open your web browser
2. Go to: https://password-reset.company.com/
3. You'll see the "Reset Your Password" form
4. Click "Forgot your password?" if you don't see the form

### Step 2: Verify Your Identity
1. Enter your username (firstname.lastname)
2. Enter your email address
3. Click "Verify Identity"
4. You'll receive an email with a verification code
5. Check your email (may take 2-5 minutes)
6. Enter the verification code in the portal

### Step 3: Set Your New Password
1. Create a new password (at least 12 characters)
2. Password must include:
   - Uppercase letters (A-Z)
   - Lowercase letters (a-z)
   - Numbers (0-9)
   - Special characters (!@#$%^&*)
3. Do NOT use your previous 5 passwords
4. Click "Reset Password"
5. Wait for confirmation (30 seconds)

### Step 4: Log In With New Password
1. Go to your email or application
2. Use your username and new password
3. If successful, you'll be logged in

## Troubleshooting

### Issue: "Verification code not received"
**Symptom**: Email doesn't arrive after 5 minutes
**Solution**:
1. Check your spam/junk folder
2. Wait another 5 minutes (emails can be slow)
3. Click "Resend code" in the portal
4. If still not received, contact IT Support

### Issue: "Password rejected - doesn't meet requirements"
**Symptom**: "Password must contain..." error
**Solution**:
1. Check you have: UPPERCASE, lowercase, 123, !@#
2. Make sure password is at least 12 characters
3. Avoid using your name or username
4. Example valid password: MyCompany2024!

### Issue: "Login still fails with new password"
**Symptom**: Password reset succeeded, but can't log in
**Solution**:
1. Wait 5-10 minutes (system needs to sync)
2. Try again
3. If still failing, clear browser cache:
   - Chrome: Ctrl+Shift+Delete (Windows) or Cmd+Shift+Delete (Mac)
   - Select "Cookies and cached images"
   - Clear
4. Try logging in again

## Related Articles
- [Setup Multi-Factor Authentication (MFA)](security/setup-mfa.md)
- [Account Locked? How to Unlock](security/account-lockout-recovery.md)
- [Change Your Password](email/change-email-password.md)

## FAQ

**Q: How often can I reset my password?**
A: As often as needed, but you cannot reuse any of your last 5 passwords.

**Q: Will resetting my password log me out everywhere?**
A: Yes, you'll be logged out of most applications. Log back in with your new password.

**Q: What if I never received the password reset email?**
A: Check spam folder, wait 10 minutes, or contact IT Support at support@company.com.

**Q: Can I reset my password from the login screen?**
A: Yes, click "Forgot password?" on the login page instead of using the web portal.
```

**Process**:
1. For each thin article in `thin_articles_priority.json`:
   - Open current article
   - Apply expansion template
   - Add prerequisites, steps, troubleshooting, FAQ
   - Write to target length (800+ chars)
   - Commit to git

**Batch Expansion**:
```bash
# Use this script to guide expansion
python3 ../scripts/expand_articles.py \
  --input thin_articles_priority.json \
  --template article_expansion_template.md \
  --output expanded_articles/

# This creates draft expansions you can refine
# Then review and enhance each one
```

**Verification**:
```bash
# Check article lengths after expansion
python3 ../scripts/verify_article_lengths.py

# Expected:
# Articles < 500 chars: 0 (was 47)
# Articles 500-800 chars: 15 (was 89)
# Articles 800+ chars: 482 (was 314)
```

---

## Step 3: Run Title Cleaning Script (30 min)

**Purpose**: Standardize and clean article titles

**Script**: `scripts/clean_article_titles.py`

```bash
cd src-tauri

# Clean titles in local SQLite KB
sqlite3 ~/.assistsupport/kb.db << 'EOF'
-- Standardize title format
UPDATE articles SET title =
  REPLACE(
    REPLACE(
      REPLACE(title, '  ', ' '),  -- Remove double spaces
      '  ', ' '
    ),
    ' _', ' '                       -- Remove trailing underscores
  );

-- Remove duplicate titles
DELETE FROM articles WHERE id NOT IN (
  SELECT MIN(id) FROM articles GROUP BY title
);

-- Verify cleanup
SELECT COUNT(*) as total_articles FROM articles;
SELECT COUNT(DISTINCT title) as unique_titles FROM articles;
EOF

# Expected output:
# total_articles: 450
# unique_titles: 449 (or close)
```

---

## Step 4: Rebuild Indexes (1 hour)

**Purpose**: Update FTS5 and vector indexes for search API

**Execution**:
```bash
cd search-api

# Stop running service
pkill -f search_api

# Rebuild all indexes
python3 rebuild_indexes.py --force-rebuild

# Expected output:
# ┌────────────────────────────────┐
# │ Index Rebuild                  │
# ├────────────────────────────────┤
# │ Loading articles...    450     │
# │ Building FTS5 index...  ✓      │
# │ Building HNSW index...  ✓      │
# │ Verifying indexes...    ✓      │
# │                                │
# │ FTS5 terms:       2,145        │
# │ Vector dimension:  1536        │
# │ HNSW M=16         ready        │
# │                                │
# │ Rebuild complete! ✅           │
# └────────────────────────────────┘

# Restart service
python3 app.py
```

**Verification**:
```bash
# Check service is healthy
curl http://localhost:3390/health

# Expected: {"status": "healthy"}
```

---

## Step 5: Re-Index Local SQLite KB (30 min)

**Purpose**: Update local FTS5 indexes for offline search

**Execution**:
```bash
# From the app, run a command to rebuild indexes
pnpm tauri dev

# In the app, go to Settings > Advanced > Maintenance
# Click "Rebuild Knowledge Base Index"
# Wait for completion

# Or run directly:
cd src-tauri
cargo run --bin rebuild_kb_indexes
```

**Progress**:
```
Rebuilding local KB indexes...
├─ Loading 450 articles... ✓
├─ Building FTS5 index... ✓
│  └─ 2,145 terms indexed
├─ Building vector index... ✓
│  └─ 450 vectors (1536-dim) indexed
└─ Complete! ✓

Time: 45 seconds
Local search now ready for: 450 articles
```

---

## Step 6: Validation - Run 10-Query Test (1 hour)

**Purpose**: Verify improved relevance after KB enrichment

**Execution**:
```bash
cd search-api

# Use same 10 test queries from Phase 2
python3 evaluate_relevance.py \
  --queries pilot_10_queries.json \
  --baseline phase2_validation_results.json \
  --output phase4_validation_results.json

# Expected output:
# ┌─────────────────────────────────┐
# │ KB Enrichment Validation        │
# ├─────────────────────────────────┤
# │ Phase 2 Baseline:               │
# │  Top-1: 70%                     │
# │  Top-3: 90%                     │
# │                                 │
# │ Phase 4 After Enrichment:       │
# │  Top-1: 88% ↑ (+18%)           │
# │  Top-3: 97% ↑ (+7%)            │
# │                                 │
# │ IMPROVEMENT: ✓ SUCCESS          │
# └─────────────────────────────────┘
```

**Success Criteria**:
- ✅ Top-1 relevance ≥ 85%
- ✅ Top-3 relevance ≥ 95%
- ✅ Response time < 500ms
- ✅ No timeout queries

**If Validation Fails**:
```bash
# Debug failing queries
python3 debug_query.py --query "your test query"

# Check:
# 1. Article exists and is long enough
# 2. Keywords are present in article
# 3. Vector was generated correctly
# 4. Scoring boost applied

# Possible fixes:
# - Article too sparse: expand more content
# - Wrong category: verify category in article metadata
# - Weak vector: regenerate embeddings
```

---

## Phase 4 Success Verification

**Checklist**:
- [ ] Step 1: Thin articles identified (47 articles)
- [ ] Step 2: Articles expanded to 800+ chars
- [ ] Step 2: All articles have troubleshooting sections
- [ ] Step 2: All articles committed to git
- [ ] Step 3: Title cleaning completed
- [ ] Step 4: Search API indexes rebuilt
- [ ] Step 5: Local KB indexes rebuilt
- [ ] Step 6: 10-query validation passed (85%+ top-1)
- [ ] Step 6: Validation results documented

**Phase 4 Status**: ✅ **COMPLETE** → **Proceed to Phase 5**

---

## Phase 4 → Phase 5 Transition

Once validation passes:

1. **Commit KB changes**:
   ```bash
   git add knowledge_base/articles/
   git commit -m "feat: Expand KB articles for Phase 4 enrichment"
   ```

2. **Tag KB version**:
   ```bash
   git tag -a "v1.0.0-phase4-kb-enriched" -m "Phase 4: KB Enrichment complete"
   ```

3. **Proceed to Phase 5**: MemoryKernel Integration
   - See: `PHASE5_MEMORYKERNEL_RUNBOOK.md`

---

## Troubleshooting

### Problem: Validation accuracy still low
```bash
# Check if rebuilds actually happened
cd search-api
python3 -c "from search_api import db; print(db.get_stats())"

# Should show updated article count and fresh index time

# Try manual rebuild
python3 rebuild_indexes.py --verbose --force-rebuild
```

### Problem: Articles too short to expand (less than 800 chars)
```bash
# Use content expansion guidelines
# Each article should have:
# - Overview (1-2 sentences)
# - Prerequisites (3-5 items)
# - Step-by-step (3-5 steps with details)
# - Troubleshooting (2-3 common issues)
# - FAQ (2-3 questions)
# - Related articles (2-3 links)

# Minimum total: 800 characters
```

### Problem: Index rebuild takes too long
```bash
# Parallel rebuild (if supported)
python3 rebuild_indexes.py --parallel --num-workers 4

# Or rebuild specific sections
python3 rebuild_indexes.py --categories policy security general
```

