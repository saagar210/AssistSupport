export interface PaginatedResponse<T> {
  items: T[];
  total: number;
  page: number;
  page_size: number;
  has_more: boolean;
}

export interface Collection {
  id: string;
  name: string;
  description: string;
  created_at: string;
  updated_at: string;
}

export interface Document {
  id: string;
  collection_id: string;
  filename: string;
  file_path: string;
  file_type: string;
  file_size: number;
  file_hash: string;
  title: string;
  author: string | null;
  page_count: number | null;
  word_count: number;
  chunk_count: number;
  status: string;
  error_message: string | null;
  created_at: string;
  updated_at: string;
}

export interface Chunk {
  id: string;
  document_id: string;
  collection_id: string;
  content: string;
  chunk_index: number;
  start_offset: number;
  end_offset: number;
  page_number: number | null;
  section_title: string | null;
  token_count: number;
  created_at: string;
}

export interface GraphEdge {
  id: string;
  source_chunk_id: string;
  target_chunk_id: string;
  collection_id: string;
  weight: number;
  relationship_type: string;
  created_at: string;
}

export interface Conversation {
  id: string;
  collection_id: string;
  title: string;
  created_at: string;
  updated_at: string;
}

export interface Message {
  id: string;
  conversation_id: string;
  role: string;
  content: string;
  created_at: string;
}

export interface Citation {
  id: string;
  message_id: string;
  chunk_id: string;
  document_id: string;
  document_title: string;
  section_title: string | null;
  page_number: number | null;
  relevance_score: number;
  snippet: string;
}

export interface Setting {
  key: string;
  value: string;
}

export interface OllamaModel {
  name: string;
  size: number;
  family: string | null;
}

export interface SearchResult {
  chunk_id: string;
  document_id: string;
  document_title: string;
  section_title: string | null;
  page_number: number | null;
  content: string;
  score: number;
}

export interface GraphNode {
  id: string;
  label: string;
  file_type: string;
  chunk_count: number;
  word_count: number;
}

export interface GraphLink {
  source: string;
  target: string;
  weight: number;
  relationship_type: string;
}

export interface GraphData {
  nodes: GraphNode[];
  links: GraphLink[];
}

export interface ChatMessage {
  id: string;
  role: "user" | "assistant";
  content: string;
  citations?: Citation[];
  created_at: string;
}

export interface IngestionProgress {
  document_id: string;
  filename: string;
  stage: "parsing" | "chunking" | "embedding" | "indexing" | "complete" | "failed";
  chunks_done: number;
  chunks_total: number;
  error?: string;
}

export interface SearchHistoryEntry {
  id: string;
  collection_id: string;
  query: string;
  result_count: number;
  created_at: string;
}

export type ViewType = "graph" | "chat" | "documents" | "search" | "settings" | "document-detail";
