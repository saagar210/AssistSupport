# AssistSupport

A local-first AI-powered customer support response generator built with Tauri, React, and Rust. All data stays on your machine — no cloud, no telemetry.

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

| Layer | Technology |
|-------|-----------|
| Frontend | React 19 + TypeScript (strict) + Vite |
| Backend | Rust + Tauri 2.x |
| Database | SQLite with SQLCipher encryption + FTS5 |
| Vector Store | LanceDB |
| LLM Runtime | llama.cpp via llama-cpp-2 bindings |
| PDF Processing | PDFium (bundled) |
| OCR | macOS Vision framework |

## Quick Start

### Prerequisites

| Requirement | Version | Notes |
|-------------|---------|-------|
| macOS | 13+ (Ventura) | Required for Vision OCR and Tauri 2 |
| Node.js | 20+ | |
| pnpm | 8+ | `npm install -g pnpm` |
| Rust | 1.75+ | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| Xcode CLT | Latest | `xcode-select --install` |

### Install & Run

```bash
# Clone the repo
git clone https://github.com/YOUR_USERNAME/AssistSupport.git
cd AssistSupport

# Install frontend dependencies
pnpm install

# Run in development mode (starts both frontend dev server and Tauri backend)
pnpm tauri dev
```

### Build for Production

```bash
pnpm tauri build
```

The `.dmg` / `.app` output will be in `src-tauri/target/release/bundle/`.

### Testing

```bash
# Frontend tests (72 tests)
pnpm test

# Backend tests (366 tests)
cd src-tauri && cargo test

# Performance benchmarks
cd src-tauri && cargo bench
```

## Project Structure

```
src/                    # React frontend (TypeScript)
├── components/         # UI components by feature
├── contexts/           # React contexts (app state, theme, toast)
├── hooks/              # Custom hooks (LLM, KB, drafts, etc.)
└── styles/             # CSS design tokens and components

src-tauri/src/          # Rust backend
├── commands/           # Tauri command handlers (~200 endpoints)
├── db/                 # SQLCipher database layer
├── kb/                 # Knowledge base (indexer, search, embeddings, vectors)
│   └── ingest/         # Web, YouTube, GitHub content ingestion
├── llm.rs              # LLM engine (llama.cpp bindings)
├── security.rs         # Encryption, key management, SecureString
└── audit.rs            # Security audit logging
```

## Documentation

| Document | Description |
|----------|-------------|
| [Architecture](docs/ARCHITECTURE.md) | System design and code structure |
| [Security](docs/SECURITY.md) | Encryption, key management, threat model |
| [Installation](docs/INSTALLATION.md) | Setup and configuration guide |
| [Performance](docs/PERFORMANCE.md) | Tuning and optimization |
| [Operations](docs/OPERATIONS.md) | Daily usage and maintenance |
| [Changelog](CHANGELOG.md) | Release history |

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

## Troubleshooting

**Rust build fails with missing system libraries**
```bash
# Ensure Xcode Command Line Tools are installed
xcode-select --install
```

**`pnpm tauri dev` fails to start**
```bash
# Rebuild from clean state
rm -rf src-tauri/target node_modules
pnpm install
pnpm tauri dev
```

**LLM model fails to load**
- Ensure model is a valid `.gguf` file
- Check available RAM (models need 2-8GB depending on size)
- Try a smaller model first (Llama 3.2 1B)

**Database encryption error on first launch**
- The app creates its database at `~/Library/Application Support/AssistSupport/`
- If migrating from a previous version, check the migration log in the app

## Security

See [SECURITY.md](docs/SECURITY.md) for the full security model.

To report a vulnerability, please see the [security policy](SECURITY.md).

## License

[MIT](LICENSE)
