# Phase 3: LLM Router V2 - Detailed Runbook

**Duration**: 2 days
**Prerequisites**: Phase 2 complete, pilot go/no-go = GO
**Success Criteria**: Golden set passes, no regressions vs Phase 2

---

## Phase 3 Overview

Implement intent-aware LLM routing with optimized generation parameters based on query type.

**Goals**:
- Route queries to appropriate response templates
- Optimize generation parameters per intent
- Validate no quality regressions vs Phase 2
- Expand golden test set with new router cases

---

## Step 1: Implement Intent-Aware Template Selection (1 hour)

**Purpose**: Route queries to appropriate response templates

**File**: `src-tauri/src/llm/prompts.rs`

**Current State**:
```rust
// Currently one-size-fits-all template
pub fn get_system_prompt() -> String {
    "You are a helpful IT support assistant...".to_string()
}
```

**After Phase 3**:
```rust
pub enum QueryIntent {
    Answer,      // Direct factual answer
    Clarify,     // Needs clarification
    Abstain,     // Cannot answer safely
}

pub fn detect_intent(query: &str) -> QueryIntent {
    // Simple heuristics (can be enhanced with classifier)
    if query.contains("?") && query.len() < 100 {
        QueryIntent::Answer
    } else if query.contains("not sure") || query.contains("unclear") {
        QueryIntent::Clarify
    } else if is_sensitive_query(query) {
        QueryIntent::Abstain
    } else {
        QueryIntent::Answer
    }
}

pub fn get_system_prompt(intent: QueryIntent) -> String {
    match intent {
        QueryIntent::Answer => answer_template(),
        QueryIntent::Clarify => clarification_template(),
        QueryIntent::Abstain => abstain_template(),
    }
}

fn answer_template() -> String {
    r#"You are a helpful IT support assistant. Answer the user's question directly with:
    1. Clear step-by-step instructions
    2. Relevant policy references
    3. Troubleshooting tips if applicable

Answer concisely and accurately."#.to_string()
}

fn clarification_template() -> String {
    r#"You are a helpful IT support assistant. The user's request is unclear.
    1. Identify what's ambiguous
    2. Ask 2-3 clarifying questions
    3. Suggest what they might be asking for

Help them refine their question."#.to_string()
}

fn abstain_template() -> String {
    r#"You are a helpful IT support assistant. This request is outside your scope.
    1. Explain why you can't help
    2. Suggest who to contact instead
    3. Offer related information if helpful

Be professional and helpful even when abstaining."#.to_string()
}
```

**Implementation Steps**:
1. Add `QueryIntent` enum to `llm.rs`
2. Implement `detect_intent()` logic
3. Update `get_system_prompt()` to use intent
4. Update command invocations to detect intent first

**Verification**:
```bash
cd src-tauri
cargo build
cargo test llm::prompts
# Expected: All tests pass, 3 templates correctly selected
```

---

## Step 2: Add Generation Parameter Profiles (1 hour)

**Purpose**: Optimize temperature and other parameters by intent

**File**: `src-tauri/src/commands/mod.rs`

**Generation Parameters Structure**:
```rust
pub struct GenerationParams {
    pub temperature: f32,
    pub top_p: f32,
    pub max_tokens: usize,
    pub repetition_penalty: f32,
}

pub fn get_params_for_intent(intent: QueryIntent) -> GenerationParams {
    match intent {
        // Policy queries: factual, deterministic
        QueryIntent::Answer => GenerationParams {
            temperature: 0.3,      // Low: factual
            top_p: 0.9,            // Narrow: less variation
            max_tokens: 500,       // Reasonable length
            repetition_penalty: 1.2, // Avoid repetition
        },
        // Clarification: needs variation
        QueryIntent::Clarify => GenerationParams {
            temperature: 0.7,      // Higher: creative questions
            top_p: 0.95,           // Broader: more options
            max_tokens: 300,       // Shorter for dialogue
            repetition_penalty: 1.1,
        },
        // Abstain: careful tone
        QueryIntent::Abstain => GenerationParams {
            temperature: 0.5,      // Balanced
            top_p: 0.9,            // Standard
            max_tokens: 200,       // Brief
            repetition_penalty: 1.3,
        },
    }
}
```

**Integration into generate command**:
```rust
#[tauri::command]
pub async fn generate_response(
    state: State<'_, AppState>,
    query: String,
    context: Option<String>,
) -> Result<String, String> {
    // Detect intent
    let intent = detect_intent(&query);

    // Get appropriate prompt
    let system_prompt = get_system_prompt(intent);

    // Get optimized parameters
    let params = get_params_for_intent(intent);

    // Generate with optimized settings
    let llm = state.llm.read().await
        .ok_or("LLM not initialized")?;

    let response = llm.generate(
        &system_prompt,
        &query,
        &context.unwrap_or_default(),
        params.temperature,
        params.top_p,
        params.max_tokens,
    ).await?;

    Ok(response)
}
```

**Verification**:
```bash
cd src-tauri
cargo test commands::generate_response
# Expected: All tests pass, parameters correctly applied
```

---

## Step 3: Run Golden Set Validation (3 hours)

**Purpose**: Verify no regressions vs Phase 2 baseline

**Test Set**: See `PHASE3_GOLDEN_SET.json`

**Execution**:
```bash
# Run golden set tests
pnpm run test:llm-golden-set

# This runs:
# - 50 benchmark queries from Phase 2
# - Compares Phase 2 vs Phase 3 responses
# - Flags any quality regressions
# - Measures performance impact

# Expected output:
# ┌─────────────────────────────────────────┐
# │ LLM Golden Set Validation                │
# ├─────────────────────────────────────────┤
# │ Phase 2 Baseline (50 queries)            │
# │ ├─ Avg Quality Score: 4.2/5.0           │
# │ ├─ Avg Response Time: 340ms             │
# │ └─ Pass Rate: 100%                       │
# │                                          │
# │ Phase 3 with Router V2 (50 queries)     │
# │ ├─ Avg Quality Score: 4.3/5.0 ↑        │
# │ ├─ Avg Response Time: 345ms             │
# │ └─ Pass Rate: 100%                       │
# │                                          │
# │ RESULT: PASS ✅ (no regressions)        │
# └─────────────────────────────────────────┘
```

**If Regression Detected**:
```bash
# Identify which queries regressed
pnpm run test:llm-golden-set --verbose

# Example output:
# REGRESSION: Query 12 (policy redirect)
#   Phase 2 Score: 5.0
#   Phase 3 Score: 3.5 ✗
#   Reason: Temperature too high for policy answer

# FIX: Adjust temperature for QueryIntent::Answer
# From: temperature: 0.4
# To: temperature: 0.3

# Re-test
pnpm run test:llm-golden-set
```

---

## Step 4: Expand Golden Set with Router V2 Cases (2 hours)

**Purpose**: Add new test cases that specifically validate intent-aware routing

**New Test Cases**:

```json
{
  "new_router_v2_cases": [
    {
      "id": 51,
      "query": "reset password",
      "expected_intent": "Answer",
      "expected_response_type": "step-by-step",
      "quality_metric": "factual_accuracy"
    },
    {
      "id": 52,
      "query": "how do I change my vpn settings I think",
      "expected_intent": "Clarify",
      "expected_response_type": "clarifying_questions",
      "quality_metric": "question_quality"
    },
    {
      "id": 53,
      "query": "can I access classified documents",
      "expected_intent": "Abstain",
      "expected_response_type": "safe_refusal",
      "quality_metric": "safety_score"
    },
    {
      "id": 54,
      "query": "I need help with my email setup",
      "expected_intent": "Answer",
      "expected_response_type": "guided_instructions",
      "quality_metric": "task_completion"
    },
    {
      "id": 55,
      "query": "what's the best monitor for gaming",
      "expected_intent": "Abstain",
      "expected_response_type": "scope_boundary",
      "quality_metric": "appropriate_boundaries"
    }
  ]
}
```

**Adding to Test Suite**:
```bash
# Add to test data
cat >> tests/fixtures/llm_golden_set.json << 'EOF'
{new_router_v2_cases}
EOF

# Run expanded tests
pnpm run test:llm-golden-set

# Expected output:
# Total test cases: 55 (50 baseline + 5 new)
# Pass rate: 100%
# Regression checks: PASS
# Router coverage: Answer (20), Clarify (15), Abstain (20)
```

---

## Phase 3 Success Verification

**Checklist**:
- [ ] Step 1: Intent detection implemented
- [ ] Step 1: Three templates created (answer/clarify/abstain)
- [ ] Step 1: Template selection working
- [ ] Step 2: Generation parameters by intent
- [ ] Step 2: Parameters correctly applied
- [ ] Step 3: Golden set baseline established
- [ ] Step 3: No regressions detected
- [ ] Step 3: Performance acceptable (< 100ms added latency)
- [ ] Step 4: 5 new router cases added
- [ ] Step 4: All 55 tests passing

**Phase 3 Status**: ✅ **COMPLETE** → **Proceed to Phase 4**

---

## Phase 3 → Phase 4 Transition

Once all tests pass:

1. **Commit router V2 code**:
   ```bash
   git add src-tauri/src/llm/prompts.rs
   git add src-tauri/src/commands/mod.rs
   git commit -m "feat: Add intent-aware LLM routing V2"
   ```

2. **Tag golden set version**:
   ```bash
   git tag -a "v1.0.0-phase3-router-v2" -m "Phase 3: LLM Router V2 complete"
   ```

3. **Proceed to Phase 4**: KB Enrichment
   - See: `PHASE4_KB_ENRICHMENT_RUNBOOK.md`

---

## Troubleshooting

### Problem: Golden set regression detected
```bash
# 1. Identify failing queries
pnpm run test:llm-golden-set --failing

# 2. Compare responses
diff <(cat phase2_golden_baseline.json) <(cat phase3_golden_current.json)

# 3. Adjust parameters and retry
# Edit src-tauri/src/llm/prompts.rs

# 4. Re-test
pnpm run test:llm-golden-set
```

### Problem: Intent detection wrong for a query
```bash
# Test intent detection directly
cargo run --bin test_intent -- "your test query"

# Adjust heuristics in detect_intent()
# Consider adding NLP classifier later
```

### Problem: Response time increased too much
```bash
# Profile generation
pnpm run profile:llm-generation

# Check:
# - Intent detection overhead (should be < 5ms)
# - Template loading (should be cached)
# - LLM invocation (main cost, ~300ms)

# Optimize if needed
```

