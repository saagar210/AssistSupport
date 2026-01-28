# AssistSupport

> **Local-first, fully offline AI assistant for IT support engineers**

![Version](https://img.shields.io/badge/version-0.5.2-10a37f)
![Platform](https://img.shields.io/badge/platform-macOS-lightgrey)
![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)
![Tests](https://img.shields.io/badge/tests-passing-brightgreen)
[![Compliance](https://img.shields.io/badge/compliance-HIPAA%20%7C%20GDPR%20%7C%20FISMA%20%7C%20SOC2-blue)](docs/compliance/COMPLIANCE_REPORT.md)

**AssistSupport** generates professional IT support responses using your knowledge base and local AI models — completely offline, completely private, completely encrypted.

```
You receive:  "Can't connect to VPN on Windows 11"
Search finds: Your 47 VPN troubleshooting docs
AI drafts:    "1. Verify Cisco AnyConnect version... 2. Check Windows 11 network..."
You refine:   Adjust tone, add ticket reference
You copy:     Paste into Jira — done in under a minute
```

---

## Why AssistSupport?

**The problem**: IT support teams spend 40% of their time drafting responses. Responses lack consistency. KB articles go unconsulted. Cloud AI tools risk exposing sensitive ticket data. And they require internet.

**The solution**: AssistSupport drafts consistent, KB-informed responses instantly — all on your machine, with zero cloud dependencies.

| | |
|---|---|
| **Instant** | Draft responses in seconds, not minutes |
| **100% Private** | All data stays on your device — no cloud, no telemetry |
| **Works Offline** | No internet required. Works on airplane mode. |
| **KB-Powered** | Automatically searches and cites your knowledge base |
| **IT-Specific** | Built for support workflows — device requests, troubleshooting, access |
| **Enterprise-Ready** | AES-256 encryption, HIPAA/GDPR/FISMA/SOC2 ready |
| **No Per-User Fees** | One-time setup, unlimited use |

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
# Output: src-tauri/target/release/bundle/dmg/
```

### First Run

1. **Key Storage** — Choose Keychain (recommended) or passphrase mode
2. **Model Selection** — Pick an LLM model (Llama 3.2 3B recommended)
3. **Knowledge Base** — Point to your team's documentation folder
4. **Generate** — Type a ticket summary, search your KB, get a draft response

---

## Features

### Response Generation
- Generate professional IT support responses with local LLM inference (llama.cpp)
- Responses automatically cite relevant KB articles
- Generate multiple alternatives for side-by-side comparison
- Rate responses (1-5 stars) to track quality over time
- Save top-rated responses as reusable templates
- Conversation-style input with context threading
- Draft versioning with diff viewer

### Knowledge Base
- Index markdown, PDF, DOCX, XLSX, code files, and images
- Hybrid search: FTS5 full-text + LanceDB vector/semantic search
- OCR support via macOS Vision framework (screenshots, scanned PDFs)
- Web page, YouTube transcript, and GitHub repo ingestion
- Namespace organization for multi-team KB separation
- KB health monitoring with staleness indicators
- Article-level analytics (citation frequency, search hits)

### Jira Integration
- Fetch ticket context (title, description, assignee, status)
- Post responses directly to Jira tickets
- Transition tickets to new status after responding
- Template variables (`{{ticket_id}}`, `{{reporter}}`, `{{company_name}}`)

### Analytics Dashboard
- Response quality tracking (ratings, trends)
- KB usage metrics (search frequency, top queries, article citations)
- Article-level drill-down (which articles get cited most)
- Generation performance metrics

### Productivity
- Session tokens — 24h auto-unlock, no password friction on every launch
- Fast startup — background model loading with cached state (2-3 seconds)
- Batch processing for similar tickets
- Auto-suggest based on ticket content
- Command palette (Cmd+K) and full keyboard-first workflow
- Draft management with autosave
- CLI tool for search and indexing outside the GUI

### Security & Privacy
- **Fully local** — all processing on your machine, zero cloud dependencies
- **AES-256 database encryption** via SQLCipher
- **AES-256-GCM token encryption** for stored credentials (Jira, HuggingFace)
- **macOS Keychain** or Argon2id passphrase-wrapped key storage
- **SSRF protection** with DNS pinning for web ingestion
- **Path traversal protection** with home directory restriction
- **Audit logging** for security events
- **Compliance validated** against [HIPAA, GDPR, FISMA, SOC2, ISO 27001, PCI DSS, NIST SP 800-53](docs/compliance/COMPLIANCE_REPORT.md)

### Design (v0.5.2)
- Two-section response format: OUTPUT (copy-paste ready) + IT SUPPORT INSTRUCTIONS (engineer guidance)
- ChatGPT-inspired dark-first UI with green accent
- Clickable KB suggestion chips — navigate directly to matching sources
- Fully responsive layout at any window size (no clipped buttons or hidden content)
- Smooth animations with `prefers-reduced-motion` support
- Full keyboard navigation and WCAG AA contrast
- System font stack for instant rendering
- Polished sidebar, lift-on-hover buttons, glow effects

---

## Why Not Just Use ChatGPT?

| Feature | AssistSupport | ChatGPT / Claude API | Zendesk / Freshdesk |
|---------|---------------|----------------------|---------------------|
| **Works Offline** | Yes | No | No |
| **Data Stays Local** | Yes — on your machine | Sent to cloud | Sent to cloud |
| **Searches Your KB** | Yes — automatic | No — manual prompt | Partial |
| **HIPAA Compliant** | Yes | No | Depends on plan |
| **IT-Specific** | Yes — built for support | Generic | Generic AI add-on |
| **Cost** | Free (MIT) | $0.001-0.003/token | $50-500+/month |
| **Customizable** | Your KB, your models | No | Limited |
| **Setup Time** | Minutes | N/A | Days |

---

## Architecture

### Tech Stack

| Layer | Technology |
|-------|-----------|
| Frontend | React 19 + TypeScript (strict) + Vite |
| Backend | Rust + Tauri 2.x |
| Database | SQLite + SQLCipher (AES-256) + FTS5 |
| Vector Store | LanceDB |
| LLM Runtime | llama.cpp via llama-cpp-2 (GGUF models) |
| PDF | PDFium (bundled) |
| OCR | macOS Vision framework |

### Data Flow

```
Knowledge Base (markdown, PDF, DOCX, web, YouTube)
    |
    v
SQLite Database (encrypted, AES-256)
    ├── FTS5 Index (full-text search)
    ├── Vector Index (LanceDB, semantic search)
    └── Response History (drafts, ratings, templates)
    |
    v
User Input + KB Context + Jira Context
    |
    v
LLM Inference (llama.cpp, fully local)
    |
    v
Generated Response → Edit → Rate → Copy/Post to Jira
```

### Project Structure

```
src/                        # React frontend
├── components/
│   ├── Analytics/          # Dashboard, article drill-down
│   ├── Batch/              # Batch processing
│   ├── Draft/              # Response drafting, alternatives, ratings
│   ├── Layout/             # Header, sidebar, command palette
│   ├── Settings/           # Model, KB, Jira configuration
│   ├── Sources/            # KB browser, ingestion, health
│   └── shared/             # Onboarding, status indicators
├── contexts/               # AppStatusContext (centralized state)
├── hooks/                  # useLlm, useKb, useInitialize
└── styles/                 # CSS design tokens, themes

src-tauri/src/              # Rust backend
├── commands/               # Tauri command handlers (~200 endpoints)
├── db/                     # SQLCipher database layer
├── kb/                     # Knowledge base
│   ├── indexer.rs          # File indexing and chunking
│   ├── search.rs           # Hybrid FTS + vector search
│   ├── embeddings.rs       # Embedding model
│   ├── vectors.rs          # LanceDB vector store
│   └── ingest/             # Web, YouTube, GitHub ingestion
├── llm.rs                  # LLM engine (llama.cpp)
├── jira.rs                 # Jira API integration
├── security.rs             # Encryption, key management
├── audit.rs                # Security audit logging
└── diagnostics.rs          # Health checks, maintenance

docs/                       # Documentation
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
| `Cmd+1-6` | Switch tabs |

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

## Roadmap

### v0.5.2 (Current)
- [x] Two-section response format: OUTPUT (clean, copy-paste ready) + IT SUPPORT INSTRUCTIONS
- [x] Tab UI for switching between response sections
- [x] Copy button copies only the OUTPUT section
- [x] Engineer gets actionable pre-send checks, post-send actions, and KB references

### v0.5.1
- [x] Interactive KB suggestion chips — click to navigate to Sources tab
- [x] Responsive layout fixes — all content visible at any window size
- [x] Button wrapping and overflow handling across all panels

### v0.5.0
- [x] ChatGPT-inspired UI redesign (dark-first, green accent)
- [x] Fast startup with background model loading (2-3 seconds)
- [x] Session tokens (24h auto-unlock)
- [x] Analytics dashboard with ratings and article drill-down
- [x] Response alternatives and template recycling
- [x] Jira post + transition workflow
- [x] KB health and staleness monitoring
- [x] CLI with real search and indexing

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

**Database encryption error on first launch**
- The app creates its database at `~/Library/Application Support/AssistSupport/`
- If migrating from a previous version, check the migration log in the app

**"Could not determine which binary to run"**
- Ensure `default-run = "assistsupport"` is set in `src-tauri/Cargo.toml`

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

### Areas for Contribution
- **Windows support** — porting and testing
- **Performance** — search optimization, LLM inference tuning
- **Documentation** — guides, examples, case studies
- **Bug fixes** — check GitHub Issues

---

## Security

See [SECURITY.md](docs/SECURITY.md) for the full security model and [Compliance Report](docs/compliance/COMPLIANCE_REPORT.md) for validation against 7 security standards.

To report a vulnerability, please open a security advisory on GitHub.

---

## License

[MIT](LICENSE)

---

## Acknowledgments

Built with [React](https://react.dev), [Tauri](https://tauri.app), [Rust](https://www.rust-lang.org), [llama.cpp](https://github.com/ggerganov/llama.cpp), [SQLite](https://sqlite.org), and [LanceDB](https://lancedb.com).
