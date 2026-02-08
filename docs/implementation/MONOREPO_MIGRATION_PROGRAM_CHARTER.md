# Monorepo Migration Program Charter

## Objective
Consolidate AssistSupport and MemoryKernel into a single monorepo rooted at `/Users/d/Projects/AssistSupport` while preserving runtime behavior, governance controls, security posture, and compliance evidence continuity.

## In Scope
- History-preserving MemoryKernel import into `services/memorykernel`
- Unified CI and governance gates
- Compliance control/evidence mapping
- Clean-machine reproducibility validation
- Legacy repo transition to mirror/read-only mode

## Out of Scope
- Feature development unrelated to migration
- Runtime behavior redesign
- Runtime contract cutover without explicit bilateral GO record

## Non-Negotiable Invariants
1. Enrichment remains optional and non-blocking.
2. Deterministic fallback remains intact.
3. No direct MemoryKernel calls outside the defined adapter boundary.
4. Runtime cutover remains NO-GO by default.
5. Pin/matrix/manifest/handoff checks remain atomic and enforced.
6. No production/user data may enter fixtures, logs, or migration evidence.

## Authority Model
- Technical execution lead: AssistSupport migration owner
- Producer contract authority: MemoryKernel owner
- Compliance signoff authority: security/compliance approver
- Runtime cutover authority: bilateral GO record only

## Stop Conditions
- Any required gate fails and cannot be corrected without scope expansion
- Any invariant regression is detected
- Any compliance control lacks enforceable evidence

## Completion Definition
Migration is complete only when all phases pass, evidence is committed, monorepo bootstrap passes from a clean clone, and legacy repo operating mode is updated.
