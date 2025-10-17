# LLM-Guard Implementation Plan

> **AI Coding Hackathon Project** | Real-time development progress tracker

**Plan owner:** hendrik.reh@outlook.com
**Last updated:** 2025-10-17
**Context:** [AI Coding Accelerator](https://maven.com/nila/ai-coding-accelerator) hackathon

**Status key:** `[ ]` not started • `[~]` in progress • `[x]` done

---

## About This Plan

This living document tracks implementation progress for the LLM-Guard project, developed through AI-assisted development workflows. Each phase represents a sprint in the hackathon, with tasks designed for incremental delivery and AI collaboration.

**Related Documentation:**
- **[`PRD.md`](./PRD.md)** — Product requirements and technical specifications
- **[`AGENTS.md`](./AGENTS.md)** — AI assistant onboarding and coding conventions
- **[`README.md`](./README.md)** — Project overview and hackathon context

---

## Phase 0 — Project Bootstrapping & Tooling

**Goal:** Establish project foundation and development environment
**AI Collaboration:** Project structure design, tooling configuration

- [x] Scaffold Cargo workspace per [`AGENTS.md`](./AGENTS.md) conventions (workspace `Cargo.toml`, `.cargo/config.toml`, `rust-toolchain.toml`, `justfile`, lint configs)
- [x] Configure tooling aliases (`fmt`, `lint`, `test-all`, `cov`, `audit`, `deny`) and CI placeholders
- [x] Establish repo structure (`src/`, `rules/`, `tests/`, `docs/`, `examples/`) and add `.gitignore`, `README` skeleton
- [ ] Add pre-commit hooks or documentation for formatting/linting workflow

## Phase 1 — Core Domain Modeling

**Goal:** Define type system and domain contracts
**AI Collaboration:** Data model design, trait definitions, documentation

- [x] Create `Rule`, `RuleKind`, `Finding`, `ScanReport`, and `LlmVerdict` structs in `agent-core`-style module (`src/scanner/mod.rs`), ensuring serde derives and documentation
- [x] Define traits/ports for scanning (`Scanner`), rule sourcing (`RuleRepository`), and verdict generation (`VerdictProvider`) to keep adapters isolated
- [x] Document invariants (e.g., weights in `0.0..=100.0`, span ordering) and add unit tests for type behavior/value guards

## Phase 2 — Rule Loading & Data Management

**Goal:** Build rule loading infrastructure
**AI Collaboration:** File parsing logic, validation, error handling

- [x] Implement rule loader capable of reading `rules/keywords.txt` and `rules/patterns.json`, using `once_cell` for cache
- [x] Add validation (duplicate IDs, invalid regex, weight bounds) with typed errors (`thiserror`)
- [x] Provide sample rule pack and guidance for extending policy packs; include regression tests for parsing
- [x] Create CLI command or flag to list rules with metadata for operators

## Phase 3 — Scanner Engine

**Goal:** Implement core detection engine
**AI Collaboration:** Pattern matching algorithms, finding generation, test coverage

- [x] Build keyword scanning with `aho-corasick` (batched patterns) and regex scanning with precompiled `RegexSet`/`Regex`
- [x] Merge findings with context windows and excerpts, respecting length caps and redaction rules
- [x] Ensure deterministic ordering (by severity then span) and add unit/integration tests covering overlaps, zero-width matches, and Unicode edge cases
- [x] Expose instrumentation hooks (`tracing` spans) for debug visibility

## Phase 4 — Risk Scoring & Rubric

**Goal:** Implement transparent risk scoring
**AI Collaboration:** Heuristic algorithms, explainability, test scenarios

- [x] Implement scoring heuristic (weight aggregation, family dampening, length normalization) per [`PRD.md`](./PRD.md)
- [x] Externalize rubric thresholds (Low/Medium/High) in config with defaults and document tuning guidance
- [x] Add tests that cover representative finding sets and ensure scores clamp to `[0, 100]`
- [x] Produce explainability payload (e.g., sorted findings, cumulative weights) for reporters

## Phase 5 — Reporting & CLI

**Goal:** Build user-facing interface
**AI Collaboration:** CLI design, output formatting, input handling

- [x] Build `report` module with human-readable (ANSI-aware) and JSON reporters; include error paths and quiet mode
- [x] Design `cli.rs` using `clap` with subcommands/flags (`--input`, `--tail`, `--with-llm`, `--json`)
- [x] Implement input readers (stdin, file, optional live tail stub) with streaming support and size limits
- [x] Wire CLI to scanner pipeline, ensuring graceful exit codes (0 safe, 2 medium, 3 high, 1 error) for CI

## Phase 6 — Optional LLM Adapter

**Goal:** Integrate external LLM analysis
**AI Collaboration:** API client implementation, prompt engineering, error handling

- [x] Define `LlmClient` trait with async interface and implement OpenAI adapter (feature-gated, requires API key)
- [x] Add request shaping (prompt template, truncation) and response parsing with guardrails/timeouts
- [x] Provide dry-run/mock adapter for tests; record usage metrics (latency, token count logging via `tracing`)
- [x] Update CLI flag `--with-llm` to call adapter, support tail streaming, and merge verdicts into `ScanReport`
- [ ] Add additional providers (e.g., Anthropic, Azure OpenAI, local models) using the `LlmClient` interface

## Phase 7 — Quality Engineering

**Goal:** Comprehensive testing and validation
**AI Collaboration:** Test generation, edge case identification, CI configuration

- [ ] Establish unit, integration (`tests/e2e.rs`), and property-based tests (e.g., fuzz suspicious inputs) aligned with [`AGENTS.md`](./AGENTS.md) testing pyramid
- [ ] Configure `cargo-nextest`, coverage (`llvm-cov`), and CI tasks (fmt, lint, test, deny, audit)
- [ ] Add fixture corpus for common jailbreak patterns and regression cases; automate through snapshot tests (`insta`)
- [ ] Document security posture (timeouts, redactions) and add assertions preventing panic paths

## Phase 8 — Documentation, DX, and Release Prep

**Goal:** Polish and prepare for wider use
**AI Collaboration:** Documentation writing, example creation, demo scripting

- [ ] Expand [`README.md`](./README.md) with usage guide, risk rubric, demo script, and troubleshooting
- [ ] Add `docs/` entries (architecture overview, rule authoring how-to) and ADR for heuristic design
- [ ] Provide `examples/` (safe, suspicious, malicious sample files) and scripted demo
- [ ] Prepare release checklist (versioning, changelog, policy pack publishing) and note future stretch goals (policy packs, sanitization, feedback loop)

## Phase 9 — Migration to `rig.rs`

**Goal:** Transition multi-provider orchestration to [`rig.rs`](https://rig.rs/)
**AI Collaboration:** Adapter refactor, validation, regression testing

- [~] Replace existing LLM adapter wiring with rig.rs (OpenAI now routes through the rig adapter)
- [ ] Map current provider implementations (Anthropic, Gemini, Azure, noop) into rig.rs abstractions
- [ ] Ensure configuration precedence (config → env → flags) is preserved via rig.rs
- [ ] Update CLI tests and documentation to reflect the new runtime

---

## Cross-Cutting Concerns & Tracking

**Observability:**
- Integrate `tracing`, structured logging, metrics hooks during Phases 3–6
- AI assistants help identify appropriate logging levels and span boundaries

**Security:**
- Enforce input size limits, redact sensitive data, avoid logging raw prompts (unless `--debug`)
- Human review required for all security-critical paths

**Configuration:**
- Centralize settings with `config` crate, environment overrides, `.env.example`
- AI collaboration on configuration schema design

**Performance:**
- Profile scanner against large prompts post-Phase 3; capture benchmarks (`criterion`) before release
- AI-assisted benchmark design and optimization suggestions

**Compliance:**
- Ensure data files (rules) load under permissive license and document update cadence
- Human oversight for licensing and compliance decisions

---

## Development Notes

**AI-Assisted Development Process:**
- Each phase includes "AI Collaboration" notes indicating where AI assistants contributed
- Human review required for: architecture decisions, security logic, testing strategy
- AI assistants excel at: boilerplate generation, documentation, pattern application, test scaffolding

**Phase Completion Criteria:**
- All tasks marked `[x]` with working code
- Tests passing for completed features
- Documentation updated in relevant files
- Changes reviewed by human developer

**Using This Plan:**
1. Review this plan before starting each phase
2. Update statuses (`[ ]` → `[~]` → `[x]`) as work progresses
3. Add sub-tasks or notes when scope changes
4. Reference related documentation ([`PRD.md`](./PRD.md), [`AGENTS.md`](./AGENTS.md)) for details
5. Mark AI collaboration points to track human-AI workflow patterns
