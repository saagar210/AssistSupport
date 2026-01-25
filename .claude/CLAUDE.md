# AssistSupport - Project Context

## Project Overview
Local-first AI-powered customer support response generator. Fully offline, encrypted, privacy-focused.

## Tech Stack
- **Frontend**: React 19 + TypeScript (strict mode) + Vite
- **Backend**: Rust + Tauri 2.x
- **Database**: SQLite with SQLCipher encryption + FTS5
- **Vector Store**: LanceDB for hybrid semantic search
- **LLM**: llama.cpp via llama-cpp-2 bindings (GGUF models)
- **PDF**: PDFium (bundled)
- **OCR**: macOS Vision framework

## Architecture Quick Reference

```
src/                  # React frontend
├── components/       # UI (Draft/, Layout/, Settings/, Sources/)
├── contexts/         # AppStatusContext (centralized state)
├── hooks/            # useLlm, useKb

src-tauri/src/        # Rust backend
├── lib.rs            # AppState, Tauri setup
├── commands/         # Tauri commands (~200 endpoints)
│   └── mod.rs        # Main command file
├── db/               # SQLCipher database layer
├── kb/               # Knowledge base
│   ├── indexer.rs    # File indexing
│   ├── search.rs     # Hybrid FTS + vector
│   ├── embeddings.rs # Embedding model
│   ├── vectors.rs    # LanceDB
│   └── ingest/       # web.rs, youtube.rs, github.rs
├── llm.rs            # LLM engine
├── security.rs       # Encryption, key management
└── audit.rs          # Security logging
```

## Key Patterns

### Commands
All backend operations use `#[tauri::command]`:
```rust
#[tauri::command]
pub async fn command_name(state: State<'_, AppState>, ...) -> Result<T, String>
```

### Frontend Invocation
```typescript
const result = await invoke<Type>('command_name', { arg1, arg2 });
```

### AppState
```rust
pub struct AppState {
    pub db: Mutex<Option<Database>>,
    pub llm: Arc<RwLock<Option<LlmEngine>>>,
    pub embeddings: Arc<RwLock<Option<EmbeddingEngine>>>,
    pub vectors: Arc<TokioRwLock<Option<VectorStore>>>,
    pub jobs: Arc<JobManager>,
}
```

## Security Model
- **Database**: SQLCipher AES-256 encryption
- **Key Storage**: macOS Keychain OR Argon2id passphrase-wrapped
- **Tokens**: AES-256-GCM encrypted at rest
- **File Access**: Restricted to `$HOME`, sensitive dirs blocked
- **Network**: SSRF protection (private IP blocking, DNS rebinding prevention)

## Testing

```bash
# Backend (225 tests)
cd src-tauri && cargo test

# Frontend (72 tests)
pnpm test

# Benchmarks
cd src-tauri && cargo bench

# Security audit
cd src-tauri && cargo audit
```

## Development

```bash
pnpm install
pnpm tauri dev      # Development
pnpm tauri build    # Production
```

## Current State (v0.3.0)
- All core features complete
- Security hardening complete
- Comprehensive test coverage
- Documentation complete

## Documentation
- [Architecture](docs/ARCHITECTURE.md) - Full system design
- [Security](docs/SECURITY.md) - Encryption, threat model
- [Installation](docs/INSTALLATION.md) - Setup guide
- [Performance](docs/PERFORMANCE.md) - Tuning guide
- [Operations](docs/OPERATIONS.md) - User guide

## Code Standards
- Rust: Handle errors explicitly, use `Result<T, E>`
- TypeScript: Strict mode, explicit types
- Tests: Run before commits
- No hedging/apologizing in code comments
