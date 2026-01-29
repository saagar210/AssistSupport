# AssistSupport

**Intelligent IT Support Powered by Semantic Search**

![Version](https://img.shields.io/badge/version-1.0.0-10a37f)
![Platform](https://img.shields.io/badge/platform-macOS-lightgrey)
![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)
![Tests](https://img.shields.io/badge/tests-passing-brightgreen)
[![Compliance](https://img.shields.io/badge/compliance-HIPAA%20%7C%20GDPR%20%7C%20FISMA%20%7C%20SOC2-blue)](docs/compliance/COMPLIANCE_REPORT.md)

AssistSupport combines local AI models with hybrid semantic search to generate accurate, KB-informed IT support responses. Fully local, fully encrypted, zero cloud dependencies.

```
User asks:    "Can I use a flash drive?"
Intent:       POLICY detected (98% confidence)
Search finds: USB/removable media policy in 22ms
AI drafts:    "Per IT Security Policy 4.2, removable storage devices..."
You copy:     Paste into Jira — done in under a minute
```

---

## What's New in v1.0.0

**Hybrid semantic search** — the system now understands the *meaning* behind questions, not just keywords.

| Before (keyword search) | Now (semantic search) |
|---|---|
| "USB policy" returns 50 docs | "Can I use a flash drive?" returns the right policy |
| "password" returns noise | "How do I reset it?" returns step-by-step guide |
| "VPN" returns networking docs | "Can I work from home?" returns remote work policy |

**Key additions:**
- **BM25 + HNSW vector search** across 3,536 articles via PostgreSQL 16 + pgvector
- **Intent detection** — automatic POLICY / PROCEDURE / REFERENCE classification
- **User feedback** — rate results to track and improve accuracy
- **Monitoring** — live metrics dashboard (latency, accuracy, intent distribution)
- **Performance** — p50: 8ms, p95: 82ms (target <100ms)

---

## How It Works

```
User Question: "Can I work from home?"
        │
        ▼
┌─────────────────────┐
│  Intent Detection   │
│  → POLICY (98%)     │
└────────┬────────────┘
         │
    ┌────┴────┐
    │         │
    ▼         ▼
┌────────┐ ┌────────┐
│ BM25   │ │ Vector │
│ Search │ │ Search │
│(keyw.) │ │(384dim)│
└───┬────┘ └───┬────┘
    │          │
    ▼          ▼
┌─────────────────────┐
│  Adaptive Fusion    │
│  Policy boost: 1.5x │
│  Rank & combine     │
└────────┬────────────┘
         │
         ▼
┌─────────────────────┐
│  Ranked Results     │
│  1. Remote Policy   │
│  2. WFH Procedure   │
│  3. Guidelines      │
└────────┬────────────┘
         │
         ▼
┌─────────────────────┐
│  User Feedback      │
│  Helpful ✓          │
│  → Logged, tracked  │
└─────────────────────┘
```

---

## Quick Start

### Prerequisites

| Requirement | Version | Notes |
|-------------|---------|-------|
| macOS | 13+ (Ventura) | Apple Silicon or Intel |
| Node.js | 20+ | |
| pnpm | 8+ | `npm install -g pnpm` |
| Rust | 1.75+ | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| Xcode CLT | Latest | `xcode-select --install` |
| System libs | | `brew install protobuf pkgconf cmake leptonica tesseract` |

### Install & Run

```bash
git clone https://github.com/saagar210/AssistSupport.git
cd AssistSupport
pnpm install
pnpm tauri dev
```

### Build for Production

```bash
pnpm tauri build
# Output: src-tauri/target/release/bundle/dmg/AssistSupport_1.0.0_aarch64.dmg
```

### Hybrid Search Backend

To enable the PostgreSQL Hybrid Search tab (Cmd+8):

```bash
# Install PostgreSQL + pgvector
brew install postgresql@16
brew services start postgresql@16

# Create database and user
createuser -s assistsupport_dev
createdb -U assistsupport_dev assistsupport_dev

# Start the search API
cd ~/assistsupport-semantic-migration/week4
source venv/bin/activate
python3 search_api.py
# API runs on http://localhost:3000
```

### First Run

1. **Key Storage** — Choose Keychain (recommended) or passphrase mode
2. **Model Selection** — Pick an LLM model (Llama 3.2 3B recommended)
3. **Knowledge Base** — Point to your team's documentation folder
4. **Generate** — Type a ticket summary, search your KB, get a draft response
5. **Hybrid Search** — Click Search tab (Cmd+8) for PostgreSQL semantic search

---

## Features

### Hybrid Semantic Search (Cmd+8)
- **BM25 + HNSW vector search** across 3,536 knowledge base articles via PostgreSQL 16 + pgvector
- **Intent detection** — classifies queries as POLICY, PROCEDURE, or REFERENCE with confidence scoring
- **Adaptive score fusion** — BM25 keyword and 384-dim vector scores combined based on detected intent
- **Category boosting** — policy articles boosted 1.5x when policy intent is detected
- **User feedback** — rate individual results as helpful, not helpful, or incorrect
- **Monitoring dashboard** — live metrics: query volume, p50/p95/p99 latency, accuracy, intent distribution

### Response Generation
- Generate professional IT support responses with local LLM inference (llama.cpp)
- Responses automatically cite relevant KB articles
- Generate multiple alternatives for side-by-side comparison
- Rate responses (1-5 stars) to track quality over time
- Save top-rated responses as reusable templates
- Conversation-style input with context threading
- Two-section format: OUTPUT (copy-paste ready) + IT SUPPORT INSTRUCTIONS (engineer guidance)

### Knowledge Base
- Index markdown, PDF, DOCX, XLSX, code files, and images
- Hybrid search: FTS5 full-text + LanceDB vector/semantic search
- Policy-first search ranking for permission/restriction queries
- Disk ingestion pipeline with source/run tracking and incremental re-indexing
- OCR support via macOS Vision framework (screenshots, scanned PDFs)
- Web page, YouTube transcript, and GitHub repo ingestion
- Namespace organization for multi-team KB separation
- KB health monitoring with staleness indicators

### Jira Integration
- Fetch ticket context (title, description, assignee, status)
- Post responses directly to Jira tickets
- Transition tickets to new status after responding
- Template variables (`{{ticket_id}}`, `{{reporter}}`, `{{company_name}}`)

### Analytics & Monitoring
- Response quality tracking (ratings, trends)
- KB usage metrics (search frequency, top queries, article citations)
- Pilot feedback system with CSV export
- Search monitoring dashboard (latency percentiles, accuracy, intent distribution)

### Security & Privacy
- **Fully local** — all processing on your machine, zero cloud dependencies
- **AES-256 database encryption** via SQLCipher
- **AES-256-GCM token encryption** for stored credentials
- **macOS Keychain** or Argon2id passphrase-wrapped key storage
- **SSRF protection** with DNS pinning for web ingestion
- **Audit logging** for security events
- **Compliance validated** against [HIPAA, GDPR, FISMA, SOC2, ISO 27001, PCI DSS, NIST SP 800-53](docs/compliance/COMPLIANCE_REPORT.md)

### Productivity
- Command palette (Cmd+K) and full keyboard-first workflow
- Session tokens — 24h auto-unlock, no password friction on every launch
- Fast startup — background model loading with cached state (2-3 seconds)
- Batch processing for similar tickets
- Draft management with autosave
- CLI tool for search and indexing outside the GUI

---

## Why Not Just Use ChatGPT?

| Feature | AssistSupport | ChatGPT / Claude API | Zendesk / Freshdesk |
|---------|---------------|----------------------|---------------------|
| **Works Offline** | Yes | No | No |
| **Data Stays Local** | Yes — on your machine | Sent to cloud | Sent to cloud |
| **Searches Your KB** | Yes — automatic | No — manual prompt | Partial |
| **Semantic Search** | Yes — intent-aware | No | Basic |
| **HIPAA Compliant** | Yes | No | Depends on plan |
| **IT-Specific** | Yes — built for support | Generic | Generic AI add-on |
| **Cost** | Free (MIT) | $0.001-0.003/token | $50-500+/month |

---

## Performance

| Metric | Target | Actual | Status |
|--------|--------|--------|--------|
| p50 latency | <50ms | 8ms | 6x faster than target |
| p95 latency | <100ms | 82ms | Meets target |
| Avg latency | <50ms | 25ms | 2x faster than target |
| Embedding coverage | 100% | 3,536/3,536 | Complete |
| Intent detection | Functional | POLICY/PROCEDURE/REFERENCE | Active |
| Production readiness | 5/5 | 5/5 PASS | Production ready |

---

## Architecture

### Tech Stack

| Layer | Technology |
|-------|-----------|
| Frontend | React 19 + TypeScript (strict) + Vite |
| Backend | Rust + Tauri 2.x |
| Database | SQLite + SQLCipher (AES-256) + FTS5 |
| Search Backend | PostgreSQL 16 + pgvector 0.8 (BM25 + HNSW) |
| Search API | Python Flask on localhost:3000 |
| Vector Store | LanceDB (local), pgvector (PostgreSQL) |
| LLM Runtime | llama.cpp via llama-cpp-2 (GGUF models) |
| PDF | PDFium (bundled) |
| OCR | macOS Vision framework |

### Data Flow

```
┌───────────────────────────────────────────────────────────────┐
│  AssistSupport.app (Tauri 2.x + React 19)                    │
└──────────────────────────┬────────────────────────────────────┘
                           │
              ┌────────────┴────────────┐
              │                         │
              ▼                         ▼
┌──────────────────────┐   ┌──────────────────────────────────┐
│  Local LLM Pipeline  │   │  Hybrid Search API               │
│                      │   │  (Flask on localhost:3000)        │
│  SQLite (encrypted)  │   │                                   │
│  ├─ FTS5 Index       │   │  POST /search  → BM25 + HNSW     │
│  ├─ LanceDB Vectors  │   │  POST /feedback → User ratings   │
│  └─ Response History │   │  GET  /stats   → Live metrics     │
│                      │   │  GET  /health  → Service health   │
│  llama.cpp (GGUF)    │   │                                   │
│  └─ Draft generation │   │  PostgreSQL 16 + pgvector         │
└──────────────────────┘   │  ├─ 3,536 articles               │
                           │  ├─ HNSW index (384-dim)          │
                           │  ├─ GIN FTS index (BM25)          │
                           │  └─ query_performance + feedback  │
                           └──────────────────────────────────┘
```

### Project Structure

```
src/                        # React frontend
├── components/
│   ├── Analytics/          # Dashboard, article drill-down
│   ├── Batch/              # Batch processing
│   ├── Draft/              # Response drafting, alternatives, ratings
│   ├── Layout/             # Header, sidebar, command palette
│   ├── Pilot/              # Pilot feedback: query tester, dashboard
│   ├── Search/             # Hybrid PostgreSQL search UI, feedback, stats
│   ├── Settings/           # Model, KB, Jira configuration
│   ├── Sources/            # KB browser, ingestion, health
│   └── shared/             # Onboarding, status indicators
├── contexts/               # AppStatusContext (centralized state)
├── hooks/                  # useLlm, useKb, useHybridSearch, useInitialize
└── styles/                 # CSS design tokens, themes

src-tauri/src/              # Rust backend
├── commands/               # Tauri command handlers (~209 endpoints)
│   └── search_api.rs       # PostgreSQL hybrid search proxy (4 commands)
├── db/                     # SQLCipher database layer (schema v11)
├── feedback/               # Pilot feedback logger, stats, CSV export
├── kb/                     # Knowledge base (indexer, search, embeddings, vectors, ingest)
├── llm.rs                  # LLM engine (llama.cpp)
├── jira.rs                 # Jira API integration
├── security.rs             # Encryption, key management
├── audit.rs                # Security audit logging
└── diagnostics.rs          # Health checks, maintenance
```

See [ARCHITECTURE.md](docs/ARCHITECTURE.md) for the full system design.

---

## For IT Support Teams

### Individual Setup
Each engineer clones, installs, and runs. Point the KB to a local docs folder or shared drive.

### Team Shared KB (Recommended)
Set up a shared documentation folder and have each engineer point AssistSupport at it:

```
IT_KnowledgeBase/
├── Windows/          # connectivity, accounts, software
├── Network/          # VPN, printing, email
├── Accounts/         # password resets, MFA
└── Procedures/       # onboarding, offboarding
```

See the [IT Support Guide](docs/IT_SUPPORT_GUIDE.md) for detailed deployment options and workflow examples.

---

## Testing

```bash
# Frontend tests
pnpm test

# Backend tests (unit + integration)
cd src-tauri && cargo test

# Performance benchmarks
cd src-tauri && cargo bench

# Security audit
cd src-tauri && cargo audit
```

See [Testing Guide](docs/TESTING.md) for the full test suite documentation.

---

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
| `Cmd+1-9` | Switch tabs |
| `Cmd+8` | Open Hybrid Search |

---

## Documentation

| Document | Description |
|----------|-------------|
| [Architecture](docs/ARCHITECTURE.md) | System design, data flow, extension points |
| [Security](docs/SECURITY.md) | Encryption, key management, threat model |
| [Compliance Report](docs/compliance/COMPLIANCE_REPORT.md) | HIPAA/GDPR/FISMA/SOC2/ISO 27001 validation |
| [Installation](docs/INSTALLATION.md) | Setup and configuration guide |
| [Quick Start for IT](docs/QUICKSTART_IT_SUPPORT.md) | 5-minute setup guide |
| [IT Support Guide](docs/IT_SUPPORT_GUIDE.md) | Workflows, team setup, Jira integration |
| [Performance](docs/PERFORMANCE.md) | Tuning and optimization |
| [Operations](docs/OPERATIONS.md) | Daily usage and maintenance |
| [Testing](docs/TESTING.md) | Test suite, health checks, CI/CD |
| [Roadmap](docs/ROADMAP.md) | Feature priorities and release plan |
| [Changelog](CHANGELOG.md) | Release history |

---

## Changelog

### v1.0.0 (Current) — Production Release
- Hybrid PostgreSQL search (BM25 + HNSW vector, 3,536 articles)
- Intent detection (POLICY/PROCEDURE/REFERENCE classification)
- Adaptive score fusion with category boosting
- Search tab (Cmd+8) with result cards, score breakdowns, API health indicator
- User feedback collection (helpful/not_helpful/incorrect ratings)
- Live monitoring dashboard (query volume, latency percentiles, accuracy, intent distribution)
- Flask REST API (5 endpoints on localhost:3000 with rate limiting)
- 4 Tauri commands proxying to Flask API via reqwest
- p95 latency 82ms, 100% embedding coverage, 5/5 production readiness checks

### v0.6.0
- Pilot feedback system (query tester, star ratings, dashboard, CSV export)
- Disk ingestion pipeline with source/run tracking
- Incremental re-indexing via SHA-256 hash comparison
- Policy-first search ranking with confidence scoring

### v0.5.x
- ChatGPT-inspired UI redesign (dark-first, green accent)
- Fast startup with background model loading (2-3 seconds)
- Analytics dashboard with ratings and article drill-down
- Response alternatives and template recycling
- Jira post + transition workflow
- KB health and staleness monitoring

### Next
- [ ] Draft management improvements (save, resume, history)
- [ ] KB management UI (create/edit articles in-app)
- [ ] Advanced analytics (ROI metrics, team benchmarking)
- [ ] Windows support
- [ ] ServiceNow integration

See [ROADMAP.md](docs/ROADMAP.md) for the full release plan.

---

## Troubleshooting

**Rust build fails with missing system libraries**
```bash
brew install protobuf pkgconf cmake leptonica tesseract
xcode-select --install
```

**`pnpm tauri dev` fails to start**
```bash
rm -rf src-tauri/target node_modules
pnpm install
pnpm tauri dev
```

**LLM model fails to load**
- Ensure model is a valid `.gguf` file
- Check available RAM (models need 2-8GB depending on size)
- Try a smaller model first (Llama 3.2 1B)

**Search tab shows "API Offline"**
- Ensure PostgreSQL is running: `brew services start postgresql@16`
- Ensure Flask API is running: `cd ~/assistsupport-semantic-migration/week4 && python3 search_api.py`
- Check API health: `curl http://localhost:3000/health`

**Database encryption error on first launch**
- The app creates its database at `~/Library/Application Support/AssistSupport/`
- If migrating from a previous version, check the migration log in the app

---

## Contributing

Contributions welcome. See the [roadmap](docs/ROADMAP.md) for planned features.

```bash
# Fork and clone
git clone https://github.com/<your-fork>/AssistSupport.git
cd AssistSupport

# Create feature branch
git checkout -b feature/your-feature

# Install and develop
pnpm install
pnpm tauri dev

# Run tests before submitting
pnpm test
cd src-tauri && cargo test && cargo clippy

# Push and create PR
git push origin feature/your-feature
```

---

## Security

See [SECURITY.md](docs/SECURITY.md) for the full security model and [Compliance Report](docs/compliance/COMPLIANCE_REPORT.md) for validation against 7 security standards.

To report a vulnerability, please open a security advisory on GitHub.

---

## License

[MIT](LICENSE)

---

Built with [React](https://react.dev), [Tauri](https://tauri.app), [Rust](https://www.rust-lang.org), [llama.cpp](https://github.com/ggerganov/llama.cpp), [SQLite](https://sqlite.org), [LanceDB](https://lancedb.com), [PostgreSQL](https://www.postgresql.org), [pgvector](https://github.com/pgvector/pgvector), and [Flask](https://flask.palletsprojects.com).
