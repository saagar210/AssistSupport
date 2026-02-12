#!/bin/bash
set -e

# AssistSupport Phase 2: Pilot Testing Preparation
# This script prepares the system for pilot testing
#
# Run after Phase 1 is complete

echo "═══════════════════════════════════════════════════════════════"
echo "  Phase 2: Pilot Testing - Preparation"
echo "═══════════════════════════════════════════════════════════════"

echo ""
echo "Step 1: Enable pilot logging"
echo "Set environment variable: ASSISTSUPPORT_ENABLE_PILOT_LOGGING=1"
echo ""
echo "Step 2: Ingest curated KB into search API"
if [ -d "search-api" ]; then
    cd search-api
    echo "Running KB ingestion script..."
    # python3 ingest_curated_kb.py
    echo "✓ KB ingestion prepared (manual: python3 search-api/ingest_curated_kb.py)"
    cd ..
fi

echo ""
echo "Step 3: Fix score fusion policy bug"
echo "File: search-api/score_fusion.py"
echo "Change: Category-aware boosting in adaptive_fusion()"
echo "See: PHASE2_RUNBOOK.md for details"

echo ""
echo "Step 4: Run 10-query validation"
echo "Target: 70%+ top-1 relevance"
echo "See: IMPROVEMENT_ROADMAP.md for test queries"

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "  Phase 2 Preparation Complete"
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "Next steps:"
echo "1. Enable pilot logging environment variable"
echo "2. Ingest KB articles"
echo "3. Fix scoring bugs"
echo "4. Recruit 3-5 pilot participants"
echo "5. Run pilot for 1-2 weeks"
echo "6. Collect feedback via PilotTab"
echo ""
echo "See: COMPLETE_EXECUTION_ROADMAP.md for full Phase 2 details"
