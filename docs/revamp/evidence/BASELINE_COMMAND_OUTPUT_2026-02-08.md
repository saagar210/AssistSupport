# Baseline Command Output (2026-02-08)

## Environment
- cwd: /Users/d/Projects/AssistSupport
- branch: master
- commit: bc77ae5286e6a9fdabe6b6e64322202f3cb9f989

## pnpm run typecheck

> assistsupport@1.0.0 typecheck /Users/d/Projects/AssistSupport
> tsc --noEmit


## pnpm run test

> assistsupport@1.0.0 test /Users/d/Projects/AssistSupport
> vitest run


 RUN  v4.0.18 /Users/d/Projects/AssistSupport

 ✓ src/hooks/useMemoryKernelEnrichment.test.ts (3 tests) 9ms
 ✓ src/hooks/useLlm.test.ts (9 tests) 17ms
 ✓ src/hooks/useKb.test.ts (10 tests) 19ms
 ✓ src/components/Draft/ResponsePanel.test.tsx (13 tests) 111ms
 ✓ src/components/Draft/InputPanel.test.tsx (22 tests) 168ms
 ✓ src/hooks/useInitialize.test.ts (7 tests) 383ms
 ✓ src/components/Settings/SettingsTab.test.tsx (18 tests) 513ms

 Test Files  7 passed (7)
      Tests  82 passed (82)
   Start at  10:07:19
   Duration  1.36s (transform 835ms, setup 559ms, import 1.65s, tests 1.22s, environment 3.01s)


## pnpm run check:memorykernel-governance

> assistsupport@1.0.0 check:memorykernel-governance /Users/d/Projects/AssistSupport
> node scripts/validate_memorykernel_governance_bundle.mjs

MemoryKernel governance bundle validation passed.

## pnpm run check:memorykernel-handoff:service-v3-candidate

> assistsupport@1.0.0 check:memorykernel-handoff:service-v3-candidate /Users/d/Projects/AssistSupport
> ASSISTSUPPORT_REQUIRE_HANDOFF_PAYLOAD=1 ASSISTSUPPORT_HANDOFF_REQUIRE_PIN_MATCH=0 MEMORYKERNEL_EXPECTED_SERVICE_CONTRACT_VERSION=service.v3 node scripts/validate_memorykernel_handoff_payload.mjs

MemoryKernel handoff payload validation passed.
Wrote MemoryKernel handoff evidence: /Users/d/Projects/AssistSupport/artifacts/memorykernel-handoff-evidence.json
