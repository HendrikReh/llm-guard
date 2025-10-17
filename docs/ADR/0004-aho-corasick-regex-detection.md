# ADR-0004: Aho-Corasick for Keywords, Regex for Complex Patterns

**Status:** Accepted
**Date:** 2025-10-17
**Deciders:** Hendrik Reh, GPT-5 Codex
**AI Collaboration:** Performance analysis and implementation with GPT-5 Codex

## Context

LLM-Guard needs to scan text for prompt injection indicators. We have two types of patterns:
1. **Exact keywords:** "ignore previous", "jailbreak", "reveal system prompt"
2. **Complex patterns:** Case-insensitive variants, Unicode tricks, encoded payloads

Performance requirements (PRD 2.2):
- Scan 10K+ characters in <100ms
- Support 100+ detection rules
- Handle streaming/tail mode efficiently

We need to choose pattern-matching algorithms that balance:
- Speed (fast scanning of large texts)
- Expressiveness (support complex patterns)
- Maintainability (rules easy to add/modify)

## Decision

Use **two complementary pattern-matching engines**:

### 1. Aho-Corasick for Keywords
Load simple keywords into `aho_corasick::AhoCorasick`:
```rust
let keywords = vec!["ignore previous", "jailbreak", "bypass safety"];
let ac = AhoCorasick::new(&keywords)?;
for mat in ac.find_iter(text) {
    // Generate Finding
}
```

**Used for:**
- Exact string matches (case-sensitive by default)
- High-frequency patterns
- Simple keyword lists in `rules/keywords.txt`

### 2. Regex for Complex Patterns
Compile regex patterns individually:
```rust
let patterns = vec![
    r"(?i)\b(ignore|disregard)\s+(previous|prior)",
    r"[\u200B-\u200F]",  // Zero-width chars
];
for pattern in &patterns {
    let re = Regex::new(pattern)?;
    for mat in re.find_iter(text) {
        // Generate Finding
    }
}
```

**Used for:**
- Case-insensitive matching (`(?i)`)
- Unicode character classes
- Encoded payload detection (base64, hex)
- Variable whitespace/word boundaries

### Rule Format

**keywords.txt:**
```
ignore previous instructions
disregard prior context
jailbreak
bypass safety
reveal system prompt
```

**patterns.json:**
```json
[
  {
    "id": "INSTR_OVERRIDE",
    "kind": "Regex",
    "pattern": "(?i)\\b(ignore|disregard)\\s+(previous|prior)\\s+(message|instruction)",
    "weight": 16.0,
    "description": "Instruction override attempt"
  }
]
```

## Consequences

### Positive
- **Optimal performance:** Aho-Corasick is O(n+m) for multi-pattern search
- **Expressiveness:** Regex handles complex cases keywords can't
- **Separation of concerns:** Simple rules stay simple (keywords), complex stay explicit (regex)
- **Maintainability:** Non-technical users can add keywords easily
- **Precompilation:** Both engines compile once at startup
- **Streaming friendly:** Both work incrementally for tail mode

### Negative
- **Two rule formats:** Users must understand when to use each
- **Regex compilation cost:** Paid at startup for each pattern
- **Regex performance variability:** Complex patterns can be slow
- **Unicode regex cost:** Unicode character classes slower than ASCII
- **Duplicate matches possible:** Same text span could match both engines

### Neutral
- **Rule maintenance split:** Keywords in .txt, patterns in .json
- **Finding deduplication:** May need to merge overlapping matches

## Alternatives Considered

### Option 1: Regex Only
- **Pros:** Single engine, maximum expressiveness
- **Cons:** Slower than Aho-Corasick for simple keywords, harder to write simple rules
- **Why rejected:** Performance penalty for common case (exact keywords)

### Option 2: Aho-Corasick Only
- **Pros:** Maximum speed, single engine
- **Cons:** Cannot handle case-insensitive, Unicode tricks, variable patterns
- **Why rejected:** Too limited for complex attack patterns

### Option 3: RegexSet for All Patterns
- **Pros:** Single regex engine, some optimization for multiple patterns
- **Cons:** Slower than Aho-Corasick for exact matches, more complex rule syntax
- **Why rejected:** Aho-Corasick faster for our keyword-heavy use case

### Option 4: Hyperscan (Intel)
- **Pros:** Extremely fast multi-pattern regex engine
- **Cons:** Complex setup, platform-specific, harder to install, overkill for v1
- **Why rejected:** Added complexity not justified by current performance requirements

## Implementation Notes

### Pattern Compilation
```rust
pub struct RuleEngine {
    keyword_matcher: AhoCorasick,
    regex_patterns: Vec<(String, Regex)>,  // (rule_id, compiled_regex)
}

impl RuleEngine {
    pub fn new(rules: &[Rule]) -> Result<Self> {
        let keywords: Vec<_> = rules.iter()
            .filter(|r| r.kind == RuleKind::Keyword)
            .map(|r| r.pattern.as_str())
            .collect();

        let keyword_matcher = AhoCorasick::new(&keywords)?;

        let regex_patterns: Vec<_> = rules.iter()
            .filter(|r| r.kind == RuleKind::Regex)
            .map(|r| (r.id.clone(), Regex::new(&r.pattern)))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self { keyword_matcher, regex_patterns })
    }
}
```

### Performance Optimization
- Use `AhoCorasick::builder()` with `.ascii_case_insensitive(true)` if all keywords ASCII
- Consider `RegexSet` if regex count grows >20 patterns
- Profile with `cargo bench` on representative inputs
- Document performance characteristics in README

### Rule Validation
At load time, validate:
- No duplicate rule IDs
- Regex patterns compile successfully
- Weights in valid range (0.0-20.0)
- Keywords are non-empty

### Testing
- Unit tests: verify each engine separately
- Integration tests: both engines together, check for overlaps
- Performance tests: benchmark 10K char input with 100 rules
- Edge cases: empty strings, Unicode, very long patterns

## References

- **PRD Section 2.2:** Performance requirement (<100ms for 10K chars)
- **PRD Section 4.1 F2:** Rule-Based Detection Engine
- **PRD Section 6:** Detection Rules & Heuristics
- **PLAN.md Phase 3:** Scanner Engine implementation
- **Aho-Corasick paper:** Efficient string matching (1975)
