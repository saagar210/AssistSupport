# Full-Scale Code Review & Audit Prompt for Codex

Copy and paste the following prompt into your Codex chat:

---

## TASK: Full-Scale Code Review & Security Audit for AssistSupport v0.3.0

### PROJECT CONTEXT
I have a production-ready desktop application called **AssistSupport** - a local-first AI-powered customer support response generator built with:
- **Frontend**: React 19 + TypeScript (strict mode) + Vite
- **Backend**: Rust + Tauri 2.x
- **Database**: SQLCipher (AES-256 encryption) + FTS5
- **Vector Store**: LanceDB for hybrid semantic search
- **LLM Runtime**: llama.cpp via llama-cpp-2 bindings

**Current Status**: v0.3.0 - All features complete, security hardened, 225 backend tests + 72 frontend tests passing.

### DOCUMENTATION (Start here - read in this order)
1. **`.claude/CLAUDE.md`** - Quick project context and architecture overview
2. **`docs/ARCHITECTURE.md`** - Full system design, directory structure, data flows, API patterns, extension points
3. **`docs/SECURITY.md`** - Encryption, key management, threat model, security boundaries
4. **`docs/PERFORMANCE.md`** - Baseline metrics, tuning, model selection
5. **`docs/INSTALLATION.md`** - Setup and configuration
6. **`docs/OPERATIONS.md`** - User guide for daily workflows
7. **`README.md`** - Project overview, features, test coverage

### CRITICAL FILES TO AUDIT
**Security & Encryption**:
- `src-tauri/src/security.rs` - Master key, encryption, key derivation
- `src-tauri/src/audit.rs` - Security event logging
- `src-tauri/src/kb/network.rs` - SSRF protection

**Data Layer**:
- `src-tauri/src/db/mod.rs` - SQLCipher database, all queries
- `src-tauri/src/db/executor.rs` - Async query executor

**Knowledge Base**:
- `src-tauri/src/kb/indexer.rs` - File indexing, chunking
- `src-tauri/src/kb/search.rs` - Hybrid FTS + vector search
- `src-tauri/src/kb/embeddings.rs` - Embedding model
- `src-tauri/src/kb/vectors.rs` - LanceDB vector store
- `src-tauri/src/kb/ingest/web.rs` - Web ingestion with SSRF protection
- `src-tauri/src/kb/ingest/youtube.rs` - YouTube transcript extraction
- `src-tauri/src/kb/ingest/github.rs` - GitHub repo indexing

**LLM & Generation**:
- `src-tauri/src/llm.rs` - LLM engine, streaming generation
- `src-tauri/src/commands/mod.rs` - ~200 Tauri command endpoints

**Input Validation**:
- `src-tauri/src/validation.rs` - Path validation, URL validation, size limits

**Frontend State**:
- `src/contexts/AppStatusContext.tsx` - Centralized app state management
- `src/components/Draft/` - Response generation workflow
- `src/components/Sources/` - KB search and content ingestion UI
- `src/components/Settings/` - Configuration and integration setup

### TEST SUITE (All Passing)
**Backend Tests**: `src-tauri/tests/`
- `kb_pipeline.rs` - 20 E2E tests (file ingestion → indexing → search → generation)
- `security.rs` - 36 security tests (encryption, key wrapping, SSRF, path traversal)
- `path_validation.rs` - 18 path validation tests
- Unit tests: 151 embedded in source files

**Frontend Tests**: `src/**/*.test.{ts,tsx}` - 72 tests

**Benchmarks**: `src-tauri/benches/performance.rs` - Criterion benchmarks for encryption, key derivation, FTS, database ops

### RECENT COMMITS (Context on what changed)
```
11fc05a - chore: Clean up outdated files and update documentation
52bc3e2 - feat: Complete security hardening and testing phases (v0.3.0)
92dbb59 - docs: Update documentation for completed modernization
73fc7a4 - feat: Complete modernization phases 5-10
c694c44 - chore: Clean up outdated files and update documentation
de8fbff - docs: Mark IMPLEMENTATION_PLAN.md as complete with summary
29919e5 - test: Add comprehensive E2E workflow integration tests
6241576 - feat(security): Restrict file paths to home directory only
bb5f0af - feat(security): Update commands to use FileKeyStore
8ed8339 - feat(security): Replace Keychain with file-based credential storage
```

### AUDIT SCOPE & REQUIREMENTS

#### 1. SECURITY AUDIT
- [ ] Review all cryptographic operations (AES-256-GCM, Argon2id)
- [ ] Verify key management (generation, storage, rotation, zeroization)
- [ ] Check SSRF protection completeness (IP blocking, DNS rebinding, scheme validation)
- [ ] Validate path traversal protections
- [ ] Review input validation across all command endpoints
- [ ] Check sensitive data handling (tokens, credentials, passwords)
- [ ] Verify encryption-at-rest for database and token storage
- [ ] Review audit logging completeness
- [ ] Check error handling for information disclosure

#### 2. CODE QUALITY AUDIT
- [ ] TypeScript: Strict mode adherence, type correctness
- [ ] Rust: Memory safety, error handling (Result<T, E> patterns)
- [ ] Code duplication, maintainability
- [ ] API consistency across 200+ command endpoints
- [ ] Frontend/backend communication patterns
- [ ] State management correctness

#### 3. ARCHITECTURE AUDIT
- [ ] Tauri 2.x integration correctness
- [ ] React 19 hooks and context usage
- [ ] Database connection pooling and lifecycle
- [ ] Background job management
- [ ] Error propagation chains
- [ ] Feature flag implementation

#### 4. PERFORMANCE AUDIT
- [ ] LLM streaming implementation
- [ ] Vector search efficiency (batch embeddings, indexing)
- [ ] FTS5 query optimization
- [ ] Memory management (model loading, context windows)
- [ ] Database query performance
- [ ] Frontend rendering optimization

#### 5. TESTING AUDIT
- [ ] Coverage gaps in current 225 backend + 72 frontend tests
- [ ] Test quality and assertions
- [ ] E2E workflow coverage
- [ ] Security test effectiveness
- [ ] Edge cases not covered

#### 6. DOCUMENTATION AUDIT
- [ ] Accuracy of architecture documentation
- [ ] Security documentation completeness
- [ ] API documentation for developers
- [ ] Installation/setup accuracy
- [ ] Operations guide usefulness

### DELIVERABLES EXPECTED
1. **Security Issues Summary**: Critical/High/Medium/Low issues with remediation priority
2. **Code Quality Report**: Style, patterns, maintainability recommendations
3. **Architecture Assessment**: Strengths, weaknesses, scalability concerns
4. **Test Coverage Analysis**: Gaps and recommended additional tests
5. **Performance Observations**: Bottlenecks, optimization opportunities
6. **Risk Assessment**: Overall security posture, residual risks
7. **Recommendations**: Top 5-10 actionable improvements with effort/impact analysis

### INSTRUCTIONS FOR CODEX
- **Be thorough**: This is a security-critical application handling encryption and sensitive data
- **Be specific**: Quote code locations (filename:line_number) for all issues
- **Be constructive**: Provide remediation paths, not just problem identification
- **Prioritize**: Focus on security first, then code quality, then performance
- **Flag assumptions**: Call out any areas where you need clarification
- **Use evidence**: Base findings on code review, not speculation

### START AUDIT NOW
Begin by reading the documentation files in the order listed above, then systematically review the source code in the order: security → data layer → knowledge base → LLM → frontend. Use the provided test suite as a guide to understand expected behavior.

---

## NOTES FOR YOU (Not for Codex)

**Prompting Best Practices Applied**:
1. ✅ **Clear Role Definition** - "Full-Scale Code Review & Security Audit"
2. ✅ **Comprehensive Context** - Project overview, tech stack, current state
3. ✅ **Structured Scope** - Specific audit categories (Security, Quality, Architecture, etc.)
4. ✅ **File Roadmap** - Clear path to documentation and source files
5. ✅ **Explicit Success Criteria** - Specific deliverables listed
6. ✅ **Quality Instructions** - "Be thorough/specific/constructive/evidence-based"
7. ✅ **Test References** - Points to 225 backend + 72 frontend tests
8. ✅ **Commit History** - Context on what changed recently
9. ✅ **Audit Checklist** - Checkboxes for tracking progress
10. ✅ **Prioritization** - Security > Quality > Performance
