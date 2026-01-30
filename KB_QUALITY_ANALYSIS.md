# Knowledge Base Quality Analysis

## Date: January 29, 2026

---

## Executive Summary

The AssistSupport system has **two separate knowledge bases** that are not synchronized:

1. **Local KB** (`knowledge_base/` directory): 27 curated, high-quality markdown files covering policies, procedures, and references. These are ingested by the Tauri app into SQLite/LanceDB.

2. **PostgreSQL KB** (Flask API): 3,536 articles bulk-imported from a Confluence/wiki export. These are searched by the Flask hybrid search API on port 3000.

**The curated articles were never loaded into PostgreSQL.** This is the primary cause of poor response quality from the hybrid search API.

---

## PostgreSQL KB Analysis (3,536 articles)

### Article Length Distribution

| Length Range | Count | Percentage | Quality Level |
|---|---|---|---|
| Under 100 chars | 293 | 8.3% | Unusable |
| 100-499 chars | 1,617 | 45.7% | Very Low |
| 500-999 chars | 711 | 20.1% | Low-Medium |
| 1,000-1,999 chars | 522 | 14.8% | Medium |
| 2,000-4,999 chars | 389 | 11.0% | Good |
| 5,000+ chars | 4 | 0.1% | Excellent |

**Key Statistics:**
- Average: 753 chars
- Median: 445 chars
- P25: 203 chars (25th percentile)
- P75: 1,037 chars (75th percentile)
- Min: 50 chars
- Max: 17,849 chars

**54% of all articles are under 500 characters.** This is the core quality problem.

### Category Distribution

| Category | Count | Percentage | Avg Content Length |
|---|---|---|---|
| REFERENCE | 2,134 | 60.4% | 758 chars |
| PROCEDURE | 1,300 | 36.8% | 758 chars |
| POLICY | 102 | 2.9% | 579 chars |

**POLICY articles are severely underrepresented at only 2.9%.** For an IT support system where users frequently ask permission-related questions ("Can I...?", "Am I allowed to...?"), this is a critical gap.

### Embedding Coverage

| Metric | Value |
|---|---|
| Total articles | 3,536 |
| With embeddings | 3,536 |
| Missing embeddings | 0 |
| Coverage | 100.00% |

Embeddings are complete — the problem is content quality, not embedding coverage.

### Title Quality

| Metric | Value |
|---|---|
| Average title length | 76.4 chars |
| Titles under 10 chars | 12 |
| Titles over 100 chars | 829 (23.4%) |

**829 titles are over 100 characters** because they contain Confluence breadcrumb paths:
- `"OS X > ASD tests > Installing"`
- `"How to Complete Cost Center Change Tickets > How to Complete"`
- `"Event Support Runbook (EUS) > Overview > A/V Policy > 3. Scope > 6.4 USB/Device Trust/Other"`

These breadcrumb titles hurt BM25 keyword matching — they contain noise words that match irrelevant queries.

### Sample Very Short Articles (Under 100 chars)

These articles are essentially unusable:

| Title | Length | Content |
|---|---|---|
| How to Complete Cost Center Change Tickets | 50 | `IT-87634 Okta - Cost Center Changes Complete )` |
| ServiceDesk Access Approvals Ticket Workflow | 50 | `Table of Contents: - New Approvals Process in JSD` |
| Events > Tech Tests | 50 | `Interested in a video walkthrough? Check out this!` |
| Deactivation of Slack Customization | 51 | `Screenshot 2023-07-21 at 8.43.52 AM.png (image/png)` |
| EMEA EUS Knowledge base | 51 | `TV Firmware Updates how to update TV firmware` |

These are Confluence fragment artifacts — ticket numbers, image references, table-of-contents stubs. They should be removed entirely.

---

## Local KB Analysis (27 files)

### File Distribution

| Category | Count | Avg Size |
|---|---|---|
| POLICIES | 15 files | ~950 chars |
| PROCEDURES | 8 files | ~1,050 chars |
| REFERENCE | 4 files | ~1,050 chars |

### Quality Assessment

All 27 files are **high quality**:

- Well-structured with markdown headers and lists
- Actionable content with clear instructions
- Specific details (e.g., "12-character minimum password")
- Escalation paths and contact information
- Troubleshooting steps where relevant

**Standout example**: `flash_drives_forbidden.md` (2,286 chars)
- Clear policy statement
- 7 prohibited USB device types listed
- 5 reasons why
- 5 approved alternatives
- Explicit "NO EXCEPTIONS" statement
- Consequences listed

### Missing From PostgreSQL

These critical curated articles are NOT in the PostgreSQL database:

| File | Category | Size | Status |
|---|---|---|---|
| flash_drives_forbidden.md | POLICY | 2,286 | NOT IN DB |
| removable_media_policy.md | POLICY | 1,495 | NOT IN DB |
| password_policy.md | POLICY | 898 | NOT IN DB |
| vpn_requirements.md | POLICY | 1,132 | NOT IN DB |
| email_security_policy.md | POLICY | 821 | NOT IN DB |
| cloud_storage_policy.md | POLICY | 1,218 | NOT IN DB |
| software_installation_policy.md | POLICY | 881 | NOT IN DB |
| acceptable_use_policy.md | POLICY | 1,013 | NOT IN DB |
| byod_policy.md | POLICY | 859 | NOT IN DB |
| data_classification.md | POLICY | 868 | NOT IN DB |
| incident_reporting_policy.md | POLICY | 1,034 | NOT IN DB |
| remote_access_policy.md | POLICY | 756 | NOT IN DB |
| vpn_setup.md | PROCEDURE | 1,168 | NOT IN DB |
| password_reset.md | PROCEDURE | 781 | NOT IN DB |
| data_backup.md | PROCEDURE | 1,013 | NOT IN DB |
| file_sharing_guide.md | PROCEDURE | 1,212 | NOT IN DB |
| software_request.md | PROCEDURE | 876 | NOT IN DB |
| access_request.md | PROCEDURE | 991 | NOT IN DB |
| report_technical_issue.md | PROCEDURE | 1,260 | NOT IN DB |
| request_new_laptop.md | PROCEDURE | 1,445 | NOT IN DB |
| supported_software.md | REFERENCE | 777 | NOT IN DB |
| contact_directory.md | REFERENCE | 879 | NOT IN DB |
| approved_devices.md | REFERENCE | 938 | NOT IN DB |
| network_architecture.md | REFERENCE | 1,017 | NOT IN DB |
| compliance_overview.md | REFERENCE | 1,247 | NOT IN DB |
| common_error_codes.md | REFERENCE | 1,518 | NOT IN DB |

---

## Search Quality Test Results

10 queries tested against Flask API (PostgreSQL):

| Query | Intent Detected | Top Result | Relevant? |
|---|---|---|---|
| Can I use a flash drive? | policy (0.6) | Laptop Provisioning | NO |
| How do I reset my password? | procedure (0.6) | Password Reset for Mac and Windows | YES |
| What is the remote work policy? | policy (0.4) | Remote employee home office equipment | PARTIAL |
| VPN setup instructions | procedure (0.6) | How to Clock In on Workday | NO |
| Cloud storage allowed | unknown | mxHero & Email at Box | WEAK |
| Laptop approval process | procedure (0.8) | Processing Laptop Refresh | YES |
| Email security guidelines | unknown | Corp Email Gmail Allowlist | WEAK |
| Data backup requirements | unknown | High Risk Travel Remote Locks | NO |
| Software license approval | unknown | App Support Index Z | NO |
| USB device policy | policy (0.6) | Duo Enforcement Policy | NO |

**Relevance rate: 2/10 fully relevant, 2/10 partial, 6/10 wrong.**

The queries that work ("password reset", "laptop approval") succeed because the Confluence export happens to contain decent articles on those topics. The queries that fail ("flash drive", "VPN setup", "data backup") fail because the relevant content doesn't exist in PostgreSQL.

---

## Topic Coverage Gaps

Queries were tested against the database for topic coverage:

| Topic | Articles Found | Quality |
|---|---|---|
| Flash drive / USB policy | 10 | WRONG — AV hardware, endpoint mgmt, not policy |
| VPN | 10 | MIXED — Some relevant (GlobalProtect), some irrelevant |
| Password reset | 10 | GOOD — Multiple relevant articles exist |
| Cloud storage | ~5 | WEAK — Mostly about Box/AWS, not policy |
| Data backup | ~5 | WEAK — Disaster recovery, not user-facing |
| Email security | ~5 | WEAK — Gmail config, not security policy |

---

## Recommendations

### Immediate (Hours 1-2): Ingest Curated KB

Load all 27 curated knowledge_base/ articles into PostgreSQL with proper embeddings. This alone will fix the top root cause.

### Short-term (Hours 3-4): Quality Filter

1. Remove or archive articles under 100 characters (293 articles)
2. Flag articles under 300 characters for review (estimated ~1,000)
3. Clean breadcrumb titles — extract the final segment
4. Re-index after cleanup

### Medium-term (Hours 5-8): Content Enrichment

1. Identify top 50 most-searched topics from query_performance
2. Find and expand thin articles on those topics
3. Add missing topics identified in coverage gaps
4. Regenerate embeddings for modified articles

### Long-term: Continuous Quality

1. Set minimum content length threshold for new articles (300+ chars)
2. Automated quality scoring for ingested content
3. Feedback-driven article improvement
4. Regular coverage gap analysis from query logs
