# VaultMind

**Local-first AI-powered knowledge management**

VaultMind is a privacy-focused desktop application that lets you ingest documents, search them with semantic understanding, have conversations grounded in your own data, and explore the connections between your knowledge through an interactive graph.

Everything runs locally on your machine. Your documents never leave your computer.

## Key Features

- **Document Ingestion** -- Import PDF, Markdown, HTML, Plain Text, DOCX, CSV, and EPUB files with automatic chunking and embedding
- **Semantic Search** -- Find information by meaning, not just keywords, using local embedding models
- **Hybrid Search** -- Combines vector similarity and keyword (BM25) search with Reciprocal Rank Fusion for best results
- **Conversational RAG** -- Chat with your documents using retrieval-augmented generation powered by local LLMs
- **Knowledge Graph** -- Visualize entity relationships extracted from your documents in an interactive force-directed graph
- **Collections** -- Organize documents into collections for scoped search and conversation
- **Command Palette** -- Keyboard-driven navigation (Cmd+K) for power users
- **Dark/Light Mode** -- Automatic theme detection with manual override
- **100% Local** -- No cloud services, no API keys, no data leaving your machine

## Screenshots

### Document Library
*Coming soon*

### Semantic Search
*Coming soon*

### Chat with Documents
*Coming soon*

### Knowledge Graph
*Coming soon*

## Tech Stack

| Layer      | Technology                        |
|------------|-----------------------------------|
| Framework  | Tauri 2 (Rust)                    |
| Frontend   | React 19, TypeScript, Tailwind 4  |
| State      | Zustand                           |
| Database   | SQLite (rusqlite)                 |
| AI Runtime | Ollama (local LLMs + embeddings)  |
| Build      | Vite 7, Cargo                     |

## Prerequisites

- **macOS** (Apple Silicon or Intel)
- **Ollama** installed and running -- [Download Ollama](https://ollama.com)
- **Node.js** 18+ and **pnpm**
- **Rust** toolchain (rustup)

## Quick Start

1. Install Ollama and pull the required models:

   ```bash
   ollama pull nomic-embed-text && ollama pull llama3.2
   ```

2. Clone the repository:

   ```bash
   git clone https://github.com/your-username/vaultmind.git
   cd vaultmind
   ```

3. Install frontend dependencies:

   ```bash
   pnpm install
   ```

4. Run in development mode:

   ```bash
   pnpm tauri dev
   ```

## Development

### Commands

```bash
# Development server with hot reload
pnpm tauri dev

# Build production release
pnpm tauri build

# Run frontend tests
pnpm test

# Run frontend tests in watch mode
pnpm test:watch

# Type-check frontend
pnpm lint

# Check Rust code
cd src-tauri && cargo check

# Run Rust tests
cd src-tauri && cargo test
```

### Project Structure

```
vaultmind/
├── src/                    # React frontend
│   ├── components/         # Reusable UI components
│   ├── views/              # Page-level views
│   ├── stores/             # Zustand state stores
│   ├── hooks/              # Custom React hooks
│   ├── types/              # TypeScript type definitions
│   └── styles/             # Global styles
├── src-tauri/              # Rust backend
│   └── src/
│       ├── commands/       # Tauri IPC command handlers
│       ├── parsers/        # Document format parsers
│       ├── chunker.rs      # Text chunking engine
│       ├── embedder.rs     # Ollama embedding client
│       ├── vector_store.rs # Vector similarity search
│       ├── graph.rs        # Knowledge graph extraction
│       ├── ollama.rs       # Ollama API client
│       ├── db.rs           # SQLite database layer
│       ├── models.rs       # Data models
│       ├── error.rs        # Error types
│       └── state.rs        # Application state
└── public/                 # Static assets
```

## Architecture Overview

VaultMind follows a standard Tauri 2 architecture with a React frontend communicating with a Rust backend over IPC.

**Frontend** renders the UI and manages client-side state with Zustand. User actions invoke Tauri commands that execute in the Rust backend.

**Backend** handles all data processing: parsing documents into text, splitting text into chunks, generating embeddings via Ollama, storing everything in SQLite, and orchestrating search and chat pipelines.

**Data Flow:**
1. **Ingestion** -- Documents are parsed, chunked, embedded, and stored in SQLite with their vector representations
2. **Search** -- Queries run through both vector similarity and keyword search, combined via Reciprocal Rank Fusion
3. **Chat** -- User messages trigger hybrid search to find relevant chunks, which are injected as context into LLM prompts
4. **Graph** -- Entity extraction runs over document chunks to build a knowledge graph of relationships

See [ARCHITECTURE.md](./ARCHITECTURE.md) for the full technical breakdown.

## Supported File Formats

| Format     | Extension  |
|------------|------------|
| PDF        | `.pdf`     |
| Markdown   | `.md`      |
| HTML       | `.html`    |
| Plain Text | `.txt`     |
| DOCX       | `.docx`    |
| CSV        | `.csv`     |
| EPUB       | `.epub`    |

## License

[MIT](./LICENSE)
