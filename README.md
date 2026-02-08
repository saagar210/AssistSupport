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

## What can it do?

| Feature | What it means for you |
|---------|----------------------|
| **7-format ingestion** | Drop in PDF, Markdown, HTML, TXT, DOCX, CSV, EPUB — VaultMind parses, chunks, and embeds them automatically |
| **Semantic search** | Find things by *meaning*, not just exact keywords. "papers about climate impact" finds relevant docs even if they never use those exact words |
| **Hybrid search** | Combines vector similarity + BM25 keyword search with Reciprocal Rank Fusion — best of both worlds |
| **Chat with your docs** | Ask questions, get answers grounded in YOUR data with inline citations pointing to exact sources |
| **Knowledge graph** | See how your documents connect through an interactive force-directed graph — discover relationships you didn't know existed |
| **Collections** | Organize documents into separate knowledge bases for different projects, topics, or clients |
| **Command palette** | `Cmd+K` to do anything. Power users rejoice. |
| **Dark mode** | Because of course. (Light mode and system-auto too.) |

## Tech Stack

```
Frontend:  React 19 + TypeScript + Tailwind CSS 4 + Zustand
Backend:   Tauri 2 + Rust + SQLite (WAL mode + FTS5)
AI:        Ollama (local embeddings + chat)
Graph:     react-force-graph-2d
Build:     Vite 7 + Cargo
Tests:     Vitest + Cargo test (76 tests)
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

That's it. Drop some documents in and start exploring.

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

**Ingestion**: Documents are parsed into text, split into overlapping chunks with section context, embedded via Ollama, and stored in SQLite alongside FTS5 full-text indexes.

**Search**: Your query hits both a vector similarity search and a BM25 keyword search simultaneously. Results are fused using Reciprocal Rank Fusion for the best of both approaches.

**Chat**: Questions trigger hybrid search to find the most relevant chunks, which get injected as context into Ollama's chat model. Every answer comes with clickable citations back to the source material.

**Graph**: Document chunks are compared by embedding similarity. Cross-document connections above a configurable threshold become edges in an interactive force-directed graph, revealing hidden relationships across your knowledge base.

## Project Structure

```
Vaultmind/
├── src/                          # React frontend
│   ├── components/               #   Sidebar, Header, StatusBar, Toast, CommandPalette
│   ├── views/                    #   Documents, Search, Chat, Graph, Settings, Detail
│   ├── stores/                   #   Zustand state (app, collections, docs, chat, settings)
│   ├── hooks/                    #   useTheme, useOllamaStatus, useKeyboardShortcuts
│   ├── utils/                    #   Shared utilities (file type colors)
│   └── types/                    #   TypeScript interfaces
├── src-tauri/                    # Rust backend
│   └── src/
│       ├── commands/             #   Tauri IPC handlers (7 modules)
│       ├── parsers/              #   Format parsers (PDF, MD, HTML, TXT, DOCX, CSV, EPUB)
│       ├── chunker.rs            #   Text chunking with overlap + section context
│       ├── embedder.rs           #   Concurrent embedding via Ollama
│       ├── vector_store.rs       #   Vector similarity search
│       ├── graph.rs              #   Knowledge graph edge generation
│       ├── ollama.rs             #   Ollama HTTP client (embed + chat stream)
│       ├── db.rs                 #   SQLite setup + migrations
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
| `Cmd+N` | New Conversation |
| `Cmd+,` | Settings |
| `Cmd+Enter` | Send Message |

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
pnpm test               # Frontend tests (22 tests)
pnpm lint               # TypeScript type check
cd src-tauri && cargo test   # Rust tests (54 tests)
cd src-tauri && cargo clippy # Rust linting
```

## License

[MIT](./LICENSE) — do whatever you want with it.

---

<div align="center">

**Built with Rust, React, and a stubborn belief that your data should stay yours.**

</div>
