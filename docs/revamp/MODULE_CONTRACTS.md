# Module Contracts (Revamp)

Status: Drafted for Phase 2 Gate  
Date: 2026-02-08

## Contract Template
Each module must document:
1. Inputs
2. Outputs
3. Side effects
4. Error contract
5. Test obligations

## 1) app-shell
### Inputs
- active route/tab state
- global notifications
- command palette actions

### Outputs
- navigation events
- action dispatch requests

### Side Effects
- none outside UI state and keyboard bindings

## 2) inbox
### Inputs
- ticket list provider
- SLA metadata provider
- filter/sort state

### Outputs
- selected ticket event
- assignment/reprioritization action request

### Side Effects
- ticket metadata update via integration adapter only

## 3) workspace
### Inputs
- selected ticket
- related context (history, notes, KB suggestions)

### Outputs
- drafted response
- transition/escalation requests
- operator diagnostics notes

### Side Effects
- may invoke local generation and adapters through domain services

## 4) integration-memorykernel
### Inputs
- normalized query payload
- runtime pin/contract expectations

### Outputs
- enrichment payload or deterministic fallback metadata

### Side Effects
- network calls to configured MemoryKernel endpoint

### Hard Rules
1. No caller bypasses this adapter.
2. Non-2xx envelopes normalized internally.
3. No direct UI dependency on raw producer envelope shape.

## 5) integration-jira
### Inputs
- ticket IDs
- workflow transition actions

### Outputs
- ticket state data
- transition result status

### Side Effects
- Jira API calls with strict validation and audit logging

## 6) llm-runtime
### Inputs
- prompt contract ID
- model profile ID
- context bundle

### Outputs
- generated response
- confidence/guardrail metadata

### Side Effects
- local model inference only

## Test Obligations per Module
1. Unit tests for contract behavior.
2. Boundary tests for invalid/malformed inputs.
3. Integration tests for critical domain paths.
