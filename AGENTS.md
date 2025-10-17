# AI Coding Assistant Onboarding Guide

**Welcome to the AI Coding Hackathon!**

This document provides essential context, conventions, and collaboration guidelines for AI coding assistants participating in hackathon projects. Follow these practices to contribute effectively and maintain code quality during rapid development.

---

## 1. Hackathon Context

### 1.1 Event Overview

You are assisting in the **AI Coding Accelerator Hackathon** organized by [Maven](https://maven.com/nila/ai-coding-accelerator).

**Instructors:** Vignesh Mohankumar and Jason Liu

**Key Characteristics:**
- Fast-paced development (typically 1-day sprints)
- Focus on working prototypes over perfect architecture
- Balance between speed and code quality
- Emphasis on demonstrable results

### 1.2 Current Project: LLM-Guard

**Project Goal:** Build a fast, explainable prompt injection detection tool in Rust

**Core Objectives:**
- Scan prompts for injection/jailbreak indicators
- Provide transparent risk scoring (0-100)
- Support multiple output formats (CLI, JSON)
- Optional LLM-powered analysis

**Key Documents:**
- `LLM-Guard.md` - Product Requirements Document
- `PLAN.md` - Implementation roadmap and progress tracking
- `README.md` - User-facing documentation

---

## 2. Collaboration Philosophy

### 2.1 Speed vs Quality Balance

**DO:**
- ✅ Implement features incrementally
- ✅ Write tests for core logic
- ✅ Use simple, explicit designs
- ✅ Prioritize working code over perfect abstractions
- ✅ Document non-obvious decisions in comments

**DON'T:**
- ❌ Over-engineer solutions
- ❌ Add unnecessary abstractions prematurely
- ❌ Skip error handling entirely
- ❌ Ignore the PRD requirements
- ❌ Create features not explicitly requested

### 2.2 Communication Style

- **Be concise:** Explain what you're doing and why, briefly
- **Ask when uncertain:** Clarify requirements before implementing
- **Show progress:** Use comments to indicate WIP sections
- **Acknowledge constraints:** Call out hackathon trade-offs explicitly

---

## 3. Rust Coding Conventions

### 3.1 Project Structure

```
llm-guard/
├── Cargo.toml              # Dependencies and metadata
├── src/
│   ├── main.rs             # Entry point
│   ├── cli.rs              # Command-line interface
│   ├── scanner/            # Core detection engine
│   │   ├── mod.rs
│   │   ├── rules.rs        # Rule loading and matching
│   │   ├── heuristics.rs   # Scoring algorithms
│   │   └── explain.rs      # Finding attribution
│   ├── llm_adapter.rs      # LLM integration (optional)
│   └── report.rs           # Output formatting
├── rules/
│   ├── keywords.txt        # Detection keywords
│   └── patterns.json       # Regex patterns
├── tests/                  # Integration tests
└── samples/                # Test data
```

### 3.2 Code Style

**Formatting:**
- Use `rustfmt` with default settings
- Run `cargo fmt` before committing

**Linting:**
- Address `clippy` warnings: `cargo clippy -- -D warnings`
- Avoid `#[allow]` without comments explaining why

**Naming:**
- Types: `PascalCase` (e.g., `ScanReport`, `RuleKind`)
- Functions/variables: `snake_case` (e.g., `risk_score`, `load_rules`)
- Constants: `SCREAMING_SNAKE_CASE` (e.g., `MAX_SCORE`)

**Error Handling:**
- Use `anyhow::Result<T>` for application code
- Use `thiserror` for library error types (if needed)
- Provide context with `.context()` or `.with_context()`

### 3.3 Essential Patterns

**Reading Files:**
```rust
use std::fs;
use anyhow::{Context, Result};

fn load_keywords(path: &str) -> Result<Vec<String>> {
    let content = fs::read_to_string(path)
        .context("Failed to read keywords file")?;
    Ok(content.lines().map(String::from).collect())
}
```

**Using Aho-Corasick:**
```rust
use aho_corasick::AhoCorasick;

let ac = AhoCorasick::new(&keywords)?;
for mat in ac.find_iter(text) {
    // Process match
}
```

**Structuring Output:**
```rust
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize)]
pub struct ScanReport {
    pub risk_score: f32,
    pub findings: Vec<Finding>,
}
```

**CLI with Clap:**
```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Scan {
        #[arg(long)]
        file: Option<String>,
        #[arg(long)]
        json: bool,
    },
}
```

---

## 4. Development Workflow

### 4.1 Before Writing Code

1. **Review requirements:** Check `LLM-Guard.md` for feature specs
2. **Check the plan:** Review `PLAN.md` for current phase and tasks
3. **Understand the context:** Read related code if modifying existing features
4. **Clarify unknowns:** Ask questions before making assumptions

### 4.2 Implementation Process

1. **Start simple:** Get basic functionality working first
2. **Test as you go:** Write tests for core logic
3. **Handle errors:** Don't use `.unwrap()` in production paths
4. **Document intent:** Add comments for non-obvious logic
5. **Format and lint:** Run `cargo fmt` and `cargo clippy`

### 4.3 Testing Expectations

**Minimum Requirements:**
- Unit tests for scoring algorithms
- Integration tests for CLI commands
- Test data for different risk levels (safe/suspicious/malicious)

**Test Structure:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_findings_zero_score() {
        let score = risk_score(&[], 100);
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_score_clamped_to_100() {
        let findings = vec![
            Finding { weight: 50.0, ..Default::default() },
            Finding { weight: 60.0, ..Default::default() },
        ];
        let score = risk_score(&findings, 1000);
        assert!(score <= 100.0);
    }
}
```

### 4.4 Common Commands

```bash
# Build the project
cargo build --release

# Run tests
cargo test

# Format code
cargo fmt

# Check for issues
cargo clippy

# Run the CLI
cargo run -- scan --file samples/test.txt

# Generate JSON output
cargo run -- scan --file samples/test.txt --json
```

---

## 5. Key Dependencies

### 5.1 Core Crates

| Crate | Purpose | Usage |
|-------|---------|-------|
| `clap` | CLI parsing | Define commands and arguments |
| `aho-corasick` | Fast keyword matching | Multi-pattern string search |
| `regex` | Pattern matching | Complex text patterns |
| `serde` + `serde_json` | Serialization | JSON input/output |
| `anyhow` | Error handling | Application-level errors |
| `colored` | Terminal colors | Pretty CLI output |

### 5.2 Optional Crates

| Crate | Purpose | Usage |
|-------|---------|-------|
| `tokio` | Async runtime | LLM API calls |
| `reqwest` | HTTP client | External API requests |
| `notify` | File watching | Streaming mode |
| `comfy-table` | Tables | Rule listing |

---

## 6. Security & Privacy Guidelines

### 6.1 Data Handling

**DO:**
- ✅ Redact sensitive data in excerpts (emails, tokens, keys)
- ✅ Truncate inputs before sending to external APIs
- ✅ Accept API keys from environment variables only
- ✅ Add timeouts to all external calls

**DON'T:**
- ❌ Log raw user input by default
- ❌ Hardcode API keys or secrets
- ❌ Store sensitive data without explicit user consent
- ❌ Use `panic!` on invalid input (return errors instead)

### 6.2 Input Validation

```rust
// Good: Validate and handle errors
fn scan_file(path: &str) -> Result<ScanReport> {
    if !Path::new(path).exists() {
        anyhow::bail!("File not found: {}", path);
    }

    let content = fs::read_to_string(path)
        .context("Failed to read file")?;

    if content.len() > 1_000_000 {
        anyhow::bail!("File too large (max 1MB)");
    }

    scan_text(&content)
}
```

---

## 7. Hackathon-Specific Guidance

### 7.1 Time Management

**Phase-Based Development:**
- Hour 1-2: Foundation (project setup, CLI scaffolding)
- Hour 3-4: Core engine (scanning, scoring)
- Hour 5: Output formatting
- Hour 6: LLM integration (if time permits)
- Hour 7: Testing and polish
- Hour 8+: Stretch features

### 7.2 MVP vs Stretch Features

**Must Have (P0):**
- File/stdin input
- Keyword and regex detection
- Risk scoring with explanation
- Human and JSON output

**Should Have (P1):**
- LLM verdict integration
- Rule management commands

**Nice to Have (P2):**
- Streaming/tail mode
- Result caching
- Custom rule sets

### 7.3 When to Cut Scope

**Signs you should simplify:**
- Implementation taking >2x estimated time
- Complex abstractions emerging
- Blocking other critical features
- Unclear requirements

**How to simplify:**
- Return to PRD and confirm P0 features
- Stub out optional features
- Use hardcoded values for configurability
- Defer error handling refinements

---

## 8. Common Patterns & Examples

### 8.1 Rule Loading

```rust
use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct Rule {
    pub id: String,
    pub pattern: String,
    pub weight: f32,
}

fn load_rules(path: &str) -> Result<Vec<Rule>> {
    let content = fs::read_to_string(path)?;
    let rules: Vec<Rule> = serde_json::from_str(&content)?;
    Ok(rules)
}
```

### 8.2 Pattern Matching

```rust
use aho_corasick::AhoCorasick;
use regex::Regex;

pub fn find_keywords(text: &str, keywords: &[String]) -> Vec<Finding> {
    let ac = AhoCorasick::new(keywords).unwrap();
    ac.find_iter(text)
        .map(|mat| Finding {
            span: (mat.start(), mat.end()),
            excerpt: text[mat.start()..mat.end()].to_string(),
            rule_id: keywords[mat.pattern()].clone(),
            weight: 10.0,
        })
        .collect()
}
```

### 8.3 Score Calculation

```rust
pub fn risk_score(findings: &[Finding], text_len: usize) -> f32 {
    let base_score: f32 = findings.iter().map(|f| f.weight).sum();
    let len_norm = (text_len as f32 / 800.0).clamp(0.5, 1.5);
    (base_score * len_norm).clamp(0.0, 100.0)
}
```

### 8.4 JSON Output

```rust
use serde_json;

fn output_json(report: &ScanReport) -> Result<()> {
    let json = serde_json::to_string_pretty(report)?;
    println!("{}", json);
    Ok(())
}
```

---

## 9. Interaction Guidelines

### 9.1 Responding to Requests

**When asked to implement a feature:**
1. Confirm understanding of requirements
2. Outline your approach briefly
3. Implement incrementally
4. Test with sample data
5. Report completion with usage example

**When asked to fix a bug:**
1. Reproduce the issue if possible
2. Identify root cause
3. Propose fix approach
4. Implement with test coverage
5. Verify fix resolves the issue

**When asked to refactor:**
1. Understand motivation for refactoring
2. Ensure tests exist first
3. Make changes incrementally
4. Verify tests still pass
5. Document significant changes

### 9.2 Asking for Clarification

**Ask when:**
- Requirements are ambiguous
- Multiple implementation approaches exist
- Feature conflicts with existing code
- Scope seems too large for timeframe
- You need test data or examples

**Example clarifying questions:**
- "Should the `--json` flag suppress human-readable output entirely?"
- "What should happen when both stdin and --file are provided?"
- "Should we fail fast on invalid rules, or skip and warn?"

### 9.3 Reporting Progress

**Good progress updates:**
- "✓ Implemented keyword matching with Aho-Corasick"
- "✓ Added risk scoring with length normalization"
- "⚠ LLM integration needs API key in environment"
- "⏸ Blocked: Need sample regex patterns for testing"

---

## 10. Quality Checklist

Before marking a feature complete, verify:

- [ ] Code compiles without warnings
- [ ] `cargo fmt` applied
- [ ] `cargo clippy` passes
- [ ] Unit tests written and passing
- [ ] Error cases handled (not just happy path)
- [ ] Documentation/comments for non-obvious logic
- [ ] Feature matches PRD requirements
- [ ] Manual testing with sample inputs completed

---

## 11. Reference Quick Links

### Key Documents
- **PRD:** `LLM-Guard.md` - Complete product specification
- **Plan:** `PLAN.md` - Implementation roadmap
- **README:** `README.md` - User documentation

### External Resources
- [Rust Book](https://doc.rust-lang.org/book/)
- [Clap Documentation](https://docs.rs/clap/)
- [Aho-Corasick Crate](https://docs.rs/aho-corasick/)
- [Serde Guide](https://serde.rs/)
- [OWASP LLM Top 10](https://owasp.org/www-project-top-10-for-large-language-model-applications/)

### Rust Cheat Sheets
- **Result handling:** `?` operator, `.context()`, `.map_err()`
- **Iterators:** `.map()`, `.filter()`, `.collect()`, `.fold()`
- **String types:** `String` (owned), `&str` (borrowed)
- **Collections:** `Vec<T>`, `HashMap<K,V>`, `HashSet<T>`

---

## 12. Troubleshooting Common Issues

### Compilation Errors

**"cannot find type X in this scope"**
→ Add missing `use` statement or check spelling

**"trait bound not satisfied"**
→ Add required trait derivation or import

**"cannot borrow as mutable"**
→ Make variable mutable with `let mut` or redesign

### Runtime Issues

**"thread panicked at unwrap()"**
→ Replace `.unwrap()` with proper error handling

**"No such file or directory"**
→ Check paths are relative to project root

**"command not found"**
→ Use `cargo run --` before CLI arguments

### Testing Issues

**Tests pass individually but fail together**
→ Check for shared state or race conditions

**JSON serialization fails**
→ Ensure all types have `#[derive(Serialize)]`

---

## 13. Success Criteria

Your contribution is successful when:

1. **Functional:** Feature works as specified in PRD
2. **Tested:** Core logic has unit test coverage
3. **Clean:** Code passes `fmt` and `clippy`
4. **Documented:** Non-obvious decisions explained
5. **Integrated:** Works with existing codebase
6. **Demonstrated:** Can show working example

---

## 14. Final Reminders

### Development Mindset
- **Iterate quickly:** Working code beats perfect code
- **Test early:** Don't wait until "feature complete"
- **Keep it simple:** Complexity is the enemy of velocity
- **Ask questions:** Clarify before implementing

### Code Quality
- **Explicit over clever:** Readable code is maintainable code
- **Errors matter:** Handle them properly even in hackathons
- **Types are documentation:** Use them to communicate intent
- **Tests are safety nets:** Write them for peace of mind

### Collaboration
- **Communicate clearly:** Say what you're doing and why
- **Show your work:** Commit incrementally
- **Respect time constraints:** Know when to cut scope
- **Learn and adapt:** Hackathons are learning experiences

---

**Ready to code!** Review the PRD (`LLM-Guard.md`) and dive into the implementation. When in doubt, ask questions. Good luck! 🚀
