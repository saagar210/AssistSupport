import { mockIPC, mockWindows } from '@tauri-apps/api/mocks';

const kbStats = {
  document_count: 2,
  chunk_count: 8,
  total_words: 4200,
};

const kbDocuments = [
  {
    id: 'doc-1',
    file_path: '/mock/kb/remote-work-policy.md',
    title: 'Remote Work Policy',
    indexed_at: '2026-02-03T10:00:00Z',
    chunk_count: 4,
  },
  {
    id: 'doc-2',
    file_path: '/mock/kb/security-baseline.md',
    title: 'Security Baseline',
    indexed_at: '2026-02-03T10:05:00Z',
    chunk_count: 4,
  },
];

function mockSearchResults() {
  return [
    {
      chunk_id: 'chunk-1',
      document_id: 'doc-1',
      file_path: '/mock/kb/remote-work-policy.md',
      title: 'Remote Work Policy',
      heading_path: 'Policy > VPN',
      content: 'Use approved VPN and MFA when working remotely.',
      snippet: 'Use approved VPN and MFA when working remotely.',
      score: 0.95,
      source: 'Hybrid',
      namespace_id: 'default',
      source_type: 'file',
    },
  ];
}

export function setupE2eTauriMock(): void {
  const deploymentArtifacts = [
    {
      id: 'artifact-1',
      artifact_type: 'app_bundle',
      version: '1.0.0',
      channel: 'stable',
      sha256: 'abc123',
      is_signed: true,
      created_at: '2026-02-03T10:00:00Z',
    },
  ];

  const deploymentRuns = [
    {
      id: 'run-1',
      target_channel: 'stable',
      status: 'succeeded',
      preflight_json: JSON.stringify(['Database integrity: pass', 'Model status: loaded']),
      rollback_available: true,
      created_at: '2026-02-03T10:00:00Z',
      completed_at: '2026-02-03T10:01:00Z',
    },
  ];

  const evalRuns = [
    {
      id: 'eval-1',
      suite_name: 'ops-regression-suite',
      total_cases: 2,
      passed_cases: 2,
      avg_confidence: 0.76,
      details_json: '[]',
      created_at: '2026-02-03T11:00:00Z',
    },
  ];

  const triageClusters = [
    {
      id: 'cluster-1',
      cluster_key: 'vpn',
      summary: '2 tickets about vpn',
      ticket_count: 2,
      tickets_json: JSON.stringify([{ id: 'INC-1001', summary: 'VPN timeout' }]),
      created_at: '2026-02-03T11:05:00Z',
    },
  ];

  const runbookSessions = [
    {
      id: 'runbook-1',
      scenario: 'security-incident',
      status: 'active',
      steps_json: JSON.stringify(['Acknowledge incident', 'Contain access', 'Notify stakeholders']),
      current_step: 0,
      created_at: '2026-02-03T11:10:00Z',
      updated_at: '2026-02-03T11:10:00Z',
    },
  ];

  const integrationConfigs = [
    {
      id: 'integration-1',
      integration_type: 'servicenow',
      enabled: true,
      config_json: '{"endpoint":"https://servicenow.example.com"}',
      updated_at: '2026-02-03T11:15:00Z',
    },
    {
      id: 'integration-2',
      integration_type: 'slack',
      enabled: false,
      config_json: null,
      updated_at: '2026-02-03T11:15:00Z',
    },
    {
      id: 'integration-3',
      integration_type: 'teams',
      enabled: false,
      config_json: null,
      updated_at: '2026-02-03T11:15:00Z',
    },
  ];

  mockWindows('main');
  mockIPC(
    async (cmd, payload) => {
      switch (cmd) {
        case 'initialize_app':
          return {
            is_first_run: false,
            vector_enabled: false,
            vector_store_ready: false,
            key_storage_mode: 'Keychain',
            passphrase_required: false,
          };
        case 'check_fts5_enabled':
          return true;
        case 'get_memory_kernel_integration_pin':
          return {
            memorykernel_repo: 'https://github.com/saagar210/MemoryKernel',
            release_tag: 'v0.3.0',
            commit_sha: 'b9e1b397558dfba1fa8a4948fcf723ed4b505e1c',
            expected_service_contract_version: 'service.v2',
            expected_api_contract_version: 'api.v1',
            expected_integration_baseline: 'integration/v1',
            default_base_url: 'http://127.0.0.1:4010',
            default_timeout_ms: 2500,
          };
        case 'get_memory_kernel_preflight_status':
          return {
            enabled: true,
            ready: false,
            enrichment_enabled: false,
            status: 'offline',
            message: 'MemoryKernel service is unavailable at http://127.0.0.1:4010',
            base_url: 'http://127.0.0.1:4010',
            service_contract_version: null,
            api_contract_version: null,
            expected_service_contract_version: 'service.v2',
            expected_api_contract_version: 'api.v1',
            integration_baseline: 'integration/v1',
            release_tag: 'v0.3.0',
            commit_sha: 'b9e1b397558dfba1fa8a4948fcf723ed4b505e1c',
          };
        case 'memory_kernel_query_ask':
          return {
            applied: false,
            status: 'fallback',
            message: 'MemoryKernel enrichment currently unavailable',
            fallback_reason: 'offline',
            machine_error_code: null,
            context_package_id: null,
            enrichment_text: null,
            preflight: {
              enabled: true,
              ready: false,
              enrichment_enabled: false,
              status: 'offline',
              message: 'MemoryKernel service is unavailable at http://127.0.0.1:4010',
              base_url: 'http://127.0.0.1:4010',
              service_contract_version: null,
              api_contract_version: null,
              expected_service_contract_version: 'service.v2',
              expected_api_contract_version: 'api.v1',
              integration_baseline: 'integration/v1',
              release_tag: 'v0.3.0',
              commit_sha: 'b9e1b397558dfba1fa8a4948fcf723ed4b505e1c',
            },
          };
        case 'check_db_integrity':
          return true;
        case 'init_llm_engine':
        case 'init_embedding_engine':
        case 'cancel_generation':
        case 'configure_jira':
        case 'clear_jira_config':
        case 'set_context_window':
        case 'set_vector_consent':
        case 'set_kb_folder':
        case 'index_kb':
        case 'generate_kb_embeddings':
        case 'log_analytics_event':
          return null;
        case 'create_session_token':
          return 'mock-session-token';
        case 'validate_session_token':
          return true;
        case 'get_model_state':
          return {
            llm_model_id: null,
            llm_model_path: null,
            llm_loaded: true,
            embeddings_model_path: null,
            embeddings_loaded: false,
          };
        case 'is_model_loaded':
          return true;
        case 'get_model_info':
          return {
            id: 'llama-3.2-1b-instruct',
            name: 'Llama 3.2 1B Instruct',
            n_ctx_train: 8192,
          };
        case 'list_downloaded_models':
          return ['llama-3.2-1b-instruct'];
        case 'get_context_window':
          return 4096;
        case 'is_embedding_model_loaded':
        case 'is_embedding_model_downloaded':
          return false;
        case 'get_embedding_model_info':
          return null;
        case 'get_embedding_model_path':
          return null;
        case 'get_vector_consent':
          return { enabled: false, consented_at: null, encryption_supported: true };
        case 'get_kb_folder':
          return '/mock/kb';
        case 'get_kb_stats':
          return kbStats;
        case 'list_kb_documents':
          return kbDocuments;
        case 'list_namespaces':
          return [{ id: 'default', name: 'default' }];
        case 'search_kb':
        case 'search_kb_with_options':
          return mockSearchResults();
        case 'get_search_context':
          return 'Source: Remote Work Policy';
        case 'list_templates':
        case 'list_saved_response_templates':
        case 'find_similar_saved_responses':
        case 'get_alternatives_for_draft':
        case 'list_drafts':
        case 'list_autosaves':
        case 'get_draft_versions':
          return [];
        case 'save_draft':
          return (payload as { draft?: { id?: string } })?.draft?.id ?? 'mock-draft-id';
        case 'generate_streaming':
        case 'generate_with_context':
          return {
            text: 'Per Remote Work Policy, use the approved VPN and complete MFA before accessing internal systems.',
            tokens_generated: 48,
            duration_ms: 900,
            source_chunk_ids: ['chunk-1'],
            sources: mockSearchResults(),
            metrics: {
              tokens_per_second: 53.3,
              sources_used: 1,
              word_count: 16,
              length_target_met: true,
              context_utilization: 0.22,
            },
            prompt_template_version: 'e2e-mock',
            confidence: {
              mode: 'answer',
              score: 0.86,
              rationale: 'Strong grounded evidence across cited sources',
            },
            grounding: [
              {
                claim: 'Use approved VPN and complete MFA.',
                source_indexes: [0],
                support_level: 'strong',
              },
            ],
          };
        case 'get_deployment_health_summary': {
          const lastRun = deploymentRuns[deploymentRuns.length - 1] ?? null;
          const signedArtifacts = deploymentArtifacts.filter(a => a.is_signed).length;
          return {
            total_artifacts: deploymentArtifacts.length,
            signed_artifacts: signedArtifacts,
            unsigned_artifacts: deploymentArtifacts.length - signedArtifacts,
            last_run: lastRun,
          };
        }
        case 'list_deployment_artifacts':
          return deploymentArtifacts.slice().reverse();
        case 'record_deployment_artifact': {
          const body = payload as {
            artifactType?: string;
            version?: string;
            channel?: string;
            sha256?: string;
            isSigned?: boolean;
          };
          const next = {
            id: `artifact-${deploymentArtifacts.length + 1}`,
            artifact_type: body.artifactType ?? 'artifact',
            version: body.version ?? '0.0.0',
            channel: body.channel ?? 'stable',
            sha256: body.sha256 ?? '',
            is_signed: !!body.isSigned,
            created_at: new Date().toISOString(),
          };
          deploymentArtifacts.push(next);
          return next.id;
        }
        case 'run_deployment_preflight': {
          const checks = ['Database integrity: pass', 'Model status: loaded', 'Signed artifacts: pass'];
          const run = {
            id: `run-${deploymentRuns.length + 1}`,
            target_channel: 'stable',
            status: 'succeeded',
            preflight_json: JSON.stringify(checks),
            rollback_available: true,
            created_at: new Date().toISOString(),
            completed_at: new Date().toISOString(),
          };
          deploymentRuns.push(run);
          return { ok: true, checks };
        }
        case 'verify_signed_artifact': {
          const body = payload as { artifactId?: string; expectedSha256?: string | null };
          const artifact = deploymentArtifacts.find(a => a.id === body.artifactId) ?? deploymentArtifacts[0];
          const expected = body.expectedSha256 ?? null;
          const hashMatches = expected ? artifact.sha256.toLowerCase() === expected.toLowerCase() : true;
          return {
            artifact,
            is_signed: artifact.is_signed,
            hash_matches: hashMatches,
            status: artifact.is_signed && hashMatches ? 'verified' : artifact.is_signed ? 'hash_mismatch' : 'unsigned',
          };
        }
        case 'rollback_deployment_run': {
          const body = payload as { runId?: string };
          const run = deploymentRuns.find(r => r.id === body.runId) ?? deploymentRuns[deploymentRuns.length - 1];
          if (run) {
            run.status = 'rolled_back';
            run.completed_at = new Date().toISOString();
          }
          return null;
        }
        case 'list_eval_runs':
          return evalRuns.slice().reverse();
        case 'run_eval_harness': {
          const body = payload as { suiteName?: string; cases?: Array<{ query?: string }> };
          const total = body.cases?.length ?? 0;
          const passed = total;
          const avgConfidence = 0.74;
          const run = {
            id: `eval-${evalRuns.length + 1}`,
            suite_name: body.suiteName ?? 'suite',
            total_cases: total,
            passed_cases: passed,
            avg_confidence: avgConfidence,
            details_json: JSON.stringify(body.cases ?? []),
            created_at: new Date().toISOString(),
          };
          evalRuns.push(run);
          return {
            run_id: run.id,
            total_cases: total,
            passed_cases: passed,
            avg_confidence: avgConfidence,
          };
        }
        case 'list_recent_triage_clusters':
          return triageClusters.slice().reverse();
        case 'cluster_tickets_for_triage': {
          const body = payload as { tickets?: Array<{ id?: string; summary?: string }> };
          const tickets = body.tickets ?? [];
          const byKey = new Map<string, Array<{ id?: string; summary?: string }>>();
          for (const ticket of tickets) {
            const key = ticket.summary?.split(/\s+/)[0]?.toLowerCase() || 'general';
            if (!byKey.has(key)) byKey.set(key, []);
            byKey.get(key)!.push(ticket);
          }
          const output = Array.from(byKey.entries()).map(([cluster_key, group], idx) => {
            const record = {
              id: `cluster-${triageClusters.length + idx + 1}`,
              cluster_key,
              summary: `${group.length} tickets about ${cluster_key}`,
              ticket_count: group.length,
              tickets_json: JSON.stringify(group),
              created_at: new Date().toISOString(),
            };
            triageClusters.push(record);
            return {
              cluster_key,
              summary: record.summary,
              ticket_ids: group.map(ticket => ticket.id ?? ''),
            };
          });
          return output;
        }
        case 'list_runbook_sessions':
          return runbookSessions.slice().reverse();
        case 'start_runbook_session': {
          const body = payload as { scenario?: string; steps?: string[] };
          const next = {
            id: `runbook-${runbookSessions.length + 1}`,
            scenario: body.scenario ?? 'runbook',
            status: 'active',
            steps_json: JSON.stringify(body.steps ?? []),
            current_step: 0,
            created_at: new Date().toISOString(),
            updated_at: new Date().toISOString(),
          };
          runbookSessions.push(next);
          return next;
        }
        case 'advance_runbook_session': {
          const body = payload as { sessionId?: string; currentStep?: number; status?: string | null };
          const session = runbookSessions.find(s => s.id === body.sessionId);
          if (session) {
            session.current_step = body.currentStep ?? session.current_step;
            if (body.status) session.status = body.status;
            session.updated_at = new Date().toISOString();
          }
          return null;
        }
        case 'list_integrations':
          return integrationConfigs;
        case 'configure_integration': {
          const body = payload as { integrationType?: string; enabled?: boolean; configJson?: string | null };
          const type = body.integrationType ?? 'unknown';
          const existing = integrationConfigs.find(item => item.integration_type === type);
          if (existing) {
            existing.enabled = !!body.enabled;
            existing.config_json = body.configJson ?? null;
            existing.updated_at = new Date().toISOString();
          } else {
            integrationConfigs.push({
              id: `integration-${integrationConfigs.length + 1}`,
              integration_type: type,
              enabled: !!body.enabled,
              config_json: body.configJson ?? null,
              updated_at: new Date().toISOString(),
            });
          }
          return null;
        }
        case 'check_search_api_health':
          return true;
        case 'get_search_api_health_status':
          return {
            healthy: true,
            status: 'ok',
            message: 'Connected',
            base_url: 'http://localhost:3000',
          };
        case 'hybrid_search':
          return {
            status: 'success',
            query: (payload as { query?: string })?.query ?? 'mock query',
            query_id: 'query-e2e-1',
            intent: 'POLICY',
            intent_confidence: 0.93,
            results_count: 1,
            results: [
              {
                rank: 1,
                article_id: 'article-1',
                title: 'Removable Media Policy',
                category: 'POLICY',
                preview: 'USB drives are restricted unless approved.',
                source_document: 'doc-2',
                section: 'Section 4.2',
                scores: { bm25: 0.91, vector: 0.88, fused: 0.9 },
              },
            ],
            metrics: {
              latency_ms: 18.7,
              embedding_time_ms: 2.1,
              search_time_ms: 6.3,
              rerank_time_ms: 4.1,
              result_count: 1,
              timestamp: '2026-02-03T10:00:00Z',
            },
          };
        case 'submit_search_feedback':
          return 'Feedback submitted';
        case 'get_search_api_stats':
          return {
            queries_24h: 12,
            queries_total: 240,
            latency_ms: { avg: 21, p50: 12, p95: 55, p99: 91 },
            feedback_stats: { helpful: 8, not_helpful: 1, incorrect: 0 },
            intent_distribution: { POLICY: 6, PROCEDURE: 4, REFERENCE: 2 },
          };
        case 'is_jira_configured':
          return false;
        case 'get_jira_config':
          return null;
        case 'get_startup_metrics':
          return { total_ms: 1200, init_app_ms: 420, models_cached: true };
        case 'get_audit_entries':
          return [];
        case 'list_custom_variables':
          return [];
        default:
          // Keep the app responsive even when new commands are added.
          if (cmd.startsWith('list_') || cmd.startsWith('get_')) return [];
          if (cmd.startsWith('is_') || cmd.startsWith('has_')) return false;
          if (cmd.startsWith('save_') || cmd.startsWith('create_')) return 'ok';
          return null;
      }
    },
    { shouldMockEvents: true }
  );
}
