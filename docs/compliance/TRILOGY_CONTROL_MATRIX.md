# Trilogy Compliance Control Matrix

This matrix maps monorepo technical controls to seven major standards:

- NIST SP 800-53
- ISO/IEC 27001
- SOC 2
- HIPAA (technical safeguards)
- PCI DSS
- GDPR
- FISMA/FedRAMP alignment

## Scope Boundary

The checks in this repository validate technical and process controls implemented in code, CI, and operational runbooks.

- Included: deterministic behavior, contract governance, trust gating, replay/audit substrate, cryptographic snapshot controls, CI policy gates.
- Excluded: legal, policy, contracting, and external assessor activities required for formal certifications.

FedRAMP note: this suite demonstrates technical readiness and evidence quality; authorization still requires external SSP/3PAO/agency processes.

## Standard Mapping

| Standard | Core control intent | Trilogy technical evidence | Automated check |
|---|---|---|---|
| NIST SP 800-53 | AC/AU/SC/SI baseline controls | threat model, trust controls, deterministic query guarantees, parity enforcement | `scripts/compliance/check_nist_80053.sh` |
| ISO 27001 | governance and change control discipline | versioning policy, release gate, closeout runbook, integration contract lock | `scripts/compliance/check_iso27001.sh` |
| SOC 2 | secure operations and monitoring consistency | CI governance gates, compatibility matrix, trace/audit docs, integration handoff | `scripts/compliance/check_soc2.sh` |
| HIPAA | technical safeguards over protected operational data | encryption/signature controls, provenance/accountability fields, trace schema | `scripts/compliance/check_hipaa.sh` |
| PCI DSS | secure coding and integrity verification controls | warning-free lint gate, benchmark threshold enforcement, trust-gate artifact checks | `scripts/compliance/check_pci_dss.sh` |
| GDPR | privacy-by-design and explainability | append-only lineage, truth/confidence separation, explainable context package, recovery runbook | `scripts/compliance/check_gdpr.sh` |
| FISMA/FedRAMP | continuous monitoring and baseline hardening | deterministic release closeout, soak/smoke gates, threshold semantics, release criteria docs | `scripts/compliance/check_fisma_fedramp.sh` |

## Suite Entry Point

Run the complete mapped suite:

```bash
./scripts/run_trilogy_compliance_suite.sh
```

For fast local iteration when baseline runtime checks were already run:

```bash
./scripts/run_trilogy_compliance_suite.sh --skip-baseline
```
