# Monorepo Workstation Bootstrap

Owner: AssistSupport Platform Engineering  
Last updated: 2026-02-08

## Purpose
Provide a deterministic bootstrap procedure for a fresh workstation using the unified AssistSupport monorepo (including embedded MemoryKernel at `services/memorykernel`).

## Scope
This bootstrap applies to:
1. Local development.
2. CI parity checks.
3. Release-candidate verification before merge to `master`.

## Preconditions
1. `git`, `node`, `pnpm`, `rustup`, `cargo` are installed.
2. Workstation has outbound network access for dependency restore.
3. Repository clone target directory exists and is writable.

## Canonical Bootstrap
1. Clone repository:
```bash
git clone https://github.com/saagar210/AssistSupport.git
cd AssistSupport
```
2. Install frontend dependencies:
```bash
pnpm install --frozen-lockfile
```
3. Validate consumer integration governance:
```bash
pnpm run check:memorykernel-pin
pnpm run check:memorykernel-governance
pnpm run check:memorykernel-handoff
pnpm run check:memorykernel-handoff:service-v3-candidate
pnpm run check:memorykernel-boundary
pnpm run check:memorykernel-cutover-policy
```
4. Validate consumer runtime and contract behavior:
```bash
pnpm run typecheck
pnpm run test
pnpm run test:memorykernel-contract
pnpm run test:memorykernel-phase3-dry-run
pnpm run test:memorykernel-cutover-dry-run
pnpm run test:ci
```
5. Validate imported MemoryKernel stack:
```bash
cd services/memorykernel
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
./scripts/verify_service_contract_alignment.sh --memorykernel-root "$(pwd)"
./scripts/verify_contract_parity.sh --canonical-root "$(pwd)"
./scripts/verify_trilogy_compatibility_artifacts.sh --memorykernel-root "$(pwd)"
./scripts/run_trilogy_smoke.sh --memorykernel-root "$(pwd)"
./scripts/run_trilogy_compliance_suite.sh --memorykernel-root "$(pwd)" --skip-baseline
./scripts/verify_producer_contract_manifest.sh --memorykernel-root "$(pwd)"
./scripts/verify_producer_handoff_payload.sh --memorykernel-root "$(pwd)"
```
6. Return to repository root:
```bash
cd ../..
```

## Expected Outcome
1. All commands pass without manual patching.
2. Consumer governance artifacts are internally consistent.
3. Embedded MemoryKernel contracts and compliance gates pass in-place.

## Fail-Fast Rules
1. If `check:memorykernel-pin` fails, stop and repair pin/matrix/manifest atomicity.
2. If `check:memorykernel-boundary` fails, stop and fix direct-call boundary violations.
3. If any MemoryKernel contract/compliance script fails, stop and resolve before merge.

## Security and Compliance Controls
1. No secrets are required for baseline local verification.
2. Governance gates enforce controlled contract drift.
3. Compliance checks include NIST/ISO/SOC2/FedRAMP/GDPR/CCPA/OWASP-intent mappings through documented gates.
