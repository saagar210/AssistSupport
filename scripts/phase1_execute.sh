#!/bin/bash
set -e

# AssistSupport Phase 1: Production Deployment
# Automated execution script for macOS
# 
# Usage: bash scripts/phase1_execute.sh
# 
# Prerequisites:
# - macOS machine
# - Xcode Command Line Tools installed
# - Rust toolchain (rustup)
# - Node.js 22+ with pnpm 10.29.2+
# - TAURI_SIGNING_PRIVATE_KEY environment variable set

echo "═══════════════════════════════════════════════════════════════"
echo "  AssistSupport Phase 1: Production Deployment"
echo "═══════════════════════════════════════════════════════════════"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Step tracking
STEPS_PASSED=0
STEPS_FAILED=0

# Function to run a step
run_step() {
    local step_num=$1
    local step_name=$2
    local step_cmd=$3
    
    echo ""
    echo "${YELLOW}Step $step_num: $step_name${NC}"
    echo "Command: $step_cmd"
    echo "──────────────────────────────────────────────────────────"
    
    if eval "$step_cmd"; then
        echo "${GREEN}✓ Step $step_num PASSED${NC}"
        ((STEPS_PASSED++))
    else
        echo "${RED}✗ Step $step_num FAILED${NC}"
        ((STEPS_FAILED++))
        return 1
    fi
}

# Prerequisites check
echo ""
echo "${YELLOW}Checking prerequisites...${NC}"

if ! command -v cargo &> /dev/null; then
    echo "${RED}✗ Rust toolchain not found. Install via: rustup.rs${NC}"
    exit 1
fi

if ! command -v pnpm &> /dev/null; then
    echo "${RED}✗ pnpm not found. Install via: npm install -g pnpm${NC}"
    exit 1
fi

if ! command -v node &> /dev/null; then
    echo "${RED}✗ Node.js not found. Install from nodejs.org${NC}"
    exit 1
fi

echo "${GREEN}✓ All prerequisites found${NC}"

# Step 1: Full CI Validation
run_step 1 "TypeScript Type Check" "pnpm run typecheck" || true
run_step 1 "Frontend Unit Tests" "pnpm test -- --run" || true
run_step 1 "Backend Unit Tests" "cd src-tauri && cargo test --lib" || true
run_step 1 "Clippy Linting" "cd src-tauri && cargo clippy -- -D warnings" || true
run_step 1 "Code Format Check" "cd src-tauri && cargo fmt --check" || true

# Step 2: Security Audit
run_step 2 "NPM Security Audit" "pnpm audit --audit-level high" || true
run_step 2 "Rust Security Audit" "cd src-tauri && cargo audit" || true
run_step 2 "Security Regression Tests" "pnpm run test:security-regression" || true

# Step 3: MemoryKernel Contract
run_step 3 "MemoryKernel Contract Suite" "pnpm run test:memorykernel-contract" || true
run_step 3 "MemoryKernel Governance" "pnpm run check:memorykernel-governance" || true
run_step 3 "Rollback Readiness" "pnpm run check:rollback-readiness" || true

# Step 4: Production Build
echo ""
echo "${YELLOW}Step 4: Production Build${NC}"
echo "──────────────────────────────────────────────────────────"

# Verify version
VERSION=$(cat src-tauri/tauri.conf.json | grep '"version"' | head -1 | sed 's/.*"version": "\([^"]*\)".*/\1/')
echo "Building version: $VERSION"

if [ -z "$TAURI_SIGNING_PRIVATE_KEY" ]; then
    echo "${RED}✗ TAURI_SIGNING_PRIVATE_KEY not set. Build will not be signed.${NC}"
    echo "Set it via: export TAURI_SIGNING_PRIVATE_KEY='your-key-here'"
    exit 1
fi

run_step 4 "Production Build" "pnpm tauri build" || true

# Step 5: Record Artifact
echo ""
echo "${YELLOW}Step 5: Record Deployment Artifact${NC}"
echo "──────────────────────────────────────────────────────────"

DMG_PATH="src-tauri/target/release/bundle/dmg/AssistSupport_${VERSION}_x64.dmg"
if [ -f "$DMG_PATH" ]; then
    DMG_SHA=$(shasum -a 256 "$DMG_PATH" | awk '{print $1}')
    echo "DMG SHA256: $DMG_SHA"
    echo "${GREEN}✓ Step 5 PASSED${NC}"
    ((STEPS_PASSED++))
else
    echo "${RED}✗ DMG not found at $DMG_PATH${NC}"
    ((STEPS_FAILED++))
fi

# Step 6: Deployment Preflight
echo ""
echo "${YELLOW}Step 6: Deployment Preflight${NC}"
echo "──────────────────────────────────────────────────────────"
echo "Manual step: Launch AssistSupport app and run preflight checks in Ops tab"
echo "Press ENTER to confirm preflight passed..."
read -p "> "

# Step 7: Deploy to Work Machine
echo ""
echo "${YELLOW}Step 7: Deploy to Work Machine${NC}"
echo "──────────────────────────────────────────────────────────"
echo "Manual step: Install app or run via:"
echo "  open src-tauri/target/release/bundle/macos/AssistSupport.app"
echo "Press ENTER to confirm app is running..."
read -p "> "

# Step 8: Health Verification
echo ""
echo "${YELLOW}Step 8: Post-Deploy Health Verification${NC}"
echo "──────────────────────────────────────────────────────────"
echo "Manual step: In the app, check Settings tab status"
echo "Expected: All components healthy (Database ✅, etc.)"
echo "Press ENTER to confirm health check passed..."
read -p "> "
echo "${GREEN}✓ Step 8 PASSED${NC}"
((STEPS_PASSED++))

# Summary
echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "  Phase 1 Execution Summary"
echo "═══════════════════════════════════════════════════════════════"
echo "${GREEN}Steps Passed: $STEPS_PASSED${NC}"
echo "${RED}Steps Failed: $STEPS_FAILED${NC}"

if [ $STEPS_FAILED -eq 0 ]; then
    echo ""
    echo "${GREEN}✓ PHASE 1 COMPLETE${NC}"
    echo ""
    echo "Next: Proceed to Phase 2 (Pilot Testing)"
    echo "See: COMPLETE_EXECUTION_ROADMAP.md"
else
    echo ""
    echo "${RED}✗ Phase 1 has failures. Review errors above.${NC}"
    exit 1
fi
