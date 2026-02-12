# AssistSupport v1.0.0 → Completion: Full Execution Roadmap

**Status**: Ready for execution  
**Current Location**: Phase 1 verification complete (Linux); runbook prepared for macOS  
**Next Step**: Phase 1 macOS execution (8 steps, 15-30 minutes)

---

## Phase Execution Sequence

### ✅ Phase 1: Production Deployment (8 steps, ~30 min)
**Location**: See `PHASE1_MACOS_RUNBOOK.md`  
**Status**: Verification complete on Linux, ready for macOS  
**Outcomes**:
- Build passes all tests (frontend 129, backend 271+)
- Production .dmg bundle signed and verified
- Artifact recorded with SHA256
- Health check returns healthy
- **Unlocks**: Phase 2

---

### 📊 Phase 2: Pilot Testing (12 steps, ~2-4 weeks)
**Run sequentially after Phase 1 completes**

#### Quick Summary
1. Enable pilot logging in production
2. Ingest 27 curated KB articles into PostgreSQL search API
3. Fix score fusion policy bug (category-aware boosting)
4. Remove ~293 junk articles (< 100 chars)
5. Add category boosting to adaptive fusion
6. Run 10-query validation (target: 70%+ top-1 relevance)
7. Distribute build to 3-5 pilot participants
8. Collect feedback 1-2 weeks via PilotTab component
9. Export pilot data to CSV
10. Address top issues (KB expansion, prompt tuning, scoring weights)
11. Run eval harness on pilot queries
12. Pilot go/no-go decision

#### Files Modified
- `search-api/score_fusion.py` (fix policy_boost logic, add category boosting)
- `knowledge_base/*.md` (expand 10-15 thin articles)
- `search-api/rebuild_indexes.py` (run to regenerate FTS/HNSW)
- `/docs/WEEK3_PILOT_RESULTS.md` (document findings)

#### Success Criteria
- Accuracy ≥ 90%
- 50+ feedback entries
- Team confidence ≥ Medium
- Decision checkpoint populated
- **Unlocks**: Phase 3

---

### 🧠 Phase 3: LLM Router V2 (4 steps, ~2 days)

#### Quick Summary
1. Implement intent-aware template selection in `prompts.rs`
   - `"answer"` → standard_template
   - `"clarify"` → clarification_template  
   - `"abstain"` → abstain_template
2. Add generation parameter profiles by intent in `commands/mod.rs`
   - Policy queries: temp=0.3 (factual)
   - Troubleshooting: temp=0.7 (creative)
3. Run golden set validation (no regressions)
4. Expand golden set with new router V2 cases

#### Success Criteria
- Golden set passes with expanded cases
- No quality regressions vs Phase 2 baseline
- **Unlocks**: Phase 4

---

### 📚 Phase 4: KB Enrichment (5 steps, ~1 week)

#### Quick Summary
1. Expand thin articles (< 500 chars) to 800+ chars
   - Add context, step-by-step instructions, troubleshooting
   - Prioritize articles from Phase 2 pilot gaps
2. Run title cleaning script on PostgreSQL
3. Rebuild FTS5 + HNSW indexes in search API
4. Re-index local SQLite KB
5. Validate: 10-query test should show 85%+ top-1 relevance

#### Success Criteria
- Top-1 relevance ≥ 85%
- Top-3 relevance ≥ 95%
- **Unlocks**: Phase 5

---

### 🔌 Phase 5: MemoryKernel Integration (8 steps, ~1 week)

#### Quick Summary
1. Verify service availability (pin/manifest valid)
2. Build and start MemoryKernel service locally
3. Set auth token via secure storage
4. Enable feature flag
5. Test enrichment end-to-end (Draft generation with enrichment)
6. Verify all 4 fallback scenarios:
   - Service offline → `fallback_reason: "offline"`
   - Wrong auth → `fallback_reason: "auth-required"`
   - Timeout (100ms) → `fallback_reason: "timeout"`
   - Feature disabled → `fallback_reason: "feature-disabled"`
7. Run contract suite + governance negative tests
8. **Decision point**: Service V3 cutover readiness (GO or NO-GO)
   - Document decision with evidence
   - If GO → unlocks Phase 6 Step 42
   - If NO-GO → system continues on service.v2 indefinitely (valid outcome)

#### Success Criteria
- All 4 fallbacks work correctly
- Contract suite green
- Enrichment applied in generation
- Decision recorded
- **Unlocks**: Phase 6 (with optional Step 42 if GO)

---

### 🔐 Phase 6: Admin + Network Ingest (6 steps, ~3-5 days)

#### Quick Summary
1. Deploy search API to production config
   - Set `ENVIRONMENT=production`, rate limiting, port 3390
2. Enable cross-encoder reranking
   - +30-50ms latency, higher precision
3. Configure network ingest allow rules
   - Trusted domains (internal docs, approved GitHub)
4. Schedule KB re-ingestion
   - Auto-mark stale sources
5. **CONDITIONAL Step 42**: Execute Service V3 cutover
   - **Only if Step 5 decision = GO**
   - Update pin/manifest/matrix files
   - All CI gates must pass
   - **If NO-GO**: skip entirely, system fully functional on service.v2
6. Ops hardening validation

#### Success Criteria
- Search API production-ready
- Reranking enabled and measured
- Network ingest rules configured
- Ops hardening checks pass
- Rollback readiness confirmed
- **Unlocks**: Phase 7

---

### 🏗️ Phase 7: Monorepo Migration (15 steps, ~1 week)

#### Quick Summary
**Gates**: A (source-of-truth), B (boundary/safety), C (handoff), D (negative drift), E (full suite), F (rollback)

1. Readiness check
2. Verify subtree integrity
3. **Gate A**: Source-of-truth alignment (pin/manifest/governance)
4. **Gate B**: Boundary & runtime safety
5. **Gate C**: Handoff integrity
6. **Gate D**: Negative drift proof
7. **Gate E**: Full program verification (all tests)
8. **Gate F**: Rollback readiness
9. Verify CI pipeline covers all lanes
10. Workstation bootstrap verification
11. Archive legacy MemoryKernel repo (read-only)
12. Record monorepo decision checkpoint
13. Program closeout documentation
14. Establish ongoing integration review
15. Tag v1.0.0-monorepo release

#### Success Criteria
- All gates A-F pass
- CI pipeline verified
- Workstation bootstrap works
- Release tagged
- **Project Complete** ✅

---

## Parallel Activities (Not Sequential)

### Phase 4 Parallel: KB Enrichment Loop (Ongoing)
- Runs during Phase 3-4
- Uses `kb_gap_candidates` table to identify low-confidence topics
- Continuous quality improvement

### MemoryKernel Version Monitoring (Background)
- Monitor service.v3 availability
- Run contract validation in CI
- Decision point is Phase 5 Step 8 (not blocking)

---

## Testing at Each Phase

| Phase | Key Test Commands |
|---|---|
| 1 | `cargo test`, `pnpm test`, `pnpm test:e2e:smoke` |
| 2 | 10-query validation, `get_pilot_stats`, `run_eval_harness` |
| 3 | `pnpm run check:llm-golden-set`, regression suite |
| 4 | 10-query validation, `get_kb_health_stats` |
| 5 | `pnpm run test:memorykernel-contract`, fallback scenarios |
| 6 | `pnpm run check:phase6-ops-hardening`, `pnpm run check:rollback-readiness` |
| 7 | Gates A-F checks, `pnpm run check:monorepo-readiness:full` |

---

## Decision Points (Async, Non-Blocking)

### Step 37 (Phase 5): Service V3 Cutover Decision
- **Options**: GO (proceed to Phase 6 Step 42) or NO-GO (skip, use service.v2)
- **Criteria**: All integration tests pass, MemoryKernel service stable
- **Timeline**: Make after Phase 5 Step 36 passes
- **Impact if NO-GO**: Zero impact; system fully functional on service.v2

### Step 42 (Phase 6): Conditionally Execute
- **Prerequisite**: Step 37 = GO
- **If NO-GO from Step 37**: **Entire step is skipped**
- **Success** if either:
  - Step 42 executed and cutover completed (GO path)
  - Step 42 skipped because NO-GO was decision (valid alternative)

---

## Rollback Plan (All Phases)

| Phase | Rollback | Command |
|---|---|---|
| 1 | Revert to previous release | `pnpm tauri build [previous-version]` |
| 2 | Disable pilot logging | `ASSISTSUPPORT_ENABLE_PILOT_LOGGING=0` |
| 3 | Revert prompt changes | `git revert [prompt-commit]` |
| 4 | Restore KB from backup | `restore_kb_from_backup()` |
| 5 | Disable MemoryKernel | `ASSISTSUPPORT_ENABLE_MEMORY_KERNEL=0` |
| 6 | Disable reranker | `ASSISTSUPPORT_ENABLE_CROSS_ENCODER=0` |
| 7 | Revert pin/manifest | `git revert [monorepo-commit]` |

---

## Time Estimates

| Phase | Duration | Notes |
|---|---|---|
| 1 | 30 min | macOS build + testing |
| 2 | 2-4 weeks | Pilot recruitment, feedback collection |
| 3 | 2 days | Prompt tuning, validation |
| 4 | 1 week | Article expansion, re-indexing |
| 5 | 1 week | Service integration + decision |
| 6 | 3-5 days | Ops hardening, optional cutover |
| 7 | 1 week | Monorepo gates + stabilization |
| **Total** | **5-7 weeks** | Including 2-4 week pilot + parallel phases |

---

## Success Definition

**All 7 phases complete when**:
- ✅ Phase 1: Production app running, health check healthy
- ✅ Phase 2: Pilot accuracy ≥ 90%, no blocking issues
- ✅ Phase 3: Golden set green, no regressions
- ✅ Phase 4: KB health metrics 85%+ top-1 relevance
- ✅ Phase 5: Integration tests pass, cutover decision recorded
- ✅ Phase 6: Ops hardening green, rollback ready (±optional Step 42)
- ✅ Phase 7: All gates A-F pass, release tagged

**Project Status**: PRODUCTION-READY ✅

---

## Key Documents

- `PHASE1_EXECUTION_CHECKPOINT.md` - What was verified on Linux
- `PHASE1_MACOS_RUNBOOK.md` - Step-by-step macOS execution (8 steps)
- `PHASE2_RUNBOOK.md` - Pilot testing (see below)
- `PHASE3_RUNBOOK.md` - LLM routing (see below)
- (etc. for remaining phases)

---

## Next Immediate Action

**Execute Phase 1 on macOS machine**:
1. Clone this repo or sync latest changes
2. Run commands from `PHASE1_MACOS_RUNBOOK.md` steps 1-8
3. Estimated time: 15-30 minutes
4. Once complete, proceed to Phase 2

**Current Status**: Ready to execute ✅
