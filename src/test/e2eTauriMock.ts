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
          };
        case 'check_search_api_health':
          return true;
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
