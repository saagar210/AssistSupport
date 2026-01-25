# AssistSupport: Self-Contained Local KB + LLM Implementation Plan

## Summary

Transform AssistSupport into a fully self-contained local app with embedded knowledge base and LLM inference. **No external services required for core workflows** (LLM + KB) - no Ollama service, no KAS, no separate databases. **Outbound network is allowed** for model downloads and on-demand web status checks.

**Bootstrap note**: Workspace is empty; initialize a new Tauri + React (Vite + TS) app before following the phases below.

### Development Context
- **Dev machine**: MacBook Pro 16" M4 Pro, 48GB RAM (personal)
- **Target machine**: Same spec (work MacBook)
- **Workflow**: Build framework here → GitHub → deploy on work machine
- **Content**: IT runbooks, past ticket solutions, PDFs (on work machine)
- **Platform scope (v1)**: macOS only

### Work Machine Constraints (All Satisfied)
- No Docker
- Outbound network allowed (model downloads + web status checks)
- Internal distribution via Box/GitHub is acceptable
- **Solution**: Single DMG app + local model files, no external services required for core workflows

### Network Access & Privacy
- **Core offline**: LLM + KB features work without network
- **Online optional**: Model downloads and web status checks only when explicitly triggered
- **Web search default**: Disabled until user enables in Settings
- **PII minimization**: Redact names, email addresses, ticket IDs before outbound web queries
- **Policy controls**: Allowlist domains, proxy support, per-source toggles

---

## Key Decisions

| Component | Choice | Rationale |
|-----------|--------|-----------|
| **LLM** | `llama-cpp-2` crate (embedded) | No external service required, Metal acceleration |
| **KB Metadata** | SQLCipher database | Docs, chunks text, FTS5 index |
| **Text Search** | FTS5 (SQLCipher build with FTS5 enabled) | Fast, proven |
| **Vector Search** | LanceDB (Rust-native) | Better performance, scales to 100K+ vectors |
| **Hybrid Search** | RRF fusion | Combines FTS5 + LanceDB results |
| **Embeddings** | Separate model (nomic-embed-text) | Fast, 768 dimensions, parallel with gen model |
| **Threading** | Parallel (both models loaded) | ~14GB RAM, faster operations |
| **App Sandbox (v1)** | Disabled | Simplifies local file access; revisit if signing/notarization is added |

### Storage Architecture
```
~/Library/Application Support/AssistSupport/
├── assistsupport.db          # SQLCipher: drafts, follow-ups, KB metadata, FTS5
├── attachments/              # Encrypted screenshots
├── models/                   # GGUF models (generation + embeddings)
├── downloads/                # Temporary download/resume state
└── vectors/                  # LanceDB: chunk embeddings (encrypted if supported)
    └── chunks.lance/
~/Library/Caches/AssistSupport/   # Temporary caches
```

### Encryption Strategy
- **SQLCipher**: Random 32-byte master key generated on first run
- **Key storage**: Store master key in macOS Keychain by default; fallback to user passphrase if Keychain unavailable
- **Passphrase**: Derive KEK with Argon2id (store salt + params) to wrap master key
- **LanceDB**: Use master key if crate supports AES-256-GCM; if unsupported, require explicit user consent before enabling unencrypted vector search
- **Default**: Vector search disabled until encryption support confirmed or explicit opt-in recorded
- **Attachments**: Encrypt at rest with master key (AES-256-GCM), store metadata in DB

### Security/DB Dependencies
- **SQLCipher integration**: Use `rusqlite` + `libsqlite3-sys` built against SQLCipher (vendored source) with compile flags `SQLITE_HAS_CODEC` and `SQLITE_ENABLE_FTS5`
- **Crypto**: `aes-gcm`, `argon2`, `rand`, `zeroize`
- **Keychain**: `keyring` crate (macOS Keychain backend)

**Why NOT fork KAS:**
- KAS requires PostgreSQL + pgvector (external service)
- LanceDB + FTS5 achieves same hybrid search locally
- Simpler architecture, fully embedded, no external processes

---

## Detailed Specifications

### LLM Integration
- **Loading**: On app launch, auto-load last used model
- **Persistence**: Remember model path across restarts
- **Error handling**: Auto-retry with backoff (3 attempts, exponential)
- **Threading**: Generation model always loaded in memory
- **Scheduling**: Prioritize generation over embedding; throttle background indexing to avoid GPU contention
- **GPU Memory**: Metal layers set to max (1000+), auto-adjust if OOM
- **Preflight**: Estimate VRAM use from model metadata; clamp `n_gpu_layers` before load
- **Model validation**: SHA256 checksum verification on first load
- **Model switching**:
  - Unload current model completely (free GPU/RAM)
  - Show progress: "Unloading model... Loading new model..."
  - Estimated time based on model size
  - Cancel button during load
  - Rollback to previous model if load fails

### Model Management & Downloads
- **Storage**: All models stored under `~/Library/Application Support/AssistSupport/models/`
- **Download manager**: Resume support, checksum verification, progress + ETA
- **Sources**: HuggingFace direct URLs; allow custom URL entry
- **Auth**: Optional HuggingFace token for gated models (store in Keychain)
- **Manual import**: Drag/drop or file picker for offline installs

### Database & Migration
- **Schema version**: Stored in `settings` table (`schema_version` key)
- **Migrations**: Run sequentially on app start if version < current
- **Backup**: Auto-backup before migration (`assistsupport.db.bak`)
- **Corruption detection**: PRAGMA integrity_check on launch
- **LanceDB versioning**: Separate version tracking for vector schema

### Knowledge Base Indexing
- **File watching**: Auto-watch KB folder, reindex on changes
- **Debounce/ignore rules**: Coalesce bursts and ignore app-generated temp files
- **Chunking**: Heading-based (H1/H2), target 200-500 words, hard cap 500
- **Folder depth**: Full recursive (all subfolders)
- **File types**: Markdown (.md), PDF (.pdf), Plain text (.txt)
- **Large folders**: Paginated indexing (batches of 50-100 files)
- **Model change**: Re-embed all chunks when switching embedding model
- **Dim changes**: If embedding size changes, recreate vector table

**PDF Handling** (Text + OCR Fallback):
- First pass: Extract embedded text using PDFium (`pdfium-render`)
- Per-page text threshold: If page has <50 characters of text
  - Run OCR provider on that page
  - Combine OCR text with extracted text
- Annotated screenshots in PDFs → captured via OCR
- Progress indicator: "Processing page X of Y..."
- **Bundling**: Ship PDFium with the app (no system library dependency)
- **Mixed PDFs**: Track OCR quality (char count/confidence); if low and Tesseract is available, run a fallback pass
- **PDFium load**: Bundle `libpdfium.dylib` in Tauri resources and bind via `Pdfium::bind_to_library` at runtime

**OCR Strategy (Pluggable)**
- **macOS default**: Vision OCR provider
- **Cross-platform fallback**: Tesseract provider (optional, if bundled/available)
- **v1 scope**: macOS only; ship Vision OCR (no Tesseract by default)
- **Tesseract language data**: English only (if bundled later)
- **No OCR available**: Index only extracted text and flag file as partial
- **Decision gate**: After OCR benchmark on mixed PDFs, decide whether to ship an optional Tesseract variant
- **Vision helper**: Build a small Swift helper binary (bundled in Tauri resources) that accepts image paths and returns text via stdout
- **Feature flag**: Guard Tesseract provider behind a Cargo feature (e.g., `tesseract`) for a separate DMG build

### Search Behavior
- **Snippet count**: Top 5 snippets per query
- **Injection**: Auto-inject into prompt, show in Sources tab after
- **No results**: Automatically lower threshold / broaden search
- **Source tracking**: Persist which KB chunks were used per draft
- **Prompt safety**: Treat KB as untrusted; add system guardrail to ignore instructions inside sources
- **Vector disabled**: If vector search is disabled, run FTS-only search
- **Embedding missing**: If no embedding model configured, run FTS-only search

### UI/UX

**Layout:**
- **Tabs**: Draft | Follow-ups | Sources | Settings (4 tabs)
- **Responsive layouts**: User toggle between three-panel, two-panel, stacked
- **Default size**: Medium (half screen), resizable, remembers position
- **Color scheme**: User choice (light/dark toggle in Settings)

**Draft Tab Layout (Three-Panel Default):**
```
┌──────────┬──────────────┬──────────────┐
│  INPUT   │  DIAGNOSIS   │   RESPONSE   │
└──────────┴──────────────┴──────────────┘
```

**Panel Controls:**
- **Draggable dividers**: Drag borders between panels to resize
  - Minimum width: 200px per panel
  - Positions remembered across sessions
- **Collapsible panels**: Each panel has collapse button (▶/◀)
  - Collapsed panel shows as thin strip with title
  - Click strip or button to expand
  - Useful for simple tickets: collapse Diagnosis, expand Input+Response
- **Responsive (narrow window)**:
  - Below 900px width → auto-stack vertically
  - Order: Input → Diagnosis → Response (scrollable)
  - Each section collapsible in stacked mode too

**Interactions:**
- **Copy**: One-click + "Copied!" toast notification
- **Screenshot paste**: Auto-OCR immediately
- **Related tickets**: Auto-detect and group similar issues
- **Ticket switch**: Auto-save current progress

**Long Input Handling** (Summarization):
- **Context budget**: Reserve ~20K chars for input after system/KB context
- **Token-aware**: Use model tokenizer to estimate token budget; char count is a fallback heuristic
- **When input exceeds budget**:
  1. Show warning: "Input is long, summarizing for context..."
  2. Run quick summarization pass on entire input
  3. LLM prompt: "Extract the key IT issue, symptoms, error messages, and user actions from this support ticket"
  4. Use summarized version (~2-3K chars) for generation
  5. Store original + summary in draft record
- **Reviewability**: Show summary preview with editable text and a toggle to use original input
- **Why summarize** (not truncate first/last):
  - First/last paragraphs often greetings, apologies, pleasantries
  - Important issue details buried in middle
  - Summarization preserves critical information

**Ticket Number Auto-Detection:**
- **Patterns recognized**:
  - Jira: `PROJ-1234`, `IT-567`, `HELP-89`
  - ServiceNow: `INC0012345`, `REQ0012345`
  - Zendesk: `#12345`
  - Generic: `Ticket #1234`, `Case 1234`
- **When detected**:
  - Show badge with ticket ID in Input panel header
  - Click badge → copy ticket ID
  - Auto-link to Jira/ServiceNow (configurable base URL in Settings)
  - Store ticket ID with draft for searching later

**Keyboard Shortcuts:**
| Shortcut | Action |
|----------|--------|
| `Cmd+G` | Generate response |
| `Cmd+Shift+C` | Copy output to clipboard |
| `Cmd+V` | Paste into input (with auto-OCR for images) |
| `Cmd+1/2/3/4` | Switch to tab 1/2/3/4 |
| `Cmd+N` | New draft (clear input) |
| `Cmd+F` | Focus search |
| `Cmd+,` | Open Settings |
| `Cmd+D` | Toggle diagnosis panel |
| `Esc` | Close modal/clear selection |

**Sources Tab (KB Dashboard):**
- **Summary stats bar**:
  - Total docs | Total chunks | Last indexed | Index health (✓ or ⚠)
- **Per-file breakdown table**:
  - File path | Chunks | Last modified | Index status
  - Sortable by any column
  - Filter by: indexed/pending/failed
- **Search analytics**:
  - Recent searches with hit counts
  - Most-referenced docs (top 10)
  - Unused docs (indexed but never matched)
- **Coverage analysis**:
  - Docs without embeddings (FTS5-only)
  - Failed files with error messages
  - Suggested: "These 5 docs may need re-indexing"
- **Used sources per draft**:
  - Expandable section showing which KB chunks were injected
  - Links to view original doc

**Toast Notifications:**
- **Auto-dismiss**: Success toasts disappear after 3 seconds
- **Persistent errors**: Error toasts stay until dismissed (X button)
- **Position**: Top-right corner, stack vertically if multiple
- **Types**: Success (green), Error (red), Info (blue), Warning (yellow)

**Loading States** (Skeleton + Spinner):
- **Panel skeletons**: Gray pulsing rectangles matching content layout
- **Action spinners**: Small spinner with action text
  - "Generating response..." (with word count as it streams)
  - "Searching KB..." (with chunk count as found)
  - "Processing PDF..." (with page number)
- **Estimated time**: Show for operations >3s expected
  - "Indexing 47 files (~2 min remaining)"
- **Cancelable**: Long operations show Cancel button

**Settings Tab Organization** (Accordion Sections):
1. **Models** - Generation model path, Embedding model path, download manager, HuggingFace token, benchmark button
2. **Knowledge Base** - KB folder path, reindex button, import files button
3. **Appearance** - Theme (light/dark/system), panel layout default
4. **Integrations** - Jira base URL, ServiceNow URL, web search enable toggle (default off), sources, allowlist + proxy
5. **Advanced** - Safe mode, diagnostics mode, export/import, wipe data, unencrypted vector search opt-in

**Draft Management** (Single + History):
- **One active draft** at a time
- **Quick switch dropdown**: Top of Draft tab shows recent 10 drafts
  - Format: "Timestamp - First 30 chars of input..."
  - Click to switch (auto-saves current)
- **New draft button**: Clears current, starts fresh
- **History search**: Full-text search of all past drafts (modal)

**Other:**
- **Indexing progress**: Progress bar with percentage + cancel button
- **Draft output**: Copy button + editable text area
- **Follow-ups**: Integrated KB search

### Word Limits (Configurable)
- **Toggle**: Short / Medium (sets default)
- **Fine-tune**: Dropdown with presets + "Custom..." option
- **Defaults**: Slack Short=80, Medium=160 | Jira Short=120, Medium=220
- **Custom range**: 50-500 words

### Diagnostic Assistant (NEW - Core Feature)

**Workflow**: Hybrid parallel mode
- Diagnosis panel + Response draft shown side-by-side
- Expand diagnosis for complex issues, collapse for simple ones
- Auto-update diagnosis as context is added (screenshots, OCR)

**Diagnostic Features:**
1. **What to Check First** (LLM-Generated)
   - LLM analyzes ticket text + KB context + OCR evidence
   - Generates 3-7 prioritized troubleshooting steps
   - Each step: checkbox + description + optional KB link
   - Interactive: check off items as verified
   - Add notes about findings per item
   - Notes saved and searchable
   - Regenerate button if initial suggestions unhelpful

2. **Root Cause Suggestions**
   - Likely causes based on symptoms
   - Confidence scores for each suggestion
   - Links to KB articles explaining each cause

3. **Similar Past Tickets** (Hybrid Matching)
   - **Matching algorithm**: Embedding similarity + keyword/entity matching
     - Extract error codes, app names, symptoms from ticket
     - Vector similarity on full ticket text
     - RRF fusion of both scores
   - Shows: Summary + resolution + similarity score
   - Link to original source (Jira/Slack thread)
   - Searchable diagnostic history
   - Top 3-5 most similar shown by default

4. **Decision Trees**
   - **8 Built-in Trees** (ship with app):
     1. Authentication Failure
     2. VPN / Network Connectivity
     3. Email / Calendar Issues
     4. Password Reset
     5. SSO / Single Sign-On
     6. Hardware (laptop, peripherals)
     7. Software Installation
     8. Account Provisioning / Access Requests
   - Auto-generated trees from indexed KB/runbooks (future)
   - Custom trees you can create (JSON editor in Settings)

   **Auto-Detection Algorithm:**
   ```rust
   fn detect_relevant_tree(ticket_text: &str) -> Option<DecisionTree> {
       // Each tree has keyword sets
       let tree_keywords = [
           ("auth", vec!["login", "password", "403", "unauthorized", "can't sign in"]),
           ("vpn", vec!["vpn", "network", "connection", "timeout", "unreachable"]),
           ("email", vec!["email", "outlook", "calendar", "meeting", "inbox"]),
           // ... etc
       ];

       // Score each tree by keyword matches
       let scores: Vec<(f32, &str)> = tree_keywords.iter()
           .map(|(id, keywords)| {
               let matches = keywords.iter()
                   .filter(|kw| ticket_text.to_lowercase().contains(*kw))
                   .count();
               (matches as f32 / keywords.len() as f32, *id)
           })
           .collect();

       // Return highest scoring tree if score > 0.3
       scores.into_iter()
           .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
           .filter(|(score, _)| *score > 0.3)
           .map(|(_, id)| load_tree(id))
   }
   ```
   - Threshold: Tree suggested only if 30%+ keywords match
   - User can dismiss suggestion and pick manually

5. **Escalation Support**
   - Button: "Generate Escalation Note" in Diagnosis panel
   - **Structured template output**:
     ```
     ## Issue Summary
     [Brief description of the problem]

     ## Environment
     - User: [name/ID]
     - System: [OS, browser, affected app]
     - First reported: [date/time]

     ## Troubleshooting Performed
     - [Checklist item 1] → [Finding/result]
     - [Checklist item 2] → [Finding/result]
     - Decision tree path: [nodes taken]

     ## Findings
     - [Key observation 1]
     - [Key observation 2]

     ## Suggested Next Steps
     - [Recommended action for Tier 2/3]
     ```
   - Auto-populated from diagnostic session data
   - Editable before copying

6. **Web Search (On-Demand)**
   - Button: "Check Web for Outages"
   - No confirmation needed (user clicked intentionally)
   - Results shown in diagnosis panel
   - Local by default, web only when button clicked
   - Button disabled until enabled in Settings
   - PII redaction for outbound queries (names/emails/ticket IDs removed)
   - Optional query preview/edit before send (Settings toggle)
   - Domain allowlist + proxy support
   - Prefer official status pages; scrape only where allowed and fail gracefully
   - Disabled by default until enabled in Settings

   **Default Sources** (queried in parallel):
   - Down Detector (`downdetector.com/status/{service}`) - HTML scrape
   - Vendor status pages (auto-detected from service name):
     - `status.microsoft.com`, `status.okta.com`, `status.zoom.us`, etc.
   - **Note**: Twitter/X removed (requires $100/mo API access)

   **Alternative Social Sources** (no API needed):
   - Reddit r/sysadmin search via RSS/JSON API (free)
   - Hacker News search API (free)
   - Google News RSS for `{service} outage`

   **Configurable Sources** (Settings → Advanced):
   - Add custom status page URLs
   - Add custom RSS feed URLs
   - Enable/disable default sources
   - Set timeout per source (default 5s)

**Progress Tracking:**
- Save troubleshooting progress
- Resume if switching tickets
- Link to follow-up for ongoing issues

**Learning (Opt-In):**
- Toggle in Settings → Advanced: "Enable learning from my interactions"
- **Data tracked** (with consent):
  - Checklist items: which were checked, skipped, added
  - Decision tree paths: which branches taken, dead ends
  - Response edits: what user changed before copying
  - Time spent: per panel, per checklist item
  - Resolution outcome: did user mark ticket as resolved?
- **Local storage only**: All learning data stays in SQLCipher
- **Benefits**:
  - Prioritize checklist items that were checked most often
  - Highlight decision tree branches that led to resolutions
  - Pre-fill common response patterns for similar tickets
  - Show "users typically check this first" hints

**Learning Algorithm** (Simple Frequency-Based):
```sql
-- Track checklist effectiveness
CREATE TABLE learning_checklist_stats (
    item_text_hash TEXT,       -- Normalized hash of checklist item
    times_shown INTEGER,
    times_checked INTEGER,
    times_led_to_resolution INTEGER,
    avg_time_to_check_ms INTEGER
);

-- Track decision tree effectiveness
CREATE TABLE learning_tree_stats (
    tree_id TEXT,
    node_id TEXT,
    times_visited INTEGER,
    times_led_to_resolution INTEGER
);
```

**Ranking formula**: `score = (times_checked / times_shown) * (1 + resolution_bonus)`
- No ML model needed - pure SQL aggregation
- Items with higher scores shown first
- Decays over time (multiply by 0.9 per week)

**Diagnosis → Response Integration:**
- Diagnostic findings auto-incorporated into response
- Response reflects what you checked and what you found
- Searchable history of all diagnostic sessions

### Output Features
- **Clarifying questions**: Include suggested follow-up questions in output
- **Variants**: Single output for now (multi-variant is future feature)
- **Quick Capture**: Future feature (minimal UI via hotkey)

### Data Management
- **Wipe Data**:
  - **Tickets Only**: Wipes drafts, follow-ups, diagnostic sessions, attachments linked to drafts, and learning data (preserves KB index)
  - **Full Wipe**: Wipes KB index, vectors, attachments, models, settings, logs, downloads, caches, DB backups, and Keychain entries (master key + HuggingFace token)
- **Source links**: Each draft records which KB chunks were used

**Export/Import** (Full Data Portability):
- **Export** (Settings → Advanced → Export Data):
  - Creates encrypted `.assbackup` archive containing:
    - All drafts and diagnostic sessions
    - All follow-ups
    - All decision trees (including custom)
    - Attachments (encrypted)
    - Settings and preferences
    - Learning data (if enabled)
  - Does NOT include: KB folder contents, model files, embeddings
  - Password-protected (user sets export password)
  - Re-encrypt all encrypted content with export password (Argon2id + AES-256-GCM)
  - Format: `manifest.json` + data payload (zip or tar), then encrypted with versioned header (salt, nonce, kdf params)
  - Filename: `assistsupport-backup-{date}.assbackup`

- **Import** (Settings → Advanced → Import Data):
  - Prompts for export password
  - Shows preview: "X drafts, Y follow-ups, Z trees"
  - Options: Merge with existing OR Replace all
  - Re-embeds imported drafts if embedding model available
  - Re-wraps imported data with local master key after import

**Theme/Appearance:**
- **System default**: Follow macOS system appearance
- **Manual override**: Toggle in Settings → Appearance
  - Light mode (always)
  - Dark mode (always)
  - System (follow OS)
- **Accent color**: Uses macOS system accent color

### Error Handling

| Error | Recovery Strategy |
|-------|-------------------|
| Model load fails | Show error, suggest re-download, offer alternatives |
| KB folder missing | Disable KB features, show warning, keep existing index |
| Index fails | Abort file, log error, continue with remaining files |
| Context overflow | Smart summarize older context, warn user |
| DB corruption | Detect on launch, offer restore from backup |
| Disk space low | Pre-check before operations, graceful failure with message |
| PDF extraction fails | Try OCR provider, skip if still fails |
| OCR provider unavailable | Index extracted text only, mark as partial |
| LanceDB error | Fall back to FTS5-only search |
| LanceDB encryption unsupported | Warn user, require explicit opt-in for unencrypted vectors, otherwise disable vector search |
| Keychain unavailable | Prompt for passphrase, store encrypted key in DB |
| Network timeout (web search) | Show "unable to check" message |
| Model download fails | Offer retry or manual import |

### Crash Recovery

**Auto-save Strategy:**
- Draft input auto-saved every 5 seconds to `drafts` table
- Diagnostic session state saved on every checklist change
- On crash: Next launch detects unsaved draft, offers to restore

**Implementation:**
```rust
// Periodic auto-save in background
tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_secs(5));
    loop {
        interval.tick().await;
        if let Some(draft) = current_draft.lock().as_ref() {
            if draft.is_dirty {
                db.save_draft_autosave(draft).ok();
            }
        }
    }
});
```

**Corruption Detection:**
- On launch: `PRAGMA integrity_check` on SQLCipher DB
- On launch: Verify LanceDB directory exists and is readable
- If corrupted: Show dialog with options:
  1. "Restore from backup" (if .bak exists)
  2. "Start fresh" (wipes data)
  3. "Export what's readable" (salvage partial data)

**Logging for Debugging:**
- Rotating log files: `~/Library/Logs/AssistSupport/`
- Keep last 7 days by default, configurable in Settings
- Log levels: ERROR (always), WARN, INFO, DEBUG (dev only)
- Include: Timestamps, operation names, durations, error messages
- Redact PII and avoid logging full ticket/KB content
- Never log secrets (tokens, keys, passphrases)

### Distribution
- **Lean DMG**: ~50-100MB, models downloaded separately
- **Full DMG (optional)**: ~10GB, includes recommended models; optional variant can bundle Tesseract if needed
- **Tesseract size impact (estimate)**: +30-80MB for lean DMG (arm64, English only); confirm in Phase 0B
- **Model download**: From HuggingFace (work machine has internet)
- **Unsigned internal distribution**: Provide IT deployment steps to allow app execution (remove quarantine flag if needed)
- **Install note**: If blocked by Gatekeeper, remove quarantine attribute after copy
- **Sandbox (v1)**: Disabled; no entitlements required

### Diagnostics & Logging
- **Diagnostics mode**: Explicit toggle; developer view (generation time, tokens/sec, memory, cache hits, raw logs, query traces, error history)
- **Logging**: Rotating logs (keep last 7 days, auto-delete old)
- **Updates**: Manual (check GitHub releases)

### Performance Targets

| Metric | Target | Failure Threshold |
|--------|--------|-------------------|
| Generation latency | 2-10s | >15s |
| Hybrid search | <100ms | >500ms |
| KB indexing | 100 docs/min | <50 docs/min |
| Embedding batch | 50 chunks/sec | <20 chunks/sec |
| Memory (app only, models unloaded) | <500MB | >1GB |
| Memory (models loaded, idle) | <14GB | >20GB |
| Memory (generating) | <16GB | >22GB |

### Quality Gates (Go/No-Go)
- **All tests must pass**: Unit, integration, E2E, and security tests
- **All benchmarks must meet targets**: Performance table above is a release gate
- **Stress tests must pass**: No crashes, no unbounded memory growth, no data loss
- **Security checks must pass**: Encryption behaviors, key storage, PII redaction, log redaction
- **Vector safety**: Vector search enabled only if encryption supported or explicit opt-in recorded
- **OCR gate**: Tiered WER thresholds (text-based pages <= 5%, image-only pages <= 12%, mixed overall <= 10%); if not, ship optional Tesseract DMG
- **Release criteria**: Zero open P0/P1 defects; all P2 fixes scheduled

### Testing Strategy

**Unit Tests** (Rust `cargo test` + TypeScript Vitest):
- Chunking algorithms
- RRF fusion math
- Keyword extraction
- Decision tree traversal
- Export/import format
- Web query PII redaction
- Prompt injection guardrail (source text cannot alter system rules)

**Integration Tests** (Tauri command tests):
- DB migrations run correctly
- Settings persist across restarts
- File watcher triggers reindex
- LanceDB queries return correct types
- SQLCipher build has FTS5 enabled
- FTS5 index updates on insert/update/delete
- Keychain read/write and passphrase fallback
- Vector encryption verified if supported; warning + explicit opt-in flow tested
- OCR provider selection logic and fallback behavior

**LLM Testing Strategy**:
```rust
// Use a mock LLM for CI tests
#[cfg(test)]
mod tests {
    use crate::llm::MockLlmEngine;

    fn mock_engine() -> MockLlmEngine {
        MockLlmEngine::with_responses(vec![
            "Here's a helpful response...",
            "Step 1: Check X\nStep 2: Verify Y",
        ])
    }
}
```
- Mock responses stored in `test_fixtures/llm_responses/`
- Real LLM tests run manually (too slow for CI)
- Output quality validated by: word count, contains expected sections

**E2E Tests** (Playwright):
1. First-run wizard completes successfully
2. Model download resumes and checksum validates
3. Model loading shows progress, handles errors
4. KB indexing shows progress bar
5. Generation streams word-by-word
6. Copy button copies to clipboard
7. Theme toggle persists
8. Export creates file, import restores
9. Web search remains disabled until enabled; allowlist enforced

**Web Search Mocking**:
```typescript
// Mock fetch for web search tests
vi.mock('fetch', () => ({
  default: vi.fn((url) => {
    if (url.includes('downdetector')) {
      return Promise.resolve({ text: () => mockDownDetectorHtml });
    }
  })
}));
```

**Performance Benchmarks** (run manually):
- Generation: Time to first token, total time, tokens/sec
- Search: Query latency at 1K, 10K, 100K chunks
- Indexing: Docs/minute for Markdown, PDF
- Memory: Baseline, during generation, peak
- OCR: Mixed PDF accuracy and throughput on a representative sample (tiered WER thresholds)

**Stress Tests** (run manually):
- Large KB: 10K docs / 100K chunks indexing, repeated reindex
- Mixed PDFs: 100+ PDFs with image-only pages
- Long session: 2+ hours of generation/search without restart
- Model churn: Load/unload 10x, verify no leaks or crashes
- Low disk simulation: Trigger graceful failure paths

**Security Tests** (run manually):
- Keychain storage and fallback passphrase flow
- SQLCipher encryption verified (no plaintext DB)
- Vector encryption warning + explicit opt-in flow
- Export/import crypto: data re-encrypted with export password, no plaintext artifacts
- PII redaction before outbound web queries
- Log redaction (no ticket/KB plaintext)
- Prompt injection guardrail test using malicious KB snippets
- Dependency audit (cargo audit, pnpm audit) has zero critical/high findings

**Reliability/Recovery Tests** (run manually):
- Crash recovery: autosave restore works
- Corruption handling: integrity_check and recovery paths
- Network drop: model download resume + web search timeout handling
- Full wipe: removes logs, downloads, caches, backups, and Keychain entries

### E2E Test Scenarios
1. First-run: Load models, set KB folder, index, generate response
2. Diagnostic flow: Paste ticket → checklist → check items → generate response
3. Decision tree: Start tree → navigate → reach resolution
4. Similar tickets: Create session → search → find similar
5. Offline: Core features work without network, web search/downloads disabled gracefully
6. Error recovery: Model load fails → shows error → user retries
7. Long input: Paste 50K chars → summarization triggers → generation works

### Go/No-Go Procedure
- Run full test matrix and benchmarks
- Capture evidence (logs, metrics, screenshots where applicable)
- Any gate failure blocks release until fixed and retested
- Formal signoff required before packaging the DMG (store QA report under `docs/qa/`)
- OCR benchmark datasets:
  - Public/synthetic subset stored in-repo under `docs/qa/ocr-bench/`
  - Internal mixed set stored in secure share; record version/hash in QA report

---

## Phase 0A: Project Bootstrap (Day 0)

### Goals
- Install prerequisites: Rust stable, Node LTS, pnpm, Tauri CLI, Xcode CLT
- Initialize a new Tauri + React (Vite + TS) app in `/Users/d/AssistSupport`
- Verify `pnpm tauri dev` launches a blank window

---

## Phase 0B: Platform + Security Spike (Days 0-2)

### Goals
- Validate SQLCipher build with FTS5 enabled
- Verify LanceDB encryption support (or confirm fallback strategy)
- Confirm OCR provider integration on macOS (Vision) and optional Tesseract fallback
- Validate Keychain storage + retrieval for master key
- Validate unsigned DMG install path (Gatekeeper/quarantine handling)
- Validate PDFium bundling for text extraction (no system dependency)
- Run OCR benchmark on a mixed PDF sample set (text-only + image-only)
- Create curated OCR benchmark set (>=50 pages, >=30% image-only) with ground truth for WER
- Define OCR evaluation method: WER via `jiwer`, normalize case, punctuation, whitespace, and line-break hyphenation
- If bundling Tesseract, validate libtesseract + leptonica packaging on macOS
 - Verify Vision helper binary executes from bundled resources

### Verification
```bash
# Confirm FTS5 in SQLCipher build
sqlite3 assistsupport.db "CREATE VIRTUAL TABLE t USING fts5(content);"
```

---

## Phase 1: Real LLM Integration (Days 1-3)

### Files to Modify

**1. `/Users/d/AssistSupport/src-tauri/Cargo.toml`**
- Add `llama-cpp-2 = "0.1.130"` (verified on crates.io)
- Add feature flags for Metal acceleration

**2. `/Users/d/AssistSupport/src-tauri/src/llm.rs`**
- Replace mock `generate()` with real llama-cpp-2 inference
- Implement proper model loading with `LlamaModelParams`
- Add Metal GPU offloading (`n_gpu_layers = 1000`)
- Implement streaming via existing `mpsc` channel

### Verification
```bash
cd /Users/d/AssistSupport
pnpm tauri dev
# Load a GGUF model, generate response, verify <10s latency
```

---

## Phase 2: KB Database Schema (Days 3-4)

**Prerequisite**: Ensure SQLCipher is built with FTS5 enabled (compile flags or bundled build).

### Files to Modify

**1. `/Users/d/AssistSupport/src-tauri/src/db.rs`**
Add new tables in migration:

```sql
-- Documents indexed from KB folder
CREATE TABLE kb_documents (
    id TEXT PRIMARY KEY,
    file_path TEXT NOT NULL UNIQUE,
    file_hash TEXT NOT NULL,
    title TEXT,
    indexed_at TEXT,
    chunk_count INTEGER
);

-- Document chunks for retrieval
CREATE TABLE kb_chunks (
    id TEXT PRIMARY KEY,
    document_id TEXT NOT NULL,
    chunk_index INTEGER NOT NULL,
    heading_path TEXT,
    content TEXT NOT NULL,
    word_count INTEGER,
    FOREIGN KEY (document_id) REFERENCES kb_documents(id) ON DELETE CASCADE
);
-- Note: keep rowid enabled; FTS5 uses kb_chunks.rowid for joins

-- FTS5 full-text search
CREATE VIRTUAL TABLE kb_fts USING fts5(
    content, heading_path,
    content='kb_chunks',
    tokenize='porter unicode61'
);

-- FTS5 triggers (keep index in sync; join on rowid)
CREATE TRIGGER kb_chunks_ai AFTER INSERT ON kb_chunks BEGIN
    INSERT INTO kb_fts(rowid, content, heading_path)
    VALUES (new.rowid, new.content, new.heading_path);
END;
CREATE TRIGGER kb_chunks_ad AFTER DELETE ON kb_chunks BEGIN
    INSERT INTO kb_fts(kb_fts, rowid, content, heading_path)
    VALUES ('delete', old.rowid, old.content, old.heading_path);
END;
CREATE TRIGGER kb_chunks_au AFTER UPDATE ON kb_chunks BEGIN
    INSERT INTO kb_fts(kb_fts, rowid, content, heading_path)
    VALUES ('delete', old.rowid, old.content, old.heading_path);
    INSERT INTO kb_fts(rowid, content, heading_path)
    VALUES (new.rowid, new.content, new.heading_path);
END;

-- Diagnostic sessions
CREATE TABLE diagnostic_sessions (
    id TEXT PRIMARY KEY,
    draft_id TEXT,
    checklist_json TEXT,      -- JSON array of checklist items + notes
    findings_json TEXT,       -- JSON array of findings
    decision_tree_id TEXT,    -- Which tree was used (if any)
    created_at TEXT,
    updated_at TEXT,
    FOREIGN KEY (draft_id) REFERENCES drafts(id) ON DELETE SET NULL
);

-- Decision trees (pre-built + custom)
CREATE TABLE decision_trees (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    category TEXT,            -- 'auth', 'vpn', 'email', etc.
    tree_json TEXT NOT NULL,  -- JSON structure of nodes/branches
    source TEXT,              -- 'builtin', 'learned', 'custom'
    created_at TEXT,
    updated_at TEXT
);

-- Learning stats (if enabled)
CREATE TABLE learning_checklist_stats (
    item_text_hash TEXT,       -- Normalized hash of checklist item
    times_shown INTEGER,
    times_checked INTEGER,
    times_led_to_resolution INTEGER,
    avg_time_to_check_ms INTEGER
);
CREATE TABLE learning_tree_stats (
    tree_id TEXT,
    node_id TEXT,
    times_visited INTEGER,
    times_led_to_resolution INTEGER
);

-- NOTE: Vector embeddings stored in LanceDB (separate encrypted file)
-- ~/Library/Application Support/AssistSupport/vectors/chunks.lance/
```

**2. `/Users/d/AssistSupport/src-tauri/Cargo.toml`**
- Add LanceDB dependency: `lancedb = "0.23"` (verified on crates.io)
- Add SQLCipher-backed DB deps: `rusqlite` + `libsqlite3-sys` (build SQLCipher with FTS5)

**Security dependencies (Cargo.toml)**
- `aes-gcm` (verify version)
- `argon2` (verify version)
- `rand` (verify version)
- `zeroize` (verify version)
- `keyring` (verify version)

### Verification
```bash
# Dev sanity (system sqlite3; not a release gate)
sqlite3 ~/Library/Application\ Support/AssistSupport/assistsupport.db ".tables"
sqlite3 ~/Library/Application\ Support/AssistSupport/assistsupport.db "SELECT sqlite_compileoption_used('SQLITE_ENABLE_FTS5');"
```
```
# Release gate: in-app check via SQLCipher connection (Tauri command)
check_fts5_enabled() -> true
```

---

## Phase 3: KB Indexer Module (Days 4-6)

### New Files to Create

**1. `/Users/d/AssistSupport/src-tauri/src/kb/mod.rs`**
```rust
pub mod indexer;
pub mod embeddings;
pub mod search;
pub mod ocr;
```

**2. `/Users/d/AssistSupport/src-tauri/src/kb/indexer.rs`**
- Scan folder for `.md` files
- Parse Markdown with `pulldown-cmark` crate
- Chunk by heading (H1/H2 boundaries)
- Target 200-500 words per chunk
- Track file hashes for incremental updates

**3. `/Users/d/AssistSupport/src-tauri/src/kb/ocr.rs`**
- Define `OcrEngine` trait and provider selection
- Providers:
  - `VisionOcr` (macOS) via small Swift helper binary
  - `TesseractOcr` (optional, if bundled/available)

**4. Add to Cargo.toml**
```toml
pulldown-cmark = "0.9"    # Markdown parsing
pdfium-render = "0.8"     # PDF text extraction (verify version)
tesseract = "0.13"        # Optional OCR provider (feature-gated; enabled only in Tesseract DMG build)
image = "0.25"            # Image decode for OCR pipeline
```

**Natively Indexed file types:**
- `.md` - Markdown (primary)
- `.pdf` - PDF documents (text + OCR for images)
- `.txt` - Plain text
- Images (.png, .jpg, .jpeg, .gif, .tif, .tiff) - OCR provider (Vision on macOS, Tesseract optional)

**Convert & Import feature** (converts to Markdown, then indexes):
- Confluence HTML exports → `scraper` crate, extract article content
- Excel/Google Sheets (.xlsx, .csv) → `calamine` crate
- Box notes → JSON parse + content extraction
- HTML pages → `scraper` crate
- Word documents (.docx) → `docx-rs` crate or zip + xml parsing

**Implementation:**
- Settings UI: "Import Files" button → native file picker (multi-select)
- Progress bar for batch conversion
- Converted files saved to KB folder as `.md`
- Then indexed normally with file watcher

### Cargo.toml additions
```toml
pulldown-cmark = "0.9"    # Markdown parsing
pdfium-render = "0.8"     # PDF text extraction (verify version)
tesseract = "0.13"        # Optional OCR provider (feature-gated; enabled only in Tesseract DMG build)
image = "0.25"            # Image decode for OCR pipeline
calamine = "0.24"         # Excel/CSV reading
scraper = "0.18"          # HTML parsing
zip = "0.6"               # For .docx (zip archive)
quick-xml = "0.31"        # For .docx XML content
```

### Verification
```bash
# Index a test folder, verify chunks in database
# Import a Confluence HTML export, verify conversion
# Import an Excel file, verify rows become Markdown tables
```

---

## Phase 4: Embedding Engine (Days 6-7)

### New File

**1. `/Users/d/AssistSupport/src-tauri/src/kb/embeddings.rs`**
- Load separate embedding model (smaller than generation model)
- Use llama-cpp-2 for embedding inference
- Batch process chunks for efficiency (50 per batch)
- Store embeddings in LanceDB `chunks` table

### Recommended Models

**Embedding Model** (separate, stays loaded for indexing):
- Primary: `nomic-embed-text-v1.5.Q5_K_M.gguf` (~550MB, 768 dims)
- Fallback: `bge-small-en-v1.5.Q8_0.gguf` (~130MB, 384 dims)

**Generation Models** (test both, benchmark on your hardware):
- Fast: `Llama-3.2-3B-Instruct.Q5_K_M.gguf` (~2.5GB) - fastest, good for simple responses
- Balanced: `Qwen2.5-7B-Instruct.Q5_K_M.gguf` (~5.5GB) - best quality/speed tradeoff
- Quality: `Qwen2.5-14B-Instruct.Q4_K_M.gguf` (~9GB) - higher quality, slower

**RAM Budget** (48GB available):
- Embedding model: ~1-2GB loaded
- Generation model: ~6-12GB loaded
- App + OS: ~4GB
- **Headroom**: 20-30GB for other apps

---

## Phase 5: Hybrid Search (Days 7-8)

### New Files

**1. `/Users/d/AssistSupport/src-tauri/src/kb/vectors.rs`**
- Initialize LanceDB at `~/Library/Application Support/AssistSupport/vectors/`
- Create `chunks` table with schema: `id: String, embedding: FixedSizeList[768]`
- Enable encryption if supported; otherwise require explicit opt-in before enabling vector search
- Implement `insert_embedding()`, `search_similar()`, `delete_by_id()`

**2. `/Users/d/AssistSupport/src-tauri/src/kb/search.rs`**

```rust
pub async fn hybrid_search(query: &str, limit: usize) -> Vec<SearchResult> {
    // 1. FTS5 keyword search (SQLite)
    let keyword_results = fts5_search(query, limit * 2);

    // 2. Vector similarity search (LanceDB)
    let query_embedding = embed(query).await;
    let semantic_results = lancedb_search(&query_embedding, limit * 2).await;

    // 3. RRF fusion (k=60)
    let fused = rrf_merge(keyword_results, semantic_results, 60);

    // 4. Return top N with sources
    fused.into_iter().take(limit).collect()
}
```
- **FTS5 join**: Use `kb_fts.rowid = kb_chunks.rowid` to fetch chunk metadata

### Performance Targets
- FTS5 search: <50ms for 10K chunks
- LanceDB search: <50ms for 100K vectors
- Combined hybrid: <100ms total

### Verification
```bash
# Search for "authentication error", verify:
# - Both keyword and semantic matches returned
# - Results properly de-duplicated by RRF
# - Response time <100ms
```

---

## Phase 6: Generation Integration (Days 8-9)

### Files to Modify

**1. `/Users/d/AssistSupport/src-tauri/src/prompts.rs`**
- Add KB context injection between OCR and user input
- Format: `[Source: filename.md > Heading]\nContent...`

**2. `/Users/d/AssistSupport/src-tauri/src/commands.rs`**
- Add `search_kb` command
- Add `index_kb` command
- Add `set_kb_folder` command
- Modify `generate` to optionally include KB context

**3. `/Users/d/AssistSupport/src/App.tsx`**
- Add KB folder selector in Settings tab
- Add "Index KB" button with progress
- Add Sources panel (in-app only, never in output)

---

## Phase 7: Diagnostic Assistant (Days 8-10)

### New Files

**1. `/Users/d/AssistSupport/src-tauri/src/kb/diagnosis.rs`**
- `generate_checklist()`: LLM generates troubleshooting steps from context
- `find_similar_sessions()`: Search past diagnostic sessions by embedding similarity
- `suggest_root_causes()`: LLM analyzes symptoms, returns causes + confidence
- `generate_escalation_note()`: Format findings for Tier 2/3 handoff

**2. `/Users/d/AssistSupport/src-tauri/src/kb/trees.rs`**
- `DecisionTree` struct: nodes, branches, conditions
- `load_builtin_trees()`: Auth, VPN, Email, SSO built-in trees
- `detect_relevant_tree()`: Keyword/embedding match to suggest tree
- `traverse_tree()`: Navigate based on user selections

**3. `/Users/d/AssistSupport/src/components/DiagnosisPanel.tsx`**
- Three sections: Checklist | Suggestions | Similar Tickets
- Interactive checklist with notes field per item
- Collapsible decision tree navigator
- "Check Web for Outages" button

**4. `/Users/d/AssistSupport/src/components/DecisionTree.tsx`**
- Visual tree with current position highlighted
- Click to navigate branches
- Shows resolution when reaching leaf node

### Built-in Decision Trees (JSON format)
```json
{
  "id": "auth-failure",
  "name": "Authentication Failure",
  "category": "auth",
  "nodes": [
    {"id": "start", "text": "Can user reach login page?", "yes": "page-loads", "no": "network-issue"},
    {"id": "page-loads", "text": "Does password field accept input?", "yes": "creds-check", "no": "browser-issue"},
    ...
  ]
}
```

### Decision Tree Content Outlines

**1. Authentication Failure** (~15 nodes)
- Start: Can reach login page? → Network vs Auth issue
- Check: Correct username format?
- Check: Password expired?
- Check: Account locked?
- Check: MFA device available?
- Resolution paths: Reset password, unlock account, re-enroll MFA

**2. VPN / Network Connectivity** (~12 nodes)
- Start: On corporate network or remote?
- Check: VPN client installed and updated?
- Check: Internet working without VPN?
- Check: Can ping internal resources?
- Resolution paths: Reinstall VPN, check firewall, contact network team

**3. Email / Calendar Issues** (~15 nodes)
- Start: Outlook desktop or web?
- Check: Can send? Can receive? Both?
- Check: Specific recipient or all?
- Check: Calendar invites working?
- Resolution paths: Repair Outlook, clear cache, check rules

**4. Password Reset** (~8 nodes)
- Start: Which system? (AD, Okta, app-specific)
- Check: Self-service available?
- Check: Security questions set up?
- Resolution paths: Self-service reset, IT-assisted reset, manager approval

**5. SSO / Single Sign-On** (~12 nodes)
- Start: Which app failing?
- Check: Other SSO apps working?
- Check: Session expired?
- Check: Browser cookies enabled?
- Resolution paths: Clear SSO session, re-authenticate, check app assignment

**6. Hardware** (~18 nodes)
- Start: Laptop, monitor, keyboard, mouse, other?
- Check: Power? Connections? Drivers?
- Check: Under warranty?
- Resolution paths: Troubleshooting steps, replacement request, repair

**7. Software Installation** (~10 nodes)
- Start: Self-Service Portal or IT request?
- Check: Admin rights needed?
- Check: Software approved?
- Resolution paths: Portal install, request approval, IT ticket

**8. Account Provisioning** (~10 nodes)
- Start: New hire, role change, or access request?
- Check: Manager approval obtained?
- Check: Which systems needed?
- Resolution paths: Submit request form, escalate to IAM team

### Web Search Integration

**Service Detection:**
- Extract service names from ticket (Okta, Zoom, Slack, Microsoft 365, etc.)
- Map to known status page URLs (configurable mapping table)
- Default mappings for 20+ common enterprise services
- Sanitize outbound queries (service names only, remove PII/ticket IDs)
- Respect allowlist + proxy settings

**Query Execution:**
```rust
async fn check_outages(services: Vec<&str>) -> Vec<OutageResult> {
    // Run all queries in parallel with 5s timeout each
    let futures = services.iter().map(|s| async {
        let dd = scrape_downdetector(s);      // HTML parse
        let status = fetch_status_page(s);    // JSON/RSS
        let reddit = search_reddit_sysadmin(s); // JSON API (free)
        let news = fetch_google_news_rss(s);  // RSS feed
        join!(dd, status, reddit, news)
    });
    join_all(futures).await
}
```

**Why no Twitter/X:**
- X API requires paid Basic tier ($100/month)
- OAuth complexity for a desktop app
- Reddit + HN + Google News provide similar signal for free

**Rate Limiting:**
- Down Detector: Max 1 request per service per 5 minutes
- Status pages: Max 1 request per page per 1 minute
- Reddit/HN: Max 1 request per 10 seconds (respect API limits)
- Cache results in memory for duration above
- Show "cached X minutes ago" indicator

**Result Display:**
- Green: "No issues reported"
- Yellow: "Some reports in last hour"
- Red: "Widespread outage detected"
- Show: Source, timestamp, report count, link to details

---

## Phase 8: Polish (Days 10-12)

- Add native file picker for KB folder (Tauri dialog API)
- Add file watcher for auto-reindexing (`notify` crate)
- Add indexing progress UI with cancel button
- Keyboard shortcuts (Cmd+G generate, Cmd+C copy, Cmd+1/2/3/4 tabs)
- Toast notifications for copy, index complete, errors
- Benchmark and optimize (target <10s generation, <100ms search)
- First-run experience with model download guidance

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                      AssistSupport                          │
├─────────────────────────────────────────────────────────────┤
│  React Frontend                                             │
│  └─ Draft │ Follow-ups │ Sources │ Settings                │
│     ├─ Input Panel                                          │
│     ├─ Diagnosis Panel (checklists, trees, suggestions)    │
│     └─ Response Panel                                       │
├─────────────────────────────────────────────────────────────┤
│  Tauri Backend (Rust)                                       │
│  ├─ llm.rs ──────────► llama-cpp-2 (embedded, Metal GPU)   │
│  ├─ kb/indexer.rs ───► Markdown/PDF parsing + chunking     │
│  ├─ kb/embeddings.rs ► Embedding model (separate)          │
│  ├─ kb/search.rs ────► FTS5 + LanceDB + RRF fusion         │
│  ├─ kb/diagnosis.rs ─► Diagnostic assistant logic          │
│  └─ prompts.rs ──────► Context injection                   │
├─────────────────────────────────────────────────────────────┤
│  Storage Layer                                              │
│  ├─ SQLCipher (assistsupport.db) - encrypted               │
│  │   ├─ drafts, followups, attachments                     │
│  │   ├─ kb_documents, kb_chunks, kb_fts                    │
│  │   └─ diagnostic_sessions, decision_trees                │
│  └─ LanceDB (vectors/) - encrypted if supported             │
│      └─ chunks.lance (768-dim embeddings)                   │
└─────────────────────────────────────────────────────────────┘
```

---

## Critical Files Summary

| File | Action | Purpose |
|------|--------|---------|
| `src-tauri/Cargo.toml` | Modify | Add llama-cpp-2, lancedb, pulldown-cmark, pdfium-render |
| `src-tauri/src/llm.rs` | Rewrite | Replace mock with real llama-cpp-2 |
| `src-tauri/src/db.rs` | Modify | Add KB + diagnostic table migrations |
| `src-tauri/src/kb/mod.rs` | Create | KB module root |
| `src-tauri/src/kb/indexer.rs` | Create | Document parsing + chunking |
| `src-tauri/src/kb/embeddings.rs` | Create | Embedding generation |
| `src-tauri/src/kb/ocr.rs` | Create | OCR provider abstraction (Vision/Tesseract) |
| `src-tauri/src/kb/search.rs` | Create | Hybrid search (FTS5 + LanceDB + RRF) |
| `src-tauri/src/kb/vectors.rs` | Create | LanceDB operations |
| `src-tauri/src/kb/diagnosis.rs` | Create | Diagnostic assistant logic |
| `src-tauri/src/kb/trees.rs` | Create | Decision tree storage + traversal |
| `src-tauri/src/commands.rs` | Modify | Add KB + diagnostic commands |
| `src-tauri/src/prompts.rs` | Modify | Add KB context + diagnostic injection |
| `src-tauri/src/lib.rs` | Modify | Register kb module |
| `src/components/DiagnosisPanel.tsx` | Create | Diagnostic UI component |
| `src/components/DecisionTree.tsx` | Create | Interactive tree visualization |

---

## Verification Plan

**All verification items are release gates (go/no-go).**

1. **LLM works**: Load model, generate response in <10s
2. **KB indexes**: Scan folder, parse Markdown, create chunks
3. **FTS5 works**: Keyword search returns relevant chunks (<100ms)
4. **Embeddings work**: Generate 768-dim vectors, store in LanceDB
5. **Hybrid search works**: RRF combines FTS5 + LanceDB results (<100ms)
6. **Integration works**: Generation uses KB context, sources hidden in output
7. **Diagnostic works**: Checklist + suggestions populate from context
8. **Decision trees work**: Navigate tree, auto-detect relevant tree
9. **Similar tickets work**: Past sessions searchable, similarity matching
10. **Encryption works**: SQLCipher + vectors encrypted if supported, otherwise warning + explicit opt-in

---

## Fallback Strategies

| Issue | Fallback |
|-------|----------|
| llama-cpp-2 build fails | Keep LLM disabled with setup guidance and diagnostics |
| LanceDB issues | FTS5-only search (no semantic, still useful) |
| LanceDB encryption unsupported | Warn user, require explicit opt-in for unencrypted vectors, otherwise disable vector search |
| Embedding too slow | Use smaller model (bge-small-en, 384 dims) |
| Metal not available | CPU inference (slower but functional) |
| PDF extraction fails | Try OCR provider, skip if still fails |
| Keychain unavailable | Prompt for passphrase, store wrapped key in DB |
| Web search fails | Show "unable to check" message, continue locally |

---

## Estimated Timeline

Building in **4 phases** with clear milestones (plus Phase 0A/0B pre-work):

### Phase A: Foundation (Week 1-2)
| Task | Days | Deliverable | Risk |
|------|------|-------------|------|
| 1. LLM Integration | 3-4 | Real llama-cpp-2 inference working | High - crate build issues |
| 2. DB Schema Migration | 1-2 | All new tables created | Low |
| 3. Model Loading UI | 2-3 | File picker + download manager + wizard step 1-3 | Low |
| 4. Settings Tab Refactor | 2-3 | Accordion sections working | Low |

**Milestone A**: Can load model, generate response, see streaming output

### Phase B: Knowledge Base (Week 2-3)
| Task | Days | Deliverable | Risk |
|------|------|-------------|------|
| 5. KB Indexer | 3-4 | Markdown + PDF parsing, chunking | Medium |
| 6. FTS5 Search | 2-3 | Keyword search working | Low |
| 7. Embedding Engine | 3-4 | Nomic model, batch embedding | Medium |
| 8. LanceDB Integration | 3-4 | Vector store + queries | High - new crate |
| 9. Hybrid Search + RRF | 2-3 | Combined results | Low |

**Milestone B**: KB indexed, hybrid search returns results, sources shown

### Phase C: Diagnostic Assistant (Week 3-4)
| Task | Days | Deliverable | Risk |
|------|------|-------------|------|
| 10. Three-Panel UI | 3-4 | Draggable, collapsible panels | Medium |
| 11. Checklist Component | 2-3 | LLM-generated, interactive | Low |
| 12. Decision Trees (4 of 8) | 4-5 | Auth, VPN, Email, Password | Content work |
| 13. Similar Tickets | 3-4 | Embedding similarity matching | Medium |
| 14. Web Search | 3-4 | Down Detector, status pages | Medium - scraping |

**Milestone C**: Full diagnostic workflow works end-to-end

### Phase D: Polish (Week 4-5)
| Task | Days | Deliverable | Risk |
|------|------|-------------|------|
| 15. Decision Trees (4 more) | 3-4 | SSO, Hardware, Software, Provisioning | Content work |
| 16. Escalation Notes | 2-3 | Template generation | Low |
| 17. Learning System | 2-3 | Stats tracking + ranking | Low |
| 18. Export/Import | 2-3 | Backup/restore working | Low |
| 19. Keyboard Shortcuts | 1-2 | All shortcuts wired | Low |
| 20. First-Run Wizard | 2-3 | Complete setup flow | Low |
| 21. Testing + Fixes | 3-5 | All tests/benchmarks pass | Variable |

**Milestone D**: Formal go/no-go review, production-ready if all gates pass

---

**Total: ~5 weeks** (25 working days, assumes some parallel work)

**Buffer**: Add 1 week for unexpected issues = **6 weeks total**
**Note**: Formal QA gates can extend timeline if any failures occur

**Critical Path**: Project Bootstrap → Platform + Security Spike → LLM Integration → Embedding Engine → LanceDB → Hybrid Search
- If any of these block, everything after is delayed
- Recommend: Spike LLM + LanceDB crates in first 2 days

---

## First-Run Experience

**Guided Setup Wizard** (modal on first launch):

### Step 1: Welcome
- Brief explanation: "AssistSupport needs a local AI model to generate responses"
- In-app download option (recommended models) or manual import
- Optional HuggingFace token field for gated models

### Step 2: Select Generation Model
- Native file picker for `.gguf` file or pick from downloaded models
- Show file size, verify it's a valid GGUF
- SHA256 checksum verification (optional, show progress)
- Recommended: `Qwen2.5-7B-Instruct.Q5_K_M.gguf`

### Step 3: Test Generation
- Auto-run a test prompt: "Say hello in one sentence"
- Show: tokens/sec, total time, success/fail
- If fails: offer troubleshooting (wrong model type, insufficient RAM)
- "Model loaded successfully! ✓"

### Step 4: Select Embedding Model (Optional)
- Explain: "For KB search, you need a separate embedding model"
- Skip option: "I'll set this up later"
- Recommended: `nomic-embed-text-v1.5.Q5_K_M.gguf`

### Step 5: Select KB Folder (Optional)
- Explain: "Point to your runbooks/docs folder for context-aware responses"
- Skip option: "I'll set this up later"
- Native folder picker

### Step 6: Ready!
- Summary of what's configured
- "Get Started" → goes to Draft tab

**Graceful degradation**:
- No generation model → show wizard again
- No embedding model → KB indexing disabled, FTS5-only search
- No KB folder → Generation works without KB context

**Streaming Display**: Word-by-word
- Buffer tokens until whitespace, display complete words
- Better UX than character-by-character (less jittery)
- Faster perceived response than sentence chunks
