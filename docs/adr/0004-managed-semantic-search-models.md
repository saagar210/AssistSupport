# 0004: Managed Semantic Search Models

- Status: Accepted
- Date: 2026-03-13

## Context

AssistSupport uses two separate semantic-search runtimes:

1. The desktop app uses a local GGUF embedding model for the knowledge base.
2. The Python search API uses a SentenceTransformer model for hybrid search.

Before this change, the Python side fetched its embedding model implicitly at runtime. That created a supply-chain blind spot because the desktop app exposed one explicit model-management flow while the search API used a second, unmanaged one. The product also allowed custom GGUF files to load even when they were not allowlisted, which weakened the trust boundary around native model parsing.

## Decision

- Keep the two embedding engines separate.
- Manage both explicitly through the app.
- Pin the search API embedding model to a fixed Hugging Face revision and install it into the app data directory with a local manifest containing file hashes.
- Load the search API embedding model from local disk only at runtime.
- Block unverified custom GGUF files by default.
- Allow unverified custom GGUF files only through an explicit advanced override with operator confirmation and audit logging.
- Keep full semantic-search model setup in Settings, while onboarding only points users there.

## Consequences

### Positive

- The search API no longer performs hidden runtime model downloads.
- Operators have one visible place to manage both semantic-search model paths.
- Custom-model loading now fails closed by default instead of silently accepting unverified files.
- Release metadata and UI state are easier to audit because model readiness is explicit.

### Tradeoffs

- The Python search API still depends on a Python-managed model format instead of sharing the desktop GGUF embedding runtime.
- Advanced users who load local custom models now need to make an intentional override choice.
- Search API model installation depends on Python availability in environments that use the local search sidecar.

## Follow-up

- Consider signing or attesting model artifacts in addition to hash verification.
- Evaluate whether the optional reranker should be promoted into the same managed-install flow.
