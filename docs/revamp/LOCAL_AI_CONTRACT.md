# Local AI Contract (Phase 4)

Status: Active (pre-UX revamp)  
Audience: Internal IT support engineers  
Purpose: Make local AI **predictable, explainable, and safe-by-default**.

## What "Local AI" Means Here

1. Responses are generated on-device using a local GGUF model (llama.cpp).
2. Knowledge Base context is loaded from local files (no cloud retrieval).
3. Optional enrichments (MemoryKernel) remain non-blocking; the core workflow must still work without them.

## Default Model Policy

1. **Recommended default**: `Llama 3.1 8B Instruct (Q4_K_M)`.
2. Other supported models exist, but are hidden behind progressive disclosure in Settings.
3. Custom GGUF models can be loaded, but are treated as unverified unless allowlisted; the app logs a local audit warning.

## Output Contract (What the Model Must Produce)

Every response must have two sections:

1. `### OUTPUT`
   - End-user response, ready to copy and send.
2. `### IT SUPPORT INSTRUCTIONS`
   - Internal operator guidance (customization steps, checks, follow-ups).

## Citation Contract (Trust and Verifiability)

1. The prompt enforces a strict rule: **NO CITATION = NO CLAIM** for technical facts.
2. Citations are presented as `[Source N]` and map to local KB sources.
3. **Copy gating** requires:
   - confidence mode = `answer`
   - at least one KB source present
4. If copy gating is overridden, the operator must enter a reason; an audit event is logged locally (no response text logged).

## Operator Controls (Without Prompt Editing)

Operators can:

1. Load/unload the model
2. Choose context window size (bounded)
3. Enable/disable vector embeddings
4. Configure KB folder
5. View "AI Status & Guarantees" (model, embeddings, KB, MemoryKernel preflight)

Operators cannot:

1. Edit system prompts
2. Disable security/citation policies without an explicit override flow (audited)

## Degraded/Offline Behavior (Deterministic)

1. If no model is loaded: generation is disabled; the UI instructs how to proceed.
2. If KB is not configured: the UI warns that citations may be missing and recommends configuring KB.
3. If MemoryKernel is unavailable: the app continues with deterministic fallback messaging.

