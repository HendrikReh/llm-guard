# LLM-Guard Implementation Plan

Plan owner: hendrik.reh@outlook.com • Last updated: 2025-10-17

Status key: `[ ]` not started • `[~]` in progress • `[x]` done

---

## Phase 0 — Project Bootstrapping & Tooling

- [x] Scaffold Cargo workspace per `AGENTS.md` conventions (workspace `Cargo.toml`, `.cargo/config.toml`, `rust-toolchain.toml`, `justfile`, lint configs).
- [x] Configure tooling aliases (`fmt`, `lint`, `test-all`, `cov`, `audit`, `deny`) and CI placeholders.
- [x] Establish repo structure (`src/`, `rules/`, `tests/`, `docs/`, `examples/`) and add `.gitignore`, `README` skeleton.
- [ ] Add pre-commit hooks or documentation for formatting/linting workflow.

## Phase 1 — Core Domain Modeling

- [x] Create `Rule`, `RuleKind`, `Finding`, `ScanReport`, and `LlmVerdict` structs in `agent-core`-style module (`src/scanner/mod.rs`), ensuring serde derives and documentation.
- [x] Define traits/ports for scanning (`Scanner`), rule sourcing (`RuleRepository`), and verdict generation (`VerdictProvider`) to keep adapters isolated.
- [x] Document invariants (e.g., weights in `0.0..=100.0`, span ordering) and add unit tests for type behavior/value guards.

## Phase 2 — Rule Loading & Data Management

- [x] Implement rule loader capable of reading `rules/keywords.txt` and `rules/patterns.json`, using `once_cell` for cache.
- [x] Add validation (duplicate IDs, invalid regex, weight bounds) with typed errors (`thiserror`).
- [x] Provide sample rule pack and guidance for extending policy packs; include regression tests for parsing.
- [x] Create CLI command or flag to list rules with metadata for operators.

## Phase 3 — Scanner Engine

- [x] Build keyword scanning with `aho-corasick` (batched patterns) and regex scanning with precompiled `RegexSet`/`Regex`.
- [x] Merge findings with context windows and excerpts, respecting length caps and redaction rules.
- [x] Ensure deterministic ordering (by severity then span) and add unit/integration tests covering overlaps, zero-width matches, and Unicode edge cases.
- [x] Expose instrumentation hooks (`tracing` spans) for debug visibility.

## Phase 4 — Risk Scoring & Rubric

- [x] Implement scoring heuristic (weight aggregation, family dampening, length normalization) per `LLM-Guard.md`.
- [x] Externalize rubric thresholds (Low/Medium/High) in config with defaults and document tuning guidance.
- [x] Add tests that cover representative finding sets and ensure scores clamp to `[0, 100]`.
- [x] Produce explainability payload (e.g., sorted findings, cumulative weights) for reporters.

## Phase 5 — Reporting & CLI

- [x] Build `report` module with human-readable (ANSI-aware) and JSON reporters; include error paths and quiet mode.
- [x] Design `cli.rs` using `clap` with subcommands/flags (`--input`, `--tail`, `--with-llm`, `--json`).
- [x] Implement input readers (stdin, file, optional live tail stub) with streaming support and size limits.
- [x] Wire CLI to scanner pipeline, ensuring graceful exit codes (0 safe, 2 medium, 3 high, 1 error) for CI.

## Phase 6 — Optional LLM Adapter

- [x] Define `LlmClient` trait with async interface and implement Codex adapter (feature-gated, requires API key).
- [x] Add request shaping (prompt template, truncation) and response parsing with guardrails/timeouts.
- [x] Provide dry-run/mock adapter for tests; record usage metrics (latency, token count logging via `tracing`).
- [ ] Update CLI flag `--with-llm` to call adapter and merge verdict into `ScanReport`.

## Phase 7 — Quality Engineering

- [ ] Establish unit, integration (`tests/e2e.rs`), and property-based tests (e.g., fuzz suspicious inputs) aligned with `AGENTS.md` testing pyramid.
- [ ] Configure `cargo-nextest`, coverage (`llvm-cov`), and CI tasks (fmt, lint, test, deny, audit).
- [ ] Add fixture corpus for common jailbreak patterns and regression cases; automate through snapshot tests (`insta`).
- [ ] Document security posture (timeouts, redactions) and add assertions preventing panic paths.

## Phase 8 — Documentation, DX, and Release Prep

- [ ] Expand `README.md` with usage guide, risk rubric, demo script, and troubleshooting.
- [ ] Add `docs/` entries (architecture overview, rule authoring how-to) and ADR for heuristic design.
- [ ] Provide `examples/` (safe, suspicious, malicious sample files) and scripted demo.
- [ ] Prepare release checklist (versioning, changelog, policy pack publishing) and note future stretch goals (policy packs, sanitization, feedback loop).

---

## Cross-Cutting Concerns & Tracking

- Observability: integrate `tracing`, structured logging, metrics hooks during Phases 3–6.
- Security: enforce input size limits, redact sensitive data, avoid logging raw prompts (unless `--debug`).
- Configuration: centralize settings with `config` crate, environment overrides, `.env.example`.
- Performance: profile scanner against large prompts post-Phase 3; capture benchmarks (`criterion`) before release.
- Compliance: ensure data files (rules) load under permissive license and document update cadence.

Review this plan before starting each phase; update statuses and add sub-tasks as work progresses.
