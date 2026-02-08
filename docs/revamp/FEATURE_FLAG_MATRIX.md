# Feature Flag Matrix (Revamp)

Status: Active  
Date: 2026-02-08

## Purpose
Define existing and planned feature toggles so revamp rollout is reversible and low-risk.

## Existing Flags / Runtime Controls
| Control | Type | Default | Scope | Purpose |
|---|---|---:|---|---|
| MemoryKernel integration enabled (settings) | Runtime setting | true | App runtime | Enable/disable enrichment behavior |
| `ASSISTSUPPORT_REQUIRE_HANDOFF_PAYLOAD` | Env var | 0 | Governance scripts | Require handoff payload validation |
| `ASSISTSUPPORT_HANDOFF_REQUIRE_PIN_MATCH` | Env var | 1 | Governance scripts | Enforce pin and handoff SHA alignment |
| `MEMORYKERNEL_EXPECTED_SERVICE_CONTRACT_VERSION` | Env var | from pin | Governance scripts | Candidate/stable contract validation |
| `ASSISTSUPPORT_VALIDATE_REMOTE_MANIFEST` | Env var | 0 | Governance scripts | Optional remote producer manifest validation |

## Planned Revamp Flags (to implement in Phase 3/4)
| Flag | Default | Owner | Controlled Surface | Rollback Action |
|---|---:|---|---|---|
| `ASSISTSUPPORT_REVAMP_APP_SHELL` | 0 | Frontend | New global shell/nav | Revert to legacy shell |
| `ASSISTSUPPORT_REVAMP_INBOX` | 0 | Frontend | Queue-first inbox | Revert to legacy draft landing |
| `ASSISTSUPPORT_REVAMP_WORKSPACE` | 0 | Frontend | New tri-pane workspace | Revert to legacy draft panel |
| `ASSISTSUPPORT_REVAMP_COMMAND_PALETTE_V2` | 0 | Frontend | New command system | Revert to current palette |
| `ASSISTSUPPORT_LLM_ROUTER_V2` | 0 | Runtime | Model routing/prompt contracts | Revert to legacy LLM path |

## Implemented Revamp Flag Runtime (Phase 3)
1. Flag parser implemented in `src/features/revamp/flags.ts`.
2. Flag read precedence:
   1. Local storage override (`assistsupport.flag.<FLAG_NAME>`)
   2. Vite env (`VITE_<FLAG_NAME>`)
   3. Safe default (`false`)
3. Current active flag wiring:
   1. `ASSISTSUPPORT_REVAMP_INBOX` toggles queue-first inbox wrapper.
   2. `ASSISTSUPPORT_REVAMP_WORKSPACE` toggles workspace revamp shell wrapper.
   3. `ASSISTSUPPORT_REVAMP_COMMAND_PALETTE_V2` enables queue jump commands (unassigned, at risk, in progress, resolved).
      Commands remain disabled unless `ASSISTSUPPORT_REVAMP_INBOX` is also enabled.
   4. Other flags are parsed and available for subsequent feature slices.

## Flag Governance Rules
1. New revamp surface cannot replace legacy by default until phase gate exit criteria pass.
2. Every flag must have a documented rollback command path.
3. Flag defaults can only flip in release-candidate phase with evidence.
4. Flag removal requires successful stabilization window and closure ADR.

## Default-On Gate Criteria (First Candidate: `ASSISTSUPPORT_REVAMP_INBOX`)
1. Technical criteria:
   1. `pnpm run typecheck` PASS
   2. `pnpm run test` PASS
   3. `pnpm run test:memorykernel-contract` PASS
   4. `pnpm run test:ci` PASS
   5. `pnpm run test:revamp-queue-rehearsal` PASS
2. UX criteria:
   1. Queue-first mode verified in desktop and mobile shell without navigation regressions.
   2. Follow-up history operations (search, load, delete, template use) remain functional.
3. Governance criteria:
   1. Rollback path documented and rehearsed (`localStorage` override and env default).
   2. Phase evidence packet recorded in `docs/revamp/evidence/`.
4. Approval criteria:
   1. Explicit GO in revamp checkpoint note.
   2. Runtime cutover posture remains independent and unchanged (still governed by MemoryKernel checkpoint).
