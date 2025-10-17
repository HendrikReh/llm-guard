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

### Formula
```
base_score = Σ (finding.weight × multiplier)
  where multiplier = 1.0 for first occurrence per rule family
                   = 0.5 for subsequent occurrences (diminishing returns)

length_normalization = clamp(text_length / 800, 0.5, 1.5)

synergy_bonus = +5 if high-severity rules co-occur within 200 chars

final_score = clamp(base_score × length_normalization + synergy_bonus, 0, 100)
```

### Rule Weight Ranges
- Instruction override: 12-18
- Data exfiltration: 12-20
- Policy subversion: 10-16
- Model exploitation: 8-12
- Obfuscation: 5-10
- Prompt leak bait: 12-18

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
- Core algorithm: `crates/llm-guard-core/src/scanner/heuristics.rs`
- Rule weights: `rules/patterns.json` and `rules/keywords.txt`
- Rubric thresholds: Configurable via CLI or config file

### Testing Strategy
- Unit tests for scoring components (length norm, synergy, diminishing returns)
- Integration tests with known attack samples
- Property tests to ensure score always in [0, 100] range

### Tuning Guidance
Document in `README.md`:
- How to adjust rule weights
- How to customize thresholds
- How to monitor false positive/negative rates

## References

- **PRD Section 2.2:** Success Criteria #4 (Transparency)
- **PRD Section 6:** Detection Rules & Heuristics (detailed formula)
- **PLAN.md Phase 4:** Risk Scoring & Rubric implementation
- **OWASP LLM Top 10:** Attack pattern taxonomy
