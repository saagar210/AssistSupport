# VaultMind Architecture

## System Overview

```
┌─────────────────────────────────────────────────────┐
│                   VaultMind Desktop                  │
│                                                     │
│  ┌───────────────────────────────────────────────┐  │
│  │              React 19 Frontend                 │  │
│  │                                               │  │
│  │  Views: Documents | Search | Chat | Graph     │  │
│  │  State: Zustand stores                        │  │
│  │  UI: Tailwind CSS 4 + Lucide icons            │  │
│  └──────────────────┬────────────────────────────┘  │
│                     │ Tauri IPC (invoke)             │
│  ┌──────────────────▼────────────────────────────┐  │
│  │              Rust Backend (Tauri 2)            │  │
│  │                                               │  │
│  │  Commands: collections | documents | search   │  │
│  │            chat | graph | settings | ollama    │  │
│  │                                               │  │
│  │  ┌─────────┐ ┌─────────┐ ┌────────────────┐  │  │
│  │  │ Parsers │ │ Chunker │ │   Embedder     │  │  │
│  │  │ (7 fmt) │ │         │ │ (Ollama API)   │  │  │
│  │  └────┬────┘ └────┬────┘ └───────┬────────┘  │  │
│  │       │           │              │            │  │
│  │  ┌────▼───────────▼──────────────▼────────┐   │  │
│  │  │              SQLite Database            │   │  │
│  │  │  collections | documents | chunks      │   │  │
│  │  │  embeddings  | conversations | graph   │   │  │
│  │  └────────────────────────────────────────┘   │  │
│  └───────────────────────────────────────────────┘  │
│                     │                               │
└─────────────────────┼───────────────────────────────┘
                      │ HTTP (localhost:11434)
              ┌───────▼───────┐
              │    Ollama     │
              │  LLM + Embed  │
              └───────────────┘
```

## Frontend

| Component       | Technology       | Purpose                                      |
|-----------------|------------------|----------------------------------------------|
| Framework       | React 19         | UI rendering with functional components      |
| Language        | TypeScript       | Strict-mode type safety                      |
| Styling         | Tailwind CSS 4   | Utility-first styling with Vite plugin       |
| State           | Zustand          | Lightweight stores for app, documents, chat  |
| Icons           | Lucide React     | Consistent icon set                          |
| Command Palette | cmdk             | Keyboard-driven navigation                   |
| Graph Viz       | react-force-graph-2d | Interactive force-directed graph         |

### Views

- **DocumentsView** -- Library of ingested documents with collection filtering
- **DocumentDetailView** -- Single document with chunk preview
- **SearchView** -- Hybrid search interface with result ranking
- **ChatView** -- Conversational RAG with citation links
- **GraphView** -- Interactive knowledge graph explorer
- **SettingsView** -- Ollama configuration and model selection

### State Stores

- `appStore` -- Navigation, sidebar, theme, global UI state
- `collectionStore` -- CRUD for document collections
- `documentStore` -- Document list and ingestion status
- `chatStore` -- Conversations, messages, streaming state
- `settingsStore` -- Ollama URL, selected models, preferences
- `toastStore` -- Notification queue

## Backend

### Tauri 2 Commands

Each command module exposes `#[tauri::command]` functions that the frontend invokes over IPC:

- **collections** -- Create, list, get, update, delete collections
- **documents** -- Ingest files, list, get, delete documents; get chunks and stats
- **search** -- Vector search, keyword search, hybrid search (RRF)
- **chat** -- Create conversations, send messages (with RAG), list/get/delete conversations
- **graph** -- Build knowledge graph from document chunks, retrieve graph data
- **settings** -- Get/update application settings
- **ollama** -- Connection check, model listing, connection testing

### Core Modules

- **parsers/** -- Format-specific text extraction (PDF, Markdown, HTML, TXT, DOCX, CSV, EPUB)
- **chunker.rs** -- Splits extracted text into overlapping chunks for embedding
- **embedder.rs** -- Sends text chunks to Ollama's embedding endpoint (`nomic-embed-text`)
- **vector_store.rs** -- Cosine similarity search over stored embeddings
- **graph.rs** -- Entity and relationship extraction from document chunks using LLM
- **ollama.rs** -- HTTP client for Ollama API (generate, embed, list models)
- **db.rs** -- SQLite schema initialization, migrations, and query helpers
- **models.rs** -- Shared data structures (Document, Chunk, Embedding, Conversation, etc.)
- **error.rs** -- Centralized error types using `thiserror`
- **state.rs** -- Application state (database connection wrapped in Mutex)

## Data Flow

### Document Ingestion Pipeline

```
File → Parser → Raw Text → Chunker → Chunks → Embedder → Vectors
                                        │                    │
                                        ▼                    ▼
                                   chunks table       embeddings table
```

1. User selects files through the native file dialog
2. The appropriate parser extracts text based on file extension
3. The chunker splits text into overlapping segments (configurable size/overlap)
4. Each chunk is sent to Ollama for embedding generation
5. Documents, chunks, and embeddings are persisted to SQLite

### Search Pipeline (Hybrid with RRF)

```
Query → ┬→ Embedder → Vector Search (cosine similarity) → Ranked Results ─┐
        │                                                                   │
        └→ Keyword Search (BM25-style) → Ranked Results ──────────────────┤
                                                                           │
                                                              Reciprocal Rank Fusion
                                                                           │
                                                                    Merged Results
```

### RAG Pipeline

```
User Message → Hybrid Search → Top-K Chunks → Prompt Assembly → Ollama LLM → Response
                                                    │
                                            System prompt +
                                            retrieved context +
                                            conversation history
```

## Database Schema Overview

The SQLite database stores all application data:

- **collections** -- Named groups for organizing documents
- **documents** -- File metadata (name, path, hash, format, size, timestamps)
- **chunks** -- Text segments with position info, linked to documents
- **embeddings** -- Vector representations of chunks (stored as BLOBs)
- **conversations** -- Chat sessions linked to optional collections
- **messages** -- Chat messages with role (user/assistant) and content
- **citations** -- Links from assistant messages back to source chunks
- **graph_entities** -- Extracted entities (name, type, description)
- **graph_relationships** -- Directed edges between entities
- **settings** -- Key-value application configuration
