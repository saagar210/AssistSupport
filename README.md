# AssistSupport

**ML-Powered Semantic Search for IT Support — Local-First, Encrypted Core Data, Zero Cloud**

![Version](https://img.shields.io/badge/version-1.0.0-10a37f)
![Platform](https://img.shields.io/badge/platform-macOS-lightgrey)
![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)
![Tests](https://img.shields.io/badge/tests-gated-blue)
![Coverage](https://img.shields.io/badge/coverage-enforced-blue)
[![Security](https://img.shields.io/badge/security-local--first-blue)](#security-and-privacy)

AssistSupport combines local LLM inference with an ML-powered hybrid search pipeline to generate accurate, KB-informed IT support responses. An ML intent classifier understands query meaning, a cross-encoder reranker sharpens relevance, and a feedback loop continuously improves results while keeping the core app and search stack on your machine.

Core workspace data is encrypted locally. Optional vector-search embeddings are stored locally as well, but they are not currently encrypted at rest when vector search is enabled.

```
User asks:    "Can I use a flash drive?"
ML Intent:    POLICY detected (86% confidence, ML classifier)
Search finds: USB/removable media policy in 21ms
Reranker:     Cross-encoder confirms top result relevance
AI drafts:    "Per IT Security Policy 4.2, removable storage devices..."
You copy:     Paste into Jira — done in under a minute
```

---

## Key Strengths

| Strength | Details |
|----------|---------|
| **ML-Powered Search** | TF-IDF + Logistic Regression intent classifier (85.7% accuracy), cross-encoder reranker (ms-marco-MiniLM-L-6-v2), adaptive score fusion |
| **Sub-25ms Latency** | p50: 8ms, p95: 82ms, avg: 21ms across 3,536 articles — 6x faster than target |
| **Local-First Runtime** | The app, search sidecar, and model inference run on the local machine. No cloud dependency is required for the core workflow |
| **Encrypted Core Data** | The main local database and stored secrets are protected locally with wrapped keys and encrypted-at-rest storage for the primary SQLite workspace |
| **Security Review in Progress** | The repo includes hardening gates, dependency audits, and local-only guardrails, but formal compliance validation is not currently claimed |
| **Trust-Gated Responses** | Confidence modes (answer/clarify/abstain), claim grounding map, citation-aware copy safety for low-confidence output |
| **Self-Improving** | Feedback loop + KB gap detector surfaces repeated low-confidence/low-rating topics and tracks remediation |
| **Ops-Ready** | Built-in Operations workspace for deployment preflight/rollback, eval harness runs, triage clustering, and runbook sessions |
| **Quality Gates Enabled** | Frontend static checks, unit tests, visual/a11y regression, Rust tests, API smoke checks, and diff coverage gates |
| **190+ API Commands** | Expanded Tauri command surface for trust signals, ops workflows, evaluations, integrations, and diagnostics |

---

## What's New in v1.0.0

### ML Intent Classifier
Replaced keyword heuristics with a trained ML model. TF-IDF vectorization + Logistic Regression trained on 182 examples achieves **85.7% cross-validation accuracy** classifying queries as POLICY, PROCEDURE, REFERENCE, or UNKNOWN — with average confidence jumping from 0.4 to 0.8+.

### Cross-Encoder Reranker
A `ms-marco-MiniLM-L-6-v2` cross-encoder rescores search candidates after initial retrieval. Blended scoring (15% cross-encoder + 85% fusion) surfaces the most relevant results while filtering noisy content from attachments and related-article sections.

### Feedback Loop
User ratings (helpful / not helpful / incorrect) feed back into search scoring. Per-article quality scores (0.5x-1.5x) activate after 3+ ratings, continuously tuning result ranking without manual intervention.

### Content Quality Pipeline
Cleaned 2,912 article titles (avg length 76 -> 57 chars), merged 672 thin chunks into 50 consolidated articles, enriched 40 popular-topic articles, and regenerated 2,597 vector embeddings — raising search quality validation from 20% to 100% on core queries.

### Hybrid Semantic Search
BM25 keyword + HNSW vector search across 3,536 articles via PostgreSQL 16 + pgvector, with intent-aware adaptive fusion, category boosting, and live monitoring dashboard.

| Before (keyword search) | Now (ML-powered semantic search) |
|---|---|
| "USB policy" returns 50 docs | "Can I use a flash drive?" returns the right policy |
| "password" returns noise | "How do I reset it?" returns step-by-step guide |
| "VPN" returns networking docs | "Can I work from home?" returns remote work policy |

### Confidence-Gated Answers + Source Grounding
Generation now returns a confidence assessment (`answer` / `clarify` / `abstain`) and a per-claim grounding map that links claims to cited sources. This adds a trust layer before copy/paste and helps reduce unsupported responses.

### Operations Workspace (Ops Tab)
A new Ops workspace consolidates deployment safety checks and operational tooling:
- Deployment preflight checks and rollback marking
- Signed artifact verification workflow
- Eval harness run execution + history
- Ticket triage clustering + history
- Runbook session tracking and progression

### KB Gap Detector
Low-confidence/unsupported generation events are logged and aggregated into ranked KB gap candidates in Analytics, with status actions (`accepted`, `resolved`, `ignored`) to close the quality loop.

### Integration and Control Foundations
Support for ServiceNow/Slack/Teams configuration records and workspace role mappings has been added to support enterprise rollout patterns.

---

## How It Works

```
User Question: "Can I work from home?"
        │
        ▼
┌─────────────────────────┐
│  ML Intent Classifier   │
│  TF-IDF + LogReg        │
│  → POLICY (86%)         │
└────────┬────────────────┘
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
┌─────────────────────────┐
│  Adaptive Score Fusion  │
│  RRF (k=60) + category  │
│  boost (1.2x policy)    │
└────────┬────────────────┘
         │
         ▼
┌─────────────────────────┐
│  Cross-Encoder Reranker │
│  ms-marco-MiniLM-L-6   │
│  Blend: 15% CE + 85% F │
└────────┬────────────────┘
         │
         ▼
┌─────────────────────────┐
│  Feedback-Adjusted      │
│  Quality Scores         │
│  Per-article (0.5-1.5x) │
└────────┬────────────────┘
         │
         ▼
┌─────────────────────────┐
│  Ranked Results         │
│  1. Remote Work Policy  │
│  2. WFH Procedure       │
│  3. VPN Setup Guide     │
└─────────────────────────┘
```

---

## Quick Start

Key operator runbooks now live in `docs/runbooks/` for safe mode recovery, vector rebuilds, search-api startup, and dependency advisory handling.

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
pnpm run check:monorepo-readiness
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
cd search-api
python3 -m venv venv
source venv/bin/activate
pip install -r requirements.txt
python3 search_api.py
# API runs on http://localhost:3000
```

Production-safe local startup (enforces API key + non-memory rate limit storage):

```bash
# Start local Redis for limiter storage
redis-server --daemonize yes

# Create production env from template (one-time)
cd search-api
cp .env.example .env.production

# Edit .env.production:
# - set ENVIRONMENT=production
# - set ASSISTSUPPORT_API_KEY to a strong random value
# - (optional) set ASSISTSUPPORT_SEARCH_API_REQUIRE_AUTH=1 (default)
# - set ASSISTSUPPORT_RATE_LIMIT_STORAGE_URI=redis://127.0.0.1:6379/0

# Launch with production env
set -a
source .env.production
set +a
python3 -m venv venv
source venv/bin/activate
pip install -r requirements.txt
python3 validate_runtime.py --check-backends
python3 search_api.py
```

Search API auth is enabled by default in all environments. AssistSupport desktop requests attach a bearer token from secure storage (`search_api_bearer_token`) or `ASSISTSUPPORT_SEARCH_API_KEY` / `ASSISTSUPPORT_API_KEY`.

For a fast production smoke check without launching a long-running server:

```bash
ENVIRONMENT=production ASSISTSUPPORT_API_KEY=test-key ASSISTSUPPORT_RATE_LIMIT_STORAGE_URI=redis://127.0.0.1:6379/0 python3 smoke_search_api.py
```

### First Run

1. **Key Storage** — New workspaces use the keychain-backed default path; existing passphrase-protected workspaces unlock through the dedicated recovery/startup flow
2. **Model Selection** — Pick an LLM model (Llama 3.2 3B recommended)
3. **Knowledge Base** — Point to your team's documentation folder
4. **Generate** — Type a ticket summary, search your KB, get a draft response
5. **Hybrid Search** — Click Search tab (Cmd+8) for ML-powered semantic search

---

## Features

### ML-Powered Hybrid Search (Cmd+8)
- **ML intent classifier** — TF-IDF + Logistic Regression trained on 182 examples (85.7% accuracy), classifies POLICY / PROCEDURE / REFERENCE / UNKNOWN
- **Cross-encoder reranker** — ms-marco-MiniLM-L-6-v2 rescores candidates with blended scoring (15% CE + 85% fusion)
- **BM25 + HNSW vector search** across 3,536 knowledge base articles via PostgreSQL 16 + pgvector
- **Adaptive score fusion** — RRF (k=60) combining BM25 keyword and 384-dim vector scores based on detected intent
- **Category boosting** — 1.2x boost for results matching detected query intent
- **Feedback loop** — per-article quality scores (0.5-1.5x) computed from user ratings, activates at 3+ entries
- **Content quality pipeline** — title cleaning, article consolidation, embedding regeneration
- **Score deduplication** — 0.85 similarity threshold to remove near-duplicates
- **Monitoring dashboard** — live metrics: query volume, p50/p95/p99 latency, accuracy, intent distribution

### Response Generation
- Generate professional IT support responses with local LLM inference (llama.cpp)
- Responses automatically cite relevant KB articles
- Confidence-gated output modes: answer / clarify / abstain
- Claim-level source grounding panel with support level indicators
- Generate multiple alternatives for side-by-side comparison
- Rate responses (1-5 stars) to track quality over time
- Save top-rated responses as reusable templates
- Conversation-style input with context threading
- Two-section format: OUTPUT (copy-paste ready) + IT SUPPORT INSTRUCTIONS (engineer guidance)

### Knowledge Base
- Index markdown, PDF, DOCX, XLSX, code files, and images
- Hybrid search: FTS5 full-text + LanceDB vector/semantic search
- Policy-first search ranking for permission/restriction queries
- Disk ingestion pipeline with source/run tracking and incremental re-indexing (SHA-256 hash comparison)
- OCR support via macOS Vision framework (screenshots, scanned PDFs)
- Web page, YouTube transcript, and GitHub repo ingestion
- Namespace organization for multi-team KB separation
- KB health monitoring with staleness indicators
- Content quality pipeline: title cleaning (2,912 cleaned), article expansion (672 chunks merged), embedding regeneration (2,597 vectors)

### Jira Integration
- Fetch ticket context (title, description, assignee, status)
- Post responses directly to Jira tickets
- Transition tickets to new status after responding
- Template variables (`{{ticket_id}}`, `{{reporter}}`, `{{company_name}}`)

### Analytics & Monitoring
- Response quality tracking (ratings, trends)
- KB usage metrics (search frequency, top queries, article citations)
- KB Gap Detector panel with actionable candidate queue
- Pilot feedback system with CSV export
- Search monitoring dashboard (latency percentiles, accuracy, intent distribution)

### Operations Workspace
- Deployment preflight checks with persisted run history
- Artifact metadata tracking and signed-pack verification
- Rollback workflow markers with audit-ready reason capture
- Eval harness with suite runs and historical comparison
- Ticket triage autopilot clustering + persisted cluster history
- Runbook mode sessions with step progression tracking
- Integration control panel for ServiceNow, Slack, and Teams

### Security & Privacy
- **Fully local** — all processing on your machine, zero cloud dependencies, no telemetry
- **Encrypted core database** via SQLCipher with 0600 file permissions
- **Encrypted stored tokens** for local credentials (Jira, HuggingFace, GitHub)
- **Keychain-backed local key storage** for the primary setup path, with dedicated passphrase unlock support for existing protected workspaces
- **Model integrity** — SHA-256 verification with built-in allowlist; custom models flagged as unverified
- **SSRF protection** — private IP blocking, IPv6-mapped IPv4 detection, cloud metadata endpoint blocking, DNS pinning
- **Path traversal protection** — home directory restriction, symlink skipping, sensitive directory blocking (.ssh, .gnupg, Keychains)
- **Secure memory** — zeroize crate for key material with ZeroizeOnDrop trait, redacted debug output
- **Audit logging** — JSON-line format, thread-safe, 5 MB rotation, covers key generation/rotation, token ops, HTTP opt-in, path failures
- **Filter injection prevention** — Unicode NFC normalization, SQL keyword detection
- **Content Security Policy** — strict CSP headers for XSS prevention
- **Vector-search note** — optional local embeddings improve semantic search, but that vector store is not currently encrypted at rest when enabled
- **Compliance posture** — the project is security-minded and local-first, but it does not currently claim formal HIPAA, GDPR, FISMA, SOC2, ISO 27001, or PCI DSS validation

### Productivity
- Command palette (Cmd+K) and full keyboard-first workflow (30+ shortcuts)
- Session tokens — 24h auto-unlock, no password friction on every launch
- Fast startup — background model loading with cached state (2-3 seconds)
- Batch processing for similar tickets
- Draft management with autosave and version history
- CLI tool for search and indexing outside the GUI

---

## Why Not Just Use ChatGPT?

| Feature | AssistSupport | ChatGPT / Claude API | Zendesk / Freshdesk |
|---------|---------------|----------------------|---------------------|
| **Works Offline** | Yes | No | No |
| **Data Stays Local** | Yes — on your machine | Sent to cloud | Sent to cloud |
| **Searches Your KB** | Yes — ML-powered | No — manual prompt | Partial |
| **Intent Detection** | ML classifier (85.7%) | No | No |
| **Reranking** | Cross-encoder | No | No |
| **Self-Improving** | Feedback loop | No | No |
| **Formal compliance validation** | Not currently claimed | No | Depends on plan |
| **IT-Specific** | Yes — built for support | Generic | Generic AI add-on |
| **Encryption** | AES-256 + Argon2id | Provider-managed | Provider-managed |
| **Cost** | Free (MIT) | $0.001-0.003/token | $50-500+/month |

---

## Performance

### Search Latency

| Metric | Target | Actual | Status |
|--------|--------|--------|--------|
| p50 latency | <50ms | 8ms | 6x faster than target |
| p95 latency | <100ms | 82ms | Meets target |
| Avg latency | <50ms | 21ms | 2.4x faster than target |
| Embedding coverage | 100% | 3,536/3,536 | Complete |
| ML intent accuracy | >80% | 85.7% | Exceeds target |
| Search quality | >90% | 92-100% | Production ready |

### Encryption Throughput

| Operation | 1 KB | 64 KB | 1 MB |
|-----------|------|-------|------|
| Encrypt | ~15 us | ~200 us | ~2.5 ms |
| Decrypt | ~12 us | ~180 us | ~2.2 ms |
| **Throughput** | — | — | **~400 MB/s** |

### Key Derivation (Argon2id — intentionally slow)

| Operation | Latency |
|-----------|---------|
| Key wrap | ~500 ms |
| Key unwrap | ~500 ms |

### Database Operations

| Operation | Latency |
|-----------|---------|
| Open + Initialize | ~50 ms |
| Integrity Check | ~1 ms |
| Read Setting | ~0.1 ms |
| Write Setting | ~0.5 ms |

### FTS Search Scaling

| Query Type | 100 docs | 1,000 docs | 10,000 docs |
|-----------|----------|-----------|-----------|
| Simple | ~1 ms | ~5 ms | ~20 ms |
| Multi-word | ~2 ms | ~8 ms | ~30 ms |
| Phrase | ~2 ms | ~10 ms | ~40 ms |

---

## Architecture

### Tech Stack

| Layer | Technology |
|-------|-----------|
| Frontend | React 19 + TypeScript (strict) + Vite |
| Backend | Rust + Tauri 2.x |
| Database | SQLite + SQLCipher (AES-256) + FTS5 |
| Search Backend | PostgreSQL 16 + pgvector 0.8 (BM25 + HNSW) |
| ML Pipeline | scikit-learn (TF-IDF + LogReg), sentence-transformers (cross-encoder) |
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
│  Local LLM Pipeline  │   │  ML Search Pipeline              │
│                      │   │  (Flask on localhost:3000)        │
│  SQLite (encrypted)  │   │                                   │
│  ├─ FTS5 Index       │   │  ML Intent Classifier (TF-IDF)   │
│  ├─ LanceDB Vectors  │   │  ├─ POLICY / PROCEDURE / REF     │
│  └─ Response History │   │  BM25 + HNSW Vector Search       │
│                      │   │  ├─ Adaptive Score Fusion (RRF)   │
│  llama.cpp (GGUF)    │   │  Cross-Encoder Reranker           │
│  └─ Draft generation │   │  ├─ ms-marco-MiniLM-L-6-v2       │
└──────────────────────┘   │  Feedback Loop                    │
                           │  ├─ Per-article quality scores    │
                           │                                   │
                           │  PostgreSQL 16 + pgvector         │
                           │  ├─ 3,536 articles               │
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
│   ├── Ops/                # Deployment, eval, triage, runbook, integrations
│   ├── Pilot/              # Pilot feedback: query tester, dashboard
│   ├── Search/             # Hybrid PostgreSQL search UI, feedback, stats
│   ├── Settings/           # Model, KB, Jira configuration
│   ├── Sources/            # KB browser, ingestion, health
│   └── shared/             # Onboarding, status indicators
├── contexts/               # AppStatusContext (centralized state)
├── hooks/                  # useLlm, useKb, useHybridSearch, useFeatureOps, useInitialize
└── styles/                 # CSS design tokens, themes

src-tauri/src/              # Rust backend
├── commands/               # Tauri command handlers (190+ endpoints)
│   └── search_api.rs       # PostgreSQL hybrid search proxy (4 commands)
├── db/                     # SQLCipher database layer (schema v12)
├── feedback/               # Pilot feedback logger, stats, CSV export
├── kb/                     # Knowledge base (indexer, search, embeddings, vectors, ingest)
├── llm.rs                  # LLM engine (llama.cpp)
├── jira.rs                 # Jira API integration
├── security.rs             # Encryption, key management
├── audit.rs                # Security audit logging
└── diagnostics.rs          # Health checks, maintenance

search-api/                 # ML search pipeline (Python)
├── search_api.py           # Flask REST API (5 endpoints)
├── hybrid_search.py        # Orchestrates ML pipeline
├── intent_detection.py     # ML intent classifier (TF-IDF + LogReg)
├── reranker.py             # Cross-encoder reranker
├── score_fusion.py         # Adaptive score fusion strategies
├── feedback_loop.py        # Per-article quality scoring
├── train_intent_classifier.py  # Model training pipeline
├── clean_titles.py         # Title cleaning (2,912 titles)
├── expand_articles.py      # Article consolidation
└── rebuild_indexes.py      # Embedding & index regeneration
```

Architecture and governance deep-dive docs were removed in this trimmed repository profile.

---

## For IT Support Teams

### Individual Setup
Each engineer clones, installs, and runs. Point the KB to a local docs folder or shared drive.

### Team Shared KB (Recommended)
Set up a shared documentation folder and have each engineer point AssistSupport at it:

```
IT_KnowledgeBase/
├── POLICIES/         # USB, remote work, software installation
├── PROCEDURES/       # Password resets, onboarding, VPN setup
└── REFERENCE/        # Architecture docs, contact lists, FAQs
```

Team-specific IT support deployment guides were removed in this trimmed repository profile.

---

## Testing

```bash
# Workstation and repo readiness checks
pnpm run check:workstation-preflight
pnpm run check:workflow-drift
pnpm run check:monorepo-readiness

# Frontend unit tests + coverage
pnpm test
pnpm test:coverage

# Frontend visual/a11y regression tests
pnpm test:e2e:smoke
pnpm ui:gate:regression

# Rust backend tests
pnpm test:ci
pnpm test:security-regression

# Security audit
pnpm git:guard:all
pnpm test:security:audit:rust
```

Security-focused backend tests cover encryption, key management, path traversal, SSRF, filter injection, namespace consistency, and data migration.

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
| `Cmd+1-9` | Switch primary tabs |
| `Cmd+8` | Open Hybrid Search |
| Sidebar: `Ops` | Open deployment/eval/triage/runbook workspace |

---

## Documentation

| Document | Description |
|----------|-------------|
| `README.md` | Consolidated setup, architecture summary, and runtime behavior |
| `SECURITY.md` | Security scope and disclosure entry point |
| `docs/SECURITY.md` | Security architecture and verification controls |
| [Changelog](CHANGELOG.md) | Release history |

---

## Changelog

### v1.0.0 (Current) — Production Release
- **ML intent classifier** — TF-IDF + Logistic Regression (85.7% accuracy, 182 training examples)
- **Cross-encoder reranker** — ms-marco-MiniLM-L-6-v2 with blended scoring (15% CE + 85% fusion)
- **Feedback loop** — per-article quality scores (0.5-1.5x) from user ratings
- **Content quality pipeline** — title cleaning (2,912), article expansion (672 merged), embedding regeneration (2,597)
- **Diagnostic analysis** — root cause identification, KB audit, 293 junk articles deactivated
- Hybrid PostgreSQL search (BM25 + HNSW vector, 3,536 articles)
- Intent detection (POLICY/PROCEDURE/REFERENCE classification)
- Adaptive score fusion with category boosting
- Search tab (Cmd+8) with result cards, score breakdowns, API health indicator
- User feedback collection (helpful/not_helpful/incorrect ratings)
- Live monitoring dashboard (query volume, latency percentiles, accuracy, intent distribution)
- Flask REST API (5 endpoints on localhost:3000 with rate limiting)
- 4 Tauri commands proxying to Flask API via reqwest
- p50: 8ms, p95: 82ms, avg: 21ms — search quality 92-100%

### Unreleased (main branch)
- New **Ops** workspace tab for deployment checks, rollback marking, eval harness, triage clusters, runbook sessions, and integrations controls
- Confidence-gated response modes (`answer` / `clarify` / `abstain`) exposed in generation results
- Claim-level source grounding map surfaced in the response UI
- KB Gap Detector analytics pipeline and action queue (accept/resolve/ignore)
- Deployment artifact tracking, signed artifact verification, and rollback command path
- New Playwright coverage for Ops workflows (`e2e/ops.spec.ts`)

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

Roadmap and execution planning artifacts were removed in this trimmed repository profile.

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
- Ensure Flask API is running: `cd search-api && python3 search_api.py`
- Check API health: `curl http://localhost:3000/health`

**Database encryption error on first launch**
- The app creates its database at `~/Library/Application Support/AssistSupport/`
- If migrating from a previous version, check the migration log in the app

---

## Contributing

Contributions welcome. This repository now focuses on runtime artifacts only.

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
cd src-tauri && cargo test && cargo clippy

# Push and create PR
git push origin feature/your-feature
```

---

## Security

See `SECURITY.md` for security reporting guidance.

To report a vulnerability, please open a security advisory on GitHub.

---

## License

[MIT](LICENSE)

---

Built with [React](https://react.dev), [Tauri](https://tauri.app), [Rust](https://www.rust-lang.org), [llama.cpp](https://github.com/ggerganov/llama.cpp), [SQLite](https://sqlite.org), [LanceDB](https://lancedb.com), [PostgreSQL](https://www.postgresql.org), [pgvector](https://github.com/pgvector/pgvector), [scikit-learn](https://scikit-learn.org), [sentence-transformers](https://www.sbert.net), and [Flask](https://flask.palletsprojects.com).
