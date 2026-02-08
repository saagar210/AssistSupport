# Producer Ownership Matrix

Updated: 2026-02-08
Owner: MemoryKernel

| Area | Primary Owner | Backup Owner | Required Evidence on Change |
|---|---|---|---|
| Service runtime envelopes (`memory-kernel-service`) | Producer Runtime | Producer API | service tests + OpenAPI + manifest/handoff checks |
| API orchestration (`memory-kernel-api`) | Producer API | Producer Runtime | cargo tests + service alignment check |
| Storage/migrations (`memory-kernel-store-sqlite`) | Producer Storage | Producer Runtime | schema tests + readiness checks |
| Contract pack (`contracts/integration/v1`) | Producer Contracts | Producer Runtime | parity checks + compatibility artifacts |
| Governance scripts (`scripts/verify_*`) | Producer Governance | Producer Contracts | negative-fixture tests + CI pass |
| Release/handoff docs (`docs/implementation`) | Producer Governance | Producer Runtime | updated checkpoint packet + decision record |

## Approval Rule

Any change touching runtime contracts or contract pack requires dual-owner review (Primary + Backup) before baseline promotion.
