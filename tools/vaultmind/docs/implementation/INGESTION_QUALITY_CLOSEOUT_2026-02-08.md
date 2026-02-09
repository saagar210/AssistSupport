# Ingestion Quality Closeout — 2026-02-08

## Commit

`8d737d3` — `feat(kb): strip confluence boilerplate and fix section mapping`
Branch `codex/ingestion-quality` merged into `main` via fast-forward.

## What Changed and Why

Two bugs in the knowledge-ingestion pipeline caused confidently-wrong RAG answers when processing Confluence-exported documentation:

### 1. Duplicate Section Title Mis-attribution (chunker.rs)

**Problem:** `build_section_map()` used `text.find(&section.title)` which always returned the *first* occurrence of a title string. When a document contained two sections with the same heading (e.g., two "Overview" sections for different products), both mapped to the same byte offset. Chunks from the second section were attributed to the first section, causing the `[Section: ...]` prefix to point to the wrong context.

**Fix:** Track used byte positions via `HashSet<usize>`. For each section, scan forward past already-claimed positions to find the next unused occurrence. Each section maps to a unique text position.

**Impact:** Eliminates wrong-section citations in documents with repeated headings — common in Confluence exports (e.g., per-product "Overview" / "Requirements" / "FAQ" sections).

### 2. Confluence Boilerplate Polluting Chunks (markdown.rs, html.rs)

**Problem:** Confluence exports include navigation chrome, metadata, and macro markers that aren't document content. These boilerplate fragments land in chunks and dilute retrieval quality — the vector search matches on "Powered by Confluence" or breadcrumb text instead of actual knowledge.

**Fix — Markdown parser:** Added `strip_confluence_boilerplate()` that runs before pulldown-cmark parsing. Strips:
- YAML frontmatter (`---` delimited blocks at file start)
- Breadcrumb lines (3+ segments separated by ` > ` or ` / `)
- Metadata lines ("Created by ...", "Last modified by ...", "Labels: ...")
- Confluence macros (`{toc}`, `{children}`, `{excerpt}`, `{info}`, `{note}`, `{warning}`, `{tip}`, `{panel}`, `{expand}`, `{status}`, `{recently-updated}`, `{page-tree}` — with optional `:param=value` syntax)
- "Powered by Confluence/Atlassian" footer lines

**Fix — HTML parser:** Added 15 CSS selectors to the existing `skip_ids` mechanism:
- Semantic elements: `nav`, `footer`, `aside`
- Class selectors: `.breadcrumb`, `.breadcrumbs`, `.page-metadata`, `.page-metadata-modification-info`, `.navigation`, `.nav-breadcrumb`, `.confluence-information-macro`, `.footer-body`, `.page-restrictions`
- ID selectors: `#footer`, `#breadcrumbs`, `#navigation`

**Impact:** Cleaner chunks = higher retrieval precision for Confluence-sourced knowledge bases.

## Files Touched

| File | Lines Changed | Change |
|------|--------------|--------|
| `src-tauri/src/chunker.rs` | +60 | `build_section_map()` rewritten + 2 tests |
| `src-tauri/src/parsers/markdown.rs` | +199 | `strip_confluence_boilerplate()` + 5 helpers + 10 tests |
| `src-tauri/src/parsers/html.rs` | +105 | 15 boilerplate selectors + 6 tests |

No database schema migrations. No public API signature changes.

## Verification Results

All checks run on `main` at commit `8d737d3`:

| Check | Command | Result |
|-------|---------|--------|
| Rust tests | `cargo test` | **178 passed**, 0 failed |
| Clippy | `cargo clippy -- -D warnings` | **0 warnings** |
| Frontend tests | `pnpm test` | **56 passed**, 0 failed |
| TypeScript lint | `pnpm lint` | **Clean** (tsc --noEmit) |

### New Tests Added (18 total)

**Chunker (2):**
- `test_duplicate_section_titles_assigned_correctly` — verifies two "Overview" sections map to different byte positions
- `test_section_map_handles_missing_title` — verifies graceful handling of sections not found in text

**Markdown boilerplate (10):**
- `test_strip_confluence_boilerplate_frontmatter` — YAML frontmatter removed
- `test_strip_confluence_boilerplate_breadcrumbs` — breadcrumb lines removed
- `test_strip_confluence_boilerplate_metadata_lines` — Created/Modified/Labels removed
- `test_strip_confluence_boilerplate_macros` — {toc}, {children} etc. removed
- `test_strip_confluence_boilerplate_powered_by` — footer removed
- `test_strip_confluence_boilerplate_preserves_normal_content` — no false positives
- `test_strip_confluence_boilerplate_unclosed_frontmatter` — unclosed `---` returns original
- `test_strip_confluence_macro_with_params` — `{toc:maxLevel=3}` style removed
- `test_breadcrumb_detection_requires_three_segments` — 2-segment lines not stripped
- `test_breadcrumb_not_triggered_by_prose` — sentences with periods not stripped

**HTML boilerplate (6):**
- `test_html_strips_nav_element`
- `test_html_strips_footer_element`
- `test_html_strips_breadcrumb_class`
- `test_html_strips_page_metadata_class`
- `test_html_strips_aside_element`
- `test_html_preserves_normal_content`

## Test Count Summary

| Suite | Before | After | Delta |
|-------|--------|-------|-------|
| Rust | 160 | 178 | +18 |
| Frontend | 56 | 56 | 0 |
| **Total** | **216** | **234** | **+18** |

## Remaining Ingestion-Quality Follow-ups

These are deferred to post-UX-revamp. None are blocking.

| # | Change | Complexity | Blocker? |
|---|--------|-----------|----------|
| 1 | Stable content-hash chunk IDs | Medium — requires DB migration (chunk.id from UUID to SHA-256 of content+position) | No |
| 3 | Hierarchical section context (`Parent > Child` prefix instead of leaf-only) | Low — modify `find_section_at()` to walk section tree | No |
| 4 | Preserve source URL from Confluence metadata | Low — add `source_url` column to documents table (migration) | No |
| 6 | Cross-encoder reranking | Medium — add reranker model call after initial retrieval | No |

## Risks

None identified. Changes are additive (new stripping + deterministic mapping). No regressions in existing test suite. No schema changes. No dependency additions.
