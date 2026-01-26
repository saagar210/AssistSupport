# AssistSupport

A local-first AI-powered customer support response generator built with Tauri, React, and Rust.

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
- **Namespace Organization**: Organize knowledge into separate namespaces

### Productivity
- **Draft Management**: Save, search, and organize response drafts with autosave
- **Template Variables**: Define custom variables (`{{company_name}}`) for consistent responses
- **Jira Integration**: Fetch and inject ticket context into generated responses
- **Command Palette**: Quick access to all actions (Cmd+K)
- **Keyboard Shortcuts**: Full keyboard-first workflow

### Security & Privacy
- **Fully Local**: All processing happens on your machine
- **Encrypted Database**: SQLCipher with AES-256 encryption
- **Dual Key Storage**: macOS Keychain or passphrase-protected
- **Secure Token Storage**: AES-256-GCM encrypted credentials
- **Path Security**: Home directory restriction, sensitive directory blocking
- **SSRF Protection**: Comprehensive network security for web ingestion
- **Audit Logging**: Security event tracking

## Tech Stack

- **Frontend**: React 19 + TypeScript + Vite
- **Backend**: Rust + Tauri 2.x
- **Database**: SQLite with SQLCipher encryption + FTS5
- **Vector Store**: LanceDB
- **LLM Runtime**: llama.cpp via llama-cpp-2 bindings
- **PDF Processing**: PDFium (bundled)
- **OCR**: macOS Vision framework

## Quick Start

### Prerequisites
- Node.js 20+
- Rust 1.75+
- pnpm 8+ (or npm)
- macOS 13+ (Ventura or later)

### Development
```bash
# Install dependencies
pnpm install

# Development mode
pnpm tauri dev

# Production build
pnpm tauri build
```

### Testing
```bash
# Frontend tests
pnpm test

# Backend tests
cd src-tauri && cargo test

# Benchmarks
cd src-tauri && cargo bench
```

## Documentation

| Document | Description |
|----------|-------------|
| [Architecture](docs/ARCHITECTURE.md) | System design and code structure |
| [Security](docs/SECURITY.md) | Encryption, key management, threat model |
| [Installation](docs/INSTALLATION.md) | Setup and configuration guide |
| [Performance](docs/PERFORMANCE.md) | Tuning and optimization |
| [Operations](docs/OPERATIONS.md) | Daily usage and maintenance |

## Project Status

**Version**: 0.3.0

### Test Coverage
- **366 backend tests** (unit, integration, security, E2E)
- **72 frontend tests**
- TypeScript strict mode enabled
- Performance benchmarks with Criterion

### Completed Features
- Full LLM integration with streaming generation
- Hybrid search (FTS5 + vector)
- Multi-format document ingestion
- Web/YouTube/GitHub content ingestion
- Encrypted database and credential storage
- Dual key storage modes (Keychain/Passphrase)
- SSRF and path traversal protection
- Audit logging
- Health diagnostics and self-repair
- CLI tool for automation

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Cmd+K` | Command palette |
| `Cmd+Enter` | Generate response |
| `Cmd+S` | Save draft |
| `Cmd+Shift+C` | Copy response |
| `Cmd+E` | Export response |
| `Cmd+N` | New draft |
| `Cmd+/` | Focus search |
| `Cmd+1-6` | Switch tabs |

## License

MIT
