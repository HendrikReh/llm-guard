# ADR-0003: Optional LLM-Powered Analysis with Multi-Provider Support

**Status:** Accepted
**Date:** 2025-10-17
**Deciders:** Hendrik Reh, GPT-5 Codex, Claude Code
**AI Collaboration:** Provider interface design with Claude Code; implementation with GPT-5 Codex

## Context

LLM-Guard's heuristic scoring provides fast, transparent detection but has limitations:
- Only detects known patterns in rule set
- Cannot reason about semantic intent
- May miss novel or sophisticated attacks

Adding LLM-powered analysis could provide:
- Secondary validation of heuristic findings
- Semantic understanding of prompt intent
- Suggestions for mitigation strategies

However, LLM analysis introduces:
- Latency (1-5 seconds per request)
- Cost (API calls)
- External dependencies
- Non-deterministic results

We need to decide if/how to integrate LLM analysis without compromising the tool's core value (speed + transparency).

## Decision

Implement **optional, multi-provider LLM analysis** with the following design:

### Opt-In by Default
- LLM analysis disabled by default
- Enabled via `--with-llm` CLI flag
- Heuristic scoring always runs first and is independent
- LLM verdict augments but doesn't replace heuristic score

### Multi-Provider Support
Support multiple LLM providers from day one:
- **OpenAI** (GPT-4o, GPT-4o-mini) - default
- **Anthropic** (Claude 3 Haiku, Sonnet, Opus)
- **Google Gemini** (Gemini 1.5 Flash, Pro)
- **Azure OpenAI** (same models, different endpoint)
- **Noop** (dry-run mode for testing)

### Configuration Hierarchy
```
CLI flags (highest priority)
  ↓
Environment variables
  ↓
Configuration file
  ↓
Built-in defaults (lowest priority)
```

### Provider Interface
```rust
#[async_trait]
pub trait LlmClient {
    async fn verdict(
        &self,
        prompt: &str,
        findings: &[Finding],
        risk_score: f32,
    ) -> Result<LlmVerdict>;
}

pub struct LlmVerdict {
    pub label: String,        // "safe" | "suspicious" | "malicious"
    pub rationale: String,    // ≤40 words explanation
    pub mitigation: String,   // Suggested action
}
```

### Prompt Template
```
You are a security reviewer for LLM prompts.

Heuristic analysis detected risk score: {risk_score}/100
Triggered rules: {rules_summary}

Classify this INPUT as one of: "safe", "suspicious", or "malicious".
Explain briefly (≤40 words) and suggest one mitigation step.

INPUT:
<<<
{snippet}
>>>

Return JSON: {"label": "...", "rationale": "...", "mitigation": "..."}
```

## Consequences

### Positive
- **Enhanced detection:** Catches semantic attacks heuristics miss
- **User flexibility:** Organizations choose their preferred provider
- **Cost control:** Opt-in model prevents surprise API bills
- **Degraded gracefully:** Tool works without LLM if API unavailable
- **Provider agnostic:** Not locked into single vendor

### Negative
- **Implementation complexity:** Multiple provider clients to maintain
- **Latency impact:** 1-5s overhead when enabled
- **API dependencies:** External service reliability risk
- **Cost consideration:** Users must provision API keys and budget
- **Non-deterministic:** LLM verdicts may vary for same input

### Neutral
- **Two-tier detection:** Heuristic + LLM provides defense-in-depth
- **Configuration surface:** More options = more to document/test

## Alternatives Considered

### Option 1: Heuristics Only (No LLM)
- **Pros:** Simpler, faster, no external deps, fully deterministic
- **Cons:** Limited to known patterns, no semantic understanding
- **Why rejected:** Leaves value on table; LLM can catch novel attacks

### Option 2: LLM Required (Always On)
- **Pros:** Maximum detection capability, simpler UX (one mode)
- **Cons:** Slow, expensive, external dependency, violates "fast" goal
- **Why rejected:** Contradicts PRD performance goals (PRD 2.2 #1)

### Option 3: Single Provider (OpenAI Only)
- **Pros:** Less code to maintain, faster initial implementation
- **Cons:** Vendor lock-in, no Azure support, limits enterprise adoption
- **Why rejected:** Reduces flexibility; multi-provider not much harder

### Option 4: Embedded Local Model
- **Pros:** No API costs, no external deps, lower latency
- **Cons:** Large binary size, GPU requirements, lower quality
- **Why rejected:** Complexity/quality trade-off not worth it for v1

## Implementation Notes

### Code Organization
```
crates/llm-guard-cli/src/llm/
├── mod.rs              # LlmClient trait, config types
├── openai.rs           # OpenAI client
├── anthropic.rs        # Anthropic client
├── gemini.rs           # Google Gemini client
├── azure.rs            # Azure OpenAI client
├── noop.rs             # Dry-run implementation
└── factory.rs          # Provider selection logic
```

### Error Handling
- Timeout after 30s (configurable)
- Retry up to 2 times (configurable)
- On failure: log error, return heuristic-only results
- Never fail the scan due to LLM unavailability

### Testing Strategy
- Unit tests for each provider client
- Integration tests with mocked HTTP responses
- Noop provider for deterministic E2E tests
- Real provider tests (optional, gated on API keys)

### Future Migration to `rig-core`
Plan to migrate to [`rig-core`](https://rig.rs/) library (Phase 9):
- Provides unified provider abstraction
- Reduces maintenance burden
- Current hand-rolled clients establish requirements
- Migration path documented in PLAN.md Phase 9

## References

- **PRD Section 2.2:** Success Criteria #5 (Extensibility)
- **PRD Section 4.1 F6:** LLM-Powered Analysis feature
- **PRD Section 8.1:** LLM Integration specifications
- **PLAN.md Phase 6:** Optional LLM Adapter implementation
- **PLAN.md Phase 9:** Migration to rig-core
