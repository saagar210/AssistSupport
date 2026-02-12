# Phase 1: Production Deployment - macOS Execution Runbook

## Verified Pre-Conditions (Linux Dev Machine)

✅ **Frontend Validation**
- TypeScript strict mode: 0 errors
- Unit tests: 129 tests PASS
- Dependencies: All 40+ npm packages installed

## Phase 1 Execution Steps (macOS Production Machine)

### Prerequisites
- macOS machine (production target)
- Xcode Command Line Tools installed
- Rust toolchain (via rustup)
- Node.js 22+ with pnpm 10.29.2+
- Code signing certificates available (for `TAURI_SIGNING_PRIVATE_KEY`)

---

### Step 1: Run Full CI Validation Suite

```bash
cd /home/user/AssistSupport

# TypeScript check
pnpm run typecheck
# Expected: No output (success)

# Frontend tests  
pnpm test -- --run
# Expected: 129 tests PASS (22 files)

# Backend tests (macOS-only, requires keyring + Vision frameworks)
cd src-tauri
cargo test --lib
# Expected: 271+ tests PASS

# Code quality checks
cargo clippy -- -D warnings
# Expected: No warnings

# Code formatting verification
cargo fmt --check
# Expected: No diffs

# Security audit
cargo audit
# Expected: No high/critical vulnerabilities
```

**Exit Criteria**: All commands exit with code 0, no errors

---

### Step 2: Run Security Audit and Regression Tests

```bash
cd /home/user/AssistSupport

# npm security
pnpm audit --audit-level high
# Expected: No high/critical vulnerabilities

# Rust security audit
cd src-tauri
cargo audit
# Expected: No advisories

# Security regression tests
pnpm run test:security-regression
# Expected: All SSRF, injection, path traversal tests PASS
```

**Exit Criteria**: No vulnerabilities, all regression tests pass

---

### Step 3: Run MemoryKernel Contract Suite

```bash
cd /home/user/AssistSupport

pnpm run test:memorykernel-contract
# Expected: Contract validation PASS

pnpm run check:memorykernel-governance
# Expected: Pin/manifest/matrix PASS

pnpm run check:rollback-readiness
# Expected: Rollback plan valid
```

**Exit Criteria**: All governance checks pass, fallback validated

---

### Step 4: Execute Production Build

```bash
cd /home/user/AssistSupport

# Verify version in tauri.conf.json
cat src-tauri/tauri.conf.json | grep '"version"'
# Should show: "version": "1.0.0"

# Set signing key (provide your Apple Developer certificate)
export TAURI_SIGNING_PRIVATE_KEY="your-signing-key-here"
export TAURI_SIGNING_KEY_PASSWORD="your-password-here"

# Production build
pnpm tauri build
# This will:
# 1. Build TypeScript/React frontend
# 2. Compile Rust backend
# 3. Bundle app
# 4. Sign with Apple certificate
# 5. Generate .dmg installer
```

**Expected Output**:
- `src-tauri/target/release/bundle/macos/AssistSupport.app` (signed app)
- `src-tauri/target/release/bundle/dmg/AssistSupport_1.0.0_x64.dmg` (installer)
- `src-tauri/target/release/bundle/macos/AssistSupport.app/Contents/Resources/` (bundled resources)

**Exit Criteria**: 
- Build succeeds (exit code 0)
- .dmg file created and signed
- App code signature valid: `codesign -v /path/to/AssistSupport.app`

---

### Step 5: Record Deployment Artifact

```bash
cd /home/user/AssistSupport

# Calculate SHA256 of the dmg
DMG_PATH="src-tauri/target/release/bundle/dmg/AssistSupport_1.0.0_x64.dmg"
DMG_SHA=$(shasum -a 256 "$DMG_PATH" | awk '{print $1}')
echo "DMG SHA256: $DMG_SHA"

# Verify code signature
codesign -v src-tauri/target/release/bundle/macos/AssistSupport.app
# Expected: valid on disk

# This would normally be called via the app, but for CI:
# record_deployment_artifact(
#   artifact_type: "dmg",
#   version: "1.0.0",
#   channel: "production",
#   sha256: $DMG_SHA,
#   is_signed: true
# )
```

**Exit Criteria**: SHA256 calculated, signature verified

---

### Step 6: Run Deployment Preflight Checks

After the app is built and on the macOS machine:

```bash
# Launch AssistSupport
open src-tauri/target/release/bundle/macos/AssistSupport.app

# In the UI, navigate to:
# Ops (tab) → Deployment Preflight

# Run the preflight check by clicking "Run Preflight"
# Expected checks:
# ✅ Database integrity
# ✅ SQLCipher accessible
# ✅ Vector store initialized
# ✅ LLM engine ready (or marked unavailable)
# ✅ File system accessible
```

**Exit Criteria**: Preflight report shows all checks GREEN

---

### Step 7: Deploy to Work Machine

```bash
# Option A: Direct app execution (dev)
open src-tauri/target/release/bundle/macos/AssistSupport.app

# Option B: Install via DMG (production)
hdiutil attach src-tauri/target/release/bundle/dmg/AssistSupport_1.0.0_x64.dmg
# Drag AssistSupport.app to /Applications
# Eject DMG

# Verify checksum on target machine
shasum -a 256 AssistSupport_1.0.0_x64.dmg
# Compare with Step 5 SHA256
```

**Exit Criteria**: App runs on target machine, no crash on launch

---

### Step 8: Post-Deploy Health Verification

Launch the app and verify:

```bash
# In the app UI (if accessible):
# - Settings tab → Check status indicators
# - Should show: Database ✅, Vector Store (if enabled) ✅

# In the database (via SQL client):
# SELECT overall_status FROM system_health_checks 
#   WHERE checked_at > datetime('now', '-5 minutes');
# Expected: "healthy"
```

**Exit Criteria**: `get_system_health()` returns `overall_status: "healthy"`

---

## Summary

After completing all 8 steps on macOS:

- ✅ All tests pass (frontend + backend)
- ✅ Security audit clean (npm + Rust)
- ✅ Production bundle signed and verified
- ✅ Artifact recorded with SHA256
- ✅ Preflight checks GREEN
- ✅ App deployed and running
- ✅ Health check returns healthy

**Phase 1 is COMPLETE** → Ready to proceed to Phase 2 (Pilot Testing)

---

## Troubleshooting

| Issue | Solution |
|---|---|
| Build fails with "keyring not found" | Install via Xcode: `xcode-select --install` |
| Code signing fails | Set `TAURI_SIGNING_PRIVATE_KEY` with valid Apple Developer cert |
| Preflight shows database unavailable | Check `/tmp/assistsupport*.db` permissions, delete and restart |
| App won't launch | Check codesign: `codesign --verify --verbose /path/to/app` |
| Health check fails | Check `startup_metrics` table for last error |

---

**Estimated Time**: 15-30 minutes on macOS (depending on network/build cache)  
**Next**: Phase 2 - Pilot Testing Runbook
