# AssistSupport

A local-first AI-powered customer support response generator built with Tauri, React, and TypeScript.

## Features

### Core Capabilities
- **Local LLM Integration**: Run GGUF models locally via llama.cpp (Qwen 2.5, Llama 3.2, Phi-4)
- **Knowledge Base Indexing**: Index markdown, PDF, DOCX, XLSX, and code files with FTS5 full-text search
- **Vector Search**: Hybrid semantic + keyword search using LanceDB and local embeddings
- **OCR Support**: Extract text from screenshots and scanned PDFs using macOS Vision framework
- **Decision Trees**: Guided diagnostic workflows for common support scenarios

### Content Ingestion
- **Web Pages**: Fetch and index public web content with SSRF protection
- **YouTube Transcripts**: Extract and index video transcripts (requires yt-dlp)
- **GitHub Repositories**: Index documentation from local or remote repos
- **Batch Import**: YAML-based batch ingestion with configurable crawl depth
- **Namespace Organization**: Organize knowledge into separate namespaces

### Productivity
- **Draft Management**: Save, search, and organize response drafts with autosave
- **Template Variables**: Define custom variables (`{{company_name}}`) for consistent responses
- **Jira Integration**: Fetch and inject ticket context into generated responses
- **Command Palette**: Quick access to all actions (Cmd+K)
- **Keyboard Shortcuts**: Full keyboard-first workflow:
  - Cmd+Enter: Generate response
  - Cmd+S: Save draft
  - Cmd+Shift+C: Copy response
  - Cmd+E: Export response
  - Cmd+1-6: Switch tabs
  - Cmd+Shift+/: View all shortcuts

### Security & Privacy
- **Fully Local**: All processing happens on your machine
- **Encrypted Database**: SQLCipher with AES-256 encryption
- **Secure Token Storage**: File-based credential storage with automatic migration
- **Input Validation**: Path traversal protection, URL/ticket ID validation, and size limits
- **Encrypted Backups**: Optional password protection for exported data (Argon2id + AES-256-GCM)
- **Content Security Policy**: Minimal CSP allowlist for app security

### Model Management
- **Pre-configured Models**: Llama 3.2, Phi-3 with one-click download
- **Custom GGUF Support**: Load any GGUF model file from your computer
- **Download Management**: Progress display with cancel support
- **Context Window Control**: Configurable context length with budget enforcement

## Tech Stack

- **Frontend**: React 19 + TypeScript + Vite
- **Backend**: Rust + Tauri 2.x
- **Database**: SQLite with SQLCipher encryption + FTS5
- **Vector Store**: LanceDB
- **LLM Runtime**: llama.cpp via llama-cpp-2 bindings
- **PDF Processing**: PDFium (bundled)
- **OCR**: macOS Vision framework

## Development

### Prerequisites
- Node.js 20+
- Rust 1.75+
- pnpm (or npm)
- macOS (for Vision OCR and Metal acceleration)

### Optional Dependencies
- **yt-dlp**: Required for YouTube transcript ingestion (`brew install yt-dlp`)

### Setup
```bash
# Install dependencies
pnpm install

# Development mode
pnpm tauri dev

# Build for production
pnpm tauri build
```

### Testing
```bash
# Frontend tests (Vitest)
pnpm test

# Backend tests (Rust)
cd src-tauri && cargo test

# Watch mode
pnpm test:watch
```

## Project Status

**Version**: 0.3.0 (Security Hardening Complete)

All 22 implementation phases are complete:

### Core Infrastructure
- Database (SQLCipher), LLM engine, KB indexer, OCR, Jira integration
- Vector search pipeline with hybrid ranking and RRF fusion
- Decision tree integration for guided diagnostics
- CLI tool for automation and scripting

### Modern UI/UX
- Design token system with consistent visual language
- Command palette (Cmd+K) for quick actions
- Full keyboard shortcut support (Cmd+1-6 tabs, Cmd+Enter generate)
- Onboarding wizard with security mode selection
- Centralized app status with header status panel
- Responsive layout with mobile TabBar

### Search & Retrieval
- Hybrid search with configurable FTS/vector weights
- Content deduplication using Jaccard similarity
- Sub-300ms search response for large KBs
- Namespace-based organization

### Performance
- Non-blocking engine initialization
- Optimized N+1 queries with single SQL JOINs
- Component memoization and result caching
- Database maintenance automation (VACUUM scheduling)
- Performance benchmarks for baseline metrics

### Security & Reliability
- Dual key storage (macOS Keychain or passphrase-protected)
- Encrypted token storage (AES-256-GCM)
- SSRF protection for web ingestion
- Model integrity verification (SHA256)
- Path traversal protection (home directory restriction)
- Audit logging for security events
- Health check system with self-repair commands

### Quality
- **225 backend tests** passing (unit, integration, security)
- **72 frontend tests** passing
- TypeScript strict mode, no errors
- Performance benchmarks with Criterion
- E2E integration tests for KB pipeline

## Documentation

- [Security Architecture](docs/SECURITY.md) - Encryption, key management, threat model
- [Installation Guide](docs/INSTALLATION.md) - Setup and configuration
- [Performance Guide](docs/PERFORMANCE.md) - Tuning and optimization
- [Operations Guide](docs/OPERATIONS.md) - Daily usage and maintenance

## License

MIT
