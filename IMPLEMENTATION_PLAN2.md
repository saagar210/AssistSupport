# AssistSupport Next-Gen Modernization - Implementation Plan 2

**Document Version**: 1.0
**Created**: 2026-01-25
**Status**: Proposed
**Classification**: Internal - Engineering

---

## Executive Summary

This plan defines the next 10 phases to modernize AssistSupport into a sharper, faster, and more premium product. The focus is a high-impact UI overhaul, tighter workflows, faster performance, stronger security, and higher quality-of-life. Each phase has concrete deliverables, acceptance criteria, and confidence gates.

---

## North Star Goals

- **Modern, premium UI**: Clear visual direction, intentional typography, and expressive layout. No generic or "AI slop" UI.
- **Faster and lighter**: Snappy launch, smooth navigation, and low memory usage under heavy KBs.
- **Higher trust**: Strong security posture, transparent data handling, and predictable reliability.
- **Delightful workflow**: Fewer clicks, better defaults, and powerful keyboard-first actions.

---

## Guiding Principles

- **Local-first** by default; network only with explicit user action.
- **Clarity over cleverness**; UI is minimal, purposeful, and consistent.
- **No regressions**; every phase has tests and performance baselines.
- **Ship in slices**; each phase delivers visible value.

---

## Phase 1 - Visual System and UX Foundations

**Goal**: Define the modern visual language and interaction baseline.

**Key Work**
- Build a **design system** (tokens, spacing, typography, color, motion).
- Select non-default fonts (example: "Space Grotesk" for headings, "IBM Plex Sans" for body).
- Create new layout grids, card styles, and input patterns.
- Define accessibility baseline (color contrast, focus, keyboard use).

**Deliverables**
- `design-tokens.css` or theme token file.
- Component style guide (buttons, inputs, panels, modals, tables).
- UX patterns for loading, empty states, errors.

**Acceptance Criteria**
- Token-driven UI across at least 3 core screens.
- WCAG AA contrast for text and interactive components.

**Risks / Dependencies**
- Requires design decisions before refactors.

---

## Phase 2 - Core UI Overhaul (App Shell, Navigation, Composition)

**Goal**: Replace current layout with a bold, modern, cohesive UI shell.

**Key Work**
- New app shell: persistent sidebar, contextual header, command bar.
- Redesign the response composition workspace (input, context, output).
- Improve visual hierarchy and spacing.

**Deliverables**
- Updated `App.tsx` layout and core screens.
- A redesigned composer view with a modern 2- or 3-pane layout.

**Acceptance Criteria**
- All primary flows available with one main UI style.
- Smooth navigation between tabs with no layout shifts.

**Risks / Dependencies**
- Requires Phase 1 tokens and components.

---

## Phase 3 - Workflow Optimization and Quality-of-Life

**Goal**: Reduce friction and make the product feel fast and intentional.

**Key Work**
- Command palette for quick actions.
- Global keyboard shortcuts and discoverability.
- Draft lifecycle improvements (autosave, history timeline, quick restore).
- "One-click" actions (copy, export, insert, rerun).

**Deliverables**
- Command palette UI and backend actions.
- Shortcut map and in-app help.

**Acceptance Criteria**
- Common tasks <= 2 steps.
- Keyboard-only workflow for primary actions.

---

## Phase 4 - Performance and Responsiveness

**Goal**: Make large KBs feel fast and the UI feel instant.

**Key Work**
- Startup and cold-launch profiling with budgets.
- DB query optimization and caching for hot paths.
- Defer expensive work to background tasks.
- UI virtualization for large lists and tables.

**Deliverables**
- Performance budget doc and regression tests.
- DB and UI profiling reports.

**Acceptance Criteria**
- Cold launch < 2 seconds on baseline hardware.
- Scrolling of KB lists remains smooth at 5,000+ items.

---

## Phase 5 - Search and Retrieval Quality

**Goal**: Make results smarter, more precise, and more readable.

**Key Work**
- Better hybrid ranking (BM25 tuning + semantic weights).
- Deduping similar chunks and improving snippet selection.
- Adjustable relevance controls in UI.
- Faster context building with preview modes.

**Deliverables**
- Search quality tuning config.
- Improved results UI (highlighting, source grouping, filters).

**Acceptance Criteria**
- Top-5 results quality improved in evaluation benchmarks.
- Search response < 300ms for common queries.

---

## Phase 6 - Ingestion and Source Lifecycle

**Goal**: Make ingestion robust, safe, and low-touch.

**Key Work**
- Scheduled re-ingestion with diffing and content hashing.
- Improved failure UX and retry controls.
- Source health dashboard (status, last sync, errors).
- Support more file types and better chunking for code.

**Deliverables**
- Source lifecycle UI and status views.
- Ingest error classification and retry strategy.

**Acceptance Criteria**
- Automatic refresh on schedule with minimal user input.
- Clear error reporting and recovery steps.

---

## Phase 7 - LLM Pipeline and Response Quality

**Goal**: Sharper output, lower latency, and better control.

**Key Work**
- Prompt templates with versioning and A/B testing.
- Response evaluation framework (golden set tests).
- Streaming optimization and token budgeting.
- Context distillation (summaries + citations).

**Deliverables**
- Prompt library with clear ownership and versioning.
- Response quality benchmarks.

**Acceptance Criteria**
- Lower hallucination rate in internal tests.
- Fast streaming with early partial results.

---

## Phase 8 - Security and Privacy Hardening

**Goal**: Raise trust and reduce attack surface.

**Key Work**
- Encrypted token storage using master key (already implemented; now validate migration in tests).
- Model integrity checks (SHA256 verification and allowlist).
- Strict network controls with opt-in policies.
- Security review checklist and periodic dependency audit.

**Deliverables**
- Security checklist and threat model doc.
- Model verification workflow.

**Acceptance Criteria**
- Zero critical findings in internal security scan.
- Verified model integrity for bundled downloads.

---

## Phase 9 - Reliability, Observability, and Self-Healing

**Goal**: Detect and recover from errors without user pain.

**Key Work**
- Structured logging with redaction rules.
- Health checks for DB, vector store, and model runtime.
- Recovery workflows (safe mode, repair tools).

**Deliverables**
- Local diagnostics panel.
- Auto-repair options for DB issues.

**Acceptance Criteria**
- Clear, actionable error messages.
- Recovery workflows for the top 5 failure modes.

---

## Phase 10 - Release Readiness and Polished Experience

**Goal**: Make it production-grade and ready to ship.

**Key Work**
- Onboarding flow and sample KB for first-run delight.
- Installer polish, packaging, and update strategy.
- Documentation refresh and support playbook.

**Deliverables**
- New onboarding UX with guided steps.
- Release checklist with quality gates.

**Acceptance Criteria**
- First-run completion < 5 minutes.
- No critical or high bugs at release gate.

---

## Quality Gates (Applies to Every Phase)

- Test suite green (frontend + backend).
- Performance budgets met.
- No new critical security findings.
- Visual QA checklist complete.

---

## Suggested Immediate Next Steps

1. Align on **visual direction** (color, type, motion, layout) and approve a design system direction.
2. Confirm performance budgets and baseline hardware.
3. Select the exact Phase 2 screens for redesign kickoff (recommend: Composer, KB Browser, Settings).

---

## Open Questions

- Do we want a single bold theme or multiple user-selectable themes?
- Should we prioritize command palette and keyboard workflows before the full redesign?
- What is the target baseline hardware for performance budgets?
