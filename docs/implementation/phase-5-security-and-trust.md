# Phase 5: Security and Trust

## Deliverables

- Signed snapshot manifests and import verification.
- Threat model and abuse-case documentation.
- Optional encrypted backup/export support.

## Non-Goals

- Broad multi-tenant authorization model.
- Cloud deployment hardening beyond local-first scope.

## Rollback Criteria

- Integrity checks bypassed on import/export paths.
- Security controls reducing deterministic behavior guarantees.

## Exit Checklist

- [x] Manifest signature verification enforced.
- [x] Threat model documented and reviewed.
- [x] Security regression tests pass.
- [x] Backup/export trust guarantees documented.
