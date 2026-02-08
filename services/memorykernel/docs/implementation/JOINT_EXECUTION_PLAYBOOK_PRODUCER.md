# Joint Execution Playbook (Producer Companion)

## 1) Producer Current State and Commitments

### Current baseline (promoted)
- `release_tag`: `v0.3.2`
- `commit_sha`: `cf331449e1589581a5dcbb3adecd3e9ae4509277`
- `service_contract_version`: `service.v2`
- `api_contract_version`: `api.v1`
- `integration baseline`: `integration/v1`

### Producer commitments (active)
- `service.v2` non-2xx envelope remains stable:
  - includes `service_contract_version`, `error.code`, `error.message`, `legacy_error`
  - excludes `api_contract_version`
- `legacy_error` remains mandatory for `service.v2`; removal only in `service.v3`.
- Producer manifest is canonical and machine-readable:
  - `contracts/integration/v1/producer-contract-manifest.json`
- CI/release governance gates remain mandatory:
  - `verify_service_contract_alignment.sh`
  - `verify_producer_contract_manifest.sh`
  - `verify_producer_handoff_payload.sh`
  - parity + compatibility + smoke + compliance suite

### Confirmed joint decisions (AssistSupport + MemoryKernel)
- `error_code_enum` validation mode: set equality (order-independent).
- Producer-manifest hash validation in consumer CI: not required for Checkpoint A/B; candidate for Phase 3 automation hardening.
- Consumer governance rule: pin + matrix + manifest updates must be atomic in one PR.

### Checkpoint status (as of 2026-02-08)
- Checkpoint A (manifest mirrored + CI checks implemented): `GO`
- Checkpoint B (full consumer suite green with manifest governance): `GO`
- Checkpoint C (steady-state service.v2 sign-off): `GO` (14-day window)
- Checkpoint D (`service.v3` RFC review kickoff): `GO` (planning only)

## 2) Next 4 Producer Phases

## Phase P1: Governance Lock and Operationalization

### Objective
Make producer contract governance operationally routine (not tribal knowledge) for every release candidate.

### Dependencies
- Producer manifest present and valid.
- Existing service/alignment/parity checks green.

### Entry criteria
- `main` includes manifest and alignment checks.
- AssistSupport pinned to promoted baseline and green.

### Exit criteria
- Release checklist includes manifest update + validation.
- Every release candidate run proves all governance gates pass.

### Deliverables
- Governance checklist section in release flow (manifest + contract policy checks).
- Evidence capture references for:
  - manifest validation
  - contract alignment
  - parity/artifact checks
- Closeout evidence report:
  - `docs/implementation/PRODUCER_PHASE1_STABILITY_CLOSEOUT_2026-02-08.md`

### Verification commands
```bash
./scripts/verify_producer_contract_manifest.sh --memorykernel-root /Users/d/Projects/MemoryKernel
./scripts/verify_producer_handoff_payload.sh --memorykernel-root /Users/d/Projects/MemoryKernel
./scripts/verify_service_contract_alignment.sh --memorykernel-root /Users/d/Projects/MemoryKernel
./scripts/verify_contract_parity.sh --canonical-root /Users/d/Projects/MemoryKernel
./scripts/verify_trilogy_compatibility_artifacts.sh --memorykernel-root /Users/d/Projects/MemoryKernel
```

## Phase P2: Cross-Repo Drift Automation

### Objective
Reduce coordination drag by turning pin/matrix/manifest drift into fast CI failures.

### Dependencies
- AssistSupport mirrors producer manifest at `config/memorykernel-producer-manifest.json`.
- AssistSupport CI has pin/matrix checks in place.
- AssistSupport enforces atomic pin+matrix+manifest updates in one PR.

### Entry criteria
- Producer manifest schema and fields are agreed across repos.

### Exit criteria
- Drift paths are machine-detected:
  - producer manifest vs consumer mirrored manifest
  - consumer pin vs producer release tag/sha
  - consumer compatibility matrix vs producer manifest versions
  - atomic update policy violations fail consumer CI

### Deliverables
- Producer-side reference workflow documentation for cross-repo manifest sync.
- Consumer-facing validation contract (field-by-field required equality list).
- Field-level rule set specifies set-equality for `error_code_enum`.

### Verification commands
```bash
./scripts/verify_producer_contract_manifest.sh --memorykernel-root /Users/d/Projects/MemoryKernel
./scripts/verify_producer_handoff_payload.sh --memorykernel-root /Users/d/Projects/MemoryKernel
cargo test --workspace --all-targets --all-features
```

## Phase P3: `service.v3` RFC and Migration Readiness

### Objective
Design `service.v3` migration so `legacy_error` removal is low-risk and reversible at rollout time.

### Dependencies
- Stable `service.v2` sprint window completed.
- AssistSupport confirms non-blocking behavior does not require `legacy_error` for correctness.

### Entry criteria
- RFC template agreed.
- Current envelope policy and tests green.

### Exit criteria
- Approved RFC with:
  - exact `service.v3` error envelope schema
  - migration window
  - rollback strategy
  - hard gates before cutover

### Deliverables
- `service.v3` RFC draft with explicit compatibility plan.
- Migration decision table (go/no-go checks).

### Verification commands
```bash
./scripts/verify_service_contract_alignment.sh --memorykernel-root /Users/d/Projects/MemoryKernel
cargo test -p memory-kernel-service --all-targets
```

## Phase P4: Producer RC-to-Release Execution

### Objective
Ship producer releases with deterministic handoff payloads and zero ambiguity for AssistSupport consumption.

### Dependencies
- P1-P3 artifacts complete.
- All trilogy quality gates green.

### Entry criteria
- Release candidate identified.
- Manifest updated to target release.

### Exit criteria
- Immutable tag published.
- Producer handoff payload delivered and acknowledged.
- AssistSupport repin complete and green.

### Deliverables
- Release tag + commit pair.
- Producer handoff payload (template in Section 6).
- Closeout evidence bundle (commands/results).
- Generated handoff artifacts:
  - `docs/implementation/PRODUCER_RELEASE_HANDOFF_PACKET_TEMPLATE.md`
  - `docs/implementation/PRODUCER_RELEASE_HANDOFF_LATEST.json`

### Verification commands
```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
./scripts/verify_producer_contract_manifest.sh --memorykernel-root /Users/d/Projects/MemoryKernel
./scripts/verify_service_contract_alignment.sh --memorykernel-root /Users/d/Projects/MemoryKernel
./scripts/verify_contract_parity.sh --canonical-root /Users/d/Projects/MemoryKernel
./scripts/verify_trilogy_compatibility_artifacts.sh --memorykernel-root /Users/d/Projects/MemoryKernel
./scripts/run_trilogy_smoke.sh --memorykernel-root /Users/d/Projects/MemoryKernel
./scripts/run_trilogy_compliance_suite.sh --memorykernel-root /Users/d/Projects/MemoryKernel --skip-baseline
```

## 3) Producer Contract-Change Policy (`service.v2`)

### Additive error-code process
- Additive codes are allowed in `service.v2` only if:
  - OpenAPI enum is updated.
  - service contract docs are updated.
  - producer manifest `error_code_enum` is updated.
  - regression tests cover mapping and envelope shape.
  - changelog includes consumer impact statement.

### Notice lead times
- Standard path: `10` business days pre-release notice.
- Emergency path: `24` hours notice + same-day docs/spec/tests update.

### Emergency process
1. Publish provisional notice to AssistSupport with code + status mapping + fallback expectations.
2. Land producer changes with all contract gates passing.
3. Publish immutable release handoff payload.
4. Require consumer repin validation before broad rollout.

### Envelope policy lock (service.v2 and planned service.v3)
- Non-2xx envelopes include:
  - `service_contract_version`
  - `error`
  - `legacy_error` (required in `service.v2`, removed in `service.v3`)
- Non-2xx envelopes exclude:
  - `api_contract_version`

## 4) `service.v3` RFC and Migration Timeline (proposal)

Reference draft:
- `/Users/d/Projects/MemoryKernel/docs/implementation/SERVICE_V3_RFC_DRAFT.md`
- `/Users/d/Projects/MemoryKernel/docs/implementation/SERVICE_V3_REHEARSAL_PLAN.md`

## Milestone M1: RFC draft
- Define final `service.v3` non-2xx envelope without `legacy_error`.
- Define transition telemetry and rollback path.

## Milestone M2: Consumer readiness validation
- AssistSupport proves green with `error.code`-only correctness path.
- Keep deterministic fallback and optional enrichment behavior unchanged.

## Milestone M3: Overlap window
- Keep `service.v2` baseline active while `service.v3` artifacts are reviewed.
- Required artifacts:
  - OpenAPI (`service.v3`)
  - updated producer manifest
  - migration notes
  - consumer impact statement

## Milestone M4: Cutover gate
- `legacy_error` removal is allowed only when all are true:
  - producer release candidate gates pass
  - consumer repin is complete
  - consumer CI is green
  - joint go decision recorded

## 5) Producer Risk Register

| Risk | Mitigation | Early warning signal |
|---|---|---|
| Runtime/spec drift | Keep alignment checks mandatory in CI/release | `verify_service_contract_alignment.sh` failure |
| Manifest drift across repos | Enforce manifest sync checks and parity workflows | consumer CI pin/manifest mismatch |
| Hidden additive-code breakage | enforce 10-day notice + tests + manifest enum update | unannounced code appears in producer diff |
| Over-tight coupling in consumer parsing | keep `error.code` primary and deterministic fallback | consumer tests fail on unknown code path |
| Premature `service.v3` cutover | hard gate on consumer repin + green CI | attempted release without recorded consumer green |
| Release evidence gaps | require full command evidence in handoff protocol | missing command outputs in release packet |

## 6) Release Handoff Protocol (Producer -> AssistSupport)

### Required artifacts
- Immutable tag + commit:
  - `release_tag`
  - `commit_sha`
- Contract versions:
  - `expected_service_contract_version`
  - `expected_api_contract_version`
- Integration baseline:
  - `integration baseline`
- Producer manifest path and content hash (optional but recommended).
- Changelog path entry.

### Required evidence
- Full verification command list with pass/fail outcomes.
- `verify_producer_contract_manifest.sh` result.
- Service alignment/parity/artifact/smoke/compliance results.
- Generated handoff payload:
  - `./scripts/generate_producer_handoff_payload.sh --memorykernel-root /Users/d/Projects/MemoryKernel --out-json /Users/d/Projects/MemoryKernel/docs/implementation/PRODUCER_RELEASE_HANDOFF_LATEST.json`

### Gate-to-evidence mapping

| Gate | Required producer artifacts | Required AssistSupport artifacts | Verification commands |
|---|---|---|---|
| Checkpoint A | `contracts/integration/v1/producer-contract-manifest.json` | `config/memorykernel-producer-manifest.json` | Producer: `./scripts/verify_producer_contract_manifest.sh --memorykernel-root /Users/d/Projects/MemoryKernel`; Consumer: `pnpm run check:memorykernel-pin` |
| Checkpoint B | Release handoff payload for active baseline | `artifacts/memorykernel-contract-evidence.json` | Consumer: `pnpm run test:memorykernel-contract` |
| Checkpoint C | Stability policy references in producer docs | Updated consumer playbook and checkpoint sign-off notes | Producer alignment + Consumer contract suite green |
| Checkpoint D kickoff | `docs/implementation/SERVICE_V3_RFC_DRAFT.md` | Service.v3 cutover acceptance criteria in consumer playbook | Joint review of RFC + no runtime cutover |
| service.v3 cutover | Immutable v3 tag/sha + OpenAPI + manifest + changelog | Atomic pin/matrix/manifest update PR + rollback rehearsal evidence | Producer full suite + Consumer `pnpm run check:memorykernel-pin && pnpm run test:memorykernel-contract && pnpm run test:ci` |

### Consumer impact statement template
```text
Consumer impact:
- Runtime behavior changes: <none|describe>
- Required consumer mapping changes: <none|describe>
- Required test updates: <none|describe>
- Repin required: <yes/no> (target tag/sha)
- Rollback instruction: repin to <previous tag/sha>
```

## 7) Cross-Repo Drift Prevention Plan

### Consistency workflow
1. Producer updates release and manifest in same change set.
2. Producer CI validates manifest + service alignment + parity.
3. Producer publishes handoff payload.
4. AssistSupport mirrors manifest and repins.
5. AssistSupport CI validates pin + matrix + mirrored manifest sync.
6. Joint checkpoint records green status before next change wave.

### Proposed automated cross-repo checks
- Producer side:
  - keep `verify_producer_contract_manifest.sh` mandatory in CI/release.
- Consumer side (AssistSupport):
  - validate `config/memorykernel-producer-manifest.json` fields against pin + matrix.
  - fail on mismatch for tag/sha/service/api/integration baseline.
  - enforce `error_code_enum` set equality (order-independent).
  - enforce atomic pin+matrix+manifest updates in one PR.
  - defer producer-manifest hash validation to Phase 3 automation hardening.

## 8) Copy/Paste Prompt for AssistSupport Codex

```text
AssistSupport Codex (GPT-5.3),

MemoryKernel producer companion playbook is now published:
- /Users/d/Projects/MemoryKernel/docs/implementation/JOINT_EXECUTION_PLAYBOOK_PRODUCER.md

MemoryKernel commitments now locked:
1) Stable producer baseline remains:
   - release_tag: v0.3.2
   - commit_sha: cf331449e1589581a5dcbb3adecd3e9ae4509277
   - service.v2 / api.v1 / integration/v1
2) Additive service.v2 error-code policy:
   - standard: 10 business days notice
   - emergency: 24h notice + same-day docs/spec/tests update
3) service.v3 guard:
   - no legacy_error removal until consumer repin + CI green + explicit joint cutover gate
4) Producer manifest governance is now mandatory in CI/release.

What MemoryKernel already implemented:
- Producer manifest:
  - contracts/integration/v1/producer-contract-manifest.json
- Producer manifest gate:
  - scripts/verify_producer_contract_manifest.sh
- CI/release/compliance wiring for manifest validation.

Exact asks for AssistSupport (next execution block):
1) Execute service.v3 consumer rehearsal branch using:
   - /Users/d/Projects/AssistSupport/docs/implementation/SERVICE_V3_CONSUMER_REHEARSAL_PLAN.md
   - /Users/d/Projects/AssistSupport/docs/implementation/SERVICE_V3_REHEARSAL_EXECUTION_TRACKER.md
2) Keep deterministic fallback + optional/non-blocking enrichment unchanged.
3) Keep atomic pin+matrix+manifest governance active; treat manifest-hash validation as optional Phase 3 hardening unless promoted jointly.

Proposed joint timeline/checkpoints:
- Checkpoint A (complete): manifest mirrored + CI checks implemented
- Checkpoint B (complete): full consumer suite green with manifest governance
- Checkpoint C (complete): joint sign-off for steady-state service.v2 execution window (2 weeks)
- Checkpoint D (active): service.v3 RFC and rehearsal planning (no runtime changes yet)

Please return:
1) files changed
2) exact command outputs
3) blockers/risks
4) readiness verdict for service.v3 rehearsal entry
```
