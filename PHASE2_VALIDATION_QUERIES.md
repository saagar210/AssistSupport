# Phase 2: Validation Query Set

**Purpose**: 10-query validation set to measure search relevance before pilot launch

**Target**: ≥ 70% top-1 accuracy, ≥ 90% top-3 accuracy

---

## Validation Queries

### Query 1: Password Reset
```
User Query: "How do I reset my Active Directory password?"
Expected Top Result: active-directory/reset-password.md
Category: policy
Relevance Explanation: Direct match for password reset process
```

### Query 2: MFA Setup
```
User Query: "enable multi-factor authentication on my account"
Expected Top Result: security/setup-mfa.md
Category: security
Relevance Explanation: MFA setup is exactly what's requested
```

### Query 3: Printer Troubleshooting
```
User Query: "My printer won't connect to the network. How do I fix it?"
Expected Top Result: hardware/printer-network-troubleshooting.md
Category: troubleshooting
Relevance Explanation: Network printer connectivity issues
```

### Query 4: VPN Configuration
```
User Query: "How to set up VPN for remote work?"
Expected Top Result: network/vpn-setup-guide.md
Category: policy
Relevance Explanation: VPN setup instructions needed
```

### Query 5: Email Forwarding
```
User Query: "Can I set up email forwarding to another account?"
Expected Top Result: email/forwarding-setup.md
Category: general
Relevance Explanation: Email forwarding feature request
```

### Query 6: License Renewal
```
User Query: "How do I renew my software license?"
Expected Top Result: software/license-renewal-process.md
Category: policy
Relevance Explanation: License renewal process
```

### Query 7: Backup & Recovery
```
User Query: "I need to back up my data. What's the procedure?"
Expected Top Result: backup/backup-recovery-procedure.md
Category: policy
Relevance Explanation: Data backup procedure
```

### Query 8: Performance Optimization
```
User Query: "My computer is running slowly. What can I do?"
Expected Top Result: troubleshooting/performance-optimization.md
Category: troubleshooting
Relevance Explanation: Performance troubleshooting
```

### Query 9: Access Control
```
User Query: "How do I request access to a shared folder?"
Expected Top Result: access-control/request-shared-access.md
Category: policy
Relevance Explanation: Access request process
```

### Query 10: Account Lockout
```
User Query: "My account is locked. What should I do?"
Expected Top Result: security/account-lockout-recovery.md
Category: security
Relevance Explanation: Account lockout recovery procedures
```

---

## JSON Format for Evaluation

```json
{
  "validation_queries": [
    {
      "id": 1,
      "query": "How do I reset my Active Directory password?",
      "expected_top_result": "active-directory/reset-password.md",
      "category": "policy",
      "difficulty": "easy"
    },
    {
      "id": 2,
      "query": "enable multi-factor authentication on my account",
      "expected_top_result": "security/setup-mfa.md",
      "category": "security",
      "difficulty": "easy"
    },
    {
      "id": 3,
      "query": "My printer won't connect to the network. How do I fix it?",
      "expected_top_result": "hardware/printer-network-troubleshooting.md",
      "category": "troubleshooting",
      "difficulty": "medium"
    },
    {
      "id": 4,
      "query": "How to set up VPN for remote work?",
      "expected_top_result": "network/vpn-setup-guide.md",
      "category": "policy",
      "difficulty": "medium"
    },
    {
      "id": 5,
      "query": "Can I set up email forwarding to another account?",
      "expected_top_result": "email/forwarding-setup.md",
      "category": "general",
      "difficulty": "easy"
    },
    {
      "id": 6,
      "query": "How do I renew my software license?",
      "expected_top_result": "software/license-renewal-process.md",
      "category": "policy",
      "difficulty": "medium"
    },
    {
      "id": 7,
      "query": "I need to back up my data. What's the procedure?",
      "expected_top_result": "backup/backup-recovery-procedure.md",
      "category": "policy",
      "difficulty": "easy"
    },
    {
      "id": 8,
      "query": "My computer is running slowly. What can I do?",
      "expected_top_result": "troubleshooting/performance-optimization.md",
      "category": "troubleshooting",
      "difficulty": "hard"
    },
    {
      "id": 9,
      "query": "How do I request access to a shared folder?",
      "expected_top_result": "access-control/request-shared-access.md",
      "category": "policy",
      "difficulty": "easy"
    },
    {
      "id": 10,
      "query": "My account is locked. What should I do?",
      "expected_top_result": "security/account-lockout-recovery.md",
      "category": "security",
      "difficulty": "easy"
    }
  ]
}
```

---

## Evaluation Criteria

**Top-1 Accuracy**: First result is relevant
- **Easy queries**: Should hit 90%+
- **Medium queries**: Should hit 70-80%
- **Hard queries**: Should hit 50%+

**Top-3 Accuracy**: One of top 3 results is relevant
- **All queries**: Should hit 90%+

**Response Time**: < 500ms
- API response: < 200ms
- Total with LLM: < 500ms

---

## Running Validation

```bash
cd search-api

# Save queries to file
cat > pilot_10_queries.json << 'EOF'
[...validation_queries...]
EOF

# Run evaluation
python3 evaluate_relevance.py \
  --queries pilot_10_queries.json \
  --output validation_results.json

# View results
cat validation_results.json
```

---

## Expected Output

```
Validation Results:
═══════════════════════════════════════════════

Query 1: "How do I reset my Active Directory password?"
  Top 1: active-directory/reset-password.md (MATCH ✓)
  Top 3: [all relevant]
  Score: 1.0

Query 2: "enable multi-factor authentication on my account"
  Top 1: security/setup-mfa.md (MATCH ✓)
  Top 3: [all relevant]
  Score: 1.0

...

═══════════════════════════════════════════════
SUMMARY:
  Top-1 Accuracy: 7/10 = 70% ✓
  Top-3 Accuracy: 10/10 = 100% ✓
  Avg Response Time: 312ms ✓
  Timeout Queries: 0 ✓

RECOMMENDATION: PASS - Ready for pilot
═══════════════════════════════════════════════
```

---

## If Validation Fails

If accuracy is below 70%:

1. **Debug failing queries**:
   ```bash
   python3 debug_query.py --query "Your test query"
   ```

2. **Check KB articles exist**:
   ```bash
   ls knowledge_base/articles/ | grep -i "keyword"
   ```

3. **Verify indexes are built**:
   ```bash
   python3 -c "from search_api import db; db.verify_indexes()"
   ```

4. **Check category boosting**:
   ```bash
   python3 -c "from search_api import boosting; print(boosting.config)"
   ```

5. **Re-ingest if needed**:
   ```bash
   python3 ingest_curated_kb.py --force-rebuild
   ```

6. **Re-run validation**:
   ```bash
   python3 evaluate_relevance.py --queries pilot_10_queries.json
   ```

