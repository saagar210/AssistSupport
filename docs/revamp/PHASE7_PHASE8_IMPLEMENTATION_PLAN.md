# Phase 7 and Phase 8 Implementation Plan

Status: Complete  
Scope: AssistSupport + MemoryKernel bilateral execution  
Intent: Complete Gate G7 and Gate G8 with explicit evidence, reproducible verification, and zero ambiguity.

## 1. Objectives
1. Close Phase 7 (`Security + Compliance Closure`) in both repos with complete control evidence and signoff packets.
2. Close Phase 8 (`Release Candidate + Handoff`) in both repos with full suite validation, rollback drill evidence, and bilateral GO/NO-GO record.
3. Preserve runtime safety posture while closing gates:
   1. Enrichment stays optional/non-blocking.
   2. Deterministic fallback stays intact.
   3. No direct MemoryKernel calls outside adapter boundary.
   4. Runtime cutover remains blocked unless bilateral decision gate is explicitly satisfied.

## 2. Non-Negotiable Rules
1. No feature work in this program slice.
2. No schema/runtime contract changes during Phase 7 and 8 closeout.
3. No gate closure without committed evidence files in both repos.
4. Any failed mandatory command blocks progression.
5. Any unresolved High/Critical security finding blocks Phase 7 closure.
6. Any rollback drill failure blocks Phase 8 closure.

## 3. Repositories and Branch Policy
1. AssistSupport repo:
   1. Path: `/Users/d/Projects/AssistSupport`
   2. Branch target: `master`
2. MemoryKernel repo:
   1. Path: `/Users/d/Projects/MemoryKernel`
   2. Branch target: `main`
3. Commit strategy:
   1. Commit after each major section closure.
   2. Push after each section closure.
   3. Keep commits scoped to section intent.

## 4. Phase 7 Plan: Security and Compliance Closure

## 4.1 Phase 7 Entry Criteria (must all pass)
1. AssistSupport `docs/revamp/PHASE_STATUS.md` shows Phases 4-6 complete.
2. MemoryKernel baseline governance docs are present and up to date.
3. Pin/matrix/manifest/handoff governance checks currently pass.
4. No uncommitted unrelated local changes in either repo.

## 4.2 Phase 7 Deliverables: AssistSupport
1. `docs/revamp/PHASE7_SECURITY_COMPLIANCE_PLAN.md`
2. `docs/revamp/CONTROL_EVIDENCE_MATRIX.md`
3. `docs/revamp/SECURITY_SIGNOFF_PACKET.md`
4. `docs/revamp/evidence/PHASE7_SECURITY_COMPLIANCE_CLOSURE_YYYY-MM-DD.md`

## 4.3 Phase 7 Deliverables: MemoryKernel
1. `docs/implementation/PHASE7_PRODUCER_SECURITY_COMPLIANCE_PLAN.md`
2. `docs/implementation/PRODUCER_CONTROL_EVIDENCE_MATRIX.md`
3. `docs/implementation/PRODUCER_SECURITY_SIGNOFF_PACKET.md`
4. `docs/implementation/PHASE7_PRODUCER_CLOSURE_YYYY-MM-DD.md`

## 4.4 Control Matrix Requirements (both repos)
1. Every control row must include:
   1. Control identifier.
   2. Standard mapping intent (FedRAMP High, GDPR, SOC2, ISO27001, NIST SSDF, OWASP, internal policy).
   3. Evidence artifact path.
   4. Verification command.
   5. Owner.
   6. Pass/fail status.
2. No row may contain “TBD”.
3. No row may reference external/untracked evidence.

## 4.5 Mandatory Verification: AssistSupport
Run exactly:
```bash
pnpm run typecheck
pnpm run test
pnpm run check:memorykernel-pin
pnpm run check:memorykernel-governance
pnpm run check:memorykernel-handoff
pnpm run check:memorykernel-handoff:service-v3-candidate
pnpm run check:llm-golden-set
pnpm run check:rollback-readiness
pnpm run check:phase6-ops-hardening
pnpm run test:security-regression
pnpm run test:memorykernel-contract
pnpm run test:memorykernel-phase3-dry-run
pnpm run test:ci
```

## 4.6 Mandatory Verification: MemoryKernel
Run exactly:
```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
./scripts/verify_service_contract_alignment.sh --memorykernel-root /Users/d/Projects/MemoryKernel
./scripts/verify_contract_parity.sh --canonical-root /Users/d/Projects/MemoryKernel
./scripts/verify_trilogy_compatibility_artifacts.sh --memorykernel-root /Users/d/Projects/MemoryKernel
./scripts/run_trilogy_smoke.sh --memorykernel-root /Users/d/Projects/MemoryKernel
./scripts/run_trilogy_compliance_suite.sh --memorykernel-root /Users/d/Projects/MemoryKernel --skip-baseline
./scripts/verify_producer_contract_manifest.sh --memorykernel-root /Users/d/Projects/MemoryKernel
./scripts/verify_producer_handoff_payload.sh --memorykernel-root /Users/d/Projects/MemoryKernel
```

## 4.7 Phase 7 Exit Criteria
1. All mandatory commands pass in both repos.
2. Control-to-evidence matrices complete in both repos.
3. Security signoff packets complete in both repos.
4. No unresolved High/Critical findings.
5. `docs/revamp/PHASE_STATUS.md` updated to `Phase 7: Complete`.

## 4.8 Phase 7 Failure Handling
1. If any command fails:
   1. Stop gate progression.
   2. Fix only failing scope.
   3. Re-run full Phase 7 mandatory suite.
2. If any High/Critical finding appears:
   1. Create remediation section in signoff packet.
   2. Fix before closure.
   3. Re-run relevant commands and full mandatory suite.

## 5. Phase 8 Plan: Release Candidate and Handoff

## 5.1 Phase 8 Entry Criteria (must all pass)
1. Phase 7 closed in both repos.
2. Security signoff packets committed and pushed in both repos.
3. Runtime baseline remains pinned and governance checks pass.

## 5.2 Phase 8 Deliverables: AssistSupport
1. `docs/revamp/PHASE8_RELEASE_CANDIDATE_PLAN.md`
2. `docs/revamp/WORK_MACHINE_HANDOFF_RUNBOOK.md`
3. `docs/revamp/GO_NO_GO_DECISION_RECORD.md`
4. `docs/revamp/evidence/PHASE8_RELEASE_CANDIDATE_CLOSURE_YYYY-MM-DD.md`

## 5.3 Phase 8 Deliverables: MemoryKernel
1. `docs/implementation/PHASE8_PRODUCER_RC_PLAN.md`
2. `docs/implementation/PRODUCER_WORK_MACHINE_HANDOFF_RUNBOOK.md`
3. `docs/implementation/PRODUCER_GO_NO_GO_DECISION_RECORD.md`
4. `docs/implementation/PHASE8_PRODUCER_CLOSURE_YYYY-MM-DD.md`

## 5.4 Rollback Drill Requirements (bilateral)
1. Execute documented rollback drill in both repos.
2. Capture:
   1. Preconditions.
   2. Commands executed.
   3. Expected vs observed result.
   4. Recovery verification commands.
   5. Final state confirmation.
3. Store evidence artifacts in-repo.
4. Rollback drill must be reproducible by runbook only.

## 5.5 GO/NO-GO Decision Record Requirements
1. Single bilateral decision structure:
   1. Target runtime baseline (tag + SHA).
   2. Gate checklist completion map.
   3. Open blockers.
   4. Risk acceptance statements.
   5. Final verdict.
2. If any blocker remains unresolved:
   1. Verdict must be NO-GO.
   2. Explicit next remediation section required.

## 5.6 Work-Machine Handoff Validation Requirements
1. Validate clean clone bootstrap instructions.
2. Validate dependency and toolchain setup instructions.
3. Validate mandatory verification suite execution instructions.
4. Validate rollback execution instructions.
5. Validate operator runbook completeness.

## 5.7 Phase 8 Exit Criteria
1. Full test/governance suites pass in both repos.
2. Rollback drill passes and is logged in both repos.
3. Bilateral GO/NO-GO record complete.
4. Work-machine handoff runbooks validated.
5. `docs/revamp/PHASE_STATUS.md` updated to `Phase 8: Complete`.

## 5.8 Phase 8 Failure Handling
1. Any rollback drill failure blocks closure.
2. Any missing handoff step blocks closure.
3. Any mismatch between decision records blocks closure.
4. Re-run full mandatory suites after remediation.

## 6. Bilateral Risk Register Addendum for Phases 7 and 8
1. R9: Evidence mismatch across repos.
   1. Trigger: conflicting closure assertions.
   2. Mitigation: bilateral closure packet diff check before push.
2. R10: False GO due to stale command results.
   1. Trigger: reused old outputs.
   2. Mitigation: command outputs timestamped in closure artifacts.
3. R11: Work-machine onboarding gaps.
   1. Trigger: bootstrap deviation from runbook.
   2. Mitigation: clean-clone rehearsal with strict runbook adherence.
4. R12: Rollback runbook drift.
   1. Trigger: drill requires undocumented manual steps.
   2. Mitigation: update runbook immediately and rerun drill.

## 7. Execution Sequence (strict order)
1. Execute Phase 7 entry check.
2. Build AssistSupport Phase 7 artifacts.
3. Build MemoryKernel Phase 7 artifacts.
4. Run all mandatory suites for Phase 7 in both repos.
5. Close Phase 7 in both repos.
6. Execute Phase 8 entry check.
7. Build AssistSupport Phase 8 artifacts.
8. Build MemoryKernel Phase 8 artifacts.
9. Execute bilateral rollback drill and capture evidence.
10. Run final mandatory suites in both repos.
11. Complete bilateral GO/NO-GO record.
12. Close Phase 8 in both repos.

## 8. Definition of Done
1. Phase 7 and Phase 8 are both marked `Complete` with evidence.
2. All command suites are green in both repos.
3. Bilateral decision record and rollback evidence are committed.
4. Work-machine handoff runbooks are validated and final.
