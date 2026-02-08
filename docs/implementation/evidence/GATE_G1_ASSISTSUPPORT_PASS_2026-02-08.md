# Gate G1 AssistSupport Baseline Evidence

- Timestamp (UTC): 2026-02-08T14:25:51Z
- Repo: /Users/d/Projects/AssistSupport
- Branch: codex/monorepo-migration
- Commit: 2deeef0356773e278df4dba496c33313eee66e27

## Commands
```bash
pnpm run typecheck
pnpm run test
pnpm run check:memorykernel-pin
pnpm run check:memorykernel-governance
pnpm run check:memorykernel-handoff
pnpm run check:memorykernel-handoff:service-v3-candidate
pnpm run test:memorykernel-contract
pnpm run test:memorykernel-phase3-dry-run
pnpm run test:memorykernel-cutover-dry-run
pnpm run test:ci
```

## Output
```text

> assistsupport@1.0.0 typecheck /Users/d/Projects/AssistSupport
> tsc --noEmit


> assistsupport@1.0.0 test /Users/d/Projects/AssistSupport
> vitest run


 RUN  v4.0.18 /Users/d/Projects/AssistSupport

 ✓ src/hooks/useMemoryKernelEnrichment.test.ts (3 tests) 10ms
 ✓ src/hooks/useLlm.test.ts (9 tests) 17ms
 ✓ src/hooks/useKb.test.ts (10 tests) 18ms
 ✓ src/components/Draft/ResponsePanel.test.tsx (13 tests) 105ms
 ✓ src/components/Draft/InputPanel.test.tsx (22 tests) 160ms
 ✓ src/hooks/useInitialize.test.ts (6 tests) 327ms
 ✓ src/components/Settings/SettingsTab.test.tsx (18 tests) 417ms

 Test Files  7 passed (7)
      Tests  81 passed (81)
   Start at  06:25:53
   Duration  1.44s (transform 928ms, setup 729ms, import 1.76s, tests 1.05s, environment 3.94s)


> assistsupport@1.0.0 check:memorykernel-pin /Users/d/Projects/AssistSupport
> node scripts/validate_memorykernel_pin_sync.mjs

MemoryKernel pin + compatibility matrix + producer manifest validation passed.

> assistsupport@1.0.0 check:memorykernel-governance /Users/d/Projects/AssistSupport
> node scripts/validate_memorykernel_governance_bundle.mjs

MemoryKernel governance bundle validation passed.

> assistsupport@1.0.0 check:memorykernel-handoff /Users/d/Projects/AssistSupport
> node scripts/validate_memorykernel_handoff_payload.mjs

MemoryKernel handoff payload validation passed.
Wrote MemoryKernel handoff evidence: /Users/d/Projects/AssistSupport/artifacts/memorykernel-handoff-evidence.json

> assistsupport@1.0.0 check:memorykernel-handoff:service-v3-candidate /Users/d/Projects/AssistSupport
> ASSISTSUPPORT_REQUIRE_HANDOFF_PAYLOAD=1 ASSISTSUPPORT_HANDOFF_REQUIRE_PIN_MATCH=0 MEMORYKERNEL_EXPECTED_SERVICE_CONTRACT_VERSION=service.v3 node scripts/validate_memorykernel_handoff_payload.mjs

MemoryKernel handoff payload validation passed.
Wrote MemoryKernel handoff evidence: /Users/d/Projects/AssistSupport/artifacts/memorykernel-handoff-evidence.json

> assistsupport@1.0.0 test:memorykernel-contract /Users/d/Projects/AssistSupport
> pnpm run check:memorykernel-pin && pnpm run check:memorykernel-governance && pnpm run check:memorykernel-boundary && pnpm run check:memorykernel-cutover-policy && vitest run src/hooks/useInitialize.test.ts src/hooks/useMemoryKernelEnrichment.test.ts && (cd src-tauri && cargo test memory_kernel -- --nocapture) && node scripts/generate_memorykernel_contract_evidence.mjs


> assistsupport@1.0.0 check:memorykernel-pin /Users/d/Projects/AssistSupport
> node scripts/validate_memorykernel_pin_sync.mjs

MemoryKernel pin + compatibility matrix + producer manifest validation passed.

> assistsupport@1.0.0 check:memorykernel-governance /Users/d/Projects/AssistSupport
> node scripts/validate_memorykernel_governance_bundle.mjs

MemoryKernel governance bundle validation passed.

> assistsupport@1.0.0 check:memorykernel-boundary /Users/d/Projects/AssistSupport
> node scripts/validate_memorykernel_boundary.mjs

MemoryKernel boundary validation passed.

> assistsupport@1.0.0 check:memorykernel-cutover-policy /Users/d/Projects/AssistSupport
> node scripts/validate_memorykernel_cutover_policy.mjs

MemoryKernel cutover policy validation passed.

 RUN  v4.0.18 /Users/d/Projects/AssistSupport

 ✓ src/hooks/useMemoryKernelEnrichment.test.ts (3 tests) 9ms
 ✓ src/hooks/useInitialize.test.ts (6 tests) 336ms

 Test Files  2 passed (2)
      Tests  9 passed (9)
   Start at  06:25:56
   Duration  845ms (transform 68ms, setup 124ms, import 188ms, tests 345ms, environment 581ms)

    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.60s
     Running unittests src/lib.rs (target/debug/deps/assistsupport_lib-60b56174ca5b0ebf)

running 13 tests
test commands::memory_kernel::tests::normalize_machine_error_code_covers_service_v2_codes ... ok
test commands::memory_kernel::tests::build_enrichment_text_includes_answer_and_selected_items ... ok
test commands::memory_kernel::tests::disabled_feature_returns_disabled_preflight_template ... ok
test commands::memory_kernel::tests::query_ask_happy_path_applies_enrichment ... ok
test commands::memory_kernel::tests::preflight_reports_offline_when_service_down ... ok
test commands::memory_kernel::tests::query_ask_keeps_deterministic_fallback_on_non_2xx_response ... ok
test commands::memory_kernel::tests::preflight_detects_malformed_payload ... ok
test commands::memory_kernel::tests::query_ask_preserves_transitional_legacy_error_compatibility ... ok
test commands::memory_kernel::tests::query_ask_can_map_machine_error_codes_to_specific_consumer_states ... ok
test commands::memory_kernel::tests::preflight_reports_timeout_as_offline ... ok
test commands::memory_kernel::tests::preflight_detects_version_mismatch ... ok
test commands::memory_kernel::tests::query_ask_uses_deterministic_fallback_when_preflight_fails ... ok
test commands::memory_kernel::tests::query_ask_includes_machine_readable_error_code_in_fallback_message ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 267 filtered out; finished in 0.53s

     Running unittests src/main.rs (target/debug/deps/assistsupport-1a16d6aff4a28689)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/bin/assistsupport-cli.rs (target/debug/deps/assistsupport_cli-1ece82262c5b9d8b)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/command_contracts.rs (target/debug/deps/command_contracts-427a085bc3eeab57)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 7 filtered out; finished in 0.00s

     Running tests/data_migration.rs (target/debug/deps/data_migration-7fbc6291f1d1b1bc)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 15 filtered out; finished in 0.00s

     Running tests/db_encrypted_roundtrip.rs (target/debug/deps/db_encrypted_roundtrip-1de4d562d0458049)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.00s

     Running tests/filter_injection.rs (target/debug/deps/filter_injection-f5a63f419374d801)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 35 filtered out; finished in 0.00s

     Running tests/kb_disk_ingestion.rs (target/debug/deps/kb_disk_ingestion-9b4fee2f2e66c77e)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 9 filtered out; finished in 0.00s

     Running tests/kb_pipeline.rs (target/debug/deps/kb_pipeline-33f6426aef8ac0a2)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 20 filtered out; finished in 0.00s

     Running tests/namespace_consistency.rs (target/debug/deps/namespace_consistency-65d37485ab8ac123)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 21 filtered out; finished in 0.00s

     Running tests/path_validation.rs (target/debug/deps/path_validation-831202e639dfcda3)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 18 filtered out; finished in 0.00s

     Running tests/security.rs (target/debug/deps/security-5535dcdbea0f1b5b)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 36 filtered out; finished in 0.00s

     Running tests/ssrf_dns_rebinding.rs (target/debug/deps/ssrf_dns_rebinding-c0c453a80dd9adb5)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 24 filtered out; finished in 0.00s

Wrote MemoryKernel contract evidence: /Users/d/Projects/AssistSupport/artifacts/memorykernel-contract-evidence.json

> assistsupport@1.0.0 test:memorykernel-phase3-dry-run /Users/d/Projects/AssistSupport
> ASSISTSUPPORT_REQUIRE_HANDOFF_PAYLOAD=1 pnpm run check:memorykernel-handoff && pnpm run test:memorykernel-contract


> assistsupport@1.0.0 check:memorykernel-handoff /Users/d/Projects/AssistSupport
> node scripts/validate_memorykernel_handoff_payload.mjs

MemoryKernel handoff payload validation passed.
Wrote MemoryKernel handoff evidence: /Users/d/Projects/AssistSupport/artifacts/memorykernel-handoff-evidence.json

> assistsupport@1.0.0 test:memorykernel-contract /Users/d/Projects/AssistSupport
> pnpm run check:memorykernel-pin && pnpm run check:memorykernel-governance && pnpm run check:memorykernel-boundary && pnpm run check:memorykernel-cutover-policy && vitest run src/hooks/useInitialize.test.ts src/hooks/useMemoryKernelEnrichment.test.ts && (cd src-tauri && cargo test memory_kernel -- --nocapture) && node scripts/generate_memorykernel_contract_evidence.mjs


> assistsupport@1.0.0 check:memorykernel-pin /Users/d/Projects/AssistSupport
> node scripts/validate_memorykernel_pin_sync.mjs

MemoryKernel pin + compatibility matrix + producer manifest validation passed.

> assistsupport@1.0.0 check:memorykernel-governance /Users/d/Projects/AssistSupport
> node scripts/validate_memorykernel_governance_bundle.mjs

MemoryKernel governance bundle validation passed.

> assistsupport@1.0.0 check:memorykernel-boundary /Users/d/Projects/AssistSupport
> node scripts/validate_memorykernel_boundary.mjs

MemoryKernel boundary validation passed.

> assistsupport@1.0.0 check:memorykernel-cutover-policy /Users/d/Projects/AssistSupport
> node scripts/validate_memorykernel_cutover_policy.mjs

MemoryKernel cutover policy validation passed.

 RUN  v4.0.18 /Users/d/Projects/AssistSupport

 ✓ src/hooks/useMemoryKernelEnrichment.test.ts (3 tests) 8ms
 ✓ src/hooks/useInitialize.test.ts (6 tests) 334ms

 Test Files  2 passed (2)
      Tests  9 passed (9)
   Start at  06:26:01
   Duration  840ms (transform 69ms, setup 111ms, import 185ms, tests 342ms, environment 587ms)

    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.48s
     Running unittests src/lib.rs (target/debug/deps/assistsupport_lib-60b56174ca5b0ebf)

running 13 tests
test commands::memory_kernel::tests::normalize_machine_error_code_covers_service_v2_codes ... ok
test commands::memory_kernel::tests::build_enrichment_text_includes_answer_and_selected_items ... ok
test commands::memory_kernel::tests::disabled_feature_returns_disabled_preflight_template ... ok
test commands::memory_kernel::tests::query_ask_keeps_deterministic_fallback_on_non_2xx_response ... ok
test commands::memory_kernel::tests::preflight_reports_offline_when_service_down ... ok
test commands::memory_kernel::tests::preflight_detects_version_mismatch ... ok
test commands::memory_kernel::tests::query_ask_uses_deterministic_fallback_when_preflight_fails ... ok
test commands::memory_kernel::tests::query_ask_includes_machine_readable_error_code_in_fallback_message ... ok
test commands::memory_kernel::tests::query_ask_can_map_machine_error_codes_to_specific_consumer_states ... ok
test commands::memory_kernel::tests::query_ask_preserves_transitional_legacy_error_compatibility ... ok
test commands::memory_kernel::tests::query_ask_happy_path_applies_enrichment ... ok
test commands::memory_kernel::tests::preflight_detects_malformed_payload ... ok
test commands::memory_kernel::tests::preflight_reports_timeout_as_offline ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 267 filtered out; finished in 0.26s

     Running unittests src/main.rs (target/debug/deps/assistsupport-1a16d6aff4a28689)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/bin/assistsupport-cli.rs (target/debug/deps/assistsupport_cli-1ece82262c5b9d8b)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/command_contracts.rs (target/debug/deps/command_contracts-427a085bc3eeab57)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 7 filtered out; finished in 0.00s

     Running tests/data_migration.rs (target/debug/deps/data_migration-7fbc6291f1d1b1bc)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 15 filtered out; finished in 0.00s

     Running tests/db_encrypted_roundtrip.rs (target/debug/deps/db_encrypted_roundtrip-1de4d562d0458049)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.00s

     Running tests/filter_injection.rs (target/debug/deps/filter_injection-f5a63f419374d801)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 35 filtered out; finished in 0.00s

     Running tests/kb_disk_ingestion.rs (target/debug/deps/kb_disk_ingestion-9b4fee2f2e66c77e)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 9 filtered out; finished in 0.00s

     Running tests/kb_pipeline.rs (target/debug/deps/kb_pipeline-33f6426aef8ac0a2)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 20 filtered out; finished in 0.00s

     Running tests/namespace_consistency.rs (target/debug/deps/namespace_consistency-65d37485ab8ac123)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 21 filtered out; finished in 0.00s

     Running tests/path_validation.rs (target/debug/deps/path_validation-831202e639dfcda3)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 18 filtered out; finished in 0.00s

     Running tests/security.rs (target/debug/deps/security-5535dcdbea0f1b5b)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 36 filtered out; finished in 0.00s

     Running tests/ssrf_dns_rebinding.rs (target/debug/deps/ssrf_dns_rebinding-c0c453a80dd9adb5)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 24 filtered out; finished in 0.00s

Wrote MemoryKernel contract evidence: /Users/d/Projects/AssistSupport/artifacts/memorykernel-contract-evidence.json

> assistsupport@1.0.0 test:memorykernel-cutover-dry-run /Users/d/Projects/AssistSupport
> pnpm run check:memorykernel-handoff:service-v3-candidate && pnpm run check:memorykernel-pin && pnpm run test:memorykernel-contract && pnpm run test:ci


> assistsupport@1.0.0 check:memorykernel-handoff:service-v3-candidate /Users/d/Projects/AssistSupport
> ASSISTSUPPORT_REQUIRE_HANDOFF_PAYLOAD=1 ASSISTSUPPORT_HANDOFF_REQUIRE_PIN_MATCH=0 MEMORYKERNEL_EXPECTED_SERVICE_CONTRACT_VERSION=service.v3 node scripts/validate_memorykernel_handoff_payload.mjs

MemoryKernel handoff payload validation passed.
Wrote MemoryKernel handoff evidence: /Users/d/Projects/AssistSupport/artifacts/memorykernel-handoff-evidence.json

> assistsupport@1.0.0 check:memorykernel-pin /Users/d/Projects/AssistSupport
> node scripts/validate_memorykernel_pin_sync.mjs

MemoryKernel pin + compatibility matrix + producer manifest validation passed.

> assistsupport@1.0.0 test:memorykernel-contract /Users/d/Projects/AssistSupport
> pnpm run check:memorykernel-pin && pnpm run check:memorykernel-governance && pnpm run check:memorykernel-boundary && pnpm run check:memorykernel-cutover-policy && vitest run src/hooks/useInitialize.test.ts src/hooks/useMemoryKernelEnrichment.test.ts && (cd src-tauri && cargo test memory_kernel -- --nocapture) && node scripts/generate_memorykernel_contract_evidence.mjs


> assistsupport@1.0.0 check:memorykernel-pin /Users/d/Projects/AssistSupport
> node scripts/validate_memorykernel_pin_sync.mjs

MemoryKernel pin + compatibility matrix + producer manifest validation passed.

> assistsupport@1.0.0 check:memorykernel-governance /Users/d/Projects/AssistSupport
> node scripts/validate_memorykernel_governance_bundle.mjs

MemoryKernel governance bundle validation passed.

> assistsupport@1.0.0 check:memorykernel-boundary /Users/d/Projects/AssistSupport
> node scripts/validate_memorykernel_boundary.mjs

MemoryKernel boundary validation passed.

> assistsupport@1.0.0 check:memorykernel-cutover-policy /Users/d/Projects/AssistSupport
> node scripts/validate_memorykernel_cutover_policy.mjs

MemoryKernel cutover policy validation passed.

 RUN  v4.0.18 /Users/d/Projects/AssistSupport

 ✓ src/hooks/useMemoryKernelEnrichment.test.ts (3 tests) 10ms
 ✓ src/hooks/useInitialize.test.ts (6 tests) 328ms

 Test Files  2 passed (2)
      Tests  9 passed (9)
   Start at  06:26:04
   Duration  847ms (transform 69ms, setup 123ms, import 177ms, tests 338ms, environment 612ms)

    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.48s
     Running unittests src/lib.rs (target/debug/deps/assistsupport_lib-60b56174ca5b0ebf)

running 13 tests
test commands::memory_kernel::tests::normalize_machine_error_code_covers_service_v2_codes ... ok
test commands::memory_kernel::tests::build_enrichment_text_includes_answer_and_selected_items ... ok
test commands::memory_kernel::tests::disabled_feature_returns_disabled_preflight_template ... ok
test commands::memory_kernel::tests::preflight_reports_offline_when_service_down ... ok
test commands::memory_kernel::tests::query_ask_can_map_machine_error_codes_to_specific_consumer_states ... ok
test commands::memory_kernel::tests::query_ask_includes_machine_readable_error_code_in_fallback_message ... ok
test commands::memory_kernel::tests::query_ask_happy_path_applies_enrichment ... ok
test commands::memory_kernel::tests::preflight_detects_version_mismatch ... ok
test commands::memory_kernel::tests::query_ask_preserves_transitional_legacy_error_compatibility ... ok
test commands::memory_kernel::tests::preflight_detects_malformed_payload ... ok
test commands::memory_kernel::tests::query_ask_uses_deterministic_fallback_when_preflight_fails ... ok
test commands::memory_kernel::tests::query_ask_keeps_deterministic_fallback_on_non_2xx_response ... ok
test commands::memory_kernel::tests::preflight_reports_timeout_as_offline ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 267 filtered out; finished in 0.26s

     Running unittests src/main.rs (target/debug/deps/assistsupport-1a16d6aff4a28689)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/bin/assistsupport-cli.rs (target/debug/deps/assistsupport_cli-1ece82262c5b9d8b)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/command_contracts.rs (target/debug/deps/command_contracts-427a085bc3eeab57)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 7 filtered out; finished in 0.00s

     Running tests/data_migration.rs (target/debug/deps/data_migration-7fbc6291f1d1b1bc)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 15 filtered out; finished in 0.00s

     Running tests/db_encrypted_roundtrip.rs (target/debug/deps/db_encrypted_roundtrip-1de4d562d0458049)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.00s

     Running tests/filter_injection.rs (target/debug/deps/filter_injection-f5a63f419374d801)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 35 filtered out; finished in 0.00s

     Running tests/kb_disk_ingestion.rs (target/debug/deps/kb_disk_ingestion-9b4fee2f2e66c77e)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 9 filtered out; finished in 0.00s

     Running tests/kb_pipeline.rs (target/debug/deps/kb_pipeline-33f6426aef8ac0a2)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 20 filtered out; finished in 0.00s

     Running tests/namespace_consistency.rs (target/debug/deps/namespace_consistency-65d37485ab8ac123)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 21 filtered out; finished in 0.00s

     Running tests/path_validation.rs (target/debug/deps/path_validation-831202e639dfcda3)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 18 filtered out; finished in 0.00s

     Running tests/security.rs (target/debug/deps/security-5535dcdbea0f1b5b)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 36 filtered out; finished in 0.00s

     Running tests/ssrf_dns_rebinding.rs (target/debug/deps/ssrf_dns_rebinding-c0c453a80dd9adb5)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 24 filtered out; finished in 0.00s

Wrote MemoryKernel contract evidence: /Users/d/Projects/AssistSupport/artifacts/memorykernel-contract-evidence.json

> assistsupport@1.0.0 test:ci /Users/d/Projects/AssistSupport
> vitest run && cd src-tauri && cargo test


 RUN  v4.0.18 /Users/d/Projects/AssistSupport

 ✓ src/hooks/useMemoryKernelEnrichment.test.ts (3 tests) 9ms
 ✓ src/hooks/useLlm.test.ts (9 tests) 19ms
 ✓ src/hooks/useKb.test.ts (10 tests) 19ms
 ✓ src/components/Draft/ResponsePanel.test.tsx (13 tests) 108ms
 ✓ src/components/Draft/InputPanel.test.tsx (22 tests) 165ms
 ✓ src/hooks/useInitialize.test.ts (6 tests) 327ms
 ✓ src/components/Settings/SettingsTab.test.tsx (18 tests) 451ms

 Test Files  7 passed (7)
      Tests  81 passed (81)
   Start at  06:26:06
   Duration  1.31s (transform 811ms, setup 573ms, import 1.68s, tests 1.10s, environment 3.04s)

    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.56s
     Running unittests src/lib.rs (target/debug/deps/assistsupport_lib-60b56174ca5b0ebf)

running 280 tests
test commands::memory_kernel::tests::normalize_machine_error_code_covers_service_v2_codes ... ok
test commands::memory_kernel::tests::disabled_feature_returns_disabled_preflight_template ... ok
test commands::memory_kernel::tests::build_enrichment_text_includes_answer_and_selected_items ... ok
test backup::tests::sanitize_imported_setting_accepts_non_kb_keys ... ok
test audit::tests::test_flush_attempt_throttling ... ok
test backup::tests::sanitize_imported_setting_rejects_missing_kb_path ... ok
test audit::tests::test_severity_display ... ok
test backup::tests::sanitize_imported_setting_rejects_outside_home_kb_path ... ok
test audit::tests::test_log_audit_without_init_returns_error ... ok
test audit::tests::test_error_callback_set_and_clear ... ok
test audit::tests::test_ring_buffer_add_and_capacity ... ok
test audit::tests::test_audit_entry_serialization ... ok
test audit::tests::test_audit_entry_with_context ... ok
test commands::search_api::tests::classify_health_response_accepts_valid_json_health ... ok
test commands::search_api::tests::classify_health_response_reports_http_errors ... ok
test commands::search_api::tests::feedback_rating_validation_accepts_only_known_values ... ok
test commands::search_api::tests::submit_feedback_rejects_invalid_rating_before_network_call ... ok
test commands::search_api::tests::sanitize_top_k_applies_defaults_and_bounds ... ok
test commands::search_api::tests::classify_health_response_detects_wrong_service_html ... ok
test commands::search_api::tests::stats_response_defaults_missing_feedback_fields_to_zero ... ok
test commands::memory_kernel::tests::preflight_detects_malformed_payload ... ok
test commands::memory_kernel::tests::preflight_detects_version_mismatch ... ok
test db::tests::test_fts5_available ... ok
test db::executor::tests::test_executor_insert_and_query ... ok
test db::executor::tests::test_executor_basic_query ... ok
test db::tests::test_database_creation ... ok
test db::tests::test_fts5_triggers ... ok
test db::tests::test_fts5_indexing ... ok
test commands::memory_kernel::tests::preflight_reports_timeout_as_offline ... ok
test commands::memory_kernel::tests::preflight_reports_offline_when_service_down ... ok
test diagnostics::tests::test_component_health_constructors ... ok
test diagnostics::tests::test_failure_modes ... ok
test diagnostics::tests::test_filesystem_health ... ok
test diagnostics::tests::test_health_status_worst ... ok
test downloads::tests::test_model_source_url ... ok
test downloads::tests::test_recommended_models ... ok
test error::tests::test_error_display ... ok
test error::tests::test_error_retryable ... ok
test error::tests::test_error_serialization ... ok
test error::tests::test_error_with_detail ... ok
test exports::tests::test_format_for_clipboard ... ok
test commands::memory_kernel::tests::query_ask_can_map_machine_error_codes_to_specific_consumer_states ... ok
test exports::tests::test_format_plaintext ... ok
test exports::tests::test_format_html ... ok
test exports::tests::test_safe_export_strips_emails ... ok
test exports::tests::test_safe_export_strips_uuids ... ok
test exports::tests::test_safe_export_strips_paths ... ok
test feedback::tests::test_detect_query_category ... ok
test commands::memory_kernel::tests::query_ask_happy_path_applies_enrichment ... ok
test commands::memory_kernel::tests::query_ask_includes_machine_readable_error_code_in_fallback_message ... ok
test commands::memory_kernel::tests::query_ask_keeps_deterministic_fallback_on_non_2xx_response ... ok
test commands::memory_kernel::tests::query_ask_preserves_transitional_legacy_error_compatibility ... ok
test commands::memory_kernel::tests::query_ask_uses_deterministic_fallback_when_preflight_fails ... ok
test jira::tests::test_parse_description_adf ... ok
test jira::tests::test_parse_description_null ... ok
test jira::tests::test_parse_description_string ... ok
test jobs::tests::test_cancellation_token ... ok
test jobs::tests::test_job_manager ... ok
test jobs::tests::test_job_status_display ... ok
test jobs::tests::test_job_status_terminal ... ok
test jobs::tests::test_job_type_display ... ok
test kb::dns::tests::test_build_ip_url_ipv4 ... ok
test kb::dns::tests::test_build_ip_url_ipv6 ... ok
test kb::dns::tests::test_build_ip_url_nonstandard_port ... ok
test kb::dns::tests::test_pinned_host_storage ... ok
test kb::dns::tests::test_pinned_resolver_blocks_private_ips ... ok
test kb::dns::tests::test_validated_url_socket_addrs ... ok
test kb::docx::tests::test_extract_nonexistent ... ok
test kb::embeddings::tests::test_cosine_similarity ... ok
test kb::embeddings::tests::test_engine_creation ... ok
test kb::embeddings::tests::test_normalize_embedding ... ok
test kb::indexer::tests::test_chunking ... ok
test kb::indexer::tests::test_document_type_detection ... ok
test kb::indexer::tests::test_file_hash ... ok
test kb::indexer::tests::test_file_size_limit_constants ... ok
test kb::indexer::tests::test_folder_scan ... ok
test db::tests::test_job_counts ... ok
test db::tests::test_list_jobs_by_status ... ok
test kb::indexer::tests::test_max_file_size_by_type ... ok
test kb::indexer::tests::test_symlink_detection ... ok
test kb::indexer::tests::test_markdown_parsing ... ok
test kb::indexer::tests::test_symlink_directories_skipped_in_scan ... ok
test kb::indexer::tests::test_symlink_files_skipped_in_scan ... ok
test kb::ingest::batch::tests::test_batch_source_parse_invalid ... ok
test kb::ingest::batch::tests::test_batch_source_parse_web ... ok
test kb::ingest::batch::tests::test_batch_source_parse_youtube ... ok
test kb::indexer::tests::test_symlink_rejected_in_parse ... ok
test db::tests::test_job_crud ... ok
test db::tests::test_job_logs ... ok
test db::tests::test_vector_consent ... ok
test db::tests::test_sqlcipher_encryption ... ok
test kb::ingest::github::tests::test_extract_code_headings_python ... ok
test kb::ingest::github::tests::test_extract_code_headings_rust ... ok
test kb::ingest::github::tests::test_is_git_repo ... ok
test kb::ingest::github::tests::test_parse_https_repo_url ... ok
test kb::ingest::github::tests::test_parse_https_repo_url_rejects_http ... ok
test kb::ingest::github::tests::test_repo_name ... ok
test kb::ingest::web::tests::test_build_ip_url ... ok
test kb::ingest::web::tests::test_dns_pinning_allows_public_ip ... ok
test kb::ingest::web::tests::test_dns_pinning_blocks_private_ip ... ok
test kb::ingest::web::tests::test_dns_pinning_returns_pinned_ips ... ok
test kb::ingest::web::tests::test_extract_headings ... ok
test kb::ingest::web::tests::test_extract_html_title ... ok
test kb::ingest::web::tests::test_html_entities_decode ... ok
test kb::ingest::web::tests::test_html_to_text ... ok
test kb::ingest::youtube::tests::test_extract_video_id ... ok
test kb::ingest::youtube::tests::test_parse_json3_transcript_basic ... ok
test kb::ingest::youtube::tests::test_parse_plain_transcript_vtt ... ok
test kb::network::tests::test_allowlist ... ok
test kb::network::tests::test_canonicalize_url ... ok
test kb::network::tests::test_extract_same_origin_links ... ok
test kb::network::tests::test_ipv4_compatible_ipv6_blocked ... ok
test kb::network::tests::test_ipv6_mapped_ipv4_blocked ... ok
test kb::network::tests::test_is_ip_blocked ... ok
test kb::network::tests::test_is_login_page ... ok
test kb::network::tests::test_is_private_ipv4 ... ok
test kb::network::tests::test_validate_url_allowed ... ok
test kb::network::tests::test_validate_url_blocked ... ok
test kb::ocr::tests::test_ocr_manager_creation ... ok
test kb::ocr::tests::test_vision_availability ... ok
test db::tests::test_wrong_key_fails ... ok
test kb::pdf::tests::test_pdf_extractor_creation ... ok
test kb::pdf::tests::test_pdf_page_count ... ok
test kb::pdf::tests::test_pdf_text_extraction ... ok
test kb::pdf::tests::test_pdfium_availability ... ok
test kb::search::tests::test_content_similarity ... ok
test feedback::tests::test_feedback_clamps_ratings ... ok
test kb::search::tests::test_deduplication ... ok
test kb::search::tests::test_format_context ... ok
test kb::search::tests::test_is_mixed_alphanumeric ... ok
test kb::search::tests::test_is_policy_result_case_insensitive ... ok
test kb::search::tests::test_is_policy_result_policies_path ... ok
test kb::search::tests::test_is_policy_result_procedures_path ... ok
test kb::search::tests::test_is_policy_result_reference_path ... ok
test kb::search::tests::test_non_policy_query_how_to ... ok
test kb::search::tests::test_policy_boost_all_denied_variants ... ok
test kb::search::tests::test_policy_boost_does_not_affect_non_policy_queries ... ok
test kb::search::tests::test_policy_boost_empty_results ... ok
test kb::search::tests::test_policy_boost_multiple_policies_both_boosted ... ok
test kb::search::tests::test_policy_boost_preserves_all_results ... ok
test kb::search::tests::test_policy_boost_no_exceptions_queries ... ok
test kb::search::tests::test_policy_boost_promotes_policy_results ... ok
test kb::search::tests::test_policy_boost_score_increase ... ok
test kb::search::tests::test_policy_confidence_capped_at_one ... ok
test kb::search::tests::test_policy_confidence_high_for_forbidden_query ... ok
test kb::search::tests::test_policy_confidence_low_for_procedure ... ok
test kb::search::tests::test_policy_confidence_moderate_for_policy_keyword ... ok
test kb::search::tests::test_policy_query_can_we ... ok
test kb::search::tests::test_policy_query_allowed_check ... ok
test kb::search::tests::test_policy_query_external_usb ... ok
test kb::search::tests::test_policy_query_flash_drive_request ... ok
test kb::search::tests::test_policy_query_forbidden_keyword ... ok
test kb::search::tests::test_policy_query_permitted_keyword ... ok
test kb::search::tests::test_policy_query_policy_keyword ... ok
test kb::search::tests::test_policy_query_rules_check ... ok
test kb::search::tests::test_policy_query_usb_stick ... ok
test kb::search::tests::test_post_process_applies_policy_boost ... ok
test kb::search::tests::test_post_process_no_boost_without_query ... ok
test kb::search::tests::test_preprocess_empty_query ... ok
test kb::search::tests::test_preprocess_mixed_alphanumeric ... ok
test kb::search::tests::test_preprocess_multi_word ... ok
test kb::search::tests::test_preprocess_simple_word ... ok
test kb::search::tests::test_preprocess_strips_double_quotes ... ok
test kb::search::tests::test_rrf_fusion ... ok
test kb::search::tests::test_sanitize_snippet ... ok
test kb::search::tests::test_score_normalization ... ok
test kb::search::tests::test_search_options_builder ... ok
test kb::search::tests::test_search_options_default_no_query_text ... ok
test kb::search::tests::test_search_options_with_query_text ... ok
test kb::search::tests::test_simhash_hamming_distance ... ok
test kb::search::tests::test_split_mixed_alphanumeric_basic ... ok
test kb::search::tests::test_split_mixed_alphanumeric_multiple ... ok
test kb::search::tests::test_simhash_similarity ... ok
test kb::search::tests::test_split_mixed_alphanumeric_trailing_digits ... ok
test kb::search::tests::test_weighted_rrf ... ok
test kb::vectors::tests::test_enable_requires_consent ... ok
test kb::vectors::tests::test_encryption_not_supported ... ok
test kb::vectors::tests::test_encryption_status ... ok
test kb::vectors::tests::test_sanitize_filter_value_case_insensitive_keywords ... ok
test kb::vectors::tests::test_sanitize_filter_value_blocks_injection ... ok
test kb::vectors::tests::test_sanitize_filter_value_escapes_quotes ... ok
test kb::vectors::tests::test_sanitize_filter_value_preserves_unicode ... ok
test kb::vectors::tests::test_sanitize_filter_value_unicode_confusables ... ok
test kb::vectors::tests::test_sanitize_filter_value_valid ... ok
test kb::vectors::tests::test_sanitize_id_blocks_injection ... ok
test kb::vectors::tests::test_sanitize_filter_value_word_boundary_keywords ... ok
test kb::vectors::tests::test_sanitize_id_length_limits ... ok
test kb::vectors::tests::test_sanitize_id_rejects_special_chars ... ok
test kb::vectors::tests::test_sanitize_id_valid ... ok
test kb::vectors::tests::test_unicode_confusable_detection ... ok
test kb::vectors::tests::test_vector_store_disabled_by_default ... ok
test kb::watcher::tests::test_watcher_creation ... ok
test kb::watcher::tests::test_watcher_nonexistent_path ... ok
test kb::watcher::tests::test_count_files ... ok
test llm::tests::test_engine_creation ... ok
test llm::tests::test_generation_params_default ... ok
test llm::tests::test_model_load_and_generate ... ignored
test kb::xlsx::tests::test_extract_nonexistent ... ok
test migration::tests::test_has_data_dir_with_contents ... ok
test migration::tests::test_has_data_empty_dir ... ok
test migration::tests::test_has_data_nonexistent ... ok
test migration::tests::test_has_data_with_file ... ok
test model_integrity::tests::test_allowlist_lookup ... ok
test model_integrity::tests::test_calculate_sha256 ... ok
test model_integrity::tests::test_strict_mode_rejects_unverified ... ok
test migration::tests::test_copy_dir_recursive ... ok
test prompts::tests::test_basic_prompt_builder ... ok
test prompts::tests::test_citation_policy_in_prompt ... ok
test model_integrity::tests::test_verification_unverified_model ... ok
test prompts::tests::test_context_truncation_tracking ... ok
test prompts::tests::test_golden_set_context_completeness ... ok
test prompts::tests::test_golden_set_kb_context_isolation ... ok
test prompts::tests::test_golden_set_prompt_structure ... ok
test prompts::tests::test_kb_context_formatting ... ok
test prompts::tests::test_max_kb_results_limit ... ok
test prompts::tests::test_ocr_context ... ok
test prompts::tests::test_policy_context_with_kb_results ... ok
test prompts::tests::test_policy_enforcement_alternatives_required ... ok
test prompts::tests::test_policy_enforcement_forbid_rule ... ok
test prompts::tests::test_policy_enforcement_no_exceptions ... ok
test prompts::tests::test_policy_enforcement_no_workarounds ... ok
test prompts::tests::test_policy_enforcement_section_exists ... ok
test prompts::tests::test_priority_score_calculation ... ok
test prompts::tests::test_prompt_metadata ... ok
test prompts::tests::test_context_budget_enforcement ... ok
test prompts::tests::test_prompt_perspective_framing ... ok
test prompts::tests::test_prompt_template_name_updated ... ok
test prompts::tests::test_prompt_version_bumped ... ok
test prompts::tests::test_prompt_versioning ... ok
test prompts::tests::test_response_length_target_words ... ok
test prompts::tests::test_response_length ... ok
test prompts::tests::test_sanitize_for_context ... ok
test prompts::tests::test_source_tracking ... ok
test prompts::tests::test_sanitize_injection_patterns ... ok
test prompts::tests::test_untrusted_fencing ... ok
test security::tests::file_key_store_tests::test_master_key_hex_roundtrip ... ok
test security::tests::file_key_store_tests::test_token_update_preserves_others ... ok
test security::tests::file_key_store_tests::test_token_delete ... ok
test security::tests::file_key_store_tests::test_tokens_json_format ... ok
test feedback::export::tests::test_export_to_csv ... ok
test security::tests::test_encryption_roundtrip ... ok
test feedback::tests::test_submit_feedback ... ok
test security::tests::property_wrong_key_cannot_decrypt_random_payloads ... ok
test security::tests::test_master_key_generation ... ok
test feedback::tests::test_feedback_validates_log_exists ... ok
test feedback::tests::test_log_query ... ok
test sources::parser::tests::test_parse_empty_namespace ... ok
test sources::parser::tests::test_parse_invalid_url ... ok
test sources::parser::tests::test_parse_minimal_source_file ... ok
test sources::parser::tests::test_parse_valid_source_file ... ok
test sources::parser::tests::test_source_type_display ... ok
test tests::test_keychain_available ... ok
test validation::tests::test_is_sensitive_path ... ok
test validation::tests::test_normalize_and_validate_namespace_id ... ok
test validation::tests::test_normalize_namespace_id_truncation ... ok
test validation::tests::test_normalize_namespace_id ... ok
test validation::tests::test_validate_namespace_id_invalid ... ok
test validation::tests::test_validate_namespace_id_valid ... ok
test validation::tests::test_validate_non_empty ... ok
test validation::tests::test_validate_path_within ... ok
test validation::tests::test_validate_text_size_ok ... ok
test validation::tests::test_validate_text_size_too_large ... ok
test validation::tests::test_validate_url ... ok
test validation::tests::test_validate_ticket_id ... ok
test validation::tests::test_validate_within_home_allows_normal_dirs ... ok
test validation::tests::test_validate_within_home_blocks_sensitive_dirs ... ok
test validation::tests::test_validate_within_home_path_traversal ... ok
test validation::tests::test_validate_within_home_blocks_system_paths ... ok
test feedback::tests::test_pilot_stats ... ok
test security::tests::property_encrypt_decrypt_roundtrip_random_payloads ... ok
test kb::indexer::tests::test_index_and_search_integration ... ok
test kb::ingest::disk::tests::test_disk_ingest_basic ... ok
test kb::ingest::disk::tests::test_disk_ingest_reindexes_changed_file ... ok
test kb::ingest::disk::tests::test_disk_ingest_creates_source_and_run ... ok
test kb::ingest::disk::tests::test_disk_ingest_incremental_skips_unchanged ... ok
test kb::ingest::disk::tests::test_disk_ingest_sets_namespace_and_source_on_docs ... ok
test kb::ocr::tests::test_vision_ocr_integration ... ok
test security::tests::test_export_crypto ... ok
test security::tests::test_key_wrapping ... ok
test security::tests::test_wrong_passphrase_fails ... ok

test result: ok. 279 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 3.06s

     Running unittests src/main.rs (target/debug/deps/assistsupport-1a16d6aff4a28689)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/bin/assistsupport-cli.rs (target/debug/deps/assistsupport_cli-1ece82262c5b9d8b)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/command_contracts.rs (target/debug/deps/command_contracts-427a085bc3eeab57)

running 7 tests
test export_format_deserialization_contract ... ok
test process_ocr_bytes_rejects_oversized_payload ... ok
test backup_summary_json_contracts ... ok
test generate_with_context_result_json_contract ... ok
test jira_contract_types_serialize_with_expected_keys ... ok
test search_api_response_contract_roundtrip ... ok
test submit_search_feedback_rejects_invalid_rating_before_network ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/data_migration.rs (target/debug/deps/data_migration-7fbc6291f1d1b1bc)

running 15 tests
test test_has_data_on_nonexistent_path ... ok
test test_has_data_on_file ... ok
test test_migration_noop_when_old_dir_missing ... ok
test test_has_data_on_empty_directory ... ok
test test_has_data_on_directory_with_contents ... ok
test test_migration_noop_when_already_migrated ... ok
test test_migration_removes_empty_old_directory ... ok
test test_migration_moves_database ... ok
test test_migration_detects_conflict ... ok
test test_migration_skips_items_not_in_old ... ok
test test_migration_keeps_old_directory_if_not_empty ... ok
test test_migration_handles_mixed_conflicts ... ok
test test_migration_moves_vectors_directory ... ok
test test_migration_preserves_nested_structure ... ok
test test_migration_moves_all_items ... ok

test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/db_encrypted_roundtrip.rs (target/debug/deps/db_encrypted_roundtrip-1de4d562d0458049)

running 2 tests
test encrypted_database_rejects_wrong_key ... ok
test encrypted_database_roundtrip_persists_data_with_correct_key ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s

     Running tests/filter_injection.rs (target/debug/deps/filter_injection-f5a63f419374d801)

running 35 tests
test test_blocks_curly_quotes ... ok
test test_allows_numbers ... ok
test test_backslash_escape ... ok
test test_allows_unicode_text ... ok
test test_blocks_tautology ... ok
test test_blocks_boolean_injection ... ok
test test_blocks_unicode_hyphens ... ok
test test_blocks_modifier_apostrophe ... ok
test test_blocks_sql_select ... ok
test test_blocks_sql_union ... ok
test test_blocks_fullwidth_quotes ... ok
test test_blocks_comment_injection ... ok
test test_blocks_sql_modify ... ok
test test_allows_simple_values ... ok
test test_blocks_unicode_operators ... ok
test test_case_insensitive_keywords ... ok
test test_case_insensitive_operators ... ok
test test_empty_value ... ok
test test_encoded_attacks ... ok
test test_escapes_straight_quotes ... ok
test test_greek_question_mark_normalizes ... ok
test test_handles_partial_keywords ... ok
test test_id_allows_alphanumeric ... ok
test test_id_allows_hyphens_underscores ... ok
test test_id_blocks_empty ... ok
test test_id_blocks_injection_attempts ... ok
test test_id_blocks_special_chars ... ok
test test_id_blocks_too_long ... ok
test test_id_blocks_unicode ... ok
test test_null_byte_injection ... ok
test test_second_order_injection ... ok
test test_stacked_queries ... ok
test test_unicode_normalization_nfc ... ok
test test_whitespace_only ... ok
test test_very_long_values ... ok

test result: ok. 35 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/kb_disk_ingestion.rs (target/debug/deps/kb_disk_ingestion-9b4fee2f2e66c77e)

running 9 tests
test common::tests::test_generate_large_document ... ok
test common::tests::test_context_creation ... ok
test common::tests::test_file_creation ... ok
test test_procedure_query_returns_procedure_first ... ok
test test_policy_boost_with_real_indexed_data ... ok
test test_incremental_reindex_skips_unchanged ... ok
test test_disk_ingest_creates_documents_and_chunks ... ok
test test_policy_query_returns_policy_first ... ok
test test_full_pipeline_policy_enforcement ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.24s

     Running tests/kb_pipeline.rs (target/debug/deps/kb_pipeline-33f6426aef8ac0a2)

running 20 tests
test common::tests::test_generate_large_document ... ok
test test_e2e_multi_format_support ... ok
test common::tests::test_context_creation ... ok
test common::tests::test_file_creation ... ok
test test_basic_ingest_index_search ... ok
test test_markdown_with_code_blocks ... ok
test test_default_namespace ... ok
test test_empty_directory ... ok
test test_indexing_with_progress ... ok
test test_e2e_incremental_reindex ... ok
test test_document_update_re_index ... ok
test test_document_removal ... ok
test test_namespace_isolation ... ok
test test_e2e_kb_ingest_search_workflow ... ok
test test_large_document_chunking ... ok
test test_special_characters_in_search ... ok
test test_unicode_content ... ok
test test_text_file_indexing ... ok
test test_nested_directories ... ok
test test_search_returns_relevance_order ... ok

test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.46s

     Running tests/namespace_consistency.rs (target/debug/deps/namespace_consistency-65d37485ab8ac123)

running 21 tests
test test_complex_mixed_input ... ok
test test_emoji_stripped ... ok
test test_empty_input ... ok
test test_german_eszett ... ok
test test_leading_hyphen_removed ... ok
test test_all_normalization_paths_produce_same_output ... ok
test test_length_truncation ... ok
test test_multiple_hyphens_collapsed ... ok
test test_numbers_preserved ... ok
test test_normalized_output_is_valid ... ok
test test_only_hyphens ... ok
test test_path_like_input ... ok
test test_special_chars_only ... ok
test test_normalization_is_idempotent ... ok
test test_trailing_hyphen_removed ... ok
test test_truncation_preserves_word_boundary ... ok
test test_turkish_i_case_folding ... ok
test test_unicode_confusables ... ok
test test_unicode_stripped_correctly ... ok
test test_url_like_input ... ok
test test_whitespace_only ... ok

test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/path_validation.rs (target/debug/deps/path_validation-831202e639dfcda3)

running 18 tests
test test_gnupg_directory_blocked ... ok
test test_home_directory_itself ... ok
test test_auto_create_nested_fails_without_parent ... ok
test test_aws_directory_blocked ... ok
test test_path_traversal_with_dotdot ... ok
test test_config_directory_blocked ... ok
test test_keychains_directory_blocked ... ok
test test_auto_create_in_sensitive_location_blocked ... ok
test test_case_sensitivity ... ok
test test_auto_create_new_directory ... ok
test test_home_directory_valid_paths ... ok
test test_ssh_directory_blocked ... ok
test test_system_paths_blocked ... ok
test test_symlink_traversal_blocked ... ok
test common::tests::test_generate_large_document ... ok
test common::tests::test_context_creation ... ok
test test_path_with_spaces ... ok
test common::tests::test_file_creation ... ok

test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s

     Running tests/security.rs (target/debug/deps/security-5535dcdbea0f1b5b)

running 36 tests
test test_audit_severity_levels ... ok
test test_audit_event_types ... ok
test test_audit_entry_with_context ... ok
test test_audit_entry_serialization ... ok
test test_encryption_produces_different_ciphertext ... ok
test test_key_storage_mode_serialization ... ok
test test_encryption_with_wrong_key_fails ... ok
test test_encryption_roundtrip ... ok
test test_encryption_empty_plaintext ... ok
test common::tests::test_generate_large_document ... ok
test test_master_key_from_bytes ... ok
test test_master_key_generation_unique ... ok
test test_master_key_hex_encoding ... ok
test test_master_key_length ... ok
test test_ssrf_allowlist_bypass ... ok
test test_ssrf_blocks_invalid_schemes ... ok
test test_ssrf_blocks_localhost_variants ... ok
test test_ssrf_blocks_private_ranges ... ok
test test_ssrf_metadata_endpoints ... ok
test test_token_constants ... ok
test test_token_encryption_with_different_keys ... ok
test test_validate_https_url ... ok
test test_file_key_store_has_master_key_false_initially ... ok
test common::tests::test_context_creation ... ok
test test_database_integrity_check ... ok
test test_database_wrong_key_fails ... ok
test common::tests::test_file_creation ... ok
test test_database_encrypted_on_disk ... ok
test test_encryption_large_data ... ok
test test_wrapped_key_components ... ok
test test_export_crypto_wrong_password_fails ... ok
test test_key_wrapping_with_passphrase ... ok
test test_key_wrapping_wrong_passphrase_fails ... ok
test test_export_crypto_roundtrip ... ok
test test_key_wrapping_produces_different_output ... ok
test test_key_wrapping_passphrase_change_simulation ... ok

test result: ok. 36 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.29s

     Running tests/ssrf_dns_rebinding.rs (target/debug/deps/ssrf_dns_rebinding-c0c453a80dd9adb5)

running 24 tests
test test_allows_ipv4_mapped_public ... ok
test test_allows_public_ipv4 ... ok
test test_allows_public_ipv6 ... ok
test test_blocks_10_network ... ok
test test_blocks_192_168_network ... ok
test test_blocks_172_network ... ok
test test_blocks_broadcast ... ok
test test_blocks_documentation_ranges ... ok
test test_blocks_ipv4_mapped_localhost ... ok
test test_blocks_ipv4_mapped_private_10 ... ok
test test_blocks_ipv4_mapped_private_172 ... ok
test test_blocks_ipv4_mapped_private_192 ... ok
test test_blocks_link_local_v4 ... ok
test test_blocks_link_local_v6 ... ok
test test_blocks_localhost_v4 ... ok
test test_blocks_localhost_v6 ... ok
test test_blocks_multicast ... ok
test test_blocks_reserved_ranges ... ok
test test_blocks_unique_local_v6 ... ok
test test_blocks_zero_address ... ok
test test_dns_rebinding_ipv6_variant ... ok
test test_dns_rebinding_scenario ... ok
test test_dns_returns_mixed_ips ... ok
test test_private_network_boundaries ... ok

test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests assistsupport_lib

running 2 tests
test src/db/executor.rs - db::executor (line 9) ... ignored
test src/validation.rs - validation::normalize_namespace_id (line 49) ... ok

test result: ok. 1 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 2.32s


> assistsupport@1.0.0 test:ci /Users/d/Projects/AssistSupport
> vitest run && cd src-tauri && cargo test


 RUN  v4.0.18 /Users/d/Projects/AssistSupport

 ✓ src/hooks/useMemoryKernelEnrichment.test.ts (3 tests) 9ms
 ✓ src/hooks/useLlm.test.ts (9 tests) 18ms
 ✓ src/hooks/useKb.test.ts (10 tests) 18ms
 ✓ src/components/Draft/ResponsePanel.test.tsx (13 tests) 107ms
 ✓ src/components/Draft/InputPanel.test.tsx (22 tests) 161ms
 ✓ src/hooks/useInitialize.test.ts (6 tests) 328ms
 ✓ src/components/Settings/SettingsTab.test.tsx (18 tests) 442ms

 Test Files  7 passed (7)
      Tests  81 passed (81)
   Start at  06:26:22
   Duration  1.37s (transform 794ms, setup 614ms, import 1.66s, tests 1.08s, environment 3.50s)

    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.25s
     Running unittests src/lib.rs (target/debug/deps/assistsupport_lib-60b56174ca5b0ebf)

running 280 tests
test backup::tests::sanitize_imported_setting_accepts_non_kb_keys ... ok
test backup::tests::sanitize_imported_setting_rejects_missing_kb_path ... ok
test audit::tests::test_severity_display ... ok
test commands::memory_kernel::tests::normalize_machine_error_code_covers_service_v2_codes ... ok
test audit::tests::test_error_callback_set_and_clear ... ok
test audit::tests::test_flush_attempt_throttling ... ok
test audit::tests::test_log_audit_without_init_returns_error ... ok
test audit::tests::test_ring_buffer_add_and_capacity ... ok
test backup::tests::sanitize_imported_setting_rejects_outside_home_kb_path ... ok
test commands::memory_kernel::tests::disabled_feature_returns_disabled_preflight_template ... ok
test commands::memory_kernel::tests::build_enrichment_text_includes_answer_and_selected_items ... ok
test commands::search_api::tests::classify_health_response_accepts_valid_json_health ... ok
test commands::search_api::tests::classify_health_response_reports_http_errors ... ok
test audit::tests::test_audit_entry_serialization ... ok
test audit::tests::test_audit_entry_with_context ... ok
test commands::search_api::tests::feedback_rating_validation_accepts_only_known_values ... ok
test commands::search_api::tests::classify_health_response_detects_wrong_service_html ... ok
test commands::search_api::tests::submit_feedback_rejects_invalid_rating_before_network_call ... ok
test commands::search_api::tests::sanitize_top_k_applies_defaults_and_bounds ... ok
test commands::search_api::tests::stats_response_defaults_missing_feedback_fields_to_zero ... ok
test commands::memory_kernel::tests::query_ask_can_map_machine_error_codes_to_specific_consumer_states ... ok
test commands::memory_kernel::tests::query_ask_includes_machine_readable_error_code_in_fallback_message ... ok
test commands::memory_kernel::tests::query_ask_preserves_transitional_legacy_error_compatibility ... ok
test commands::memory_kernel::tests::preflight_detects_version_mismatch ... ok
test db::executor::tests::test_executor_basic_query ... ok
test db::executor::tests::test_executor_insert_and_query ... ok
test db::tests::test_fts5_available ... ok
test db::tests::test_database_creation ... ok
test db::tests::test_job_crud ... ok
test db::tests::test_fts5_triggers ... ok
test diagnostics::tests::test_component_health_constructors ... ok
test diagnostics::tests::test_failure_modes ... ok
test diagnostics::tests::test_filesystem_health ... ok
test diagnostics::tests::test_health_status_worst ... ok
test downloads::tests::test_model_source_url ... ok
test downloads::tests::test_recommended_models ... ok
test error::tests::test_error_display ... ok
test error::tests::test_error_retryable ... ok
test error::tests::test_error_serialization ... ok
test error::tests::test_error_with_detail ... ok
test exports::tests::test_format_for_clipboard ... ok
test db::tests::test_fts5_indexing ... ok
test exports::tests::test_format_plaintext ... ok
test exports::tests::test_format_html ... ok
test exports::tests::test_safe_export_strips_emails ... ok
test exports::tests::test_safe_export_strips_uuids ... ok
test exports::tests::test_safe_export_strips_paths ... ok
test feedback::tests::test_detect_query_category ... ok
test db::tests::test_job_counts ... ok
test commands::memory_kernel::tests::preflight_reports_timeout_as_offline ... ok
test commands::memory_kernel::tests::preflight_reports_offline_when_service_down ... ok
test commands::memory_kernel::tests::query_ask_uses_deterministic_fallback_when_preflight_fails ... ok
test commands::memory_kernel::tests::query_ask_happy_path_applies_enrichment ... ok
test jira::tests::test_parse_description_adf ... ok
test jira::tests::test_parse_description_null ... ok
test jira::tests::test_parse_description_string ... ok
test jobs::tests::test_cancellation_token ... ok
test jobs::tests::test_job_manager ... ok
test jobs::tests::test_job_status_display ... ok
test jobs::tests::test_job_status_terminal ... ok
test jobs::tests::test_job_type_display ... ok
test kb::dns::tests::test_build_ip_url_ipv4 ... ok
test kb::dns::tests::test_build_ip_url_ipv6 ... ok
test kb::dns::tests::test_build_ip_url_nonstandard_port ... ok
test kb::dns::tests::test_pinned_host_storage ... ok
test commands::memory_kernel::tests::query_ask_keeps_deterministic_fallback_on_non_2xx_response ... ok
test kb::dns::tests::test_validated_url_socket_addrs ... ok
test kb::docx::tests::test_extract_nonexistent ... ok
test kb::embeddings::tests::test_cosine_similarity ... ok
test commands::memory_kernel::tests::preflight_detects_malformed_payload ... ok
test kb::embeddings::tests::test_normalize_embedding ... ok
test kb::indexer::tests::test_chunking ... ok
test kb::embeddings::tests::test_engine_creation ... ok
test kb::indexer::tests::test_document_type_detection ... ok
test kb::indexer::tests::test_file_size_limit_constants ... ok
test kb::indexer::tests::test_file_hash ... ok
test kb::indexer::tests::test_folder_scan ... ok
test kb::dns::tests::test_pinned_resolver_blocks_private_ips ... ok
test kb::indexer::tests::test_max_file_size_by_type ... ok
test kb::indexer::tests::test_symlink_detection ... ok
test kb::indexer::tests::test_symlink_directories_skipped_in_scan ... ok
test kb::indexer::tests::test_markdown_parsing ... ok
test kb::indexer::tests::test_symlink_files_skipped_in_scan ... ok
test kb::ingest::batch::tests::test_batch_source_parse_invalid ... ok
test kb::indexer::tests::test_symlink_rejected_in_parse ... ok
test kb::ingest::batch::tests::test_batch_source_parse_web ... ok
test kb::ingest::batch::tests::test_batch_source_parse_youtube ... ok
test db::tests::test_list_jobs_by_status ... ok
test db::tests::test_job_logs ... ok
test db::tests::test_vector_consent ... ok
test db::tests::test_sqlcipher_encryption ... ok
test kb::ingest::github::tests::test_extract_code_headings_python ... ok
test kb::ingest::github::tests::test_extract_code_headings_rust ... ok
test kb::ingest::github::tests::test_is_git_repo ... ok
test kb::ingest::github::tests::test_parse_https_repo_url ... ok
test kb::ingest::github::tests::test_parse_https_repo_url_rejects_http ... ok
test kb::ingest::github::tests::test_repo_name ... ok
test kb::ingest::web::tests::test_build_ip_url ... ok
test kb::ingest::web::tests::test_dns_pinning_allows_public_ip ... ok
test kb::ingest::web::tests::test_dns_pinning_blocks_private_ip ... ok
test kb::ingest::web::tests::test_dns_pinning_returns_pinned_ips ... ok
test kb::ingest::web::tests::test_extract_headings ... ok
test kb::ingest::web::tests::test_extract_html_title ... ok
test kb::ingest::web::tests::test_html_entities_decode ... ok
test kb::ingest::web::tests::test_html_to_text ... ok
test kb::ingest::youtube::tests::test_extract_video_id ... ok
test kb::ingest::youtube::tests::test_parse_json3_transcript_basic ... ok
test kb::ingest::youtube::tests::test_parse_plain_transcript_vtt ... ok
test kb::network::tests::test_allowlist ... ok
test kb::network::tests::test_canonicalize_url ... ok
test kb::network::tests::test_extract_same_origin_links ... ok
test kb::network::tests::test_ipv4_compatible_ipv6_blocked ... ok
test kb::network::tests::test_ipv6_mapped_ipv4_blocked ... ok
test kb::network::tests::test_is_ip_blocked ... ok
test kb::network::tests::test_is_login_page ... ok
test kb::network::tests::test_is_private_ipv4 ... ok
test kb::network::tests::test_validate_url_allowed ... ok
test kb::network::tests::test_validate_url_blocked ... ok
test kb::ocr::tests::test_ocr_manager_creation ... ok
test kb::ocr::tests::test_vision_availability ... ok
test db::tests::test_wrong_key_fails ... ok
test kb::pdf::tests::test_pdf_extractor_creation ... ok
test kb::pdf::tests::test_pdf_page_count ... ok
test kb::pdf::tests::test_pdf_text_extraction ... ok
test kb::pdf::tests::test_pdfium_availability ... ok
test kb::search::tests::test_content_similarity ... ok
test kb::search::tests::test_deduplication ... ok
test kb::search::tests::test_format_context ... ok
test kb::search::tests::test_is_mixed_alphanumeric ... ok
test kb::search::tests::test_is_policy_result_case_insensitive ... ok
test kb::search::tests::test_is_policy_result_policies_path ... ok
test kb::search::tests::test_is_policy_result_procedures_path ... ok
test kb::search::tests::test_is_policy_result_reference_path ... ok
test kb::search::tests::test_non_policy_query_how_to ... ok
test kb::search::tests::test_policy_boost_all_denied_variants ... ok
test kb::search::tests::test_policy_boost_does_not_affect_non_policy_queries ... ok
test kb::search::tests::test_policy_boost_empty_results ... ok
test kb::search::tests::test_policy_boost_multiple_policies_both_boosted ... ok
test kb::search::tests::test_policy_boost_no_exceptions_queries ... ok
test kb::search::tests::test_policy_boost_preserves_all_results ... ok
test kb::search::tests::test_policy_boost_promotes_policy_results ... ok
test kb::search::tests::test_policy_boost_score_increase ... ok
test kb::search::tests::test_policy_confidence_capped_at_one ... ok
test kb::search::tests::test_policy_confidence_high_for_forbidden_query ... ok
test kb::search::tests::test_policy_confidence_low_for_procedure ... ok
test kb::search::tests::test_policy_confidence_moderate_for_policy_keyword ... ok
test kb::search::tests::test_policy_query_allowed_check ... ok
test kb::search::tests::test_policy_query_can_we ... ok
test kb::search::tests::test_policy_query_external_usb ... ok
test kb::search::tests::test_policy_query_flash_drive_request ... ok
test kb::search::tests::test_policy_query_forbidden_keyword ... ok
test kb::search::tests::test_policy_query_permitted_keyword ... ok
test kb::search::tests::test_policy_query_policy_keyword ... ok
test kb::search::tests::test_policy_query_rules_check ... ok
test kb::search::tests::test_policy_query_usb_stick ... ok
test kb::search::tests::test_post_process_applies_policy_boost ... ok
test kb::search::tests::test_post_process_no_boost_without_query ... ok
test kb::search::tests::test_preprocess_empty_query ... ok
test kb::search::tests::test_preprocess_mixed_alphanumeric ... ok
test kb::search::tests::test_preprocess_multi_word ... ok
test kb::search::tests::test_preprocess_simple_word ... ok
test kb::search::tests::test_preprocess_strips_double_quotes ... ok
test kb::search::tests::test_rrf_fusion ... ok
test kb::search::tests::test_sanitize_snippet ... ok
test kb::search::tests::test_score_normalization ... ok
test kb::search::tests::test_search_options_builder ... ok
test kb::search::tests::test_search_options_default_no_query_text ... ok
test kb::search::tests::test_search_options_with_query_text ... ok
test kb::search::tests::test_simhash_hamming_distance ... ok
test kb::search::tests::test_simhash_similarity ... ok
test kb::search::tests::test_split_mixed_alphanumeric_basic ... ok
test kb::search::tests::test_split_mixed_alphanumeric_multiple ... ok
test kb::search::tests::test_split_mixed_alphanumeric_trailing_digits ... ok
test kb::search::tests::test_weighted_rrf ... ok
test kb::vectors::tests::test_enable_requires_consent ... ok
test kb::vectors::tests::test_encryption_not_supported ... ok
test kb::vectors::tests::test_encryption_status ... ok
test kb::vectors::tests::test_sanitize_filter_value_blocks_injection ... ok
test kb::vectors::tests::test_sanitize_filter_value_case_insensitive_keywords ... ok
test kb::vectors::tests::test_sanitize_filter_value_escapes_quotes ... ok
test kb::vectors::tests::test_sanitize_filter_value_preserves_unicode ... ok
test kb::vectors::tests::test_sanitize_filter_value_unicode_confusables ... ok
test kb::vectors::tests::test_sanitize_filter_value_valid ... ok
test kb::vectors::tests::test_sanitize_filter_value_word_boundary_keywords ... ok
test kb::vectors::tests::test_sanitize_id_blocks_injection ... ok
test kb::vectors::tests::test_sanitize_id_length_limits ... ok
test kb::vectors::tests::test_sanitize_id_rejects_special_chars ... ok
test kb::vectors::tests::test_sanitize_id_valid ... ok
test kb::vectors::tests::test_unicode_confusable_detection ... ok
test kb::vectors::tests::test_vector_store_disabled_by_default ... ok
test kb::watcher::tests::test_count_files ... ok
test kb::watcher::tests::test_watcher_creation ... ok
test kb::watcher::tests::test_watcher_nonexistent_path ... ok
test kb::xlsx::tests::test_extract_nonexistent ... ok
test llm::tests::test_engine_creation ... ok
test llm::tests::test_generation_params_default ... ok
test llm::tests::test_model_load_and_generate ... ignored
test feedback::tests::test_feedback_validates_log_exists ... ok
test migration::tests::test_has_data_dir_with_contents ... ok
test migration::tests::test_copy_dir_recursive ... ok
test migration::tests::test_has_data_nonexistent ... ok
test feedback::tests::test_feedback_clamps_ratings ... ok
test model_integrity::tests::test_allowlist_lookup ... ok
test migration::tests::test_has_data_with_file ... ok
test migration::tests::test_has_data_empty_dir ... ok
test model_integrity::tests::test_calculate_sha256 ... ok
test prompts::tests::test_basic_prompt_builder ... ok
test prompts::tests::test_citation_policy_in_prompt ... ok
test prompts::tests::test_context_budget_enforcement ... ok
test prompts::tests::test_context_truncation_tracking ... ok
test prompts::tests::test_golden_set_context_completeness ... ok
test prompts::tests::test_golden_set_kb_context_isolation ... ok
test prompts::tests::test_golden_set_prompt_structure ... ok
test prompts::tests::test_kb_context_formatting ... ok
test prompts::tests::test_max_kb_results_limit ... ok
test prompts::tests::test_ocr_context ... ok
test prompts::tests::test_policy_context_with_kb_results ... ok
test model_integrity::tests::test_verification_unverified_model ... ok
test prompts::tests::test_policy_enforcement_alternatives_required ... ok
test prompts::tests::test_policy_enforcement_forbid_rule ... ok
test model_integrity::tests::test_strict_mode_rejects_unverified ... ok
test prompts::tests::test_policy_enforcement_no_exceptions ... ok
test prompts::tests::test_policy_enforcement_no_workarounds ... ok
test prompts::tests::test_policy_enforcement_section_exists ... ok
test prompts::tests::test_priority_score_calculation ... ok
test prompts::tests::test_prompt_metadata ... ok
test prompts::tests::test_prompt_perspective_framing ... ok
test prompts::tests::test_prompt_template_name_updated ... ok
test prompts::tests::test_prompt_version_bumped ... ok
test prompts::tests::test_prompt_versioning ... ok
test prompts::tests::test_response_length_target_words ... ok
test prompts::tests::test_response_length ... ok
test prompts::tests::test_sanitize_for_context ... ok
test prompts::tests::test_source_tracking ... ok
test prompts::tests::test_sanitize_injection_patterns ... ok
test prompts::tests::test_untrusted_fencing ... ok
test security::tests::file_key_store_tests::test_master_key_hex_roundtrip ... ok
test security::tests::file_key_store_tests::test_token_update_preserves_others ... ok
test security::tests::file_key_store_tests::test_token_delete ... ok
test security::tests::file_key_store_tests::test_tokens_json_format ... ok
test security::tests::test_encryption_roundtrip ... ok
test feedback::tests::test_pilot_stats ... ok
test feedback::export::tests::test_export_to_csv ... ok
test security::tests::test_master_key_generation ... ok
test security::tests::property_wrong_key_cannot_decrypt_random_payloads ... ok
test kb::ingest::disk::tests::test_disk_ingest_creates_source_and_run ... ok
test sources::parser::tests::test_parse_empty_namespace ... ok
test sources::parser::tests::test_parse_invalid_url ... ok
test sources::parser::tests::test_parse_minimal_source_file ... ok
test sources::parser::tests::test_parse_valid_source_file ... ok
test sources::parser::tests::test_source_type_display ... ok
test tests::test_keychain_available ... ok
test validation::tests::test_is_sensitive_path ... ok
test validation::tests::test_normalize_and_validate_namespace_id ... ok
test validation::tests::test_normalize_namespace_id ... ok
test validation::tests::test_normalize_namespace_id_truncation ... ok
test validation::tests::test_validate_namespace_id_invalid ... ok
test validation::tests::test_validate_namespace_id_valid ... ok
test validation::tests::test_validate_non_empty ... ok
test validation::tests::test_validate_path_within ... ok
test validation::tests::test_validate_text_size_too_large ... ok
test validation::tests::test_validate_text_size_ok ... ok
test validation::tests::test_validate_url ... ok
test validation::tests::test_validate_within_home_allows_normal_dirs ... ok
test validation::tests::test_validate_ticket_id ... ok
test validation::tests::test_validate_within_home_blocks_sensitive_dirs ... ok
test validation::tests::test_validate_within_home_path_traversal ... ok
test validation::tests::test_validate_within_home_blocks_system_paths ... ok
test feedback::tests::test_log_query ... ok
test kb::indexer::tests::test_index_and_search_integration ... ok
test feedback::tests::test_submit_feedback ... ok
test kb::ingest::disk::tests::test_disk_ingest_basic ... ok
test security::tests::property_encrypt_decrypt_roundtrip_random_payloads ... ok
test kb::ingest::disk::tests::test_disk_ingest_incremental_skips_unchanged ... ok
test kb::ingest::disk::tests::test_disk_ingest_sets_namespace_and_source_on_docs ... ok
test kb::ingest::disk::tests::test_disk_ingest_reindexes_changed_file ... ok
test kb::ocr::tests::test_vision_ocr_integration ... ok
test security::tests::test_export_crypto ... ok
test security::tests::test_key_wrapping ... ok
test security::tests::test_wrong_passphrase_fails ... ok

test result: ok. 279 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 3.11s

     Running unittests src/main.rs (target/debug/deps/assistsupport-1a16d6aff4a28689)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/bin/assistsupport-cli.rs (target/debug/deps/assistsupport_cli-1ece82262c5b9d8b)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/command_contracts.rs (target/debug/deps/command_contracts-427a085bc3eeab57)

running 7 tests
test export_format_deserialization_contract ... ok
test process_ocr_bytes_rejects_oversized_payload ... ok
test jira_contract_types_serialize_with_expected_keys ... ok
test backup_summary_json_contracts ... ok
test generate_with_context_result_json_contract ... ok
test search_api_response_contract_roundtrip ... ok
test submit_search_feedback_rejects_invalid_rating_before_network ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/data_migration.rs (target/debug/deps/data_migration-7fbc6291f1d1b1bc)

running 15 tests
test test_has_data_on_nonexistent_path ... ok
test test_has_data_on_file ... ok
test test_migration_noop_when_old_dir_missing ... ok
test test_has_data_on_empty_directory ... ok
test test_has_data_on_directory_with_contents ... ok
test test_migration_noop_when_already_migrated ... ok
test test_migration_detects_conflict ... ok
test test_migration_moves_database ... ok
test test_migration_removes_empty_old_directory ... ok
test test_migration_skips_items_not_in_old ... ok
test test_migration_handles_mixed_conflicts ... ok
test test_migration_keeps_old_directory_if_not_empty ... ok
test test_migration_moves_vectors_directory ... ok
test test_migration_preserves_nested_structure ... ok
test test_migration_moves_all_items ... ok

test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/db_encrypted_roundtrip.rs (target/debug/deps/db_encrypted_roundtrip-1de4d562d0458049)

running 2 tests
test encrypted_database_rejects_wrong_key ... ok
test encrypted_database_roundtrip_persists_data_with_correct_key ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

     Running tests/filter_injection.rs (target/debug/deps/filter_injection-f5a63f419374d801)

running 35 tests
test test_allows_numbers ... ok
test test_backslash_escape ... ok
test test_allows_simple_values ... ok
test test_blocks_comment_injection ... ok
test test_blocks_boolean_injection ... ok
test test_blocks_sql_select ... ok
test test_allows_unicode_text ... ok
test test_blocks_modifier_apostrophe ... ok
test test_blocks_fullwidth_quotes ... ok
test test_blocks_curly_quotes ... ok
test test_blocks_sql_modify ... ok
test test_blocks_sql_union ... ok
test test_blocks_tautology ... ok
test test_blocks_unicode_hyphens ... ok
test test_blocks_unicode_operators ... ok
test test_case_insensitive_keywords ... ok
test test_case_insensitive_operators ... ok
test test_empty_value ... ok
test test_encoded_attacks ... ok
test test_escapes_straight_quotes ... ok
test test_greek_question_mark_normalizes ... ok
test test_id_allows_alphanumeric ... ok
test test_handles_partial_keywords ... ok
test test_id_allows_hyphens_underscores ... ok
test test_id_blocks_empty ... ok
test test_id_blocks_injection_attempts ... ok
test test_id_blocks_special_chars ... ok
test test_id_blocks_too_long ... ok
test test_id_blocks_unicode ... ok
test test_null_byte_injection ... ok
test test_second_order_injection ... ok
test test_stacked_queries ... ok
test test_unicode_normalization_nfc ... ok
test test_whitespace_only ... ok
test test_very_long_values ... ok

test result: ok. 35 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/kb_disk_ingestion.rs (target/debug/deps/kb_disk_ingestion-9b4fee2f2e66c77e)

running 9 tests
test common::tests::test_generate_large_document ... ok
test common::tests::test_file_creation ... ok
test common::tests::test_context_creation ... ok
test test_procedure_query_returns_procedure_first ... ok
test test_policy_boost_with_real_indexed_data ... ok
test test_policy_query_returns_policy_first ... ok
test test_disk_ingest_creates_documents_and_chunks ... ok
test test_incremental_reindex_skips_unchanged ... ok
test test_full_pipeline_policy_enforcement ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.24s

     Running tests/kb_pipeline.rs (target/debug/deps/kb_pipeline-33f6426aef8ac0a2)

running 20 tests
test common::tests::test_generate_large_document ... ok
test test_namespace_isolation ... ok
test test_document_removal ... ok
test test_e2e_multi_format_support ... ok
test test_e2e_incremental_reindex ... ok
test common::tests::test_file_creation ... ok
test common::tests::test_context_creation ... ok
test test_markdown_with_code_blocks ... ok
test test_basic_ingest_index_search ... ok
test test_document_update_re_index ... ok
test test_indexing_with_progress ... ok
test test_default_namespace ... ok
test test_empty_directory ... ok
test test_e2e_kb_ingest_search_workflow ... ok
test test_large_document_chunking ... ok
test test_special_characters_in_search ... ok
test test_text_file_indexing ... ok
test test_unicode_content ... ok
test test_search_returns_relevance_order ... ok
test test_nested_directories ... ok

test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.43s

     Running tests/namespace_consistency.rs (target/debug/deps/namespace_consistency-65d37485ab8ac123)

running 21 tests
test test_complex_mixed_input ... ok
test test_german_eszett ... ok
test test_leading_hyphen_removed ... ok
test test_emoji_stripped ... ok
test test_empty_input ... ok
test test_length_truncation ... ok
test test_multiple_hyphens_collapsed ... ok
test test_all_normalization_paths_produce_same_output ... ok
test test_normalized_output_is_valid ... ok
test test_numbers_preserved ... ok
test test_only_hyphens ... ok
test test_normalization_is_idempotent ... ok
test test_path_like_input ... ok
test test_special_chars_only ... ok
test test_trailing_hyphen_removed ... ok
test test_truncation_preserves_word_boundary ... ok
test test_turkish_i_case_folding ... ok
test test_unicode_confusables ... ok
test test_unicode_stripped_correctly ... ok
test test_url_like_input ... ok
test test_whitespace_only ... ok

test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/path_validation.rs (target/debug/deps/path_validation-831202e639dfcda3)

running 18 tests
test test_config_directory_blocked ... ok
test test_auto_create_nested_fails_without_parent ... ok
test test_aws_directory_blocked ... ok
test test_home_directory_itself ... ok
test test_gnupg_directory_blocked ... ok
test test_auto_create_in_sensitive_location_blocked ... ok
test test_case_sensitivity ... ok
test test_keychains_directory_blocked ... ok
test test_home_directory_valid_paths ... ok
test test_path_traversal_with_dotdot ... ok
test test_auto_create_new_directory ... ok
test test_ssh_directory_blocked ... ok
test test_system_paths_blocked ... ok
test test_symlink_traversal_blocked ... ok
test common::tests::test_generate_large_document ... ok
test common::tests::test_context_creation ... ok
test test_path_with_spaces ... ok
test common::tests::test_file_creation ... ok

test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s

     Running tests/security.rs (target/debug/deps/security-5535dcdbea0f1b5b)

running 36 tests
test test_audit_entry_serialization ... ok
test test_audit_event_types ... ok
test test_audit_entry_with_context ... ok
test test_audit_severity_levels ... ok
test test_encryption_empty_plaintext ... ok
test test_encryption_produces_different_ciphertext ... ok
test test_encryption_roundtrip ... ok
test test_encryption_with_wrong_key_fails ... ok
test test_key_storage_mode_serialization ... ok
test common::tests::test_generate_large_document ... ok
test test_master_key_from_bytes ... ok
test test_master_key_generation_unique ... ok
test test_master_key_hex_encoding ... ok
test test_master_key_length ... ok
test test_ssrf_allowlist_bypass ... ok
test test_ssrf_blocks_invalid_schemes ... ok
test test_ssrf_blocks_localhost_variants ... ok
test test_ssrf_blocks_private_ranges ... ok
test test_ssrf_metadata_endpoints ... ok
test test_token_constants ... ok
test test_token_encryption_with_different_keys ... ok
test test_validate_https_url ... ok
test common::tests::test_file_creation ... ok
test common::tests::test_context_creation ... ok
test test_database_integrity_check ... ok
test test_database_wrong_key_fails ... ok
test test_file_key_store_has_master_key_false_initially ... ok
test test_database_encrypted_on_disk ... ok
test test_encryption_large_data ... ok
test test_wrapped_key_components ... ok
test test_key_wrapping_wrong_passphrase_fails ... ok
test test_export_crypto_roundtrip ... ok
test test_key_wrapping_with_passphrase ... ok
test test_export_crypto_wrong_password_fails ... ok
test test_key_wrapping_produces_different_output ... ok
test test_key_wrapping_passphrase_change_simulation ... ok

test result: ok. 36 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.31s

     Running tests/ssrf_dns_rebinding.rs (target/debug/deps/ssrf_dns_rebinding-c0c453a80dd9adb5)

running 24 tests
test test_allows_ipv4_mapped_public ... ok
test test_allows_public_ipv4 ... ok
test test_allows_public_ipv6 ... ok
test test_blocks_10_network ... ok
test test_blocks_192_168_network ... ok
test test_blocks_172_network ... ok
test test_blocks_broadcast ... ok
test test_blocks_documentation_ranges ... ok
test test_blocks_ipv4_mapped_localhost ... ok
test test_blocks_ipv4_mapped_private_10 ... ok
test test_blocks_ipv4_mapped_private_172 ... ok
test test_blocks_ipv4_mapped_private_192 ... ok
test test_blocks_link_local_v4 ... ok
test test_blocks_link_local_v6 ... ok
test test_blocks_localhost_v4 ... ok
test test_blocks_localhost_v6 ... ok
test test_blocks_multicast ... ok
test test_blocks_reserved_ranges ... ok
test test_blocks_unique_local_v6 ... ok
test test_blocks_zero_address ... ok
test test_dns_rebinding_ipv6_variant ... ok
test test_dns_rebinding_scenario ... ok
test test_dns_returns_mixed_ips ... ok
test test_private_network_boundaries ... ok

test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests assistsupport_lib

running 2 tests
test src/db/executor.rs - db::executor (line 9) ... ignored
test src/validation.rs - validation::normalize_namespace_id (line 49) ... ok

test result: ok. 1 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.88s

```

## Result
PASS
