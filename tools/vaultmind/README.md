<div align="center">

# VaultMind

### Your documents. Your AI. Your machine. Nothing leaves.

[![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![React](https://img.shields.io/badge/React_19-61DAFB?style=for-the-badge&logo=react&logoColor=black)](https://react.dev/)
[![Tauri](https://img.shields.io/badge/Tauri_2-FFC131?style=for-the-badge&logo=tauri&logoColor=black)](https://tauri.app/)
[![SQLite](https://img.shields.io/badge/SQLite-003B57?style=for-the-badge&logo=sqlite&logoColor=white)](https://sqlite.org/)
[![Ollama](https://img.shields.io/badge/Ollama-000000?style=for-the-badge&logo=ollama&logoColor=white)](https://ollama.com/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=for-the-badge)](./LICENSE)

**Drop in your PDFs, Markdown, DOCX, HTML, EPUB, CSV, and plain text files.**
**Search by meaning. Chat with your knowledge. See the connections.**

*All powered by local LLMs through Ollama. Zero cloud. Zero tracking. Zero compromise.*

</div>

---

## What is VaultMind?

VaultMind is a desktop app that turns your messy pile of documents into a searchable, conversational, interconnected knowledge base — and it does it **entirely on your machine**.

No API keys. No subscriptions. No data leaving your laptop. Just you, your documents, and a local AI that actually understands them.

## Features

| Feature | What it means for you |
|---------|----------------------|
| **7-format ingestion** | Drop in PDF, Markdown, HTML, TXT, DOCX, CSV, EPUB — VaultMind parses, chunks, and embeds them automatically |
| **Real-time ingestion progress** | Per-file, per-stage progress tracking. Continue using the app while files process in the background |
| **Semantic search** | Find things by *meaning*, not just exact keywords. "papers about climate impact" finds relevant docs even if they never use those exact words |
| **Hybrid search** | Combines vector similarity + BM25 keyword search with Reciprocal Rank Fusion — best of both worlds |
| **Faceted search filters** | Filter results by file type. Search history chips for quick re-runs. "More like this" for any result |
| **Chat with your docs** | Ask questions, get answers grounded in YOUR data with expandable inline citations pointing to exact sources |
| **Markdown rendering** | Chat responses render with full markdown — headers, code blocks with syntax highlighting, tables, links |
| **Stop & regenerate** | Stop generation mid-stream or regenerate the last response. Pick your chat model per conversation |
| **Auto-titled conversations** | First exchange auto-generates a conversation title. Export any conversation as markdown |
| **Knowledge graph** | Interactive force-directed graph with file-type colored nodes, search/highlight, right-click context menus, and a legend |
| **Document tags** | Tag documents for organization and filtering |
| **Re-ingestion** | Changed chunk settings? Re-ingest individual documents or entire collections without re-importing |
| **Collections** | Organize documents into separate knowledge bases for different projects, topics, or clients |
| **Setup wizard** | First-run wizard guides you through Ollama connection and model selection |
| **Ollama recovery** | Auto-detects Ollama disconnection with a recovery banner that retries every 10 seconds |
| **Command palette** | `Cmd+K` to do anything. Power users rejoice |
| **Dark mode** | Light, dark, and system-auto themes |

## Tech Stack

```
Frontend:  React 19 + TypeScript + Tailwind CSS 4 + Zustand + Framer Motion
Backend:   Tauri 2 + Rust + SQLite (WAL mode + FTS5)
AI:        Ollama (local embeddings + chat with streaming)
Rendering: react-markdown + remark-gfm + rehype-highlight
Graph:     react-force-graph-2d
Build:     Vite 7 + Cargo
Tests:     Vitest + Cargo test (101 tests)
```

## Quick Start

### Prerequisites

- **macOS** (Apple Silicon or Intel)
- [**Ollama**](https://ollama.com) installed and running
- **Node.js** 18+ and [**pnpm**](https://pnpm.io/)
- **Rust** toolchain via [rustup](https://rustup.rs/)

### Get running in 60 seconds

```bash
# 1. Pull the AI models
ollama pull nomic-embed-text && ollama pull llama3.2

# 2. Clone and install
git clone https://github.com/saagar210/Vaultmind.git
cd Vaultmind
pnpm install

# 3. Launch
pnpm tauri dev
```

That's it. The setup wizard will guide you through connecting to Ollama and choosing your models. Drop some documents in and start exploring.

## How It Works

```
Documents ──> Parse ──> Chunk ──> Embed ──> Store
                                              │
                          ┌───────────────────┤
                          │                   │
                     Search/Chat          Knowledge Graph
                          │                   │
                    Hybrid Search         Force-directed
                   (Vector + BM25)        visualization
                          │                   │
                     RRF Fusion           Document clusters
                          │               + connections
                    Cited answers
```

**Ingestion**: Documents are parsed into text, split into overlapping chunks with section context, embedded via Ollama, and stored in SQLite alongside FTS5 full-text indexes. Real-time progress events track each stage (parsing, chunking, embedding, indexing).

**Search**: Your query hits both a vector similarity search and a BM25 keyword search simultaneously. Results are fused using Reciprocal Rank Fusion for the best of both approaches. Filter by file type, browse search history, or find similar content with one click.

**Chat**: Questions trigger hybrid search to find the most relevant chunks, which get injected as context into Ollama's streaming chat model. Every answer renders as rich markdown with expandable inline citations. Stop generation mid-stream, regenerate responses, or switch models on the fly.

**Graph**: Document chunks are compared by embedding similarity. Cross-document connections above a configurable threshold become edges in an interactive force-directed graph with file-type colored nodes, search/highlight, and right-click context menus.

## Project Structure

```
Vaultmind/
├── src/                          # React frontend
│   ├── components/               #   UI primitives (SetupWizard, MarkdownRenderer, GraphLegend, etc.)
│   ├── views/                    #   Documents, Search, Chat, Graph, Settings, Detail
│   ├── stores/                   #   Zustand state (app, collections, docs, chat, settings, toasts)
│   ├── hooks/                    #   useTheme, useOllamaStatus, useKeyboardShortcuts
│   ├── utils/                    #   Shared utilities (file type colors)
│   └── types/                    #   TypeScript interfaces
├── src-tauri/                    # Rust backend
│   └── src/
│       ├── commands/             #   Tauri IPC handlers (collections, documents, search, chat, graph, settings, ollama)
│       ├── parsers/              #   Format parsers (PDF, MD, HTML, TXT, DOCX, CSV, EPUB)
│       ├── chunker.rs            #   Text chunking with overlap + section context
│       ├── embedder.rs           #   Concurrent embedding via Ollama with progress events
│       ├── vector_store.rs       #   Vector similarity search
│       ├── graph.rs              #   Knowledge graph edge generation
│       ├── ollama.rs             #   Ollama HTTP client (embed + chat stream + cancellation)
│       ├── db.rs                 #   SQLite setup
│       ├── migrations.rs         #   Database migrations
│       ├── utils.rs              #   Shared math utilities
│       └── models.rs             #   Data structures
├── .github/workflows/ci.yml     # CI pipeline
└── ARCHITECTURE.md               # Deep technical docs
```

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Cmd+1` | Knowledge Graph |
| `Cmd+2` | Chat |
| `Cmd+3` | Documents |
| `Cmd+4` | Search |
| `Cmd+K` | Command Palette |
| `Cmd+O` | Import Files |
| `Cmd+N` | New Conversation (in Chat) |
| `Cmd+Shift+F` | Jump to Search |
| `Cmd+,` | Settings |
| `Cmd+Enter` | Send Message |
| `Escape` | Close palette / dialogs |

## Supported Formats

| Format | Extensions | Parser |
|--------|-----------|--------|
| PDF | `.pdf` | pdf-extract + lopdf |
| Markdown | `.md` | pulldown-cmark |
| HTML | `.html` | scraper |
| Plain Text | `.txt` | encoding_rs (auto-detect) |
| Word | `.docx` | zip + quick-xml |
| CSV | `.csv` | csv crate |
| EPUB | `.epub` | zip + scraper |

## Development

```bash
pnpm tauri dev          # Dev server with hot reload
pnpm tauri build        # Production build (macOS DMG)
pnpm test               # Frontend tests (44 tests)
pnpm lint               # TypeScript type check
cd src-tauri && cargo test   # Rust tests (57 tests)
cd src-tauri && cargo clippy # Rust linting
```

## License

[MIT](./LICENSE) — do whatever you want with it.

---

<div align="center">

**Built with Rust, React, and a stubborn belief that your data should stay yours.**

</div>
