# Phase 5: MemoryKernel Integration - Detailed Runbook

**Duration**: 1 week
**Prerequisites**: Phase 4 complete (KB enriched to 85%+ accuracy)
**Success Criteria**: Contract suite passes, all fallbacks work, decision recorded

---

## Phase 5 Overview

Integrate the MemoryKernel service for context-aware response enrichment. Focus on:
- Verifying service availability
- Building local integration layer
- Testing enrichment in practice
- Validating all fallback scenarios
- Making GO/NO-GO decision for Phase 6

---

## Step 1: Verify Service Availability (30 min)

**Purpose**: Ensure MemoryKernel service is available and pinned correctly

**Files to Check**:
- `src-tauri/src/integrations/memorykernel_pin.txt` - Service pin
- `src-tauri/src/integrations/memorykernel_manifest.json` - Service manifest

**Verification Steps**:

```bash
# Step 1: Check pin file exists and is valid
cat src-tauri/src/integrations/memorykernel_pin.txt

# Expected format:
# MemoryKernel/1.2.3-STABLE
# ├─ Pin: abc123def456...
# ├─ Status: AVAILABLE ✓
# └─ Last verified: 2026-02-12

# Step 2: Verify manifest is current
cat src-tauri/src/integrations/memorykernel_manifest.json | jq .

# Expected:
# {
#   "service": "MemoryKernel",
#   "version": "1.2.3",
#   "status": "stable",
#   "endpoint": "http://localhost:9999",
#   "auth_required": true,
#   "timeout_ms": 100
# }

# Step 3: Network test
curl -s http://localhost:9999/health | jq .

# Expected if running locally:
# {"status": "healthy", "version": "1.2.3"}

# Or if remote:
# Connection refused or timeout (expected, will start in Step 2)
```

**Success Criteria**:
- ✅ Pin file exists and is valid
- ✅ Manifest is current (matches latest service)
- ✅ Service info is accurate

---

## Step 2: Build and Start MemoryKernel Service Locally (1 hour)

**Purpose**: Set up MemoryKernel for local integration testing

**Prerequisites**:
- Docker installed
- 2GB RAM available
- Port 9999 free

**Build from Source**:
```bash
# Clone MemoryKernel repo
git clone https://github.com/saagar210/MemoryKernel.git
cd MemoryKernel

# Check out stable version
git checkout v1.2.3

# Build Docker image
docker build -t memorykernel:1.2.3 .

# Expected output:
# ...
# Successfully built memorykernel:1.2.3
# Successfully tagged memorykernel:1.2.3
```

**Start Service**:
```bash
# Start MemoryKernel in foreground
docker run \
  --name memorykernel \
  -p 9999:9999 \
  -e MEMORYKERNEL_MODE=local-integration \
  -e MEMORYKERNEL_LOG_LEVEL=info \
  memorykernel:1.2.3

# Expected startup output:
# 2026-02-20 14:22:10 [INFO] MemoryKernel Service Starting
# 2026-02-20 14:22:11 [INFO] Loading knowledge graphs...
# 2026-02-20 14:22:12 [INFO] Starting HTTP server on :9999
# 2026-02-20 14:22:13 [INFO] Service ready (pid 1)
```

**Verify Running**:
```bash
# In another terminal
curl http://localhost:9999/health

# Expected:
# {"status": "healthy", "version": "1.2.3"}
```

---

## Step 3: Set Auth Token via Secure Storage (30 min)

**Purpose**: Configure authentication for service communication

**Generate Auth Token**:
```bash
# On MemoryKernel service
docker exec memorykernel /usr/local/bin/mk-token-gen

# Expected output:
# Generated token: mk_token_xxxxxx.yyyyyy.zzzzzz
# Expires: 2026-05-20
# Copy this token and keep it safe!
```

**Store in AssistSupport**:
```bash
# Set token in secure storage
# On macOS:
security add-generic-password \
  -a "AssistSupport" \
  -s "MemoryKernelAuthToken" \
  -w "mk_token_xxxxxx.yyyyyy.zzzzzz"

# Verify storage
security find-generic-password -a "AssistSupport" -s "MemoryKernelAuthToken"

# Expected:
# keychain: ...MemoryKernelAuthToken
# password: "mk_token_xxxxxx.yyyyyy.zzzzzz"
```

**Use in Code**:
```rust
// File: src-tauri/src/integrations/memorykernel.rs

pub async fn get_auth_token() -> Result<String, String> {
    #[cfg(target_os = "macos")]
    {
        use security_framework::passwords;
        let token = passwords::get_password(
            passwords::SecClass::GenericPassword,
            "AssistSupport",
            "MemoryKernelAuthToken",
        ).map_err(|e| e.to_string())?;
        Ok(token)
    }

    #[cfg(not(target_os = "macos"))]
    {
        // Fallback to environment variable
        std::env::var("MEMORYKERNEL_AUTH_TOKEN")
            .map_err(|_| "MemoryKernel auth token not configured".to_string())
    }
}
```

---

## Step 4: Enable Feature Flag (15 min)

**Purpose**: Activate MemoryKernel integration in the app

**File**: `src-tauri/src/lib.rs`

```rust
// Add to AppState initialization
pub struct AppState {
    pub db: Mutex<Option<Database>>,
    pub llm: Arc<RwLock<Option<LlmEngine>>>,
    pub embeddings: Arc<RwLock<Option<EmbeddingEngine>>>,
    pub vectors: Arc<TokioRwLock<Option<VectorStore>>>,
    pub jobs: Arc<JobManager>,

    // NEW:
    pub memorykernel_enabled: Arc<AtomicBool>,
}

// Initialize with feature flag
pub async fn init_app_state() -> Result<AppState, String> {
    let memorykernel_enabled = Arc::new(AtomicBool::new(
        std::env::var("ASSISTSUPPORT_ENABLE_MEMORY_KERNEL")
            .map(|v| v.to_lowercase() == "true")
            .unwrap_or(false)
    ));

    if memorykernel_enabled.load(std::sync::atomic::Ordering::Relaxed) {
        println!("✓ MemoryKernel integration enabled");
    } else {
        println!("ℹ MemoryKernel integration disabled");
    }

    Ok(AppState {
        // ... other fields
        memorykernel_enabled,
    })
}
```

**Enable via Environment**:
```bash
# Before launching app
export ASSISTSUPPORT_ENABLE_MEMORY_KERNEL=true
pnpm tauri dev

# Or for testing
ASSISTSUPPORT_ENABLE_MEMORY_KERNEL=true pnpm tauri dev
```

---

## Step 5: Test Enrichment End-to-End (1 hour)

**Purpose**: Verify enrichment works in actual response generation

**Manual Test**:
```bash
# 1. Launch app with MemoryKernel enabled
ASSISTSUPPORT_ENABLE_MEMORY_KERNEL=true pnpm tauri dev

# 2. In the Draft tab, ask a question
#    Query: "How do I reset my password?"

# 3. In browser DevTools, check response
#    Should include in response metadata:
{
  "response": "...",
  "memorykernel_enrichment": {
    "applied": true,
    "context_added": "Previous password reset attempts",
    "confidence": 0.92,
    "sources": ["kb_article_123", "user_session_456"]
  }
}
```

**Automated Test**:
```bash
# Run integration test
pnpm run test:memorykernel-integration

# Expected output:
# ✓ MemoryKernel connection established
# ✓ Auth token valid
# ✓ Query enrichment works
# ✓ Response includes enrichment metadata
# ✓ Latency < 100ms
# Test suite: 8/8 PASS
```

---

## Step 6: Verify All 4 Fallback Scenarios (2 hours)

**Purpose**: Ensure system gracefully handles all failure cases

### Fallback Scenario 1: Service Offline

```bash
# Stop MemoryKernel
docker stop memorykernel

# Query the app
# Expected response metadata:
{
  "response": "...",
  "memorykernel": {
    "fallback_reason": "offline",
    "fallback_applied": true,
    "recovery_method": "use_kb_without_enrichment"
  }
}

# Verify: Response is still generated (no crash)
# Result: PASS ✓
```

### Fallback Scenario 2: Wrong Auth Token

```bash
# Change token to invalid value
security add-generic-password \
  -a "AssistSupport" \
  -s "MemoryKernelAuthToken" \
  -w "invalid_token_12345" \
  -U  # Update existing

# Restart app
# Query the app
# Expected response metadata:
{
  "response": "...",
  "memorykernel": {
    "fallback_reason": "auth-required",
    "fallback_applied": true,
    "recovery_method": "request_new_auth"
  }
}

# Verify: App suggests re-authenticating
# Result: PASS ✓
```

### Fallback Scenario 3: Timeout (100ms exceeded)

```bash
# Start MemoryKernel with slow responses
docker stop memorykernel
docker run \
  --name memorykernel \
  -p 9999:9999 \
  -e MEMORYKERNEL_MODE=local-integration \
  -e MEMORYKERNEL_RESPONSE_DELAY_MS=200 \  # Slow mode
  memorykernel:1.2.3

# Query the app (with 100ms timeout)
# Expected response metadata:
{
  "response": "...",
  "memorykernel": {
    "fallback_reason": "timeout",
    "fallback_applied": true,
    "recovery_method": "use_cached_context"
  }
}

# Verify: No hang, response within 500ms total
# Result: PASS ✓
```

### Fallback Scenario 4: Feature Disabled

```bash
# Disable feature flag
export ASSISTSUPPORT_ENABLE_MEMORY_KERNEL=false

# Restart app
# Query the app
# Expected response metadata:
{
  "response": "...",
  "memorykernel": {
    "fallback_reason": "feature-disabled",
    "fallback_applied": false,
    "recovery_method": "none_required"
  }
}

# Verify: System works perfectly without enrichment
# Result: PASS ✓
```

**Fallback Verification Script**:
```bash
pnpm run test:memorykernel-fallbacks

# Expected output:
# ┌──────────────────────────────────┐
# │ MemoryKernel Fallback Tests      │
# ├──────────────────────────────────┤
# │ 1. Service Offline        PASS ✓ │
# │ 2. Auth Failure          PASS ✓ │
# │ 3. Timeout (100ms)       PASS ✓ │
# │ 4. Feature Disabled      PASS ✓ │
# │                                  │
# │ All Fallbacks Working    PASS ✓ │
# └──────────────────────────────────┘
```

---

## Step 7: Run Contract Suite + Governance Tests (1 hour)

**Purpose**: Validate service integration meets requirements

**Contract Tests**:
```bash
pnpm run test:memorykernel-contract

# Expected output:
# ┌──────────────────────────────────────┐
# │ MemoryKernel Contract Suite          │
# ├──────────────────────────────────────┤
# │ Response Format Compliance    PASS ✓ │
# │ Timeout Enforcement (100ms)   PASS ✓ │
# │ Fallback Behavior             PASS ✓ │
# │ Auth Token Handling           PASS ✓ │
# │ Error Responses               PASS ✓ │
# │ Enrichment Accuracy           PASS ✓ │
# │                                      │
# │ Contract Status:         GREEN ✅   │
# └──────────────────────────────────────┘

# 6/6 tests passing
```

**Governance Tests**:
```bash
pnpm run test:memorykernel-governance

# Expected output:
# ┌──────────────────────────────────────┐
# │ MemoryKernel Governance              │
# ├──────────────────────────────────────┤
# │ Pin/Manifest Alignment       PASS ✓ │
# │ Service Matrix Compliance    PASS ✓ │
# │ Handoff Readiness            PASS ✓ │
# │ Boundary Enforcement         PASS ✓ │
# │ Negative Drift               PASS ✓ │
# │                                      │
# │ Governance Status:       GREEN ✅   │
# └──────────────────────────────────────┘

# 5/5 tests passing
```

---

## Step 8: Make Service V3 Cutover Decision (GO/NO-GO) (1 hour)

**Purpose**: Decide whether to upgrade to MemoryKernel V3 in Phase 6

**Decision Criteria**:

| Criterion | Target | Status | Weight |
|-----------|--------|--------|--------|
| Contract suite | GREEN | ✅ | 30% |
| All 4 fallbacks | PASS | ✅ | 30% |
| Integration tests | GREEN | ✅ | 20% |
| Service stability | 99%+ uptime | ✅ | 20% |

**Decision Document**: Create `PHASE5_MEMORYKERNEL_DECISION.md`

```markdown
# Phase 5 MemoryKernel Integration - GO/NO-GO Decision

## Summary
Completed all 7 steps of MemoryKernel integration. Service is stable and contract tests pass.

## Results

### Integration Testing
- ✅ Service available and responding
- ✅ Authentication working
- ✅ Enrichment applied to responses
- ✅ All 4 fallback scenarios working
- ✅ Contract suite GREEN (6/6 tests)
- ✅ Governance suite GREEN (5/5 tests)

### Performance
- ✅ Enrichment adds < 50ms latency
- ✅ Fallback response < 20ms
- ✅ No response timeouts
- ✅ Service uptime: 99.9%

### Recommendation

**DECISION: GO** ✅

Rationale:
- All acceptance criteria met
- Service is production-ready
- Fallbacks provide safety net
- Ready to upgrade to V3 in Phase 6

## Next Steps
1. Proceed with Phase 6 (Admin + Network Ingest)
2. At Phase 6 Step 5 (Step 42), execute Service V3 cutover
3. Monitor V3 service in production for 1 week
4. Proceed to Phase 7 once stable

---

**Alternative (if NO-GO):**
If any test failed or service unstable, decision would be NO-GO:
- System continues on service.v2 (fully functional)
- No Phase 6 Step 5 execution
- Still proceed to Phase 7 (monorepo migration)
- V3 upgrade attempted in future release
```

**Recording Decision**:
```bash
# Save decision to git
git add PHASE5_MEMORYKERNEL_DECISION.md
git commit -m "docs: Record Phase 5 MemoryKernel decision: GO

Service integration complete and stable.
- Contract suite: GREEN (6/6)
- Governance suite: GREEN (5/5)
- All fallbacks: PASS
- Ready for Phase 6 cutover

https://claude.ai/code/session_01DEzVFdgCbvURmuN9rv9Vbi"
```

---

## Phase 5 Success Verification

**Checklist**:
- [ ] Step 1: Service availability verified
- [ ] Step 2: MemoryKernel service running locally
- [ ] Step 3: Auth token configured and secured
- [ ] Step 4: Feature flag enabled
- [ ] Step 5: End-to-end enrichment test passed
- [ ] Step 6: All 4 fallback scenarios verified
- [ ] Step 7: Contract suite GREEN (6/6)
- [ ] Step 7: Governance suite GREEN (5/5)
- [ ] Step 8: GO/NO-GO decision made and documented

**Phase 5 Status**: ✅ **COMPLETE** → **Proceed to Phase 6**

---

## Phase 5 → Phase 6 Transition

Once Phase 5 is complete:

1. **Archive MemoryKernel decision**:
   ```bash
   git tag -a "v1.0.0-phase5-mk-integration" -m "Phase 5: MemoryKernel integration"
   ```

2. **Evaluate GO/NO-GO decision**:
   - If GO: Proceed to Phase 6 (including optional Step 5)
   - If NO-GO: Proceed to Phase 6 (skip Step 5)

3. **Proceed to Phase 6**: Admin + Network Ingest
   - See: `PHASE6_ADMIN_NETWORK_RUNBOOK.md`

---

## Troubleshooting

### Problem: MemoryKernel service won't start
```bash
# Check Docker
docker ps
docker logs memorykernel

# Common fixes
docker restart memorykernel
docker system prune -a  # If corrupted
docker build --no-cache -t memorykernel:1.2.3 .
```

### Problem: Auth token authentication fails
```bash
# Regenerate token
docker exec memorykernel /usr/local/bin/mk-token-gen

# Update storage
security add-generic-password \
  -a "AssistSupport" \
  -s "MemoryKernelAuthToken" \
  -w "mk_token_new..." \
  -U
```

### Problem: Enrichment not showing in responses
```bash
# Check feature flag enabled
echo $ASSISTSUPPORT_ENABLE_MEMORY_KERNEL
# Should be: true

# Check service is reachable
curl http://localhost:9999/health

# Check logs
pnpm tauri dev  # Look for MemoryKernel connection logs
```

