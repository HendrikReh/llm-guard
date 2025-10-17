# Architecture Decision Records (ADRs)

This directory contains Architecture Decision Records (ADRs) documenting significant architectural and design decisions made during the development of LLM-Guard.

## What is an ADR?

An ADR is a document that captures an important architectural decision along with its context and consequences. ADRs help:
- Preserve the reasoning behind decisions
- Onboard new team members (including AI agents!)
- Revisit decisions when context changes
- Learn from past choices

## Format

We use a lightweight ADR format with these sections:
- **Context:** What problem are we solving?
- **Decision:** What did we decide?
- **Consequences:** What are the implications?
- **Alternatives:** What else did we consider?

See [`0000-adr-template.md`](./0000-adr-template.md) for the full template.

## ADR Index

### Active Decisions

- [**ADR-0001:**](./0001-heuristic-risk-scoring.md) **Heuristic-Based Risk Scoring Algorithm**
  - Weighted rule-based scoring with length normalization and synergy bonuses
  - Chosen for transparency and explainability over ML approaches
  - Date: 2025-10-17 | Status: Accepted

- [**ADR-0002:**](./0002-workspace-architecture.md) **Cargo Workspace with Core/CLI Separation**
  - Two-crate structure: `llm-guard-core` (library) + `llm-guard-cli` (binary)
  - Enables library reuse while maintaining clear boundaries
  - Date: 2025-10-17 | Status: Accepted

- [**ADR-0003:**](./0003-optional-llm-integration.md) **Optional LLM-Powered Analysis with Multi-Provider Support**
  - Opt-in LLM verdicts via `--with-llm` flag
  - Supports OpenAI, Anthropic, Gemini, Azure OpenAI, and noop providers
  - Future migration to `rig-core` planned (Phase 9)
  - Date: 2025-10-17 | Status: Accepted

- [**ADR-0004:**](./0004-aho-corasick-regex-detection.md) **Aho-Corasick for Keywords, Regex for Complex Patterns**
  - Dual-engine approach: Aho-Corasick for exact keywords, Regex for complex patterns
  - Optimizes performance while maintaining expressiveness
  - Date: 2025-10-17 | Status: Accepted

### Proposed Decisions

_No proposed decisions at this time._

### Superseded/Deprecated Decisions

_No superseded decisions at this time._

## AI-Assisted Decision Making

This project was developed using AI-assisted workflows. Many ADRs document collaboration between:
- **Human developer** (Hendrik Reh)
- **GPT-5 Codex** (via Codex CLI) - primary implementation agent
- **Claude Code** - architecture review and documentation agent

Each ADR includes an "AI Collaboration" field noting which agents participated in the decision.

## Creating a New ADR

1. Copy `0000-adr-template.md` to `XXXX-short-title.md` (increment number)
2. Fill out all sections
3. Note which AI agents (if any) were involved in the decision
4. Update this README index
5. Reference the ADR from relevant code/docs

## Related Documentation

- [`../../PRD.md`](../../PRD.md) - Product Requirements Document
- [`../../PLAN.md`](../../PLAN.md) - Implementation roadmap
- [`../../AGENTS.md`](../../AGENTS.md) - AI agent onboarding guide
- [`../../README.md`](../../README.md) - Project overview

## References

- [Michael Nygard's ADR format](https://cognitect.com/blog/2011/11/15/documenting-architecture-decisions)
- [ADR GitHub Organization](https://adr.github.io/)
