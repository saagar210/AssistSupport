# AssistSupport Improvement Plan - Part 1

## Overview

This plan addresses all gaps identified during the comprehensive app audit. The goal is to transform the current bare-bones MVP into a polished, feature-complete application matching the original implementation plan vision.

**Current State**: Basic LLM generation + FTS5 KB search working
**Target State**: Full diagnostic assistant with modern UX, integrations, and production polish

---

## Priority 1: Core UX Fixes

**Goal**: Make the app feel modern and responsive
**Estimated Effort**: 1-2 days

### 1.1 Streaming Generation Display
Currently generation shows all text at once after completion. Users need to see tokens appear word-by-word.

**Files to Modify**:
- `src-tauri/src/commands.rs` - Add streaming command with Tauri events
- `src-tauri/src/llm.rs` - Already has streaming, need to expose it
- `src/hooks/useLlm.ts` - Listen to streaming events
- `src/components/Draft/ResponsePanel.tsx` - Render streaming text

**Implementation**:
```rust
// Backend: Emit tokens as events
#[tauri::command]
pub async fn generate_streaming(
    window: tauri::Window,
    state: State<'_, AppState>,
    params: GenerateWithContextParams,
) -> Result<(), String> {
    // ... setup ...
    while let Some(event) = rx.recv().await {
        match event {
            GenerationEvent::Token(t) => {
                window.emit("generation-token", &t).ok();
            }
            GenerationEvent::Done { tokens_generated, duration_ms } => {
                window.emit("generation-done", json!({
                    "tokens_generated": tokens_generated,
                    "duration_ms": duration_ms
                })).ok();
                break;
            }
            // ...
        }
    }
}
```

```typescript
// Frontend: Listen and accumulate
const [streamingText, setStreamingText] = useState('');

useEffect(() => {
  const unlisten = listen('generation-token', (event) => {
    setStreamingText(prev => prev + event.payload);
  });
  return () => { unlisten.then(fn => fn()); };
}, []);
```

**Verification**:
- Generate response, see words appear one-by-one
- Word count updates in real-time
- Cancel button stops generation mid-stream

### 1.2 Theme Support (Light/Dark/System)
Add proper theming with CSS variables.

**Files to Create/Modify**:
- `src/styles/themes.css` - CSS variables for both themes
- `src/hooks/useTheme.ts` - Theme state + system detection
- `src/components/Settings/SettingsTab.tsx` - Theme selector
- `src/App.tsx` - Apply theme class to root

**CSS Variables**:
```css
:root, [data-theme="light"] {
  --bg-primary: #ffffff;
  --bg-secondary: #f8f9fa;
  --bg-tertiary: #e9ecef;
  --text-primary: #212529;
  --text-secondary: #6c757d;
  --border: #dee2e6;
  --primary: #0066cc;
  --primary-hover: #0052a3;
  --success: #28a745;
  --warning: #ffc107;
  --danger: #dc3545;
  --shadow: rgba(0, 0, 0, 0.1);
}

[data-theme="dark"] {
  --bg-primary: #1a1a2e;
  --bg-secondary: #16213e;
  --bg-tertiary: #0f3460;
  --text-primary: #e8e8e8;
  --text-secondary: #a0a0a0;
  --border: #2d2d44;
  --primary: #4da6ff;
  --primary-hover: #80bdff;
  --success: #4caf50;
  --warning: #ffca28;
  --danger: #f44336;
  --shadow: rgba(0, 0, 0, 0.3);
}
```

**Verification**:
- Toggle between Light/Dark/System in Settings
- All panels respect theme
- System preference changes auto-update (if System selected)

### 1.3 Loading States & Skeletons
Replace basic spinners with proper loading skeletons and progress indicators.

**Files to Create/Modify**:
- `src/components/shared/Skeleton.tsx` - Reusable skeleton component
- `src/components/shared/ProgressBar.tsx` - Determinate/indeterminate progress
- Update all panels to show skeletons during load

**Skeleton Component**:
```typescript
interface SkeletonProps {
  variant: 'text' | 'rect' | 'circle';
  width?: string | number;
  height?: string | number;
  lines?: number; // For text variant
}

export function Skeleton({ variant, width, height, lines = 1 }: SkeletonProps) {
  // Pulsing animation skeleton
}
```

**Where to Use**:
- ResponsePanel: Text skeleton while generating
- SourcesTab: Table skeleton while loading files
- SettingsTab: Card skeletons while loading model info
- DiagnosisPanel: List skeleton while generating checklist

### 1.4 Toast Improvements
Move toasts to top-right, stack properly, add icons.

**Files to Modify**:
- `src/components/shared/Toast.tsx` - Add icons, improve styling
- `src/components/shared/Toast.css` - Position top-right, animations

**Improvements**:
- Position: `top: 20px; right: 20px;`
- Stack: New toasts push down existing ones
- Icons: ✓ Success, ✕ Error, ℹ Info, ⚠ Warning
- Slide-in/out animations
- Progress bar for auto-dismiss countdown

### 1.5 Panel Layout Improvements
Make panels more flexible with better collapse behavior.

**Files to Modify**:
- `src/components/Draft/DraftTab.tsx` - Add resize handles
- `src/components/Draft/DraftTab.css` - Flexbox/grid improvements
- `src/hooks/usePanelLayout.ts` - Persist panel sizes

**Features**:
- Draggable dividers between panels (min 200px each)
- Double-click divider to reset to default
- Collapsed panel shows as thin 40px strip with vertical label
- Remember sizes across sessions (localStorage)
- Responsive: Stack vertically below 900px width

### 1.6 Additional Keyboard Shortcuts
Wire up missing shortcuts from the plan.

**Shortcuts to Add**:
| Shortcut | Action | Current |
|----------|--------|---------|
| `Cmd+Shift+C` | Copy output | ❌ |
| `Cmd+F` | Focus search | ❌ |
| `Cmd+,` | Open Settings | ❌ |
| `Cmd+D` | Toggle diagnosis panel | ❌ |
| `Esc` | Close modal/clear | ❌ |

**Implementation**:
- Add global keyboard listener in App.tsx
- Use `useHotkeys` pattern or native event listener

---

## Priority 2: Diagnostic Assistant Core

**Goal**: Transform static diagnosis panel into intelligent assistant
**Estimated Effort**: 3-5 days

### 2.1 LLM-Generated Checklist
Replace static checklist with dynamic suggestions based on ticket content.

**Files to Create/Modify**:
- `src-tauri/src/kb/diagnosis.rs` - New module for diagnostic logic
- `src-tauri/src/prompts.rs` - Add diagnostic prompt templates
- `src-tauri/src/commands.rs` - Add `generate_checklist` command
- `src/components/Draft/DiagnosisPanel.tsx` - Dynamic checklist UI

**Diagnostic Prompt Template**:
```rust
const CHECKLIST_PROMPT: &str = r#"
You are an IT support diagnostic assistant. Based on the following support ticket,
generate 3-7 prioritized troubleshooting steps.

For each step, provide:
1. A clear, actionable instruction
2. What to look for (expected vs problematic results)
3. Relevance to the issue (high/medium/low)

Format as JSON:
{
  "steps": [
    {
      "instruction": "Check if user can access other internal websites",
      "look_for": "If other sites work, issue is app-specific; if all fail, network issue",
      "relevance": "high"
    }
  ]
}

Ticket content:
{ticket_text}

KB Context:
{kb_context}
"#;
```

**Checklist Item Schema**:
```typescript
interface ChecklistItem {
  id: string;
  instruction: string;
  lookFor: string;
  relevance: 'high' | 'medium' | 'low';
  checked: boolean;
  notes: string;
  checkedAt?: string;
}
```

**UI Features**:
- Relevance badges (colored: red/yellow/green)
- Expandable "What to look for" section
- Notes textarea per item
- "Regenerate" button if suggestions unhelpful
- Save checklist state with draft

### 2.2 Decision Tree Engine
Load and traverse decision trees for common issues.

**Files to Create**:
- `src-tauri/src/kb/trees.rs` - Tree loading, traversal, auto-detection
- `src-tauri/resources/trees/` - JSON tree definitions
- `src/components/Draft/DecisionTree.tsx` - Interactive tree UI

**Tree Schema**:
```json
{
  "id": "auth-failure",
  "name": "Authentication Failure",
  "category": "auth",
  "keywords": ["login", "password", "403", "unauthorized", "can't sign in", "access denied"],
  "nodes": [
    {
      "id": "start",
      "type": "question",
      "text": "Can the user reach the login page?",
      "options": [
        { "label": "Yes", "next": "page-loads" },
        { "label": "No", "next": "network-issue" }
      ]
    },
    {
      "id": "page-loads",
      "type": "question",
      "text": "Does the password field accept input?",
      "options": [
        { "label": "Yes", "next": "creds-check" },
        { "label": "No", "next": "browser-issue" }
      ]
    },
    {
      "id": "network-issue",
      "type": "resolution",
      "text": "This appears to be a network connectivity issue.",
      "actions": [
        "Check if user is connected to VPN",
        "Verify internet connectivity",
        "Try accessing other internal sites"
      ],
      "escalate_to": "Network Team"
    }
  ]
}
```

**Auto-Detection Algorithm**:
```rust
pub fn detect_relevant_tree(ticket_text: &str, trees: &[DecisionTree]) -> Option<&DecisionTree> {
    let text_lower = ticket_text.to_lowercase();

    trees.iter()
        .map(|tree| {
            let matches = tree.keywords.iter()
                .filter(|kw| text_lower.contains(*kw))
                .count();
            let score = matches as f32 / tree.keywords.len() as f32;
            (tree, score)
        })
        .filter(|(_, score)| *score > 0.3)
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .map(|(tree, _)| tree)
}
```

**UI Features**:
- Tree visualization (current node highlighted)
- Click options to navigate
- Back button to go to previous node
- "Start Over" button
- Show suggested tree with dismiss option
- Manual tree selector dropdown

### 2.3 Built-in Decision Trees (First 4)
Create the most commonly needed trees.

**Trees to Create**:

#### 2.3.1 Authentication Failure (~15 nodes)
```
Start: Can reach login page?
├─ No → Network issue branch
│   ├─ VPN connected?
│   ├─ Other sites work?
│   └─ Resolution: Network troubleshooting
└─ Yes → Auth issue branch
    ├─ Correct username format?
    ├─ Password expired?
    ├─ Account locked?
    ├─ MFA working?
    └─ Resolutions: Reset password, unlock account, re-enroll MFA
```

#### 2.3.2 VPN / Network Connectivity (~12 nodes)
```
Start: Corporate network or remote?
├─ Corporate → Internal network issue
│   ├─ Specific app or all?
│   └─ Resolution: Check firewall, contact network team
└─ Remote → VPN issue
    ├─ VPN client installed?
    ├─ Internet working without VPN?
    ├─ Can ping internal resources?
    └─ Resolution: Reinstall VPN, check credentials
```

#### 2.3.3 Email / Calendar (~15 nodes)
```
Start: Outlook desktop or web?
├─ Desktop → Client issue
│   ├─ Can send? Can receive?
│   ├─ Specific recipient or all?
│   └─ Resolution: Repair Outlook, clear cache
└─ Web → Server/account issue
    ├─ Other O365 apps working?
    └─ Resolution: Check service status, re-auth
```

#### 2.3.4 Password Reset (~8 nodes)
```
Start: Which system?
├─ Active Directory
├─ Okta
├─ App-specific
└─ Each branch: Self-service available? → IT-assisted reset
```

### 2.4 Root Cause Suggestions
LLM analyzes symptoms and suggests likely causes with confidence.

**Prompt Template**:
```rust
const ROOT_CAUSE_PROMPT: &str = r#"
Based on the support ticket and diagnostic findings, suggest the most likely root causes.

For each cause, provide:
1. Description of the cause
2. Confidence level (0-100%)
3. Evidence from the ticket supporting this cause
4. Recommended next step

Format as JSON array, max 5 causes, sorted by confidence.

Ticket: {ticket_text}
Diagnostic findings: {checklist_findings}
KB matches: {kb_context}
"#;
```

**UI**:
- Card for each suggestion
- Confidence bar (visual percentage)
- Expandable evidence section
- "Investigate" button links to relevant KB articles

### 2.5 Similar Past Tickets
Search past diagnostic sessions for similar issues.

**Implementation**:
- Store diagnostic sessions in DB (already have table)
- Embed session summary for similarity search
- Show top 3-5 matches with resolution summary

**Files to Modify**:
- `src-tauri/src/kb/diagnosis.rs` - Add session search
- `src/components/Draft/DiagnosisPanel.tsx` - Similar tickets section

**UI**:
- Collapsed section "Similar Past Tickets"
- Each card shows: Summary, resolution, similarity %, date
- Click to view full session details

### 2.6 Escalation Note Generator
Generate structured handoff notes for Tier 2/3.

**Template**:
```markdown
## Issue Summary
[Brief description from ticket]

## Environment
- User: [redacted or ID]
- System: [detected from ticket]
- First reported: [timestamp]

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

**UI**:
- "Generate Escalation Note" button in Diagnosis panel
- Modal with generated note (editable)
- Copy button
- Option to include/exclude sections

---

## Priority 3: Auto-Features

**Goal**: Reduce manual work with smart automation
**Estimated Effort**: 2-3 days

### 3.1 File Watcher for Auto-Reindex
Automatically detect KB folder changes and reindex.

**Files to Modify**:
- `src-tauri/src/kb/watcher.rs` - New file watcher module
- `src-tauri/src/lib.rs` - Start watcher on app launch
- `src-tauri/src/commands.rs` - Commands to start/stop watcher

**Implementation**:
```rust
use notify::{Watcher, RecursiveMode, watcher};
use notify_debouncer_full::{new_debouncer, DebouncedEvent};

pub struct KbWatcher {
    watcher: Option<RecommendedWatcher>,
    debouncer: Debouncer<RecommendedWatcher>,
}

impl KbWatcher {
    pub fn start(kb_folder: &Path, on_change: impl Fn(Vec<PathBuf>)) -> Result<Self> {
        let (tx, rx) = std::sync::mpsc::channel();
        let mut debouncer = new_debouncer(Duration::from_secs(2), None, tx)?;

        debouncer.watcher().watch(kb_folder, RecursiveMode::Recursive)?;

        // Spawn thread to handle events
        std::thread::spawn(move || {
            for result in rx {
                match result {
                    Ok(events) => {
                        let paths: Vec<_> = events.iter()
                            .filter_map(|e| e.path.clone())
                            .collect();
                        on_change(paths);
                    }
                    Err(e) => eprintln!("Watch error: {:?}", e),
                }
            }
        });

        Ok(Self { debouncer })
    }
}
```

**Features**:
- Debounce: Wait 2 seconds after last change before reindexing
- Ignore patterns: `.git`, `.DS_Store`, temp files
- Incremental: Only reindex changed files
- Notification: Toast when reindex completes
- Toggle: Enable/disable in Settings

### 3.2 Auto-Save Drafts
Save draft progress every 5 seconds to prevent data loss.

**Files to Modify**:
- `src-tauri/src/commands.rs` - Add `save_draft_autosave` command
- `src/hooks/useAutoSave.ts` - New hook for auto-save logic
- `src/components/Draft/DraftTab.tsx` - Integrate auto-save

**Implementation**:
```typescript
function useAutoSave(draft: Draft, interval = 5000) {
  const [lastSaved, setLastSaved] = useState<Date | null>(null);
  const [isDirty, setIsDirty] = useState(false);

  useEffect(() => {
    if (!isDirty) return;

    const timer = setTimeout(async () => {
      await invoke('save_draft_autosave', { draft });
      setLastSaved(new Date());
      setIsDirty(false);
    }, interval);

    return () => clearTimeout(timer);
  }, [draft, isDirty, interval]);

  return { lastSaved, isDirty, markDirty: () => setIsDirty(true) };
}
```

**UI**:
- Show "Saved" indicator with timestamp
- Show "Unsaved changes" when dirty
- On app crash/close: Detect unsaved draft on next launch, offer restore

### 3.3 Draft History & Management
Quick access to recent drafts with search.

**Files to Create/Modify**:
- `src-tauri/src/commands.rs` - Add `list_drafts`, `get_draft`, `delete_draft`
- `src/components/Draft/DraftHistory.tsx` - History dropdown/modal
- `src/hooks/useDrafts.ts` - Draft management hook

**UI Features**:
- Dropdown in Draft tab header showing recent 10 drafts
- Format: "Jan 24, 2:30 PM - First 30 chars of input..."
- Click to switch (auto-saves current)
- "New Draft" button clears current
- Full history modal with search
- Delete drafts (with confirmation)

### 3.4 Ticket ID Auto-Detection
Parse ticket IDs from input and display badge.

**Patterns to Detect**:
```typescript
const TICKET_PATTERNS = [
  // Jira: PROJ-1234, IT-567
  { name: 'Jira', pattern: /\b([A-Z]+-\d+)\b/g },
  // ServiceNow: INC0012345, REQ0012345
  { name: 'ServiceNow', pattern: /\b(INC|REQ|CHG|PRB)\d{7,10}\b/gi },
  // Zendesk: #12345
  { name: 'Zendesk', pattern: /#(\d{5,})\b/g },
  // Generic: Ticket #1234, Case 1234
  { name: 'Generic', pattern: /\b(?:ticket|case)\s*#?\s*(\d+)\b/gi },
];
```

**UI**:
- Badge in Input panel header showing detected ID
- Click badge to copy ID
- If Jira/ServiceNow URL configured: Click opens in browser
- Store ticket ID with draft for future reference

### 3.5 Long Input Summarization
Automatically summarize very long inputs to fit context window.

**Threshold**: ~20K characters (after system prompt + KB context)

**Implementation**:
```rust
const SUMMARIZE_PROMPT: &str = r#"
Extract the key IT issue from this support ticket. Include:
- Main problem/symptoms
- Error messages (exact text)
- User actions taken
- Affected systems/applications
- Timeline if mentioned

Be concise (under 500 words). Preserve technical details exactly.

Ticket:
{full_text}
"#;
```

**UI**:
- Warning banner when input exceeds threshold
- "Input is long. Summarizing for better results..."
- Show summary preview (expandable)
- Toggle to use original (with warning about truncation)
- Store both original and summary

---

## Priority 4: Vector Search Completion

**Goal**: Enable semantic search for better KB matches
**Estimated Effort**: 2-3 days

### 4.1 Embedding Model Loading
Wire up the embedding engine to actually load models.

**Files to Modify**:
- `src/components/Settings/SettingsTab.tsx` - Embedding model selector
- `src-tauri/src/commands.rs` - Model path persistence

**Recommended Models**:
- Primary: `nomic-embed-text-v1.5.Q5_K_M.gguf` (~550MB, 768 dims)
- Fallback: `bge-small-en-v1.5.Q8_0.gguf` (~130MB, 384 dims)

**UI**:
- Separate section in Settings for embedding model
- Download or import option
- Test button to verify embeddings work

### 4.2 Batch Embed Chunks on Index
Generate embeddings for all chunks during indexing.

**Files to Modify**:
- `src-tauri/src/kb/indexer.rs` - Add embedding step
- `src-tauri/src/kb/vectors.rs` - Batch insert embeddings

**Implementation**:
```rust
pub async fn index_with_embeddings(
    db: &Database,
    embedding_engine: &EmbeddingEngine,
    chunks: Vec<Chunk>,
) -> Result<()> {
    // Batch embed (50 at a time for efficiency)
    for batch in chunks.chunks(50) {
        let texts: Vec<_> = batch.iter().map(|c| c.content.as_str()).collect();
        let embeddings = embedding_engine.batch_generate(&texts).await?;

        // Store in LanceDB
        vectors::insert_batch(batch, embeddings).await?;
    }
    Ok(())
}
```

**Progress**:
- Show embedding progress during indexing
- "Indexing: 150/500 chunks (embedding...)"

### 4.3 Enable Hybrid Search RRF
Combine FTS5 and vector results with Reciprocal Rank Fusion.

**Files to Modify**:
- `src-tauri/src/kb/search.rs` - Implement full hybrid search

**RRF Formula**:
```rust
fn rrf_score(rank: usize, k: f32) -> f32 {
    1.0 / (k + rank as f32)
}

fn rrf_merge(
    fts_results: Vec<SearchResult>,
    vec_results: Vec<SearchResult>,
    k: f32,  // typically 60
) -> Vec<SearchResult> {
    let mut scores: HashMap<String, f32> = HashMap::new();

    for (rank, result) in fts_results.iter().enumerate() {
        *scores.entry(result.chunk_id.clone()).or_default() += rrf_score(rank, k);
    }
    for (rank, result) in vec_results.iter().enumerate() {
        *scores.entry(result.chunk_id.clone()).or_default() += rrf_score(rank, k);
    }

    // Sort by combined score, return top N
    // ...
}
```

### 4.4 User Consent Flow for Unencrypted Vectors
LanceDB may not support encryption. Require explicit opt-in.

**UI Flow**:
1. First time enabling vector search
2. Show dialog explaining vectors are unencrypted
3. Checkbox: "I understand and accept"
4. Store consent in DB (`vector_consent` table)
5. Without consent, vector search stays disabled (FTS5 only)

---

## Priority 5: Polish & Distribution

**Goal**: Production-ready application
**Estimated Effort**: 3-5 days

### 5.1 First-Run Wizard
Guided setup on first launch.

**Steps**:
1. **Welcome** - Brief intro, what the app does
2. **Generation Model** - Download or import GGUF
3. **Test Generation** - Run "Say hello" test
4. **Embedding Model** (optional) - For KB search
5. **KB Folder** (optional) - Point to runbooks
6. **Ready!** - Summary, "Get Started" button

**Files to Create**:
- `src/components/Wizard/SetupWizard.tsx`
- `src/components/Wizard/WizardStep.tsx`
- `src/components/Wizard/ModelStep.tsx`
- `src/components/Wizard/KbStep.tsx`

**Persistence**:
- Store `setup_completed` flag in DB
- Allow re-running wizard from Settings

### 5.2 Export/Import
Full data portability with encrypted backups.

**Export Format** (`.assbackup`):
```
assistsupport-backup-2024-01-24.assbackup
├── manifest.json (version, date, counts)
├── drafts.json (all drafts)
├── sessions.json (diagnostic sessions)
├── trees.json (custom decision trees)
├── settings.json (app settings)
└── attachments/ (encrypted files)
```

**Encryption**:
- User sets export password
- Argon2id key derivation
- AES-256-GCM encryption of payload
- Header: version, salt, nonce, kdf params

**Import Options**:
- Merge with existing data
- Replace all data
- Preview before import

### 5.3 DMG Packaging
Create distributable macOS app.

**Tauri Build**:
```bash
pnpm tauri build
# Creates .dmg in src-tauri/target/release/bundle/dmg/
```

**DMG Contents**:
- AssistSupport.app
- Link to /Applications
- README.txt (setup instructions)

**Size Targets**:
- Lean DMG: ~50-100MB (models downloaded separately)
- Full DMG: ~10GB (includes recommended models)

### 5.4 Error Handling & Recovery
Robust error handling throughout.

**Improvements**:
- Global error boundary in React
- Crash recovery: Detect unclean shutdown, offer restore
- Model load failures: Clear error message, suggest alternatives
- DB corruption: Offer restore from backup
- Network failures: Graceful degradation

### 5.5 Logging & Diagnostics
Debug mode and log files.

**Implementation**:
- Rotating log files: `~/Library/Logs/AssistSupport/`
- Log levels: ERROR, WARN, INFO, DEBUG
- Diagnostics toggle in Settings → Advanced
- Show: generation time, tokens/sec, memory usage

---

## Priority 6: Web Integrations

**Goal**: Connect to external services for enhanced diagnostics
**Estimated Effort**: 2-3 days

### 6.1 Jira/ServiceNow URL Configuration
Link ticket IDs to external systems.

**Settings UI**:
- Jira Base URL: `https://company.atlassian.net`
- ServiceNow Instance: `company.service-now.com`
- Auto-construct URLs: `{base}/browse/{ticket_id}`

**Features**:
- Detected ticket ID becomes clickable link
- Opens in default browser
- Optional: Fetch ticket details via API (future)

### 6.2 Web Search for Outages
Check external sources for service outages.

**Sources**:
1. **Down Detector** - HTML scrape
2. **Official Status Pages** - JSON/RSS
   - status.microsoft.com
   - status.okta.com
   - status.zoom.us
   - status.slack.com
   - etc.
3. **Reddit r/sysadmin** - JSON API (free)
4. **Google News RSS** - For `{service} outage`

**Implementation**:
```rust
pub async fn check_outages(services: Vec<&str>) -> Vec<OutageResult> {
    let futures = services.iter().map(|service| async {
        let dd = scrape_downdetector(service).await;
        let status = fetch_status_page(service).await;
        let reddit = search_reddit(service).await;
        OutageResult {
            service: service.to_string(),
            down_detector: dd,
            status_page: status,
            reddit_reports: reddit,
        }
    });
    join_all(futures).await
}
```

**Rate Limiting**:
- Down Detector: 1 req/service/5 min
- Status pages: 1 req/page/1 min
- Reddit: 1 req/10 sec

**PII Redaction**:
- Remove names, emails, ticket IDs from queries
- Service names only

**UI**:
- "Check for Outages" button in Diagnosis panel
- Disabled until enabled in Settings
- Results: Green (ok), Yellow (reports), Red (outage)

### 6.3 Convert & Import
Import documents from various formats.

**Supported Formats**:
| Format | Library | Notes |
|--------|---------|-------|
| Confluence HTML | `scraper` | Export → import |
| Excel/CSV | `calamine` | Rows → tables |
| Word (.docx) | `zip` + `quick-xml` | Extract text |
| HTML pages | `scraper` | Clean article content |

**UI**:
- "Import Files" button in Settings
- Multi-select file picker
- Progress bar for batch conversion
- Converted to Markdown, saved to KB folder

---

## Priority 7: Advanced Features

**Goal**: Differentiating capabilities
**Estimated Effort**: 3-5 days

### 7.1 Learning System
Track checklist effectiveness to improve suggestions.

**Data Tracked** (with consent):
- Checklist items: shown, checked, led to resolution
- Decision tree paths: visited, led to resolution
- Time spent per item

**Ranking Formula**:
```sql
score = (times_checked / times_shown) * (1 + resolution_bonus)
-- Decay: multiply by 0.9 per week
```

**UI**:
- Opt-in toggle in Settings → Advanced
- Items with higher scores shown first
- "Users typically check this first" hints

### 7.2 Remaining Decision Trees (4 more)
Complete the set of 8 built-in trees.

**Trees to Create**:
1. **SSO / Single Sign-On** (~12 nodes)
2. **Hardware** (~18 nodes)
3. **Software Installation** (~10 nodes)
4. **Account Provisioning** (~10 nodes)

### 7.3 Custom Decision Trees
Allow users to create their own trees.

**UI**:
- JSON editor in Settings → Advanced
- Visual tree builder (future)
- Import/export trees

### 7.4 Response Variants
Generate multiple response options.

**Implementation**:
- "Generate 3 variants" option
- Different tones: Formal, Friendly, Technical
- User picks preferred, others discarded

### 7.5 Quick Capture Mode
Minimal UI via global hotkey.

**Features**:
- Global hotkey (e.g., Cmd+Shift+A) opens mini window
- Paste ticket, get response
- Copy and close
- No distractions

---

## Summary: Implementation Order

| Phase | Priority | Effort | Key Deliverables |
|-------|----------|--------|------------------|
| 1 | Core UX | 1-2 days | Streaming, themes, loading states, shortcuts |
| 2 | Diagnostic Core | 3-5 days | Dynamic checklist, 4 trees, root causes |
| 3 | Auto-Features | 2-3 days | File watcher, auto-save, draft history |
| 4 | Vector Search | 2-3 days | Embeddings working, hybrid search |
| 5 | Polish | 3-5 days | Wizard, export/import, DMG |
| 6 | Integrations | 2-3 days | Jira links, outage checks, imports |
| 7 | Advanced | 3-5 days | Learning, more trees, variants |

**Total Estimated Effort**: 16-26 days (3-5 weeks)

---

## Verification Checklist

### Core UX
- [ ] Generation shows tokens word-by-word
- [ ] Theme toggle works (light/dark/system)
- [ ] Loading skeletons appear during operations
- [ ] Toasts appear top-right, stack properly
- [ ] Panel dividers draggable, sizes persist
- [ ] All keyboard shortcuts functional

### Diagnostic Assistant
- [ ] Checklist generated dynamically from ticket
- [ ] Decision trees load and navigate correctly
- [ ] Auto-detection suggests relevant tree
- [ ] Root cause suggestions appear with confidence
- [ ] Similar tickets found and displayed
- [ ] Escalation notes generate correctly

### Auto-Features
- [ ] File watcher detects KB changes
- [ ] Auto-save every 5 seconds
- [ ] Draft history dropdown works
- [ ] Ticket IDs detected and badged
- [ ] Long inputs summarized automatically

### Vector Search
- [ ] Embedding model loads successfully
- [ ] Chunks embedded during indexing
- [ ] Hybrid search returns better results
- [ ] Consent flow for unencrypted vectors

### Polish
- [ ] First-run wizard completes setup
- [ ] Export creates valid backup file
- [ ] Import restores data correctly
- [ ] DMG installs and runs on fresh Mac
- [ ] Logs written to correct location

### Integrations
- [ ] Ticket links open in browser
- [ ] Outage check returns results
- [ ] Document import converts correctly
