# Phase 6: Admin + Network Ingest - Detailed Runbook

**Duration**: 3-5 days
**Prerequisites**: Phase 5 complete (MemoryKernel decision made)
**Success Criteria**: Ops hardening checks pass, rollback ready

---

## Phase 6 Overview

Harden the system for production use with admin capabilities and network knowledge base ingestion.

**Goals**:
- Configure search API for production
- Enable cross-encoder reranking for accuracy
- Set up network KB ingest with security rules
- Schedule automatic KB updates
- Execute optional Service V3 cutover (if Phase 5 = GO)
- Validate operational readiness

---

## Step 1: Deploy Search API to Production Config (1 hour)

**Purpose**: Configure search API for production environment

**File**: `search-api/config/production.yaml`

```yaml
# Production Configuration
environment: production
port: 3390
host: "0.0.0.0"

database:
  engine: postgresql
  host: "localhost"
  port: 5432
  database: "assistsupport_kb"
  pool_size: 20
  ssl: true

search:
  fts_enabled: true
  vector_enabled: true
  reranking_enabled: true
  timeout_ms: 500

rate_limiting:
  enabled: true
  requests_per_minute: 1000
  requests_per_user: 100

monitoring:
  enabled: true
  metrics_port: 9090
  log_level: "info"
  slow_query_threshold_ms: 200

cache:
  enabled: true
  ttl_seconds: 3600
  max_size_mb: 1024
```

**Apply Configuration**:
```bash
cd search-api

# Copy configuration
cp config/production.yaml config/current.yaml

# Restart with production config
python3 app.py --config config/production.yaml

# Verify
curl http://localhost:3390/health
# Expected: {"status": "healthy", "environment": "production"}
```

**Environment Variables**:
```bash
export SEARCH_API_ENV=production
export SEARCH_API_PORT=3390
export SEARCH_API_LOG_LEVEL=info
export DB_CONNECTION_STRING="postgresql://user:pass@localhost:5432/assistsupport_kb?sslmode=require"
```

---

## Step 2: Enable Cross-Encoder Reranking (1 hour)

**Purpose**: Use cross-encoder model for better ranking accuracy

**Trade-off**:
- Latency: +30-50ms per query
- Accuracy: +5-10% better top-1 relevance
- Cost: Minimal (CPU-based, no API calls)

**Configuration**:
```bash
cd search-api

# Download cross-encoder model (one-time)
python3 download_model.py --model "ms-marco-cross-encoder"

# Expected output:
# Downloading model: ms-marco-cross-encoder (45MB)
# ...
# ✓ Model ready at: models/cross-encoder/

# Enable in config
cat >> config/production.yaml << 'EOF'

cross_encoder:
  enabled: true
  model: "ms-marco-cross-encoder"
  batch_size: 32
  threshold: 0.5  # Only rerank if score > 0.5
EOF
```

**Test Reranking**:
```bash
# Compare results with and without reranking
curl http://localhost:3390/search?q=password+reset&include_scores=true

# Expected output WITH reranking:
{
  "results": [
    {
      "title": "Reset Active Directory Password",
      "score": 0.89,
      "score_before_reranking": 0.75,
      "reranked": true
    }
  ]
}
```

**Performance Verification**:
```bash
# Benchmark query latency
python3 benchmark.py --queries pilot_10_queries.json --with-reranking

# Expected output:
# Query latency:
#   Before reranking:  ~250ms
#   After reranking:   ~300ms  (added 50ms)
# Top-1 accuracy:
#   Before: 88%
#   After: 93% ↑
# Recommendation: Enable
```

---

## Step 3: Configure Network Ingest Allow Rules (1 hour)

**Purpose**: Set up trusted sources for automatic KB ingestion

**File**: `search-api/config/ingest_rules.yaml`

```yaml
network_ingest:
  enabled: true
  auto_ingest_enabled: true
  ingest_schedule: "0 2 * * *"  # Daily at 2 AM

  sources:
    - name: "Internal Docs Wiki"
      url: "https://wiki.company.internal/"
      auth: "bearer_token"
      include_patterns:
        - "^/support/"
        - "^/policies/"
      exclude_patterns:
        - "^/draft/"
        - "^/private/"
      priority: "high"
      refresh_interval_hours: 24

    - name: "Approved GitHub Repos"
      url: "https://github.com/company/"
      auth: "github_token"
      repos:
        - "company/it-runbooks"
        - "company/sops"
      include_patterns:
        - "^docs/"
        - "^README"
      priority: "medium"
      refresh_interval_hours: 48

    - name: "Vendor Documentation"
      url: "https://docs.vendor.com/"
      auth: "none"
      include_patterns:
        - "^guides/"
        - "^troubleshooting/"
      exclude_patterns:
        - "^beta/"
        - "^deprecated/"
      priority: "low"
      refresh_interval_hours: 72

  # Security Rules
  security:
    allow_private_ips: false
    allow_redirects: true
    max_redirects: 3
    ssl_verify: true
    timeout_seconds: 30

  rate_limiting:
    max_documents_per_source: 1000
    max_size_per_document_mb: 5
    max_total_size_mb: 500

  # Mark stale content
  staleness:
    enabled: true
    mark_stale_after_days: 90
    delete_stale_after_days: 365
```

**Apply Rules**:
```bash
cd search-api

# Validate configuration
python3 validate_ingest_rules.py --config config/ingest_rules.yaml

# Expected output:
# ✓ Rules validated
# ✓ 3 sources configured
# ✓ All URLs reachable
# ✓ Authentication tokens valid

# Test ingest (dry-run)
python3 ingest_from_network.py --config config/ingest_rules.yaml --dry-run

# Expected output:
# Source: Internal Docs Wiki
#   Would ingest: 47 documents
#   Total size: 12.3 MB
# Source: Approved GitHub Repos
#   Would ingest: 23 documents
#   Total size: 8.7 MB
# Source: Vendor Documentation
#   Would ingest: 15 documents
#   Total size: 4.2 MB
#
# Total: 85 documents, 25.2 MB (dry-run)

# Actually run ingest
python3 ingest_from_network.py --config config/ingest_rules.yaml

# Monitor ingestion
tail -f logs/ingest.log
```

---

## Step 4: Schedule KB Re-ingestion (30 min)

**Purpose**: Set up automatic KB updates

**Cron Schedule**:
```bash
# Edit crontab
crontab -e

# Add scheduling rules:
# 2 AM daily: Re-ingest all sources
0 2 * * * cd /home/user/AssistSupport/search-api && python3 ingest_from_network.py --config config/ingest_rules.yaml

# 3 AM daily: Mark stale articles
0 3 * * * cd /home/user/AssistSupport/search-api && python3 mark_stale_articles.py --days 90

# 3:30 AM daily: Rebuild indexes
30 3 * * * cd /home/user/AssistSupport/search-api && python3 rebuild_indexes.py

# Verify scheduling
crontab -l
```

**Systemd Alternative** (if preferred):
```bash
# Create systemd service
sudo tee /etc/systemd/system/assistsupport-ingest.service << 'EOF'
[Unit]
Description=AssistSupport KB Ingestion
After=network.target

[Service]
Type=simple
User=ubuntu
WorkingDirectory=/home/user/AssistSupport/search-api
ExecStart=python3 ingest_from_network.py --config config/ingest_rules.yaml
StandardOutput=journal
StandardError=journal
SyslogIdentifier=assistsupport-ingest
Restart=on-failure
RestartSec=300

[Install]
WantedBy=multi-user.target
EOF

# Create timer
sudo tee /etc/systemd/system/assistsupport-ingest.timer << 'EOF'
[Unit]
Description=AssistSupport KB Ingestion Timer
Requires=assistsupport-ingest.service

[Timer]
OnCalendar=daily
OnCalendar=*-*-* 02:00:00

[Install]
WantedBy=timers.target
EOF

# Enable and start
sudo systemctl enable assistsupport-ingest.timer
sudo systemctl start assistsupport-ingest.timer
sudo systemctl status assistsupport-ingest.timer
```

---

## Step 5: CONDITIONAL - Execute Service V3 Cutover (1-2 hours)

**⚠️ IMPORTANT**: Only execute if Phase 5 Step 8 decision = GO

**Prerequisite Check**:
```bash
# Verify Phase 5 decision was GO
if [ ! -f PHASE5_MEMORYKERNEL_DECISION.md ]; then
  echo "ERROR: Phase 5 decision document not found"
  exit 1
fi

if ! grep -q "DECISION: GO" PHASE5_MEMORYKERNEL_DECISION.md; then
  echo "Phase 5 Decision: NO-GO"
  echo "Skipping Step 5 (Service V3 Cutover)"
  exit 0  # Skip to Step 6
fi

echo "Phase 5 Decision: GO"
echo "Proceeding with Service V3 Cutover..."
```

**If GO: Execute Cutover**:

```bash
# Step 5.1: Update pin file
cat > src-tauri/src/integrations/memorykernel_pin.txt << 'EOF'
MemoryKernel/1.3.0-STABLE
├─ Pin: v3-production-xyz789abc...
├─ Status: AVAILABLE ✓
└─ Last verified: 2026-02-25
EOF

# Step 5.2: Update manifest
jq '.version = "1.3.0" | .status = "v3-production"' \
  src-tauri/src/integrations/memorykernel_manifest.json > /tmp/mk_manifest.json
mv /tmp/mk_manifest.json src-tauri/src/integrations/memorykernel_manifest.json

# Step 5.3: Update matrix/governance files
git fetch origin memorykernel-v3  # Fetch V3 branch if separate
git merge memorykernel-v3  # Or rebase if preferred

# Step 5.4: Run all CI gates
echo "Running CI gates..."
pnpm run check:llm-golden-set
pnpm run test:memorykernel-contract
pnpm run check:phase6-ops-hardening

# Expected output:
# ✓ Golden set GREEN
# ✓ Contract suite GREEN
# ✓ Ops hardening GREEN

# Step 5.5: Commit changes
git add src-tauri/src/integrations/memorykernel_*
git add .  # Include any governance file updates
git commit -m "feat: Upgrade to MemoryKernel V3

Service V3 cutover complete:
- Pin: v3-production-xyz789abc
- All CI gates: GREEN
- Contract suite: GREEN
- Ops hardening: GREEN

Ready for Phase 7"

# Step 5.6: Monitor V3 in production
echo "Monitoring MemoryKernel V3..."
# Continue with Step 6
```

**If NO-GO: Skip Step 5**:
```bash
echo "Phase 5 Decision: NO-GO"
echo "Service V3 cutover skipped"
echo "System continues on service.v2 (fully functional)"
echo "Proceeding to Step 6..."
```

---

## Step 6: Ops Hardening Validation (1 hour)

**Purpose**: Verify operational readiness

**Hardening Checklist**:
```bash
pnpm run check:phase6-ops-hardening

# Expected output:
# ┌──────────────────────────────────────┐
# │ Phase 6: Ops Hardening Validation    │
# ├──────────────────────────────────────┤
# │ Search API Configuration       PASS ✓ │
# │ Rate Limiting Enabled          PASS ✓ │
# │ Cross-Encoder Active           PASS ✓ │
# │ Network Ingest Rules Valid     PASS ✓ │
# │ KB Ingestion Scheduled         PASS ✓ │
# │ Error Logging Active           PASS ✓ │
# │ Metrics Enabled                PASS ✓ │
# │ Backup Procedures Verified     PASS ✓ │
# │ Rollback Ready                 PASS ✓ │
# │ Load Testing (1000 QPS)        PASS ✓ │
# │                                      │
# │ All Checks:                  GREEN ✅ │
# └──────────────────────────────────────┘
```

**Rollback Readiness Check**:
```bash
pnpm run check:rollback-readiness

# This verifies:
# - Previous version can be restored
# - Database backups exist
# - Configuration rollback possible
# - Fallback mechanisms functional

# Expected output:
# ┌──────────────────────────────────────┐
# │ Rollback Readiness                   │
# ├──────────────────────────────────────┤
# │ Previous Build Available       PASS ✓ │
# │ Database Backups Exist         PASS ✓ │
# │ Fallback Config Available      PASS ✓ │
# │ Service V2 Still Available     PASS ✓ │
# │ Rollback Time Est: 15 min      GOOD   │
# │                                      │
# │ Ready for Production         READY ✅ │
# └──────────────────────────────────────┘
```

---

## Phase 6 Success Verification

**Checklist**:
- [ ] Step 1: Search API production config deployed
- [ ] Step 1: Port 3390 verified
- [ ] Step 2: Cross-encoder model downloaded and enabled
- [ ] Step 2: Reranking adds < 50ms latency
- [ ] Step 2: Top-1 accuracy improved
- [ ] Step 3: Network ingest rules configured
- [ ] Step 3: All sources reachable and authenticated
- [ ] Step 4: KB re-ingestion scheduled (daily 2 AM)
- [ ] Step 4: Cron/systemd service verified
- [ ] Step 5: Phase 5 decision recorded (GO or NO-GO)
- [ ] Step 5: If GO → Service V3 cutover executed
- [ ] Step 5: If NO-GO → Step 5 skipped, system on v2
- [ ] Step 6: Ops hardening checks GREEN
- [ ] Step 6: Rollback readiness confirmed

**Phase 6 Status**: ✅ **COMPLETE** → **Proceed to Phase 7**

---

## Phase 6 → Phase 7 Transition

Once Phase 6 is complete:

1. **Commit hardening results**:
   ```bash
   git add .
   git commit -m "docs: Phase 6 ops hardening complete

   - Search API production config deployed
   - Cross-encoder reranking enabled (+7-10% accuracy)
   - Network ingest configured (daily 2 AM)
   - Service V3 cutover: [GO/NO-GO as decided]
   - Ops hardening checks: GREEN
   - Rollback ready

   https://claude.ai/code/session_01DEzVFdgCbvURmuN9rv9Vbi"
   ```

2. **Tag Phase 6**:
   ```bash
   git tag -a "v1.0.0-phase6-ops-hardened" -m "Phase 6: Ops hardening complete"
   ```

3. **Proceed to Phase 7**: Monorepo Migration
   - See: `PHASE7_MONOREPO_MIGRATION_RUNBOOK.md`

---

## Troubleshooting

### Problem: Search API won't start with production config
```bash
# Check PostgreSQL is accessible
psql -c "SELECT version();"

# Verify connection string
echo $DB_CONNECTION_STRING

# Check config syntax
python3 -c "import yaml; yaml.safe_load(open('config/production.yaml'))"

# Start with verbose logging
python3 app.py --config config/production.yaml --log-level debug
```

### Problem: Reranking making results worse
```bash
# Disable reranking temporarily
sed -i 's/enabled: true/enabled: false/' config/production.yaml
python3 app.py

# Check if it's a model issue
python3 -c "from cross_encoder import load; model = load('ms-marco-cross-encoder')"

# Try different threshold
# Lower threshold = more aggressive reranking
# Edit: config/production.yaml -> cross_encoder.threshold
```

### Problem: Network ingest failing for a source
```bash
# Check source directly
curl -i https://wiki.company.internal/ --connect-timeout 10

# Verify auth token
echo $WIKI_AUTH_TOKEN

# Run ingest with verbose output
python3 ingest_from_network.py --verbose --source "Internal Docs Wiki"

# Check logs
tail -f logs/ingest.log | grep "Internal Docs Wiki"
```

