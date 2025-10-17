# ADR-0002: Cargo Workspace with Core/CLI Separation

**Status:** Accepted
**Date:** 2025-10-17
**Deciders:** Hendrik Reh, GPT-5 Codex
**AI Collaboration:** Architecture discussion with Claude Code; implementation with GPT-5 Codex

## Context

LLM-Guard needs a clear separation between:
1. **Core detection logic** (scanning, scoring, rules) - reusable as a library
2. **CLI interface** (argument parsing, I/O, reporting) - end-user tool

We want to enable future use cases:
- Embedding as a library in other Rust applications
- Potential web service wrapper
- Testing core logic independently of CLI

We need to decide on the project structure that best supports these goals while maintaining simplicity for a hackathon timeframe.

## Decision

Use a **Cargo workspace with two crates**:

```
llm-guard/
├── Cargo.toml              # Workspace root
├── crates/
│   ├── llm-guard-core/     # Library: core detection engine
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── scanner/    # Rule loading, pattern matching
│   │       ├── scoring/    # Heuristics, risk calculation
│   │       └── types.rs    # Rule, Finding, ScanReport
│   └── llm-guard-cli/      # Binary: CLI wrapper
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs
│           ├── cli.rs      # Clap argument parsing
│           ├── input.rs    # Stdin/file/tail readers
│           ├── output.rs   # Formatters (human/JSON)
│           └── llm.rs      # LLM adapter integration
```

### Boundaries

**llm-guard-core responsibilities:**
- Rule loading and validation
- Text scanning with Aho-Corasick and regex
- Finding generation and scoring
- Core types (Rule, Finding, ScanReport)
- No I/O, CLI, or LLM integration

**llm-guard-cli responsibilities:**
- CLI argument parsing with Clap
- Input handling (stdin, file, tail)
- Output formatting (human-readable, JSON)
- LLM adapter orchestration
- Exit code handling for CI/CD

## Consequences

### Positive
- **Library reusability:** Core can be embedded in other Rust projects
- **Clear boundaries:** I/O and CLI concerns separated from detection logic
- **Testability:** Core logic testable without CLI infrastructure
- **Future flexibility:** Easy to add web service or other interfaces later
- **Dependency hygiene:** CLI-specific deps (clap, colored) don't pollute core

### Negative
- **Initial complexity:** Two crates vs single binary adds setup overhead
- **Coordination:** Changes may span both crates
- **Build time:** Slightly longer than monolithic structure
- **Learning curve:** New contributors need to understand workspace structure

### Neutral
- **Code organization:** More files/dirs but clearer responsibilities
- **Workspace dependencies:** Shared deps managed at workspace level

## Alternatives Considered

### Option 1: Single Binary Crate
- **Pros:** Simpler setup, faster initial development, fewer files
- **Cons:** No library reuse, harder to test core logic, tight coupling
- **Why rejected:** Limits future use cases, harder to maintain boundaries

### Option 2: Three Crates (core, cli, llm-adapter)
- **Pros:** Even finer-grained separation, LLM logic fully isolated
- **Cons:** Over-engineering for v1, more coordination overhead
- **Why rejected:** YAGNI for hackathon scope; can refactor later if needed

### Option 3: Bin/Lib in Single Crate
- **Pros:** One crate, still exportable as library
- **Cons:** Harder to enforce boundaries, all deps pulled in together
- **Why rejected:** Doesn't enforce clean separation adequately

## Implementation Notes

### Workspace Configuration
- Shared dependencies in `[workspace.dependencies]`
- Common metadata (edition, license) in `[workspace.package]`
- Resolver = "2" for modern dependency resolution

### CLI Depends on Core
```toml
# crates/llm-guard-cli/Cargo.toml
[dependencies]
llm-guard-core = { path = "../llm-guard-core" }
```

### Public API Surface
Core exposes:
- `scan_text(text: &str, rules: &[Rule]) -> ScanReport`
- `load_rules(path: &Path) -> Result<Vec<Rule>>`
- `calculate_risk(findings: &[Finding]) -> f32`

### Migration Path
If we later need finer separation (e.g., `llm-guard-adapters`), we can extract LLM code from CLI into a third crate without changing core.

## References

- **PRD Section 2.2:** Success Criteria #5 (Extensibility)
- **PRD Section 5.1:** System Architecture diagram
- **AGENTS.md:** Workspace layout conventions
- **PLAN.md Phase 0:** Project bootstrapping and structure
