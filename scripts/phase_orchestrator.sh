#!/bin/bash
set -e

# AssistSupport: Phase Orchestration Script
# Coordinates execution of all 7 phases
#
# Usage: bash scripts/phase_orchestrator.sh [phase_number]
# Examples:
#   bash scripts/phase_orchestrator.sh 1
#   bash scripts/phase_orchestrator.sh 2
#   bash scripts/phase_orchestrator.sh all

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
PHASE_NUM="${1:-all}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Logging
log_info() {
  echo -e "${BLUE}ℹ${NC} $1"
}

log_success() {
  echo -e "${GREEN}✓${NC} $1"
}

log_warn() {
  echo -e "${YELLOW}⚠${NC} $1"
}

log_error() {
  echo -e "${RED}✗${NC} $1"
}

# Phase status tracking
declare -A PHASE_STATUS
declare -A PHASE_START_TIME
declare -A PHASE_END_TIME

check_phase_prerequisites() {
  local phase=$1

  case $phase in
    1)
      log_info "Phase 1 prerequisites:"
      log_info "  - macOS machine (current: $(uname))"
      log_info "  - Xcode CLT installed"
      log_info "  - Signing key configured"
      ;;
    2)
      log_info "Phase 2 prerequisites:"
      if [ ! -d "src-tauri/target/release/bundle/dmg" ]; then
        log_error "Phase 1 not complete (no .dmg found)"
        return 1
      fi
      log_success "Phase 1 outputs verified"
      ;;
    3)
      log_info "Phase 3 prerequisites:"
      if [ ! -f "PHASE2_DECISION.md" ]; then
        log_error "Phase 2 not marked complete"
        return 1
      fi
      if ! grep -q "DECISION: GO" PHASE2_DECISION.md; then
        log_error "Phase 2 decision: NO-GO (cannot proceed)"
        return 1
      fi
      log_success "Phase 2 GO decision verified"
      ;;
    *)
      log_info "Phase $phase prerequisites check"
      ;;
  esac

  return 0
}

execute_phase_1() {
  log_info "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  log_info "PHASE 1: Production Deployment"
  log_info "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

  check_phase_prerequisites 1 || return 1

  log_info "Phase 1 must be executed manually on macOS"
  log_info "See: PHASE1_MACOS_RUNBOOK.md"
  log_info ""
  log_info "Automated option available:"
  log_info "  bash scripts/phase1_execute.sh"

  PHASE_STATUS[1]="READY"
  return 0
}

execute_phase_2() {
  log_info "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  log_info "PHASE 2: Pilot Testing"
  log_info "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

  check_phase_prerequisites 2 || return 1

  log_info "Phase 2 preparation ready"
  log_info "See: PHASE2_PILOT_TESTING_RUNBOOK.md"
  log_info ""
  log_info "Manual steps:"
  log_info "  1. Enable pilot logging"
  log_info "  2. Ingest 27 KB articles"
  log_info "  3. Fix scoring policy"
  log_info "  4. Remove junk articles"
  log_info "  5. Configure category boosting"
  log_info "  6. Run 10-query validation"
  log_info "  7. Distribute to 3-5 pilots"
  log_info "  8. Collect feedback (1-2 weeks)"
  log_info "  9. Export pilot data"
  log_info "  10. Address top issues"
  log_info "  11. Run evaluation harness"
  log_info "  12. Make GO/NO-GO decision"

  PHASE_STATUS[2]="READY"
  return 0
}

execute_phase_3() {
  log_info "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  log_info "PHASE 3: LLM Router V2"
  log_info "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

  check_phase_prerequisites 3 || return 1

  log_info "Phase 3 preparation ready"
  log_info "See: PHASE3_LLM_ROUTER_RUNBOOK.md"
  log_info ""
  log_info "Implementation steps:"
  log_info "  1. Implement intent-aware template selection"
  log_info "  2. Add generation parameter profiles"
  log_info "  3. Run golden set validation"
  log_info "  4. Expand golden set with new cases"

  PHASE_STATUS[3]="READY"
  return 0
}

execute_phase_4() {
  log_info "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  log_info "PHASE 4: KB Enrichment"
  log_info "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

  log_info "Phase 4 preparation ready"
  log_info "See: PHASE4_KB_ENRICHMENT_RUNBOOK.md"
  log_info ""
  log_info "Implementation steps:"
  log_info "  1. Identify thin articles"
  log_info "  2. Expand articles to 800+ chars"
  log_info "  3. Run title cleaning"
  log_info "  4. Rebuild search API indexes"
  log_info "  5. Re-index local KB"
  log_info "  6. Run 10-query validation"

  PHASE_STATUS[4]="READY"
  return 0
}

execute_phase_5() {
  log_info "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  log_info "PHASE 5: MemoryKernel Integration"
  log_info "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

  log_info "Phase 5 preparation ready"
  log_info "See: PHASE5_MEMORYKERNEL_RUNBOOK.md"
  log_info ""
  log_info "Implementation steps:"
  log_info "  1. Verify service availability"
  log_info "  2. Build and start MemoryKernel locally"
  log_info "  3. Set auth token"
  log_info "  4. Enable feature flag"
  log_info "  5. Test enrichment end-to-end"
  log_info "  6. Verify all fallback scenarios"
  log_info "  7. Run contract suite"
  log_info "  8. Make GO/NO-GO decision (Service V3 cutover)"

  PHASE_STATUS[5]="READY"
  return 0
}

execute_phase_6() {
  log_info "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  log_info "PHASE 6: Admin + Network Ingest"
  log_info "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

  log_info "Phase 6 preparation ready"
  log_info "See: PHASE6_ADMIN_NETWORK_RUNBOOK.md"
  log_info ""
  log_info "Implementation steps:"
  log_info "  1. Deploy search API to production config"
  log_info "  2. Enable cross-encoder reranking"
  log_info "  3. Configure network ingest rules"
  log_info "  4. Schedule KB re-ingestion"
  log_info "  5. CONDITIONAL: Execute Service V3 cutover"
  log_info "  6. Ops hardening validation"

  PHASE_STATUS[6]="READY"
  return 0
}

execute_phase_7() {
  log_info "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  log_info "PHASE 7: Monorepo Migration"
  log_info "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

  log_info "Phase 7 preparation ready"
  log_info "See: PHASE7_MONOREPO_MIGRATION_RUNBOOK.md"
  log_info ""
  log_info "Implementation steps:"
  log_info "  1. Readiness check"
  log_info "  2. Verify subtree integrity"
  log_info "  3. GATE A: Source-of-truth alignment"
  log_info "  4. GATE B: Boundary & runtime safety"
  log_info "  5. GATE C: Handoff integrity"
  log_info "  6. GATE D: Negative drift proof"
  log_info "  7. GATE E: Full program verification"
  log_info "  8. GATE F: Rollback readiness"
  log_info "  9. Verify CI pipeline"
  log_info "  10. Workstation bootstrap verification"
  log_info "  11. Archive legacy repo"
  log_info "  12. Record monorepo decision"
  log_info "  13. Establish ongoing review"
  log_info "  14. Program closeout documentation"
  log_info "  15. Tag v1.0.0-monorepo release"

  PHASE_STATUS[7]="READY"
  return 0
}

print_summary() {
  echo ""
  echo "╔════════════════════════════════════════════════╗"
  echo "║  AssistSupport Phase Orchestrator - Summary    ║"
  echo "╚════════════════════════════════════════════════╝"
  echo ""

  for phase in {1..7}; do
    if [ -z "${PHASE_STATUS[$phase]}" ]; then
      continue
    fi

    status="${PHASE_STATUS[$phase]}"
    symbol="○"
    if [ "$status" = "READY" ]; then
      symbol="✓"
    fi

    printf "  $symbol Phase $phase: %-40s %s\n" \
      "$(get_phase_name $phase)" \
      "[$status]"
  done

  echo ""
  echo "All preparation complete!"
  echo "Runbooks available for manual execution."
  echo ""
}

get_phase_name() {
  case $1 in
    1) echo "Production Deployment" ;;
    2) echo "Pilot Testing" ;;
    3) echo "LLM Router V2" ;;
    4) echo "KB Enrichment" ;;
    5) echo "MemoryKernel Integration" ;;
    6) echo "Admin + Network Ingest" ;;
    7) echo "Monorepo Migration" ;;
  esac
}

main() {
  echo ""
  echo "╔════════════════════════════════════════════════╗"
  echo "║  AssistSupport Phase Orchestrator              ║"
  echo "║  Phases 1-7 (5-7 weeks to completion)         ║"
  echo "╚════════════════════════════════════════════════╝"
  echo ""

  if [ "$PHASE_NUM" = "all" ]; then
    for phase in {1..7}; do
      case $phase in
        1) execute_phase_1 ;;
        2) execute_phase_2 ;;
        3) execute_phase_3 ;;
        4) execute_phase_4 ;;
        5) execute_phase_5 ;;
        6) execute_phase_6 ;;
        7) execute_phase_7 ;;
      esac
      echo ""
    done
  else
    case "$PHASE_NUM" in
      1) execute_phase_1 ;;
      2) execute_phase_2 ;;
      3) execute_phase_3 ;;
      4) execute_phase_4 ;;
      5) execute_phase_5 ;;
      6) execute_phase_6 ;;
      7) execute_phase_7 ;;
      *)
        log_error "Invalid phase: $PHASE_NUM"
        log_info "Usage: bash scripts/phase_orchestrator.sh [1-7|all]"
        exit 1
        ;;
    esac
  fi

  print_summary
}

main "$@"
