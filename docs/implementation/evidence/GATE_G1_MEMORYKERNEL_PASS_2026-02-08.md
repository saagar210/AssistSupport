# Gate G1 MemoryKernel Baseline Evidence

- Timestamp (UTC): 2026-02-08T14:26:58Z
- Repo: /Users/d/Projects/MemoryKernel
- Branch: main
- Commit: 54da0d271da6b7dcad8e4a8411c1577e9ee988fa

## Commands
```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
./scripts/verify_service_contract_alignment.sh --memorykernel-root /Users/d/Projects/MemoryKernel
./scripts/verify_contract_parity.sh --canonical-root /Users/d/Projects/MemoryKernel
./scripts/verify_trilogy_compatibility_artifacts.sh --memorykernel-root /Users/d/Projects/MemoryKernel
./scripts/run_trilogy_smoke.sh --memorykernel-root /Users/d/Projects/MemoryKernel
./scripts/run_trilogy_compliance_suite.sh --memorykernel-root /Users/d/Projects/MemoryKernel --skip-baseline
./scripts/verify_producer_contract_manifest.sh --memorykernel-root /Users/d/Projects/MemoryKernel
./scripts/verify_producer_handoff_payload.sh --memorykernel-root /Users/d/Projects/MemoryKernel
```

## Output
```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.51s
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.32s
     Running unittests src/lib.rs (target/debug/deps/memory_kernel_api-c5a9b65741718a49)

running 3 tests
test tests::api_add_query_and_show_round_trip ... ok
test tests::api_recall_query_supports_mixed_summary_records ... ok
test tests::api_recall_query_defaults_to_non_constraint_record_types ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

     Running unittests src/main.rs (target/debug/deps/mk-aabb824e6493a4a5)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/cli_integration.rs (target/debug/deps/cli_integration-cdd1aff367031ced)

running 14 tests
test spec_docs_restrict_ambiguous_terms_to_mkr_027_exception_line ... ok
test governance_docs_exist_for_phase_checklists_and_policies ... ok
test integration_contract_schemas_validate_fixtures ... ok
test memory_link_rejects_non_ulid_version_ids ... ok
test memory_add_query_and_context_show_flow_is_consistent ... ok
test host_cli_exposes_outcome_command_tree ... ok
test key_outputs_match_golden_fixtures_after_normalization ... ok
test signed_snapshot_import_requires_and_validates_signature ... ok
test cli_outputs_validate_against_versioned_schemas ... ok
test encrypted_snapshot_round_trip_requires_explicit_decrypt_key ... ok
test trilogy_compatibility_artifacts_validate_against_memorykernel_consumer_requirements ... ok
test db_commands_cover_migrate_integrity_backup_restore_export_import ... ok
test trilogy_contract_parity_checker_matches_sibling_repos_when_available ... ok
test query_recall_returns_persisted_mixed_record_context_package ... ok

test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s

     Running unittests src/lib.rs (target/debug/deps/memory_kernel_core-b41fd0c86aae8174)

running 19 tests
test tests::retracted_constraints_are_excluded_with_reason ... ok
test tests::deterministic_sort_prefers_authority_then_truth_then_confidence ... ok
test tests::conflicting_top_precedence_constraints_return_inconclusive ... ok
test tests::context_items_include_memory_version_ids_for_selected_and_excluded ... ok
test tests::superseded_records_are_excluded ... ok
test tests::deterministic_tie_break_uses_memory_version_id ... ok
test tests::recall_defaults_to_non_constraint_record_types ... ok
test tests::recall_query_returns_mixed_records_with_explainable_exclusions ... ok
test tests::validate_rejects_inferred_without_confidence ... ok
test tests::context_package_json_is_stable_for_permuted_input ... ok
test tests::validate_rejects_invalid_source_hash_format ... ok
test tests::validate_rejects_missing_source_uri ... ok
test tests::validate_rejects_missing_justification ... ok
test tests::validate_rejects_missing_writer ... ok
test tests::recall_context_package_json_is_stable_for_permuted_mixed_input ... ok
test tests::property_policy_context_is_deterministic_under_seeded_permutations ... ok
test tests::policy_context_package_meets_baseline_budget ... ok
test tests::property_recall_context_is_deterministic_under_seeded_permutations ... ok
test tests::recall_context_package_meets_baseline_budget ... ok

test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s

     Running benches/resolver_bench.rs (target/debug/deps/resolver_bench-2be1e4135d1a2d3c)
Gnuplot not found, using plotters backend
Testing policy_context_package_1000_records
Success

Testing recall_context_package_1000_records
Success

     Running unittests src/lib.rs (target/debug/deps/memory_kernel_outcome_cli-97ce0fd0a755d9e6)

running 6 tests
test tests::parse_payload_accepts_valid_json ... ok
test tests::parse_payload_rejects_invalid_json ... ok
test tests::parse_optional_utc_rejects_non_utc ... ok
test tests::gate_json_contract_is_stable_v1 ... ok
test tests::stable_embed_api_host_path_stays_operational ... ok
test tests::cli_end_to_end_log_replay_show_and_gate_preview ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running unittests src/main.rs (target/debug/deps/mk_outcome-3913b4dda005f075)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/cli_contracts_v1.rs (target/debug/deps/cli_contracts_v1-7bef4ef3ce348af1)

running 5 tests
test outcome_help_contract_lists_expected_subcommands ... ok
test error_shape_for_missing_trust_snapshot_is_stable ... ok
test projector_json_contract_contains_versioned_payloads ... ok
test benchmark_command_exits_non_zero_on_threshold_violation ... ok
test benchmark_command_emits_contract_json_and_report_artifact ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

     Running tests/cli_snapshots_v1.rs (target/debug/deps/cli_snapshots_v1-3270e65f7b93610b)

running 3 tests
test snapshot_projector_status_json_v1 ... ok
test snapshot_projector_check_error_stderr_v1 ... ok
test snapshot_gate_preview_json_v1 ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

     Running tests/integration_contracts.rs (target/debug/deps/integration_contracts-02255129e29da6e4)

running 1 test
test integration_contract_pack_validates_fixtures ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/projector_smoke.rs (target/debug/deps/projector_smoke-29af087458552927)

running 1 test
test projector_check_smoke_after_replay ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

     Running unittests src/lib.rs (target/debug/deps/memory_kernel_outcome_core-8a247e15f94e15ba)

running 11 tests
test tests::as_of_decay_moves_toward_baseline ... ok
test tests::exploration_probe_bucket_is_deterministic ... ok
test tests::safe_gate_excludes_capped_items ... ok
test tests::manual_override_can_bypass_contradiction_cap ... ok
test tests::unknown_events_do_not_change_confidence ... ok
test tests::inheritance_resets_baseline_with_cap ... ok
test tests::contradiction_applies_cap_without_override ... ok
test tests::edited_success_uses_half_weight ... ok
test tests::manual_retire_is_sticky_until_promote ... ok
test tests::validated_requires_three_wins_and_zero_failures ... ok
test tests::manual_promote_requires_reearning_validation ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/lib.rs (target/debug/deps/memory_kernel_outcome_store_sqlite-7d3d49eb354508b4)

running 24 tests
test tests::migrate_fails_when_memory_records_table_missing ... ok
test tests::migrate_fails_without_unique_memory_id_version_key ... ok
test tests::migrate_fails_when_memory_records_version_column_missing ... ok
test tests::invalid_ruleset_json_is_reported_clearly ... ok
test tests::migrate_succeeds_with_explicit_unique_index_on_identity_columns ... ok
test tests::append_only_trigger_blocks_updates ... ok
test tests::invalid_event_timestamp_is_reported_clearly ... ok
test tests::projector_stale_keys_lists_out_of_date_identities ... ok
test tests::gate_preview_supports_safe_and_exploration_modes ... ok
test tests::golden_fixture_replay_snapshot_matches_expected ... ok
test tests::projector_status_and_check_report_lag_and_recovery ... ok
test tests::migration_is_idempotent_and_preserves_existing_data ... ok
test tests::schema_contract_contains_expected_tables_columns_and_triggers ... ok
test tests::replay_is_idempotent_on_repeated_partial_recovery ... ok
test tests::ready_to_integrate_gate_suite_passes ... ok
test tests::version_isolation_prevents_cross_version_trust_smear ... ok
test tests::mixed_ruleset_versions_replay_deterministically_without_rewrite ... ok
test tests::replay_projection_is_deterministic_incremental_vs_full ... ok
test tests::benchmark_harness_generates_report_and_respects_thresholds ... ok
test tests::sqlite_busy_timeout_allows_append_after_lock_release ... ok
test tests::long_stream_replay_stays_deterministic ... ok
test tests::scale_guardrails_append_and_replay_within_budget ... ok
test tests::prop_event_stream_replay_equivalence ... ok
test tests::prop_mixed_ruleset_replay_equivalence ... ok

test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.67s

     Running unittests src/main.rs (target/debug/deps/memory_kernel_service-27e82cd63389a943)

running 9 tests
test tests::openapi_endpoint_returns_versioned_artifact ... ok
test tests::health_endpoint_reports_ok ... ok
test tests::invalid_json_payload_returns_invalid_json_error ... ok
test tests::non_2xx_error_envelope_keeps_service_v3_shape ... ok
test tests::add_constraint_validation_failure_returns_validation_error ... ok
test tests::context_show_missing_returns_not_found_machine_error ... ok
test tests::duplicate_identity_returns_write_conflict ... ok
test tests::service_summary_add_and_recall_flow_round_trip ... ok
test tests::service_add_query_and_context_flow_round_trip ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s

     Running unittests src/lib.rs (target/debug/deps/memory_kernel_store_sqlite-60a7a45e84c0afc5)

running 13 tests
test tests::migrate_rejects_invalid_legacy_schema_without_identity_columns ... ok
test tests::schema_status_reports_pending_migration_for_legacy_v1 ... ok
test tests::integrity_check_reports_clean_database ... ok
test tests::duplicate_memory_id_version_is_rejected ... ok
test tests::sqlite_constraints_enforce_checks_and_foreign_keys ... ok
test tests::supersedes_links_round_trip_by_memory_version_id ... ok
test tests::write_and_read_constraint_round_trip ... ok
test tests::inserted_records_receive_distinct_memory_version_ids ... ok
test tests::migrate_legacy_v1_database_to_v2 ... ok
test tests::backup_and_restore_database_round_trip ... ok
test tests::import_rejects_manifest_digest_mismatch ... ok
test tests::export_and_import_snapshot_round_trip ... ok
test tests::concurrent_writes_and_reads_preserve_integrity ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s

     Running unittests src/main.rs (target/debug/deps/multi_agent_center_cli-f47c8098ede6fab9)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/integration_contracts.rs (target/debug/deps/integration_contracts-edc424b4ce356026)

running 3 tests
test trilogy_compatibility_artifact_v1_shape_and_content_is_valid ... ok
test integration_contract_v1_parity_matches_memorykernel_canonical_pack ... ok
test integration_contract_pack_validates_fixtures ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/run_with_memory_db_api.rs (target/debug/deps/run_with_memory_db_api-d0492733022fca8f)

running 2 tests
test replay_rerun_keeps_trust_memory_ref_identity_fields ... ok
test run_with_memory_db_uses_api_policy_and_recall_queries ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s

     Running unittests src/lib.rs (target/debug/deps/multi_agent_center_domain-6e137c7a6e5f742f)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/lib.rs (target/debug/deps/multi_agent_center_orchestrator-9ea69e2b0c5395ee)

running 11 tests
test tests::parse_recall_record_types_invalid_value_errors ... ok
test tests::step_context_query_recall_invalid_record_type_fails ... ok
test tests::provider_failure_marks_step_failed_instead_of_aborting_run ... ok
test tests::step_context_query_recall_missing_record_types_defaults_scope ... ok
test tests::step_context_query_recall_mode_uses_recall_resolver_rules ... ok
test tests::step_supports_multiple_context_packages ... ok
test tests::step_context_query_recall_empty_record_types_defaults_scope ... ok
test tests::step_context_query_default_mode_is_policy ... ok
test tests::trust_gate_decisions_are_persisted_per_memory_ref ... ok
test tests::dag_with_parallel_ready_steps_executes_successfully ... ok
test tests::api_memory_kernel_context_source_builds_step_packages ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s

     Running unittests src/lib.rs (target/debug/deps/multi_agent_center_policy-0f447617829ce9e3)

running 2 tests
test tests::prunes_disallowed_types_and_excess_items ... ok
test tests::empty_allowed_types_allows_all_records ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/lib.rs (target/debug/deps/multi_agent_center_provider-34f480f5ec495442)

running 2 tests
test tests::http_provider_requires_url ... ok
test tests::mock_provider_output_is_stable_for_same_input ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/lib.rs (target/debug/deps/multi_agent_center_trace_core-eb335276b0c1440b)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/lib.rs (target/debug/deps/multi_agent_center_trace_sqlite-7fdb2e02281ffbd6)

running 5 tests
test tests::trust_gate_memory_ref_requires_memory_version_id ... ok
test tests::migrate_is_idempotent_and_manifest_columns_exist ... ok
test tests::trace_events_are_append_only ... ok
test tests::context_package_round_trip_and_trust_gate_persist ... ok
test tests::migrate_keeps_legacy_rows_and_enforces_new_trust_memory_ref_identity ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running unittests src/lib.rs (target/debug/deps/multi_agent_center_workflow-e032e68456fe665c)

running 1 test
test tests::normalize_hash_is_stable ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

[check] service runtime version
[check] openapi version
[check] documented service version
[check] openapi includes structured error envelope
[check] openapi includes non-2xx responses on query endpoints
[check] error code taxonomy present
[check] ServiceErrorEnvelope policy invariants
[check] integration schemas use $id metadata
Service contract alignment checks passed.
[INFO] Comparing canonical contracts to OutcomeMemory
       canonical: /Users/d/Projects/MemoryKernel/contracts/integration/v1
       target:    /Users/d/Projects/MemoryKernel/components/outcome-memory/contracts/integration/v1
[INFO] Comparing canonical contracts to MultiAgentCenter
       canonical: /Users/d/Projects/MemoryKernel/contracts/integration/v1
       target:    /Users/d/Projects/MemoryKernel/components/multi-agent-center/contracts/integration/v1
Contract parity check passed.
[INFO] Validating OutcomeMemory artifact: /Users/d/Projects/MemoryKernel/components/outcome-memory/trilogy-compatibility.v1.json
[INFO] Validating MultiAgentCenter artifact: /Users/d/Projects/MemoryKernel/components/multi-agent-center/trilogy-compatibility.v1.json
Trilogy compatibility artifacts validated successfully.
[STEP] Contract parity check
[INFO] Comparing canonical contracts to OutcomeMemory
       canonical: /Users/d/Projects/MemoryKernel/contracts/integration/v1
       target:    /Users/d/Projects/MemoryKernel/components/outcome-memory/contracts/integration/v1
[INFO] Comparing canonical contracts to MultiAgentCenter
       canonical: /Users/d/Projects/MemoryKernel/contracts/integration/v1
       target:    /Users/d/Projects/MemoryKernel/components/multi-agent-center/contracts/integration/v1
Contract parity check passed.
[STEP] Trilogy compatibility artifact check
[INFO] Validating OutcomeMemory artifact: /Users/d/Projects/MemoryKernel/components/outcome-memory/trilogy-compatibility.v1.json
[INFO] Validating MultiAgentCenter artifact: /Users/d/Projects/MemoryKernel/components/multi-agent-center/trilogy-compatibility.v1.json
Trilogy compatibility artifacts validated successfully.
[STEP] MemoryKernel deterministic policy/recall smoke
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.13s
     Running `target/debug/mk --db /var/folders/gf/3t3h93q52d1fj7d_tldckr6r0000gn/T/tmp.w64DfdupfO/memorykernel.sqlite3 db migrate`
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.11s
     Running `target/debug/mk --db /var/folders/gf/3t3h93q52d1fj7d_tldckr6r0000gn/T/tmp.w64DfdupfO/memorykernel.sqlite3 memory add constraint --actor user --action use --resource usb_drive --effect deny --writer smoke --justification 'phase7 smoke' --source-uri 'file:///phase7/policy.md' --truth-status asserted --authority authoritative --confidence 0.95`
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.08s
     Running `target/debug/mk --db /var/folders/gf/3t3h93q52d1fj7d_tldckr6r0000gn/T/tmp.w64DfdupfO/memorykernel.sqlite3 memory add decision --summary 'Decision: USB usage requires approval' --writer smoke --justification 'phase7 smoke' --source-uri 'file:///phase7/decision.md' --truth-status observed --authority derived --confidence 0.80`
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.07s
     Running `target/debug/mk --db /var/folders/gf/3t3h93q52d1fj7d_tldckr6r0000gn/T/tmp.w64DfdupfO/memorykernel.sqlite3 query ask --text 'Am I allowed to use a USB drive?' --actor user --action use --resource usb_drive --as-of '2026-02-07T12:00:00Z'`
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.07s
     Running `target/debug/mk --db /var/folders/gf/3t3h93q52d1fj7d_tldckr6r0000gn/T/tmp.w64DfdupfO/memorykernel.clone.sqlite3 query ask --text 'Am I allowed to use a USB drive?' --actor user --action use --resource usb_drive --as-of '2026-02-07T12:00:00Z'`
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.08s
     Running `target/debug/mk --db /var/folders/gf/3t3h93q52d1fj7d_tldckr6r0000gn/T/tmp.w64DfdupfO/memorykernel.sqlite3 query recall --text 'usb approval' --as-of '2026-02-07T12:00:00Z'`
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.08s
     Running `target/debug/mk --db /var/folders/gf/3t3h93q52d1fj7d_tldckr6r0000gn/T/tmp.w64DfdupfO/memorykernel.clone.sqlite3 query recall --text 'usb approval' --as-of '2026-02-07T12:00:00Z'`
[STEP] Outcome host checks
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.08s
     Running `target/debug/mk --db /var/folders/gf/3t3h93q52d1fj7d_tldckr6r0000gn/T/tmp.w64DfdupfO/memorykernel.sqlite3 outcome projector status --json`
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.08s
     Running `target/debug/mk --db /var/folders/gf/3t3h93q52d1fj7d_tldckr6r0000gn/T/tmp.w64DfdupfO/memorykernel.sqlite3 outcome gate preview --mode safe --as-of '2026-02-07T12:00:00Z' --candidate '01KGYTC70GVYTRER2F4T4YCP8B:1' --json`
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.06s
     Running `target/debug/mk --db /var/folders/gf/3t3h93q52d1fj7d_tldckr6r0000gn/T/tmp.w64DfdupfO/memorykernel.sqlite3 outcome benchmark run --volume 100 --volume 500 --volume 2000 --repetitions 3 --append-p95-max-ms 8 --replay-p95-max-ms 250 --gate-p95-max-ms 8 --json`
[STEP] MultiAgentCenter API-backed context source smoke
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.16s
     Running `components/multi-agent-center/target/debug/multi-agent-center-cli run --workflow /Users/d/Projects/MemoryKernel/components/multi-agent-center/examples/workflow.memory.yaml --trace-db /var/folders/gf/3t3h93q52d1fj7d_tldckr6r0000gn/T/tmp.w64DfdupfO/multi-agent-center.trace.sqlite3 --memory-db /var/folders/gf/3t3h93q52d1fj7d_tldckr6r0000gn/T/tmp.w64DfdupfO/memorykernel.sqlite3 --non-interactive`
Trilogy smoke checks passed.
[suite] check_nist_80053.sh
[check] threat model present
[check] trust controls present
[check] append-only requirement documented
[check] deterministic query requirement documented
[check] contract parity requirement documented
[check] audit/replay surface documented
[suite] check_iso27001.sh
[check] versioning policy documented
[check] version bump rules enforced
[check] release gate documented
[check] closeout playbook documented
[check] change governance references integration v1 lock
[suite] check_soc2.sh
[check] ci workflow includes parity check
[check] ci workflow includes compatibility artifact check
[check] ci workflow includes service contract alignment check
[check] ci workflow includes producer manifest alignment check
[check] multi-agent trace architecture documented
[check] outcome integration handoff documented
[check] trilogy compatibility matrix documented
[suite] check_hipaa.sh
[check] encryption controls documented
[check] signature verification controls documented
[check] provenance requirements documented
[check] context package includes trust semantics
[check] audit/replay substrate present
[suite] check_pci_dss.sh
[check] ci enforces warning-free lint gate
[check] ci enforces benchmark threshold triplet
[check] trust-gate attachment fixture present
[check] outcome compatibility artifact exists
[check] multi-agent compatibility artifact exists
[suite] check_gdpr.sh
[check] append-only behavior requirement documented
[check] lineage requirements documented
[check] authority/confidence separation documented
[check] context package explainability requirement documented
[check] operations recovery runbook present
[suite] check_fisma_fedramp.sh
[check] trilogy closeout script present
[check] soak verification script present
[check] release gate references deterministic smoke requirement
[check] outcome threshold semantics locked in artifact
[check] fedramp external authorization caveat documented
Trilogy compliance suite passed.
[check] manifest JSON structure and policy fields
[check] manifest service/api versions align with runtime and docs
[check] manifest error code enum aligns with OpenAPI
[check] manifest policy aligns with service-v3 lifecycle docs
Producer contract manifest checks passed.
[check] generate stable handoff payload
[check] generate service-v3 candidate handoff payload
[check] payload schema and policy invariants
[check] policy text is documented in normative docs
Producer handoff payload checks passed.
```

## Result
PASS
