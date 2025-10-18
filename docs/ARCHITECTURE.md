# LLM-Guard Architecture Overview

This document summarises the moving pieces that turn raw prompt text into an actionable risk report. It complements the detailed ADRs under `docs/ADR/`.

## High-Level Flow

```
        ┌────────────┐
        │ CLI Inputs │  (stdin | file | --tail)
        └─────┬──────┘
              │ UTF-8, <= 1 MB enforced
              ▼
┌────────────────────────┐
│ llm-guard-core::scanner│
│ - RuleRepository       │───┐  loads from rules/keywords.txt & patterns.json
│ - DefaultScanner       │   │
└────────┬───────────────┘   │
         │ findings (rule id, span, excerpt, weight)
         ▼                   │
┌────────────────────────┐   │
│ Risk Scoring           │   │
│ - family dampening     │   │
│ - length normalisation │   │
└────────┬───────────────┘   │
         │ ScoreBreakdown    │
         ▼                   │
┌────────────────────────┐   │
│ Optional LLM verdict   │<──┘ (OpenAI / Anthropic / Gemini / Azure via rig.rs)
└────────┬───────────────┘
         │
         ▼
┌────────────────────────┐
│ Reporting              │
│ - Human / JSON output  │
│ - Exit code 0/2/3      │
└────────────────────────┘
```

## Crate Responsibilities

| Location                              | Responsibility             
|---------------------------------------|-----------------------------------------------------------------------------------------------
| `crates/llm-guard-cli`                | Parses CLI flags, enforces the 1 MB input budget, tails files, and orchestrates health checks.
| `crates/llm-guard-core/src/scanner`   | Normalises rule data, performs keyword/regex scanning, and scores findings.
| `crates/llm-guard-core/src/report.rs` | Renders human-readable and JSON summaries, including the risk band.
| `crates/llm-guard-core/src/llm`       | Handles provider-specific verdict enrichment and rig.rs integration.

## Key Design Choices

- **Rule Repositories** — The `RuleRepository` trait lets us source detection rules from files, memory, or a remote service without touching scanner internals. `FileRuleRepository` caches parsed rules with `OnceCell`.
- **Scoring Heuristics** — `ScoreBreakdown` tracks raw vs. adjusted weight, a per-family contribution list, and the length normalisation factor. A dampening factor (default `0.5`) reduces the impact of repeated hits in the same rule family.
- **1 MB Input Guardrail** — The CLI streams both stdin and files in 8 KB chunks, rejecting oversize or non-UTF-8 data early. Tail mode uses the same helper to avoid duplicating logic.
- **LLM Verdict Handling** — The rig adapter standardises retries, prompt shaping, and JSON coercion. Providers that misbehave fall back to an `"unknown"` label rather than failing the scan.
- **Exit Codes** — Risk bands map to `0` (low), `2` (medium), and `3` (high). CI/CD pipelines can gate deployments by capturing these codes.

## File Structure Cheatsheet

```
crates/
  llm-guard-cli/
    src/main.rs           # CLI entrypoint
    tests/health.rs       # Provider health integration test
  llm-guard-core/
    src/scanner/mod.rs    # Domain model & traits
    src/scanner/default_scanner.rs
    src/report.rs
    src/llm/              # Providers and settings
rules/
  keywords.txt            # pipe-delimited keyword rules
  patterns.json           # JSON regex rules
docs/
  ARCHITECTURE.md         # this file
  RULE_AUTHORING.md       # rule authoring guidance
```

## Further Reading

- [ADR 0002 — Workspace Architecture](./ADR/0002-workspace-architecture.md)
- [ADR 0004 — Aho-Corasick + Regex Detection](./ADR/0004-aho-corasick-regex-detection.md)
- [docs/RULE_AUTHORING.md](./RULE_AUTHORING.md)
