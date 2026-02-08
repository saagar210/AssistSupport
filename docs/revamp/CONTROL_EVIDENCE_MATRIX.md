# AssistSupport Control Evidence Matrix

Status: Complete  
Date: 2026-02-08

| Control ID | Control Objective | Standard Mapping | Evidence Artifact(s) | Verification Command(s) | Owner | Status |
|---|---|---|---|---|---|---|
| AS-C01 | Secure build and typed frontend contracts | NIST SSDF, SOC2, ISO27001 | `docs/revamp/evidence/PHASE7_SECURITY_COMPLIANCE_CLOSURE_2026-02-08.md` | `pnpm run typecheck` | AssistSupport Engineering | Pass |
| AS-C02 | Functional regression control across UI/runtime | NIST SSDF, SOC2 Availability | `docs/revamp/evidence/PHASE7_SECURITY_COMPLIANCE_CLOSURE_2026-02-08.md` | `pnpm run test`, `pnpm run test:ci` | AssistSupport Engineering | Pass |
| AS-C03 | MemoryKernel pin/matrix/manifest governance | SOC2, ISO27001, Internal Policy | `artifacts/memorykernel-contract-evidence.json`, `docs/revamp/evidence/PHASE7_SECURITY_COMPLIANCE_CLOSURE_2026-02-08.md` | `pnpm run check:memorykernel-pin`, `pnpm run check:memorykernel-governance` | Integration Owner | Pass |
| AS-C04 | Producer handoff and candidate payload policy validation | SOC2, ISO27001, Internal Policy | `artifacts/memorykernel-handoff-evidence.json`, `docs/revamp/evidence/PHASE7_SECURITY_COMPLIANCE_CLOSURE_2026-02-08.md` | `pnpm run check:memorykernel-handoff`, `pnpm run check:memorykernel-handoff:service-v3-candidate` | Integration Owner | Pass |
| AS-C05 | LLM quality and policy safety regression guard | OWASP Top 10 (LLM abuse intent), NIST SSDF | `docs/revamp/evidence/LLM_GOLDEN_SET_LATEST.json` | `pnpm run check:llm-golden-set` | LLM Runtime Owner | Pass |
| AS-C06 | Rollback readiness and baseline integrity | FedRAMP High (operational readiness intent), SOC2 Availability | `artifacts/rollback-readiness-evidence.json` | `pnpm run check:rollback-readiness` | Release Owner | Pass |
| AS-C07 | Ops hardening orchestration and deterministic gate execution | FedRAMP High, SOC2 Availability, ISO27001 | `docs/revamp/evidence/PHASE6_OPS_HARDENING_LATEST.json` | `pnpm run check:phase6-ops-hardening` | Release Owner | Pass |
| AS-C08 | Security regression coverage (crypto, SSRF/DNS rebinding, injection, path) | OWASP Top 10, ISO27001, FedRAMP High | `docs/revamp/evidence/PHASE7_SECURITY_COMPLIANCE_CLOSURE_2026-02-08.md` | `pnpm run test:security-regression` | Security Owner | Pass |
| AS-C09 | MemoryKernel adapter contract and fallback safety | NIST SSDF, SOC2, Internal Policy | `artifacts/memorykernel-contract-evidence.json` | `pnpm run test:memorykernel-contract`, `pnpm run test:memorykernel-phase3-dry-run` | Integration Owner | Pass |
| AS-C10 | Non-blocking enrichment and deterministic fallback invariant | Internal Policy, SOC2 Availability | `docs/revamp/evidence/PHASE4_UX_CLOSURE_2026-02-08.md`, `docs/revamp/evidence/PHASE6_OPS_HARDENING_CLOSURE_2026-02-08.md` | `pnpm run test:memorykernel-contract`, `pnpm run test:ci` | Product + Engineering | Pass |
| AS-C11 | Data minimization and safe export handling | GDPR, ISO27001 | `src-tauri/tests/security.rs`, `docs/revamp/evidence/PHASE7_SECURITY_COMPLIANCE_CLOSURE_2026-02-08.md` | `pnpm run test:ci` | Security Owner | Pass |
| AS-C12 | Change-control traceability and phase gate discipline | NIST SSDF, ISO27001, Internal Policy | `docs/revamp/RELEASE_GATES.md`, `docs/revamp/PHASE_STATUS.md`, `docs/revamp/SECURITY_SIGNOFF_PACKET.md` | Documentation review + command evidence in closure file | Program Owner | Pass |
