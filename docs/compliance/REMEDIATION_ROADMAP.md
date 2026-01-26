# Compliance Remediation Roadmap

**Date**: 2025-01-25
**Version**: AssistSupport v0.3.1

---

## Gap Analysis

### Technical Controls: No Gaps

All 41 technical controls across 7 standards pass validation. No code changes required.

### Documentation Gaps (Non-Technical)

These gaps are administrative/legal, not technical. They apply only when deploying in specific regulatory contexts.

| Gap | Applies When | Effort | Priority |
|-----|-------------|--------|----------|
| HIPAA Administrative Safeguards | Handling PHI | Policy docs, BAAs | If healthcare |
| GDPR Privacy Policy & DPA | EU user data | Legal review | If EU market |
| FISMA SSP & SAR | Federal deployment | Use this report as basis | If federal |
| SOC 2 Type II Audit | Customer requirement | 6-month observation period | If enterprise sales |
| PCI DSS Certification | Payment processing | QSA engagement | If payment integration |

---

## Dependency Advisories

Two transitive dependency advisories from Dependabot (neither exploitable in AssistSupport):

| Package | Severity | Advisory | Dependency Chain | Action |
|---------|----------|----------|-----------------|--------|
| `glib` 0.18.2 | Medium | GHSA-wrw7-89jp-8q8g | Tauri GTK bindings (Linux only) | Monitor for upstream update |
| `lru` 0.12.5 | Low | GHSA-rhfx-m35p-ff5j | tantivy -> lance -> lancedb | Monitor for upstream update |

Neither requires immediate action:
- `glib`: Linux-only dependency, not used on macOS builds
- `lru`: Unsound `IterMut` pattern not exercised by lancedb/tantivy usage

---

## Optional Enhancements

These are not required for compliance but would strengthen security posture:

### Short-term
- [ ] Add CodeQL GitHub Action for automated static analysis
- [ ] Add GitHub Actions CI to run tests on push
- [ ] Add repository topics for discoverability

### Medium-term
- [ ] Independent penetration test (after 6 months operational)
- [ ] Formal threat model review with security engineer
- [ ] SBOM (Software Bill of Materials) generation

### Long-term (If Pursuing Certification)
- [ ] SOC 2 Type II: Engage auditor, begin 6-month observation
- [ ] ISO 27001: Develop ISMS documentation, engage certification body
- [ ] FedRAMP: Prepare SSP from this evidence, engage 3PAO

---

## Certification Readiness

| Certification | Technical Readiness | Administrative Readiness | Overall |
|---------------|-------------------|------------------------|---------|
| NIST SP 800-53 | Ready | Evidence documented | Ready |
| ISO 27001 | Ready | ISMS documentation needed | Partial |
| SOC 2 Type I | Ready | Auditor engagement needed | Partial |
| SOC 2 Type II | Ready | 6-month observation needed | Not yet |
| HIPAA | Ready | Policies and BAAs needed | Partial |
| PCI DSS | Ready | QSA engagement needed | Partial |
| GDPR | Ready | Legal documentation needed | Partial |
| FISMA | Ready | SSP and SAR needed | Partial |

"Ready" = technical controls implemented and tested.
"Partial" = technical ready, administrative/legal docs needed.
