# Trilogy Compliance Test Suite

## Purpose

This suite provides a repeatable compliance regression gate for the trilogy monorepo prior to downstream integrations (for example, AssistSupport).

It validates technical controls and evidence continuity across:

- `MemoryKernel` (authoritative memory + context package resolution)
- `OutcomeMemory` (trust projection/gating)
- `MultiAgentCenter` (orchestration + trace/replay)

## Standards Covered

1. NIST SP 800-53
2. ISO/IEC 27001
3. SOC 2
4. HIPAA (technical safeguards)
5. PCI DSS
6. GDPR
7. FISMA/FedRAMP alignment

Control mapping is defined in:

- `docs/compliance/TRILOGY_CONTROL_MATRIX.md`

## Entry Command

```bash
./scripts/run_trilogy_compliance_suite.sh
```

### What the suite runs

1. Baseline runtime gates:
   - `scripts/verify_contract_parity.sh`
   - `scripts/verify_trilogy_compatibility_artifacts.sh`
   - `scripts/run_trilogy_smoke.sh`
2. Standard-specific mapped checks:
   - `scripts/compliance/check_nist_80053.sh`
   - `scripts/compliance/check_iso27001.sh`
   - `scripts/compliance/check_soc2.sh`
   - `scripts/compliance/check_hipaa.sh`
   - `scripts/compliance/check_pci_dss.sh`
   - `scripts/compliance/check_gdpr.sh`
   - `scripts/compliance/check_fisma_fedramp.sh`

## CI Integration

Root CI runs this suite in:

- `.github/workflows/ci.yml` → `Seven-standard compliance suite`

## Operational Note

Passing this suite indicates technical control readiness and evidence quality.
Formal certifications/authorizations still require external legal, policy, and assessor workflows.
