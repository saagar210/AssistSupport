// App initialization types
export interface InitResult {
  is_first_run: boolean;
  vector_enabled: boolean;
  vector_store_ready: boolean;
  key_storage_mode: string;
  passphrase_required: boolean;
  recovery_issue: StartupRecoveryIssue | null;
}

export interface StartupRecoveryConflict {
  name: string;
  old_path: string;
  new_path: string;
  reason: string;
}

export interface StartupRecoveryIssue {
  code: string;
  summary: string;
  details: string | null;
  can_repair: boolean;
  can_restore_backup: boolean;
  requires_manual_resolution: boolean;
  migration_conflicts: StartupRecoveryConflict[];
}

export interface ImportSummary {
  drafts_imported: number;
  templates_imported: number;
  variables_imported: number;
  trees_imported: number;
}

export interface ModelStateResult {
  llm_model_id: string | null;
  llm_model_path: string | null;
  llm_loaded: boolean;
  embeddings_model_path: string | null;
  embeddings_loaded: boolean;
}

export interface StartupMetricsResult {
  total_ms: number;
  init_app_ms: number;
  models_cached: boolean;
}

export interface VectorConsent {
  enabled: boolean;
  consented_at: string | null;
  encryption_supported: boolean | null;
}

export interface MemoryKernelIntegrationPin {
  memorykernel_repo: string;
  release_tag: string;
  commit_sha: string;
  expected_service_contract_version: string;
  expected_api_contract_version: string;
  expected_integration_baseline: string;
  default_base_url: string;
  default_timeout_ms: number;
}

export interface MemoryKernelPreflightStatus {
  enabled: boolean;
  ready: boolean;
  enrichment_enabled: boolean;
  status: string;
  message: string;
  base_url: string;
  service_contract_version: string | null;
  api_contract_version: string | null;
  expected_service_contract_version: string;
  expected_api_contract_version: string;
  integration_baseline: string;
  release_tag: string;
  commit_sha: string;
}

export interface MemoryKernelEnrichmentResult {
  applied: boolean;
  status: string;
  message: string;
  fallback_reason: string | null;
  machine_error_code: string | null;
  context_package_id: string | null;
  enrichment_text: string | null;
  preflight: MemoryKernelPreflightStatus;
}

// LLM types
export interface ModelInfo {
  id: string;
  name: string;
  path?: string;
  size?: string;
  description?: string;
  size_bytes?: number;
  n_params?: number | null;
  n_ctx?: number;
  n_ctx_train?: number;
  n_embd?: number;
  n_vocab?: number;
  n_gpu_layers?: number;
  verification_status?: string | null;
}

export interface GgufFileInfo {
  path: string;
  file_name: string;
  file_size: number;
  is_valid: boolean;
  integrity_status: string;
}

export interface SearchApiEmbeddingModelStatus {
  installed: boolean;
  ready: boolean;
  model_name: string;
  revision: string;
  local_path: string | null;
  error: string | null;
}

export interface GenerationParams {
  max_tokens?: number;
  temperature?: number;
  top_p?: number;
  top_k?: number;
  repeat_penalty?: number;
  stop_sequences?: string[];
  context_window?: number;
}

export interface GenerationResult {
  text: string;
  tokens_generated: number;
  duration_ms: number;
}

// Streaming token event from backend
export interface StreamToken {
  token: string;
  done: boolean;
}

export interface TreeDecisions {
  tree_name: string;
  path_summary: string;
}

export interface JiraTicketContext {
  key: string;
  summary: string;
  description: string | null;
  status: string;
  priority: string | null;
  assignee: string | null;
  reporter: string;
  created: string;
  updated: string;
  issue_type: string;
}

export interface GenerateWithContextParams {
  user_input: string;
  kb_query?: string;
  kb_limit?: number;
  ocr_text?: string;
  diagnostic_notes?: string;
  tree_decisions?: TreeDecisions;
  jira_ticket?: JiraTicketContext;
  response_length?: ResponseLength;
  gen_params?: GenerationParams;
}

export interface GenerationMetrics {
  tokens_per_second: number;
  sources_used: number;
  word_count: number;
  length_target_met: boolean;
  context_utilization: number;
}

export type ConfidenceMode = "answer" | "clarify" | "abstain";

export interface ConfidenceAssessment {
  mode: ConfidenceMode;
  score: number;
  rationale: string;
}

export interface GroundedClaim {
  claim: string;
  source_indexes: number[];
  support_level: string;
}

export interface GenerateWithContextResult {
  text: string;
  tokens_generated: number;
  duration_ms: number;
  source_chunk_ids: string[];
  sources: ContextSource[];
  metrics: GenerationMetrics;
  prompt_template_version: string;
  confidence: ConfidenceAssessment;
  grounding: GroundedClaim[];
}

export interface ContextSource {
  chunk_id: string;
  document_id: string;
  file_path: string;
  title: string | null;
  heading_path: string | null;
  score: number;
  search_method: string | null;
  source_type: string | null;
}

export type ResponseLength = "Short" | "Medium" | "Long";
export type FirstResponseTone = "slack" | "jira";

export interface FirstResponseParams {
  user_input: string;
  tone: FirstResponseTone;
  ocr_text?: string;
  jira_ticket?: JiraTicketContext;
}

export interface FirstResponseResult {
  text: string;
  tokens_generated: number;
  duration_ms: number;
}

export interface ChecklistItem {
  id: string;
  text: string;
  category?: string | null;
  priority?: string | null;
  details?: string | null;
}

export interface ChecklistState {
  items: ChecklistItem[];
  completed_ids: string[];
}

export interface ChecklistGenerateParams {
  user_input: string;
  ocr_text?: string;
  diagnostic_notes?: string;
  tree_decisions?: TreeDecisions;
  jira_ticket?: JiraTicketContext;
}

export interface ChecklistUpdateParams extends ChecklistGenerateParams {
  checklist: ChecklistState;
}

export interface ChecklistResult {
  items: ChecklistItem[];
}

// Download types
export interface ModelSource {
  name: string;
  repo: string;
  filename: string;
  size_bytes: number | null;
  sha256: string | null;
}

export interface DownloadProgress {
  model_id: string;
  percent: number;
  downloaded_bytes: number;
  total_bytes: number;
  speed_bps: number;
}

// KB types
export interface KbDocument {
  id: string;
  file_path: string;
  title?: string | null;
  indexed_at?: string | null; // ISO 8601 string
  chunk_count?: number;
}

export interface IndexedFile {
  file_path: string;
  title: string | null;
  chunk_count: number;
  indexed_at: string; // ISO 8601 string
}

export interface IndexStats {
  document_count: number;
  chunk_count: number;
  total_words: number;
}

export interface IndexResult {
  total_files: number;
  indexed: number;
  skipped: number;
  errors: number;
}

export interface SearchResult {
  chunk_id: string;
  document_id: string;
  file_path: string;
  title: string | null;
  heading_path: string | null;
  content: string;
  snippet: string;
  score: number;
  source: "Fts5" | "Vector" | "Hybrid";
  namespace_id: string | null;
  source_type: string | null;
}

export interface SearchOptions {
  /** Weight for FTS5 results (0.0-1.0, default 0.5) */
  fts_weight?: number;
  /** Weight for vector results (0.0-1.0, default 0.5) */
  vector_weight?: number;
  /** Enable deduplication (default true) */
  enable_dedup?: boolean;
  /** Deduplication threshold (0.0-1.0, default 0.85) */
  dedup_threshold?: number;
}

export interface KbGapCandidate {
  id: string;
  query_signature: string;
  sample_query: string;
  occurrences: number;
  low_confidence_count: number;
  low_rating_count: number;
  unsupported_claim_events: number;
  suggested_category: string | null;
  status: string;
  resolution_note: string | null;
  first_seen_at: string;
  last_seen_at: string;
}

export interface DeploymentRunRecord {
  id: string;
  target_channel: string;
  status: string;
  preflight_json: string | null;
  rollback_available: boolean;
  created_at: string;
  completed_at: string | null;
}

export interface DeploymentHealthSummary {
  total_artifacts: number;
  signed_artifacts: number;
  unsigned_artifacts: number;
  last_run: DeploymentRunRecord | null;
}

export interface IntegrationConfigRecord {
  id: string;
  integration_type: string;
  enabled: boolean;
  config_json: string | null;
  updated_at: string;
}

export interface DeploymentArtifactRecord {
  id: string;
  artifact_type: string;
  version: string;
  channel: string;
  sha256: string;
  is_signed: boolean;
  created_at: string;
}

export interface SignedArtifactVerificationResult {
  artifact: DeploymentArtifactRecord;
  is_signed: boolean;
  hash_matches: boolean;
  status: string;
}

export interface EvalRunRecord {
  id: string;
  suite_name: string;
  total_cases: number;
  passed_cases: number;
  avg_confidence: number;
  details_json: string | null;
  created_at: string;
}

export interface TriageClusterRecord {
  id: string;
  cluster_key: string;
  summary: string;
  ticket_count: number;
  tickets_json: string;
  created_at: string;
}

export interface RunbookSessionRecord {
  id: string;
  scenario: string;
  scope_key: string;
  status: string;
  steps_json: string;
  current_step: number;
  created_at: string;
  updated_at: string;
}

export interface RunbookTemplateRecord {
  id: string;
  name: string;
  scenario: string;
  steps_json: string;
  created_at: string;
  updated_at: string;
}

export interface RunbookStepEvidenceRecord {
  id: string;
  session_id: string;
  step_index: number;
  status: "pending" | "completed" | "skipped" | "failed";
  evidence_text: string;
  skip_reason: string | null;
  created_at: string;
}

export interface DispatchHistoryRecord {
  id: string;
  integration_type: "jira" | "servicenow" | "slack" | "teams";
  draft_id: string | null;
  title: string;
  destination_label: string;
  payload_preview: string;
  status: "previewed" | "sent" | "cancelled" | "failed";
  metadata_json: string | null;
  created_at: string;
  updated_at: string;
}

export interface CaseOutcomeRecord {
  id: string;
  draft_id: string;
  status: string;
  outcome_summary: string;
  handoff_pack_json: string | null;
  kb_draft_json: string | null;
  evidence_pack_json: string | null;
  tags_json: string | null;
  created_at: string;
  updated_at: string;
}

// Embedding types
export interface EmbeddingModelInfo {
  path: string;
  name: string;
  embedding_dim: number;
  size_bytes: number;
}

// OCR types
export interface OcrResult {
  text: string;
  confidence: number;
}

// UI State types
export type Tab =
  | "draft"
  | "followups"
  | "sources"
  | "ingest"
  | "knowledge"
  | "analytics"
  | "pilot"
  | "search"
  | "ops"
  | "settings";

export type Theme = "light" | "dark" | "system";

export interface Draft {
  id: string;
  input: string;
  output: string;
  sources: ContextSource[];
  created_at: string;
  updated_at: string;
}

// Toast types
export type ToastType = "success" | "error" | "info" | "warning";

export interface Toast {
  id: string;
  type: ToastType;
  message: string;
  duration?: number;
}

// Decision Tree types
export interface DecisionTree {
  id: string;
  name: string;
  category: string | null;
  tree_json: string; // Serialized TreeStructure
  source: "builtin" | "custom";
  created_at: string;
  updated_at: string;
}

export interface TreeStructure {
  root_node_id: string;
  nodes: Record<string, TreeNode>;
}

export interface TreeNode {
  id: string;
  type: "question" | "action" | "terminal";
  title: string;
  content: string | null;
  options?: TreeOption[];
}

export interface TreeOption {
  label: string;
  next_node_id: string | null; // null = terminal
}

export interface DiagnosticSession {
  id: string;
  draft_id: string | null;
  checklist_json: string | null;
  findings_json: string | null;
  decision_tree_id: string | null;
  tree_path_json: string | null;
  escalation_note: string | null;
  created_at: string;
  updated_at: string;
}

export interface TreePath {
  tree_id: string;
  visited_nodes: string[];
  current_node_id: string;
}

export type DraftStatus = "draft" | "finalized" | "archived";
export type NoteAudience =
  | "internal-note"
  | "customer-safe"
  | "escalation-note";
export type IntakeUrgency = "low" | "normal" | "high" | "critical";
export type NextActionKind =
  | "answer"
  | "clarify"
  | "runbook"
  | "escalate"
  | "approval"
  | "promote_kb";

export interface CaseIntake {
  issue?: string | null;
  environment?: string | null;
  impact?: string | null;
  urgency?: IntakeUrgency | null;
  affected_user?: string | null;
  affected_system?: string | null;
  affected_site?: string | null;
  symptoms?: string | null;
  steps_tried?: string | null;
  blockers?: string | null;
  likely_category?: string | null;
  missing_data?: string[];
  note_audience?: NoteAudience;
  user?: string | null;
  device?: string | null;
  os?: string | null;
  reproduction?: string | null;
  logs?: string | null;
  custom_fields?: Record<string, string>;
}

export interface HandoffPack {
  summary: string;
  actions_taken: string[];
  current_blocker: string;
  next_step: string;
  customer_safe_update: string;
  escalation_note: string;
}

export interface SearchExplanation {
  summary: string;
  matched_terms: string[];
  reasons: string[];
  authoritative: boolean;
}

export interface SimilarCase {
  draft_id: string;
  ticket_id: string | null;
  title: string;
  excerpt: string;
  response_excerpt: string;
  response_text: string;
  handoff_summary: string | null;
  status: DraftStatus;
  updated_at: string;
  match_score: number;
  explanation: SearchExplanation;
}

export interface MissingQuestion {
  id: string;
  question: string;
  reason: string;
  priority: "high" | "medium" | "low";
}

export interface NextActionRecommendation {
  id: string;
  kind: NextActionKind;
  label: string;
  rationale: string;
  confidence: number;
  prerequisites: string[];
}

export interface EvidencePackSection {
  label: string;
  content: string;
}

export interface EvidencePack {
  title: string;
  summary: string;
  sections: EvidencePackSection[];
}

export interface KbDraft {
  title: string;
  summary: string;
  symptoms: string;
  environment: string;
  cause: string;
  resolution: string;
  warnings: string[];
  prerequisites: string[];
  policy_links: string[];
  tags: string[];
}

export interface ResolutionKit {
  id: string;
  name: string;
  summary: string;
  category: string;
  response_template: string;
  checklist_items: string[];
  kb_document_ids: string[];
  runbook_scenario: string | null;
  approval_hint: string | null;
}

export interface ResolutionKitRecord {
  id: string;
  name: string;
  summary: string;
  category: string;
  response_template: string;
  checklist_items_json: string;
  kb_document_ids_json: string;
  runbook_scenario: string | null;
  approval_hint: string | null;
  created_at: string;
  updated_at: string;
}

export interface WorkspaceFavorite {
  id: string;
  kind: "runbook" | "policy" | "kb" | "kit";
  label: string;
  resource_id: string;
  metadata?: Record<string, string> | null;
  created_at?: string;
  updated_at?: string;
}

export interface WorkspaceFavoriteRecord {
  id: string;
  kind: "runbook" | "policy" | "kb" | "kit";
  label: string;
  resource_id: string;
  metadata_json: string | null;
  created_at: string;
  updated_at: string;
}

export interface GuidedRunbookTemplate {
  id: string;
  name: string;
  scenario: string;
  steps: string[];
}

export interface GuidedRunbookSession {
  id: string;
  scenario: string;
  status: string;
  steps: string[];
  current_step: number;
  evidence: RunbookStepEvidenceRecord[];
}

export interface ApprovalGuidance {
  allowed: boolean;
  required_approvers: string[];
  required_evidence: string[];
  recommended_response: string;
}

export interface CollaborationDispatchPreview {
  integration_type: "jira" | "servicenow" | "slack" | "teams";
  title: string;
  destination_label: string;
  payload_preview: string;
  draft_id?: string | null;
  metadata?: Record<string, string> | null;
}

export interface TicketWorkspaceSnapshot {
  draft: SavedDraft;
  intake: CaseIntake;
  handoff_pack: HandoffPack;
  next_actions: NextActionRecommendation[];
  missing_questions: MissingQuestion[];
  evidence_pack: EvidencePack;
  similar_cases?: SimilarCase[];
  favorites?: WorkspaceFavorite[];
  kits?: ResolutionKit[];
}

export interface WorkspacePersonalization {
  preferred_note_audience: NoteAudience;
  preferred_output_length: ResponseLength;
  favorite_queue_view:
    | "all"
    | "unassigned"
    | "at_risk"
    | "in_progress"
    | "resolved";
  default_evidence_format: "clipboard" | "markdown";
}

// Draft/Session types
export interface SavedDraft {
  id: string;
  input_text: string;
  summary_text: string | null;
  diagnosis_json: string | null;
  response_text: string | null;
  ticket_id: string | null;
  kb_sources_json: string | null;
  created_at: string;
  updated_at: string;
  is_autosave: boolean;
  /** Name of the model that generated this response (e.g., "Llama 3.2 3B Instruct") */
  model_name?: string | null;
  case_intake_json?: string | null;
  status?: DraftStatus | null;
  handoff_summary?: string | null;
  finalized_at?: string | null;
  finalized_by?: string | null;
}

export interface ResponseTemplate {
  id: string;
  name: string;
  category: string | null;
  content: string;
  created_at: string;
  updated_at: string;
}

export interface CustomVariable {
  id: string;
  name: string;
  value: string;
  created_at: string;
}

export interface TemplateContext {
  ticketId?: string;
  customerName?: string;
  agentName?: string;
}

export interface AuditEntry {
  timestamp: string;
  // Serde serializes unit variants as strings ("key_generated") but
  // data variants like Custom(String) as objects ({"custom": "value"})
  event: string | Record<string, string>;
  severity: "info" | "warning" | "error" | "critical";
  message: string;
  context?: Record<string, unknown> | null;
}

// Ingestion types
export interface Namespace {
  id: string;
  name: string;
  description?: string | null;
  color?: string | null;
  created_at: string;
  updated_at: string;
}

export interface NamespaceWithCounts extends Namespace {
  document_count: number;
  source_count: number;
}

export interface IngestSource {
  id: string;
  source_type: string;
  source_uri: string;
  namespace_id: string;
  title?: string | null;
  etag?: string | null;
  last_modified?: string | null;
  content_hash?: string | null;
  last_ingested_at?: string | null;
  status: string;
  error_message?: string | null;
  metadata_json?: string | null;
  created_at: string;
  updated_at: string;
}

export interface IngestResult {
  document_id: string;
  title: string;
  source_uri: string;
  chunk_count: number;
  word_count: number;
}

export interface BatchIngestResult {
  successful: IngestResult[];
  failed: FailedSource[];
  cancelled: boolean;
}

export interface FailedSource {
  source: string;
  error: string;
}

export interface KbDocumentInfo {
  id: string;
  file_path: string;
  title?: string | null;
  indexed_at?: string | null;
  chunk_count?: number | null;
  namespace_id: string;
  source_type: string;
  source_id?: string | null;
  last_reviewed_at?: string | null;
  last_reviewed_by?: string | null;
}

export interface DocumentChunk {
  id: string;
  chunk_index: number;
  heading_path?: string | null;
  content: string;
  word_count?: number | null;
}

// Source health types
export interface SourceHealth {
  id: string;
  source_type: string;
  source_uri: string;
  title: string | null;
  status: "pending" | "active" | "stale" | "error" | "removed";
  error_message: string | null;
  last_ingested_at: string | null;
  document_count: number;
  days_since_refresh: number | null;
}

export interface SourceHealthSummary {
  total_sources: number;
  active_sources: number;
  stale_sources: number;
  error_sources: number;
  pending_sources: number;
  sources: SourceHealth[];
}

// Diagnostics types
export type HealthStatus = "healthy" | "warning" | "error" | "unavailable";

export interface ComponentHealth {
  name: string;
  status: HealthStatus;
  message: string;
  details: string | null;
  can_repair: boolean;
}

export interface SystemHealth {
  database: ComponentHealth;
  vector_store: ComponentHealth;
  llm_engine: ComponentHealth;
  embedding_model: ComponentHealth;
  file_system: ComponentHealth;
  overall_status: HealthStatus;
  checked_at: string;
}

export interface RepairResult {
  component: string;
  success: boolean;
  action_taken: string;
  message: string | null;
}

export interface FailureMode {
  id: string;
  problem: string;
  symptoms: string[];
  resolution_steps: string[];
  auto_repair_available: boolean;
}

export interface QuickHealthResult {
  healthy: boolean;
  checks_passed: number;
  checks_total: number;
  issues: string[];
}

// v0.4.0 Types

export interface DocumentReviewInfo {
  id: string;
  file_path: string;
  title: string | null;
  indexed_at: string | null;
  last_reviewed_at: string | null;
  last_reviewed_by: string | null;
  namespace_id: string;
  source_type: string;
}

export interface ArticleAnalytics {
  document_id: string;
  title: string;
  file_path: string;
  total_uses: number;
  average_rating: number | null;
  draft_references: ArticleDraftReference[];
}

export interface ArticleDraftReference {
  draft_id: string;
  input_text: string;
  response_text: string | null;
  created_at: string;
  rating: number | null;
  feedback_text: string | null;
}

export interface SavedResponseTemplate {
  id: string;
  source_draft_id: string | null;
  source_rating: number | null;
  name: string;
  category: string | null;
  content: string;
  variables_json: string | null;
  use_count: number;
  created_at: string;
  updated_at: string;
}

export interface ResponseAlternative {
  id: string;
  draft_id: string;
  original_text: string;
  alternative_text: string;
  sources_json: string | null;
  metrics_json: string | null;
  generation_params_json: string | null;
  chosen: "original" | "alternative" | null;
  created_at: string;
}

export interface JiraTransition {
  id: string;
  name: string;
  to_status: string;
}

// PostgreSQL Hybrid Search types (Week 4)

export interface HybridSearchScores {
  bm25: number;
  vector: number;
  fused: number;
}

export interface HybridSearchResult {
  rank: number;
  article_id: string;
  title: string;
  category: string;
  preview: string;
  source_document: string | null;
  section: string | null;
  scores: HybridSearchScores | null;
}

export interface HybridSearchMetrics {
  latency_ms: number;
  embedding_time_ms: number;
  search_time_ms: number;
  result_count: number;
  timestamp: string;
}

export interface HybridSearchResponse {
  status: string;
  query: string;
  query_id: string | null;
  intent: string;
  intent_confidence: number;
  results_count: number;
  results: HybridSearchResult[];
  metrics: HybridSearchMetrics;
}

export interface SearchApiLatency {
  avg: number;
  p50: number;
  p95: number;
  p99: number;
}

export interface SearchApiFeedbackStats {
  helpful: number;
  not_helpful: number;
  incorrect: number;
}

export interface SearchApiStatsData {
  queries_24h: number;
  queries_total: number;
  latency_ms: SearchApiLatency;
  feedback_stats: SearchApiFeedbackStats;
  intent_distribution: Record<string, number>;
}

export interface SearchApiHealthStatus {
  healthy: boolean;
  status: string;
  message: string;
  base_url: string;
}
