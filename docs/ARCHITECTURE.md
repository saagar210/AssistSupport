# Architecture Overview

AssistSupport is a local-first AI-powered customer support response generator built with Tauri 2.x, React 19, and Rust.

## Directory Structure

```
AssistSupport/
├── src/                      # React frontend
│   ├── components/           # UI components
│   │   ├── Draft/           # Response drafting (InputPanel, ResponsePanel)
│   │   ├── Layout/          # Header, TabBar, CommandPalette
│   │   ├── Settings/        # Model, KB, integration settings
│   │   ├── Sources/         # KB search and ingestion
│   │   └── shared/          # OnboardingWizard, StatusIndicator
│   ├── contexts/            # React contexts
│   │   └── AppStatusContext.tsx  # Centralized app state
│   ├── hooks/               # Custom React hooks
│   │   ├── useLlm.ts        # LLM operations
│   │   └── useKb.ts         # Knowledge base operations
│   ├── styles/              # CSS and design tokens
│   └── App.tsx              # Main app component
│
├── src-tauri/               # Rust backend
│   ├── src/
│   │   ├── lib.rs           # Library entry, AppState, Tauri setup
│   │   ├── commands/        # Tauri command handlers
│   │   │   ├── mod.rs       # All commands (~200 endpoints)
│   │   │   ├── backup.rs    # Backup/restore commands
│   │   │   └── diagnostics.rs # Health check commands
│   │   ├── db/              # Database layer
│   │   │   ├── mod.rs       # SQLCipher database, all queries
│   │   │   └── executor.rs  # Async query executor
│   │   ├── kb/              # Knowledge base
│   │   │   ├── indexer.rs   # File indexing, chunking
│   │   │   ├── search.rs    # Hybrid FTS + vector search
│   │   │   ├── embeddings.rs # Embedding model
│   │   │   ├── vectors.rs   # LanceDB vector store
│   │   │   ├── ingest/      # Content ingestion
│   │   │   │   ├── web.rs   # Web page ingestion
│   │   │   │   ├── youtube.rs # YouTube transcripts
│   │   │   │   └── github.rs # GitHub repos
│   │   │   ├── network.rs   # SSRF protection
│   │   │   ├── ocr.rs       # macOS Vision OCR
│   │   │   ├── pdf.rs       # PDF extraction
│   │   │   ├── docx.rs      # DOCX extraction
│   │   │   └── xlsx.rs      # Excel extraction
│   │   ├── llm.rs           # LLM engine (llama.cpp)
│   │   ├── security.rs      # Encryption, key management
│   │   ├── audit.rs         # Security audit logging
│   │   ├── validation.rs    # Input validation
│   │   ├── prompts.rs       # Prompt engineering
│   │   ├── jira.rs          # Jira integration
│   │   ├── jobs.rs          # Background job manager
│   │   ├── exports.rs       # Draft export formatting
│   │   ├── diagnostics.rs   # Health checks, maintenance
│   │   └── error.rs         # Error types
│   ├── bin/
│   │   └── assistsupport-cli.rs  # CLI tool
│   ├── tests/               # Integration tests
│   │   ├── kb_pipeline.rs   # E2E KB tests
│   │   ├── security.rs      # Security tests
│   │   └── path_validation.rs # Path security tests
│   └── benches/
│       └── performance.rs   # Criterion benchmarks
│
└── docs/                    # Documentation
    ├── ARCHITECTURE.md      # This file
    ├── SECURITY.md          # Security architecture
    ├── INSTALLATION.md      # Setup guide
    ├── PERFORMANCE.md       # Tuning guide
    └── OPERATIONS.md        # User guide
```

## Core Data Flow

### 1. Knowledge Base Pipeline
```
Files/URLs → Ingestion → Chunking → FTS Index + Vector Embeddings
                                           ↓
User Query → Hybrid Search (FTS + Vector) → Ranked Results → Context
```

### 2. Response Generation
```
User Input + KB Context + Jira Context → Prompt Builder → LLM → Response
                                              ↓
                              Draft Storage → Export/Copy
```

### 3. Security Model
```
Master Key (Keychain or Passphrase-wrapped)
     ↓
SQLCipher DB ← AES-256-GCM → Tokens (HuggingFace, Jira)
     ↓
All data at rest encrypted
```

## Key Components

### AppState (src-tauri/src/lib.rs)
```rust
pub struct AppState {
    pub db: Mutex<Option<Database>>,           // SQLCipher database
    pub llm: Arc<RwLock<Option<LlmEngine>>>,   // LLM runtime
    pub embeddings: Arc<RwLock<Option<EmbeddingEngine>>>,
    pub vectors: Arc<TokioRwLock<Option<VectorStore>>>,
    pub jobs: Arc<JobManager>,                  // Background tasks
}
```

### Frontend State (src/contexts/AppStatusContext.tsx)
```typescript
interface AppStatusState {
    llmLoaded: boolean;
    llmModelName: string | null;
    embeddingsLoaded: boolean;
    vectorEnabled: boolean;
    kbIndexed: boolean;
    kbDocumentCount: number;
}
```

### Database Schema (SQLCipher)
- `kb_documents` - Indexed documents
- `kb_chunks` - Document chunks with FTS5
- `kb_fts` - Full-text search virtual table
- `drafts` - Saved response drafts
- `templates` - Response templates
- `settings` - App configuration
- `jobs` - Background job tracking
- `ingest_sources` - Web/YouTube/GitHub sources
- `namespaces` - KB organization

### Namespace ID Policy

Namespace IDs are used to organize knowledge base content. They follow a strict slug pattern for consistency and security:

**ID Format**: `[a-z0-9-]{1,64}` (lowercase, alphanumeric, hyphens only)

**Auto-Normalization**:
- Uppercase → lowercase: `MyNamespace` → `my-namespace`
- Spaces/underscores → hyphens: `Product Docs` → `product-docs`
- Special characters removed
- Leading/trailing hyphens trimmed
- Max 64 characters (truncated if longer)

**Display Names**: Stored separately for user-friendly labels (not subject to slug rules).

**Usage**:
```rust
use crate::validation::normalize_and_validate_namespace_id;

let ns_id = normalize_and_validate_namespace_id("My Namespace")?; // "my-namespace"
```

## API Patterns

### Tauri Commands
All backend operations exposed via `#[tauri::command]`:
```rust
#[tauri::command]
pub async fn search_kb(
    state: State<'_, AppState>,
    query: String,
    limit: usize,
) -> Result<Vec<SearchResult>, String>
```

### Frontend Invocation
```typescript
const results = await invoke<SearchResult[]>('search_kb', { 
    query, 
    limit: 10 
});
```

## Testing Strategy

| Layer | Framework | Location |
|-------|-----------|----------|
| Unit | Rust built-in | `src/**/*.rs` (mod tests) |
| Integration | Rust | `tests/*.rs` |
| Benchmarks | Criterion | `benches/performance.rs` |
| Frontend Unit | Vitest | `src/**/*.test.{ts,tsx}` |
| E2E | Manual | — |

## Security Boundaries

1. **File Access**: Restricted to `$HOME`, sensitive dirs blocked
2. **Network**: SSRF protection, HTTPS enforced
3. **Credentials**: Encrypted at rest, zeroized in memory
4. **Database**: SQLCipher AES-256
5. **Exports**: Optional Argon2id + AES-256-GCM

## Extension Points

### Adding a New Ingestion Source
1. Create `src-tauri/src/kb/ingest/newtype.rs`
2. Implement extraction logic
3. Add command in `src-tauri/src/commands/mod.rs`
4. Add UI in `src/components/Sources/`

### Adding a New Command
1. Add function in `src-tauri/src/commands/mod.rs`
2. Register in `src-tauri/src/lib.rs` invoke_handler
3. Call via `invoke()` from frontend

### Adding a Settings Panel
1. Create component in `src/components/Settings/`
2. Add tab in `SettingsTab.tsx`
3. Add Tauri commands for persistence
