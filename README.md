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
- **Keyboard Shortcuts**: Cmd+G to generate, Cmd+N to clear, Cmd+S to save

### Security & Privacy
- **Fully Local**: All processing happens on your machine
- **Encrypted Database**: SQLCipher with AES-256 encryption
- **Secure Token Storage**: API tokens protected with zeroize-on-drop
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

**Version**: 0.1.0 (Feature Complete)

All development phases are complete:
- Core infrastructure (DB, LLM, KB indexer, OCR, Jira)
- Vector search pipeline with hybrid search
- Decision tree integration
- Jira deep integration
- Content ingestion (web, YouTube, GitHub, batch import)
- Knowledge browser with namespace management
- UI/UX polish (error boundaries, accessibility, export/import)
- Performance optimizations (parallel search, background indexing)
- Security hardening (SSRF protection, token protection, input validation)
- Advanced features (code indexing, file watching, PDF OCR)
- Test coverage (72 frontend, 84 backend tests passing)

The development roadmap and implementation history are tracked in the project planning files.

## License

MIT
