// App initialization types
export interface InitResult {
  is_first_run: boolean;
  keychain_available: boolean;
  fts5_available: boolean;
  vector_store_ready: boolean;
}

export interface VectorConsent {
  enabled: boolean;
  consented_at: string | null;
  encryption_supported: boolean | null;
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

export interface GenerateWithContextResult {
  text: string;
  tokens_generated: number;
  duration_ms: number;
  source_chunk_ids: string[];
  sources: ContextSource[];
}

export interface ContextSource {
  chunk_id: string;
  document_id: string;
  file_path: string;
  title: string | null;
  heading_path: string | null;
  score: number;
}

export type ResponseLength = 'Short' | 'Medium' | 'Long';

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
  indexed_at?: string | null;  // ISO 8601 string
  chunk_count?: number;
}

export interface IndexedFile {
  file_path: string;
  title: string | null;
  chunk_count: number;
  indexed_at: string;  // ISO 8601 string
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
  source: 'Fts5' | 'Vector' | 'Hybrid';
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
export type Tab = 'draft' | 'followups' | 'sources' | 'settings';

export type Theme = 'light' | 'dark' | 'system';

export interface Draft {
  id: string;
  input: string;
  output: string;
  sources: ContextSource[];
  created_at: string;
  updated_at: string;
}

// Toast types
export type ToastType = 'success' | 'error' | 'info' | 'warning';

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
  source: 'builtin' | 'custom';
  created_at: string;
  updated_at: string;
}

export interface TreeStructure {
  root_node_id: string;
  nodes: Record<string, TreeNode>;
}

export interface TreeNode {
  id: string;
  type: 'question' | 'action' | 'terminal';
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
