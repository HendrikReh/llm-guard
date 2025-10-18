# LLM-Guard — Prompt Injection Firewall (PoC)

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](./LICENSE)
[![Rust Version](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://www.rust-lang.org)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](https://github.com/HendrikReh/llm-guard/pulls)
[![AI Coding Hackathon](https://img.shields.io/badge/AI%20Coding-Hackathon-purple)](https://maven.com/nila/ai-coding-accelerator)
[![Proof of Concept](https://img.shields.io/badge/status-Proof_of_Concept-yellow)](https://github.com/HendrikReh/llm-guard)

**LLM Provider Support:**
[![OpenAI](https://img.shields.io/badge/OpenAI-412991?logo=openai&logoColor=white)](https://openai.com)
[![Anthropic](https://img.shields.io/badge/Anthropic-191919?logo=anthropic&logoColor=white)](https://anthropic.com)
[![Google Gemini](https://img.shields.io/badge/Google_Gemini-4285F4?logo=google&logoColor=white)](https://ai.google.dev)
[![Azure OpenAI](https://img.shields.io/badge/Azure_OpenAI-0078D4?logo=microsoftazure&logoColor=white)](https://azure.microsoft.com/products/ai-services/openai-service)
[![Noop](https://img.shields.io/badge/Noop-simulator-lightgrey)](#configuration)

**Built With AI Tools:**
[![Cursor](https://img.shields.io/badge/Cursor-000000?logo=visual-studio-code&logoColor=white)](https://cursor.sh)
[![Claude Code](https://img.shields.io/badge/Claude_Code-191919?logo=anthropic&logoColor=white)](https://claude.ai)
[![Codex CLI](https://img.shields.io/badge/Codex_CLI-412991?logo=openai&logoColor=white)](https://github.com/openai/codex-cli)
[![RepoPrompt MCP](https://img.shields.io/badge/RepoPrompt-MCP-orange)](https://repoprompt.com/)
[![Context7 MCP](https://img.shields.io/badge/Context7-MCP-orange)](https://context7.com/)

> **AI Coding Hackathon Project** · Built collaboratively by a human — acting as architect, tester, product manager — and AI agents during Maven’s Accelerator. The result is a fast, explainable Rust CLI that scans prompts for jailbreak indicators, clamps scores with a transparent rubric, and now adds a configurable input guardrail plus optional LLM verdicts.

## Table of Contents

- [Quick Start](#quick-start)
  - [Installation](#installation)
  - [Quality Checks](#quality-checks)
  - [Usage Examples](#usage-examples)
  - [Configuration](#configuration)
  - [Troubleshooting](#troubleshooting)
- [Features](#features)
  - [Core Capabilities](#core-capabilities)
  - [Detection Coverage](#detection-coverage)
- [Technical Overview](#technical-overview)
  - [Architecture](#architecture)
  - [How It Works](#how-it-works)
  - [Risk Rubric](#risk-rubric)
  - [Core Dependencies](#core-dependencies)
- [Project Status](#project-status)
- [Project Documentation](#project-documentation)
- [Hackathon Context](#hackathon-context)
  - [Goals of This Hackathon](#goals-of-this-hackathon)
  - [What Makes This Project Different](#what-makes-this-project-different)
- [AI-Assisted Development Insights](#ai-assisted-development-insights)
  - [Workflow Highlights](#workflow-highlights)
  - [Collaboration Lessons](#collaboration-lessons)
- [Provider Integration Pitfalls & Fixes](#provider-integration-pitfalls--fixes)
- [Contributing](#contributing)
- [License & Disclaimer](#license--disclaimer)
  - [License](#license)
  - [Security Disclaimer](#security-disclaimer)
  - [AI Development Notice](#ai-development-notice)
- [Acknowledgments](#acknowledgments)

## Quick Start

### Installation

```bash
# Clone the workspace and build the CLI (debug build by default)
git clone https://github.com/HendrikReh/llm-guard
cd llm-guard
cargo build --workspace

# Optional: build an optimized binary
cargo build -p llm-guard-cli --release

# Run the optimized binary directly
./target/release/llm-guard-cli --help

# Or install the CLI binary into ~/.cargo/bin
cargo install --path crates/llm-guard-cli
```

The compiled binary is named `llm-guard-cli`. After `cargo install`, invoke it via `llm-guard-cli` or create an alias (`alias llm-guard=llm-guard-cli`) if you prefer the shorter name used in examples below.

### Quality Checks

```bash
cargo fmt --all               # Format source code
cargo lint                    # Clippy (alias from .cargo/config.toml)
cargo test-all                # Run tests for all crates and features
cargo cov                     # HTML coverage report (requires cargo-llvm-cov)
just test                     # Uses cargo-nextest when available, falls back to cargo test
```

CI is configured through `.github/workflows/ci.yml` to run the same checks on pull requests.

### Usage Examples

```bash
# Scan a file (reads stdin when --file is omitted)
./target/debug/llm-guard-cli scan --file samples/chat.txt

# Pipe input from another command
echo "Ignore previous instructions" | ./target/debug/llm-guard-cli scan

# Generate JSON output for automation
./target/debug/llm-guard-cli scan --file samples/chat.txt --json > report.json

# Augment with an LLM verdict (requires provider credentials)
LLM_GUARD_PROVIDER=openai \
LLM_GUARD_API_KEY=sk-... \
LLM_GUARD_MODEL=gpt-4o-mini \
./target/debug/llm-guard-cli scan --file samples/chat.txt --with-llm

# Switch providers via CLI overrides (values take precedence over env/config)
./target/debug/llm-guard-cli scan --file samples/chat.txt --with-llm \
  --provider anthropic --model claude-3-haiku-20240307

# Tail a log file and scan new content as it arrives
./target/debug/llm-guard-cli scan --file logs/chat.log --tail

# Increase the input budget to 2 MB for large transcripts
./target/debug/llm-guard-cli --max-input-bytes 2000000 scan --file transcripts/long.txt

# Run health diagnostics for a specific provider
./target/debug/llm-guard-cli --debug health --provider openai
```

> **Input size:** `llm-guard-cli` enforces a 1 MB (1,000,000 byte) cap per input. Tail mode and stdin use the same guard to avoid runaway memory usage. Override it with `--max-input-bytes` or `LLM_GUARD_MAX_INPUT_BYTES` when you deliberately need to scan larger corpora.

**Sample output:**

```
Risk: 72/100 (HIGH)

Findings:
  [INSTR_OVERRIDE] "ignore previous instructions" at 0..29  (+16)
  [PROMPT_LEAK]    "reveal system prompt" at 45..65        (+14)

Synergy bonus (override+leak within 200 chars)              (+5)
```

Exit codes: `0` (low), `2` (medium), `3` (high), `1` (error). Integrate the CLI into CI/CD pipelines by acting on those codes.
Sample prompts live under `examples/prompt_safe.txt`, `examples/prompt_suspicious.txt`, and `examples/prompt_malicious.txt` for quick demos.

### Configuration

Environment variables provide the quickest way to configure LLM access:

| Variable | Description | Default |
|----------|-------------|---------|
| `LLM_GUARD_PROVIDER` | Provider (`openai`, `anthropic`, `gemini`, `azure`, `noop`) | `openai` |
| `LLM_GUARD_API_KEY` | API key/token (required unless provider=`noop`) | – |
| `LLM_GUARD_ENDPOINT` | Custom endpoint/base URL | Provider default |
| `LLM_GUARD_MODEL` | Model identifier (`gpt-4o-mini`, `claude-3-haiku-20240307`, …) | Provider default |
| `LLM_GUARD_DEPLOYMENT` | Deployment name (Azure rig profiles) | – |
| `LLM_GUARD_PROJECT` | Project or tenant identifier (Anthropic, Gemini) | – |
| `LLM_GUARD_WORKSPACE` | Workspace identifier when required | – |
| `LLM_GUARD_TIMEOUT_SECS` | HTTP timeout in seconds | `30` |
| `LLM_GUARD_MAX_RETRIES` | Retry attempts for failed calls | `2` |
| `LLM_GUARD_API_VERSION` | API version (Azure OpenAI) | Provider default |
| `LLM_GUARD_MAX_INPUT_BYTES` | Max bytes accepted from stdin/files | `1_000_000` |

Configuration precedence: CLI flags → environment variables → profile in `llm_providers.yaml`.

`llm_providers.yaml` lets you manage multiple providers side-by-side:

```yaml
providers:
  - name: openai
    api_key: OPENAI_API_KEY
    model: gpt-4o-mini
  - name: azure
    api_key: AZURE_OPENAI_KEY
    endpoint: https://your-resource.openai.azure.com
    deployment: gpt-4o-production
    api_version: 2024-02-15-preview
    timeout_secs: 60
    max_retries: 3
```

Override the location with `--providers-config`. You can also prime the environment from a `.env` file:

```bash
set -a && source .env && set +a
cargo run -p llm-guard-cli -- scan --file samples/chat.txt --with-llm
```

Single-provider setups may prefer a TOML config file:

```toml
# llm-config.toml
[llm]
provider = "anthropic"
model = "claude-3-haiku-20240307"
timeout_secs = 45
max_retries = 3
```

```bash
cargo run -p llm-guard-cli -- --config llm-config.toml scan --with-llm --file prompt.txt
```

### Troubleshooting

- **`input exceeds 1000000 bytes`** — The scanner rejects inputs larger than 1 MB. Remove unnecessary context, chunk long transcripts, tail a filtered log, or raise the cap with `--max-input-bytes`.
- **`input contains invalid UTF-8`** — Inputs must be UTF-8. Re-encode your file (`iconv -f utf-16 -t utf-8 ...`) or ensure pipelines emit UTF-8 text.
- **Tail mode shows no updates** — The watcher only prints when file content changes. Confirm your log writer writes the entire file each update or adjust intervals via `--tail` plus periodic `sleep`.
- **LLM verdict is `unknown`** — Providers may return blank or malformed payloads; the CLI falls back to an informative placeholder. Re-run with `--debug` to inspect raw responses.

## Features

### Core Capabilities

- Fast Aho-Corasick and precompiled regex scanning (<100 ms for typical prompts)
- Transparent risk scoring (0–100) with rule attribution, excerpts, and synergy bonuses
- Multiple input sources: stdin, files, and tail mode for streaming logs
- Human-readable and JSON output, with machine-friendly exit codes
- Optional LLM verdicts via OpenAI, Anthropic, Google Gemini, or Azure OpenAI (plus `noop` simulator)
- Rig-backed provider health diagnostics (`health` subcommand, `--debug` raw payload logging)

### Detection Coverage

- **Instruction override:** “ignore previous instructions”, “reset system prompt”
- **Data exfiltration:** prompt leaks, hidden system prompt disclosure attempts
- **Policy subversion:** jailbreak / guardrail bypass patterns
- **Obfuscation:** base64 payloads, Unicode tricks, hex-encoded directives
- **Streaming resilience:** tail mode deduplicates unchanged snapshots and handles rapid log churn

## Technical Overview

### Architecture

```
llm-guard/
├── crates/
│   ├── llm-guard-core/
│   │   ├── src/
│   │   │   ├── scanner/        (rule repositories, scanning, scoring heuristics)
│   │   │   ├── report.rs       (human + JSON reporters)
│   │   │   └── llm/            (OpenAI, Anthropic, Azure, Gemini, rig adapter, settings)
│   │   └── Cargo.toml
│   └── llm-guard-cli/
│       ├── src/main.rs         (CLI, config loading, tail loop, provider health)
│       └── Cargo.toml
├── rules/                      (keywords + pattern packs)
├── tests/                      (integration + snapshot tests)
├── docs/                       (usage, ADRs, testing guide, screenshots)
└── examples/                   (sample prompts)
```

### How It Works

1. **Rule loading** – Keywords and regex patterns are loaded from `rules/` and validated.
2. **Scanning** – Inputs from stdin, files, or tail mode are analyzed for matches.
3. **Finding generation** – Each match carries spans, rule IDs, excerpts, and weights.
4. **Risk scoring** – Heuristic scoring aggregates weights, applies dampening, and clamps to 0–100.
5. **Reporting** – Human or JSON output summarizes findings; optional LLM verdicts enrich the report.

### Risk Rubric

| Score | Band   | Recommendation |
|-------|--------|----------------|
| 0–24  | Low    | Proceed – no prompt-injection indicators detected |
| 25–59 | Medium | Review – investigate before executing user instructions |
| 60–100| High   | Block – re-prompt or escalate for manual review |

Scores combine rule weights, dampened repeat hits, and a length normalization factor clamped to `[0.5, 1.5]`. The qualitative band drives exit codes (`0`, `2`, `3`) so you can fail CI jobs or gate automations based on the rubric.

### Core Dependencies

- `aho-corasick`, `regex` — high-performance pattern matching
- `serde`, `serde_json`, `serde_yaml`, `json5` — serialization formats
- `clap` — command-line parsing
- `tokio`, `reqwest`, `async-trait` — async runtime and HTTP clients
- `tracing`, `tracing-subscriber` — structured diagnostics
- `config`, `once_cell`, `thiserror`, `anyhow` — configuration and error handling
- `rig-core` — shared provider orchestration across OpenAI, Anthropic, and Azure adapters

## Project Status

**Current phase:** Active development; see [`PLAN.md`](./PLAN.md) for phase-by-phase progress (last updated 2025-10-17).

- ✅ Phases 0–6 complete (scanner, scoring, CLI, multi-provider LLM integration)
- ⚙️ Phase 7 hardening underway (expanded tests, fuzzing, CI polish)
- 📝 Phase 8 documentation tasks open (README refresh, usage deep-dives, release checklist)
- 🔄 Phase 9 rig.rs migration landed; final doc/test refresh still pending

**Test suite:** `cargo test --workspace --all-features` exercises 69 tests total (59 active, 10 ignored for loopback/TLS constraints). Snapshot fixtures live in `tests/scanner_snapshots.rs`.

## Project Documentation

| Document                                             | Purpose                                        | Audience 
|------------------------------------------------------|------------------------------------------------|--------------------------
| [`README.md`](./README.md)                           | Project overview, quick start, AI insights     | Everyone 
| [`docs/USAGE.md`](./docs/USAGE.md)                   | CLI reference and advanced command examples    | Operators 
| [`docs/ARCHITECTURE.md`](./docs/ARCHITECTURE.md)     | Component and data-flow overview               | Contributors 
| [`docs/RULE_AUTHORING.md`](./docs/RULE_AUTHORING.md) | How to extend keyword & regex rule packs       | Security engineers 
| [`PRD.md`](./PRD.md)                                 | Product requirements and success criteria      | Builders & reviewers 
| [`PLAN.md`](./PLAN.md)                               | Phase tracking, outstanding work, status notes | Contributors 
| [`AGENTS.md`](./AGENTS.md)                           | Onboarding guide for AI coding assistants      | AI agents & humans 
| [`docs/TESTING_GUIDE.md`](./docs/TESTING_GUIDE.md)   | Testing strategy, commands, troubleshooting    | Developers, QA 
| [`docs/SECURITY.md`](./docs/SECURITY.md)             | Security guardrails, runtime expectations      | Security reviewers 
| [`docs/RELEASE_CHECKLIST.md`](./docs/RELEASE_CHECKLIST.md) | Steps for shipping a tagged release        | Maintainers 
| [`docs/ADR/`](./docs/ADR/)                           | Architecture decision records                  | Technical stakeholders 

## Hackathon Context

This repository was created for Maven’s [AI Coding Accelerator](https://maven.com/nila/ai-coding-accelerator) hackathon as a focused experiment in AI-assisted software delivery.

### Goals of This Hackathon

1. Explore how far AI coding assistants can accelerate real-world development.
2. Validate product development workflows where AI contributes to design, implementation, and docs.
3. Produce a demonstrable prompt-injection firewall within a single-day sprint.
4. Capture lessons learned about human + AI collaboration.

### What Makes This Project Different

- **Multi-agent collaboration:** GPT-5 Codex handled most implementation; Claude Code reviewed and documented.
- **Living documentation:** [`AGENTS.md`](./AGENTS.md) lets any assistant join with full context.
- **Transparent planning:** [`PLAN.md`](./PLAN.md) logs granular progress and decisions.
- **PRD-first workflow:** [`PRD.md`](./PRD.md) governed scope and rubric changes.
- **MCP integration:** RepoPrompt and Context7 MCP servers kept AI agents aware of repository state.

## AI-Assisted Development Insights

### Workflow Highlights

- Requirements captured up front via PRD refinements between GPT-5 Codex and Claude Code.
- Cursor IDE hosted parallel terminals for Codex CLI and Claude Code, enabling rapid iteration.
- RepoPrompt MCP supplied curated repository slices, powering large-scale refactors (e.g., rig.rs migration).
- `just` recipes + cargo aliases standardized formatting, linting, testing, and coverage for every agent.
- Observability (`tracing`, debug flags) was prioritized early to simplify later provider debugging.

### Collaboration Lessons

- Pairing multiple AI agents with distinct strengths reduced blocker time; humans focused on review and direction.
- Documenting conventions (`AGENTS.md`, `docs/TESTING_GUIDE.md`) minimized context loss between agent hand-offs.
- Maintaining a living plan avoided scope creep and clarified which phases were safe to trim when time-boxed.
- Local fallbacks (noop provider, dry-run health checks) kept demos functional even when real APIs misbehaved.
- Capturing pitfalls immediately in docs or ADRs prevented repeated regressions as agents rotated tasks.

## Provider Integration Pitfalls & Fixes

- **Anthropic truncation & malformed JSON:** Added newline sanitisation, auto-repair for dangling quotes/braces, JSON5 fallback, and final "unknown" verdicts so scans never abort.
- **OpenAI reasoning-only replies:** Switched from strict `json_schema` to `json_object`, captured tool-call arguments, and fallback to "unknown" verdicts when content is withheld.
- **Gemini + rig.rs incompatibilities:** Bypassed rig for Gemini due to schema mismatches; implemented a native REST client that formats prompt/response JSON manually.
- **Gemini empty responses:** Treats empty candidates as warnings; emits "unknown" verdict with guidance instead of failing.
- **Debugging provider quirks:** Global `--debug` flag sets `LLM_GUARD_DEBUG=1` to log raw payloads whenever parsing fails.

## Contributing

Contributions that extend the experiment or harden the CLI are welcome.

- Follow the Rust conventions captured in [`AGENTS.md`](./AGENTS.md).
- Include automated tests for new logic (unit, integration, or snapshots as appropriate).
- Run `cargo fmt`, `cargo lint`, and `cargo test-all` before submitting.
- Document notable AI-assisted workflows or trade-offs in the PR description.
- For new detection rules, update `rules/`, add fixtures, and note expected risk scores.

## License & Disclaimer

### License

MIT License — see [`LICENSE`](./LICENSE) for full text.

### Security Disclaimer

⚠️ **Important:** The scanner relies on heuristics and optional LLM verdicts. False positives and false negatives are possible, and novel attacks outside the rule set will be missed.

- Treat risk scores as decision support, not absolute truth.
- Use LLM-Guard as one layer within a defence-in-depth strategy.
- Combine with input sanitisation, rate limiting, monitoring, and human review.
- Regularly refresh rule packs and review detection results for drift.

### AI Development Notice

⚠️ This codebase was built with significant AI assistance.

- GPT-5 Codex (via Codex CLI) generated most code.
- Claude Code contributed reviews, docs, and feasibility checks.
- Humans reviewed security-critical paths, tests, and release readiness.
- Perform your own review, threat modelling, and tuning before production use.

## Acknowledgments

**AI Coding Accelerator Hackathon**
- Course: [Maven’s AI Coding Accelerator](https://maven.com/nila/ai-coding-accelerator)
- Instructors: [Vignesh Mohankumar](https://x.com/vig_xyz), [Jason Liu](https://x.com/jxnlco)

**Tools & Technologies**
- AI agents: [Codex CLI (OpenAI)](https://github.com/openai/codex-cli), [Claude Code (Anthropic)](https://claude.ai)
- MCP servers: [RepoPrompt](https://repoprompt.com/), [Context7](https://context7.com/)
- IDE: [Cursor](https://cursor.sh)
- Research: [Perplexity](https://www.perplexity.ai/)
- Git client: [Tower](https://www.git-tower.com/mac)
