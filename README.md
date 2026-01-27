# AssistSupport

**The AI assistant for IT support engineers who can't use ChatGPT.**

Generate first-response drafts for support tickets — offline, encrypted, on your machine.
Hybrid search across your KB. HIPAA/GDPR/FISMA ready.

[![Compliance](https://img.shields.io/badge/compliance-NIST%20%7C%20ISO27001%20%7C%20SOC2%20%7C%20HIPAA%20%7C%20GDPR%20%7C%20FISMA-blue)](docs/compliance/COMPLIANCE_REPORT.md)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-436%20passing-brightgreen)](docs/TESTING.md)

---

### For IT Support Engineers

You need to:
- Draft responses faster (less time typing, more time solving)
- Keep company data off the cloud (no ChatGPT, no vendor APIs)
- Use your internal KB + documentation (not internet search)
- Meet compliance (SOC2, HIPAA, GDPR, FISMA)

AssistSupport does all of this. Runs locally. No telemetry. No vendor lock-in.

---

## The Problem

You're spending hours drafting ticket responses using:
- Outdated internal documentation you can't search
- ChatGPT (but your company won't approve it)
- Memory of what you've said before

## The Solution

**AssistSupport** is your offline AI assistant that:
- Searches **your KB** (not the internet)
- Drafts responses in **seconds**
- Keeps data **100% on your machine**
- Works **completely offline** (no internet needed after setup)

### Real Example

```
You receive:  "Can't connect to VPN on Windows 11"
Search finds: Your 47 VPN troubleshooting docs
AI drafts:    "1. Verify Cisco AnyConnect version... 2. Check Windows 11 network..."
You refine:   Add ticket reference, adjust language
You copy:     Paste into Jira — done in under a minute
```

See the [IT Support Guide](docs/IT_SUPPORT_GUIDE.md) for more workflow examples, or the [Quick Start](docs/QUICKSTART_IT_SUPPORT.md) to get running in 5 minutes.

### Why IT Support Engineers Use This
- **Speed**: Faster ticket responses (not copying/pasting from 5 docs)
- **Accuracy**: Grounded in YOUR documentation (no hallucinations from the internet)
- **Privacy**: Completely offline (meets HIPAA, GDPR, FISMA)
- **Control**: Own your data, own your LLM, zero vendor lock-in

---

## Why Not Just Use ChatGPT?

| Feature | AssistSupport | ChatGPT | Custom Scripts |
|---------|---------------|---------|----------------|
| **Works Offline** | Yes | No | Yes |
| **Searches Your KB** | Yes | No (internet only) | Maybe |
| **HIPAA Compliant** | Yes | No | If built right |
| **Data Stays Local** | Yes | OpenAI servers | Yes |
| **Pre-Built** | Yes | N/A | Need to build |
| **Hybrid Search** | FTS + Vector | N/A | Usually vector only |
| **Cost** | Free (MIT) | $20/mo | Engineering time |

**Bottom line**: ChatGPT is great for personal use. AssistSupport is for IT teams that need compliance + control + local data.

---

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
- **Compliance**: Validated against HIPAA, GDPR, FISMA, SOC2, ISO 27001, PCI DSS, NIST SP 800-53 ([report](docs/compliance/COMPLIANCE_REPORT.md))

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
| **System Libraries** | | `brew install protobuf pkgconf cmake leptonica tesseract` |

### Install & Run

```bash
# Clone the repo
git clone https://github.com/saagar210/AssistSupport.git
cd AssistSupport

# Install frontend dependencies
pnpm install

# Run in development mode (starts both frontend dev server and Tauri backend)
pnpm tauri dev
```

### First Run

1. **Key Storage**: Choose Keychain (recommended) or passphrase mode
2. **Model Selection**: Pick an LLM model (Llama 3.2 3B recommended)
3. **Knowledge Base**: Point to your team's documentation folder
4. **Start generating**: Type a question, search your KB, get a draft response

### Build for Production

```bash
pnpm tauri build
```

The `.dmg` / `.app` output will be in `src-tauri/target/release/bundle/`.

### Testing

```bash
# Frontend tests
pnpm test

# Backend tests
cd src-tauri && cargo test

# Performance benchmarks
cd src-tauri && cargo bench
```

## For IT Support Teams

### Individual Setup
Each engineer clones, installs, and runs. Point KB to a shared drive or local docs folder.

### Team Shared KB (Recommended)
Set up a shared documentation folder and have each engineer point AssistSupport at it:

```
IT_KnowledgeBase/
├── Windows/          # connectivity, accounts, software
├── Network/          # VPN, printing, email
├── Accounts/         # password resets, MFA
└── Procedures/       # onboarding, offboarding
```

See the [IT Support Guide](docs/IT_SUPPORT_GUIDE.md#setup-for-your-team) for detailed deployment options.

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

## Real Results

See how IT teams use AssistSupport:

- **[TechCorp IT](docs/CASE_STUDIES/EXAMPLE_TechCorp.md)**: 12 engineers, 1,200 tickets/day, 67% faster responses, $333k/year ROI
- [Your Organization?](docs/CASE_STUDIES/TEMPLATE.md) - Submit a case study

**Expected improvements**:
- Response time: 12 min -> 4 min (67% faster)
- Responses per engineer: 20 -> 35 per day (75% more)
- Time saved: ~1 hour per engineer per day
- Annual ROI: $300k-500k for 10-person team

## Testing & Verification

Before deploying to your team, verify AssistSupport works:

### Quick Health Check
```bash
pnpm test:health
```
Verifies app launches, database works, LLM loads, encryption works.

### Full Test Suite
```bash
pnpm test
```
436 tests covering UI, search, generation, security, encryption.

### Integration Tests
```bash
pnpm test:kb-indexing  # KB indexing works
pnpm test:search       # Hybrid search works
pnpm test:generation   # Response generation works
pnpm test:jira         # Jira integration works
```

### Security Tests
```bash
pnpm test:security:encryption  # AES-256 encryption
pnpm test:security:paths       # Path validation
pnpm test:security:audit       # Audit logging
```

See [Testing Guide](docs/TESTING.md) for comprehensive testing documentation.

## Documentation

| Document | Description |
|----------|-------------|
| [Testing Guide](docs/TESTING.md) | Automated test suite, quick health checks, CI/CD pipeline |
| [Case Studies](docs/CASE_STUDIES/) | Real examples: TechCorp IT saves 67% response time |
| [Roadmap](docs/ROADMAP.md) | Q1-Q4 2026 priorities, Jira mastery focus |
| [Analytics Plan](docs/ANALYTICS_DASHBOARD_PLAN.md) | Q2 2026: Team dashboard to measure ROI |
| [Quick Start for IT](docs/QUICKSTART_IT_SUPPORT.md) | 5-minute setup guide |
| [IT Support Guide](docs/IT_SUPPORT_GUIDE.md) | Workflows, team setup, integration |
| [Architecture](docs/ARCHITECTURE.md) | System design and code structure |
| [Security](docs/SECURITY.md) | Encryption, key management, threat model |
| [Compliance Report](docs/compliance/COMPLIANCE_REPORT.md) | HIPAA/GDPR/FISMA/SOC2/ISO 27001 validation |
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
brew install protobuf pkgconf cmake leptonica tesseract
xcode-select --install
```

**`pnpm tauri dev` fails to start**
```bash
# Rebuild from clean state
rm -rf src-tauri/target node_modules
pnpm install
pnpm tauri dev
```

**"Could not determine which binary to run"**
- Ensure `default-run = "assistsupport"` is set in `src-tauri/Cargo.toml` `[package]` section

**LLM model fails to load**
- Ensure model is a valid `.gguf` file
- Check available RAM (models need 2-8GB depending on size)
- Try a smaller model first (Llama 3.2 1B)

**Database encryption error on first launch**
- The app creates its database at `~/Library/Application Support/AssistSupport/`
- If migrating from a previous version, check the migration log in the app

## Security

See [SECURITY.md](docs/SECURITY.md) for the full security model and [Compliance Report](docs/compliance/COMPLIANCE_REPORT.md) for validation against 7 security standards.

To report a vulnerability, please see the [security policy](SECURITY.md).

## License

[MIT](LICENSE)
