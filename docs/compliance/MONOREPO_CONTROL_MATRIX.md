# Monorepo Compliance Control Matrix

## Scope
Application-level control mapping for AssistSupport + MemoryKernel monorepo.

| Standard | Control Objective | Enforced Gate(s) | Evidence Artifact |
|---|---|---|---|
| FedRAMP High | Controlled change + audit-ready release validation | `memorykernel-monorepo`, `monorepo-governance` | `docs/implementation/evidence/GATE_G6_IMPORTED_MEMORYKERNEL_VALIDATION_2026-02-08.md`, `docs/implementation/evidence/GATE_G5_ASSISTSUPPORT_MONOREPO_VALIDATION_2026-02-08.md` |
| NIST SSDF | Secure build and verification discipline | `lint-frontend`, `memorykernel-monorepo`, `memorykernel-contract` | `docs/implementation/MONOREPO_CI_GATE_SPEC.md`, `docs/implementation/evidence/GATE_G5_ASSISTSUPPORT_MONOREPO_VALIDATION_2026-02-08.md` |
| ISO/IEC 27001 | Governance, version control, and release discipline | `memorykernel-contract`, `monorepo-governance` | `docs/implementation/MONOREPO_MIGRATION_PROGRAM_CHARTER.md`, `docs/implementation/MIGRATION_EXCEPTION_LOG.md`, `docs/implementation/evidence/GATE_G7_MONOREPO_DECISION_CHECKPOINT_2026-02-08.md` |
| SOC 2 (Security/Availability) | Availability and fallback resilience with controlled rollout | `memorykernel-contract`, `monorepo-governance`, `test-e2e-smoke` | `docs/implementation/POST_CUTOVER_STABILIZATION_WINDOW_2026-02-08.md`, `docs/implementation/SERVICE_V3_CUTOVER_DECISION_CHECKPOINT_2026-02-08.md`, `docs/implementation/RUNTIME_CUTOVER_GOVERNANCE_PACKET.md` |
| GDPR | Data minimization and controlled handling in workflows | `memorykernel-monorepo` compliance suite (`check_gdpr`) | `services/memorykernel/docs/compliance/TRILOGY_COMPLIANCE_TEST_SUITE.md`, gate output evidence |
| CCPA/CPRA | Data handling controls and accountable operational procedures | `memorykernel-monorepo` compliance suite + governance lanes | `docs/implementation/RUNTIME_CUTOVER_DECISION_RECORD_2026-02-08.md`, `docs/implementation/MEMORYKERNEL_ROLLBACK_DRILL_2026-02-08.md` |
| OWASP Top 10 / ASVS intent | Input validation, boundary enforcement, secure defaults | `check:memorykernel-boundary`, `check:memorykernel-cutover-policy`, backend test suites | `scripts/validate_memorykernel_boundary.mjs`, `scripts/validate_memorykernel_cutover_policy.mjs` |

## Control Notes
1. This matrix intentionally maps controls to enforceable CI gates and evidence files.
2. Runtime cutover governance remains explicit and NO-GO by default unless bilateral decision records are updated.
