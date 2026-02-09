# Monorepo Integration Readiness — VaultMind

## Summary

VaultMind is ready for monorepo integration with AssistSupport at the **security boundary level**. All P1 hardening fixes have been implemented. One major gap remains: SQLCipher at-rest encryption (design complete, implementation pending).

## What VaultMind Provides to AssistSupport

VaultMind serves as the **Knowledge Base Preparation + Quality Engine**, outputting structured Knowledge Packs:
- 7-format document ingestion (PDF, DOCX, EPUB, Markdown, HTML, CSV, TXT)
- Chunking with configurable size/overlap
- Vector embeddings via Ollama
- Named entity extraction (NER)
- Relationship detection between entities
- Knowledge graph with community detection
- Hybrid search (vector + BM25 + RRF fusion)
- RAG with multi-hop retrieval and precise citations

## Security Posture (Post-Hardening)

| Control | Status | Details |
|---------|--------|---------|
| Path traversal prevention | DONE | `validation::validate_file_path()` — canonicalize, reject symlinks, restrict to home dir |
| SSRF protection | DONE | `validation::validate_ollama_host()` — localhost-only Ollama connections |
| Settings injection | DONE | `validation::validate_setting()` — per-key validation rules |
| Model name injection | DONE | `validation::validate_model_name()` — whitelist pattern |
| ZIP bomb defense | DONE | `validation::validate_zip_archive()` — entry count, per-entry size, total size limits |
| EPUB path traversal | DONE | URL-decode + component-level ".." check (replaces bypassable `.contains("..")`) |
| FTS5 injection | DONE | `sanitize_fts_query()` — wraps each word in double quotes |
| Pagination caps | DONE | All list endpoints capped at 500 items per page |
| Audit logging | DONE | All CRUD operations logged to audit_log table |
| GDPR compliance | DONE | Export, erasure (document/collection/all), retention policies, consent records |
| Secure delete | DONE | `PRAGMA secure_delete = ON` on all connections |
| DB integrity check | DONE | `PRAGMA integrity_check` + foreign key validation on demand |
| Encryption at rest | PENDING | SQLCipher design complete (see SQLCIPHER_DESIGN.md), implementation blocked on testing |

## Shared Crate Opportunities

If moving to a monorepo workspace, these VaultMind modules could become shared crates:

1. **`vaultmind-validation`** — path validation, SSRF protection, ZIP safety (reusable by AssistSupport)
2. **`vaultmind-parsers`** — document ingestion for all 7 formats
3. **`vaultmind-embedder`** — Ollama embedding client with batch support
4. **`vaultmind-chunker`** — configurable text chunking
5. **`vaultmind-ner`** — LLM-based named entity extraction

## What VaultMind Needs from AssistSupport

1. **Knowledge Pack format spec** — JSON schema for the interchange format between VaultMind exports and AssistSupport imports
2. **SQLCipher build recipe** — AssistSupport's proven `bundled-sqlcipher` configuration and CI setup
3. **Cross-encoder model name** — Which reranking model AssistSupport uses, so VaultMind can pre-rank chunks

## Remaining Blockers

1. **SQLCipher implementation** — Design complete, needs 2-3 days of implementation
2. **Knowledge Pack interchange format** — Needs joint design session
3. **Shared CI/CD pipeline** — Needs workspace-level Cargo.toml and unified test runner

## GO/NO-GO Assessment

**GO** — with conditions:
- VaultMind's security posture is production-ready for all non-encryption concerns
- SQLCipher can be implemented independently without blocking monorepo work
- No cross-repo changes are needed; integration happens via the Knowledge Pack API boundary
