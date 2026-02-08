# Runtime Cutover Decision Record (Producer Mirror)

Updated: 2026-02-08  
Owner: MemoryKernel + AssistSupport (bilateral)

## Decision Scope
Producer-side mirror of the bilateral Phase 8 runtime cutover decision.

## Decision Outcome
1. Rehearsal continuation: **GO**
2. Runtime cutover execution: **GO**

## Approved Runtime Target
- release_tag: `v0.4.0`
- commit_sha: `7e4806a34b98e6c06ee33fa9f11499a975e7b922`
- service_contract_version: `service.v3`
- api_contract_version: `api.v1`
- integration_baseline: `integration/v1`

## Ownership (Named Roles)
- MemoryKernel incident commander role: MemoryKernel Producer On-Call Lead
- AssistSupport incident commander role: Support Platform On-Call Lead
- MemoryKernel rollback owner role: MemoryKernel Release Owner
- AssistSupport rollback owner role: AssistSupport Runtime Integrations Owner
- Joint decision log owner role: Integration Program Owner

## Runtime Cutover Completion Evidence
1. Immutable producer runtime baseline published (`v0.4.0`).
2. Consumer repin and governance bundle aligned to the same baseline.
3. Producer and consumer verification suites are green at cutover close.
4. Rollback readiness evidence refreshed against the new runtime baseline.

## Program State
- Phase 7: **Closed**
- Phase 8: **Complete**
- Phase 9 stabilization window: **Complete**
- Phase 10 program delivery closeout: **Complete**
