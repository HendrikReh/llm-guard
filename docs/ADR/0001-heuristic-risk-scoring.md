# ADR-0001: Heuristic-Based Risk Scoring Algorithm

**Status:** Accepted
**Date:** 2025-10-17
**Deciders:** Hendrik Reh, GPT-5 Codex, Claude Code
**AI Collaboration:** Algorithm design with GPT-5 Codex; review and refinement with Claude Code

## Context

LLM-Guard needs a transparent, explainable method to score the risk of prompt injection attempts. The score must be:
- Fast to compute (<100ms for 10K+ chars)
- Transparent with clear attribution to specific patterns
- Tunable by security teams
- Bounded to 0-100 range for easy interpretation

Traditional ML-based approaches would be opaque "black boxes" that don't meet our explainability requirement. We need a deterministic, rule-based approach that users can understand and customize.

## Decision

Implement a **weighted heuristic scoring algorithm** with the following components:

### Formula (as implemented in v0.4.1)
```
base_score = Σ (finding.weight × multiplier)
  where multiplier = 1.0 for first occurrence per rule family
                   = 0.5 for subsequent occurrences (family dampening)

length_factor = clamp(text_length / baseline_chars, min_factor, max_factor)
  where baseline_chars = 800
        min_factor = 0.5
        max_factor = 1.5

final_score = clamp(base_score × length_factor, 0, 100)
```

**Note:** Synergy bonuses (proximity-based scoring boosts) were planned but not implemented in v0.4.1. Future enhancement tracked in Phase 10 (optimization).

### Rule Weight Ranges
Rule families are determined by the prefix before the first underscore in the rule ID (e.g., `PROMPT_LEAK` → family `PROMPT`, `INSTR_IGNORE` → family `INSTR`).

Current rule weights (as of v0.4.1):
- CODE family (code injection): 45
- PROMPT family (prompt leak): 40
- INSTR family (instruction override): 30-35
- MODEL family (model override): 30

**Weight Guidelines:**
- Critical attacks (code execution, prompt leak): 40-45
- High severity (instruction override): 30-35
- Medium severity (model manipulation): 25-30
- Low severity (obfuscation, edge cases): 10-20

### Risk Rubric
- 0-24: Low (proceed)
- 25-59: Medium (review required)
- 60-100: High (block/re-prompt)

## Consequences

### Positive
- **Full transparency:** Every score can be decomposed into specific rule matches
- **Fast computation:** Simple arithmetic, no ML inference overhead
- **Tunable:** Security teams can adjust weights and thresholds
- **Explainable:** Users understand why prompts are flagged
- **Deterministic:** Same input always produces same score

### Negative
- **Manual rule maintenance:** New attack patterns require manual rule additions
- **False positives:** Legitimate prompts may trigger pattern matches
- **Limited to known patterns:** Novel attacks not in rule set will be missed
- **Threshold tuning required:** Organizations need to calibrate for their use case

### Neutral
- **Simplified vs ML:** Trading accuracy for transparency
- **Rule coverage:** Effectiveness depends on rule set comprehensiveness

## Alternatives Considered

### Option 1: ML-Based Classification
- **Pros:** Could detect novel patterns, potentially higher accuracy
- **Cons:** Opaque decisions, requires training data, slower inference, hard to explain
- **Why rejected:** Fails transparency and explainability requirements (PRD Section 2.1)

### Option 2: Simple Binary Classification
- **Pros:** Even simpler, faster
- **Cons:** No risk gradation, can't prioritize review, too coarse for CI/CD
- **Why rejected:** Doesn't provide actionable risk levels for automated workflows

### Option 3: Bayesian Probability Scoring
- **Pros:** Statistically grounded, handles uncertainty
- **Cons:** Complex to tune, harder to explain to non-technical users, slower
- **Why rejected:** Complexity outweighs benefits for v1; may revisit in future

## Implementation Notes

### Code Location
- **Core algorithm:** `crates/llm-guard-core/src/scanner/default_scanner.rs` (`score_findings()` method)
- **Config:** `crates/llm-guard-core/src/scanner/mod.rs` (`RiskConfig` struct)
- **Rule weights:** `rules/patterns.json` (regex) and `rules/keywords.txt` (exact match)
- **Risk thresholds:** `RiskThresholds` in `scanner/mod.rs` (default: 25, 60)

### Testing Strategy
- **Unit tests:** Scoring components (length factor, family dampening)
- **Integration tests:** Known attack samples in `default_scanner.rs`
- **Property tests:** Score always in [0, 100] range (`proptest` in `scanner/mod.rs`)
- **Current coverage:** 44 tests (34 passing, 10 ignored for network)

### Tuning Guidance
Users can customize:
- **Rule weights:** Edit `rules/patterns.json` and `rules/keywords.txt`
- **Risk thresholds:** Modify `RiskConfig::default()` in code (CLI flags not yet exposed)
- **Length normalization:** Adjust `baseline_chars`, `min_length_factor`, `max_length_factor`
- **Family dampening:** Tune `family_dampening` (0.0 = no dampening, 1.0 = full credit for all occurrences)

## References

- **PRD Section 2.2:** Success Criteria #4 (Transparency)
- **PRD Section 6:** Detection Rules & Heuristics (detailed formula)
- **PLAN.md Phase 4:** Risk Scoring & Rubric implementation
- **OWASP LLM Top 10:** Attack pattern taxonomy
