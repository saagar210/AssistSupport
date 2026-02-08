# Non-Negotiables (Revamp Invariants)

Status: Active  
Date: 2026-02-08

## 1) Integration Safety
1. MemoryKernel enrichment must remain optional and non-blocking.
2. Deterministic fallback behavior must remain unchanged under all known failure classes.
3. No direct MemoryKernel calls outside the approved adapter boundary.
4. Pin/matrix/manifest governance must remain atomic and CI-enforced.

## 2) Runtime Governance
1. Runtime cutover decisions require explicit bilateral GO/NO-GO record.
2. No runtime service contract cutover is implied by candidate payload validation.
3. Rollback procedure must exist and be validated before any runtime gate changes.

## 3) Product and UX
1. Legacy UI components are not protected by sentiment/history.
2. Any component that fails acceptance criteria is replaced, not patched indefinitely.
3. Keyboard-first operation is required for core IT workflows.

## 4) Engineering Quality
1. No undocumented architectural changes.
2. No phase closure without evidence artifacts and passing verification suite.
3. No temporary compatibility layer without explicit deprecation/removal gate.

## 5) Security and Compliance
1. No secrets in source, logs, prompts, or test fixtures.
2. Existing security controls cannot be weakened without tests and explicit rationale.
3. Compliance evidence mapping must be maintained as implementation proceeds.
