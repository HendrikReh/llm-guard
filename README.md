# LLM-Guard — Prompt Injection Firewall

[![Build Status](https://img.shields.io/github/actions/workflow/status/HendrikReh/llm-guard/ci.yml?branch=main)](https://github.com/HendrikReh/llm-guard/actions)
[![Crates.io](https://img.shields.io/crates/v/llm-guard)](https://crates.io/crates/llm-guard)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](./LICENSE)
[![Rust Version](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://www.rust-lang.org)
[![AI Coding Hackathon](https://img.shields.io/badge/AI%20Coding-Hackathon-purple)](https://maven.com/nila/ai-coding-accelerator)

> **AI Coding Hackathon Project** | Experimenting with AI-assisted development workflows

A fast, explainable **Rust** CLI that scans prompts and logs for **prompt-injection & jailbreak indicators**, scores the risk (0–100), and optionally asks an LLM for a short verdict and mitigation tip.

## Hackathon Context

This project was developed during the **[AI Coding Accelerator](https://maven.com/nila/ai-coding-accelerator)** hackathon (Maven) as an experiment in **AI-assisted software development**.

**Instructors:** Vignesh Mohankumar and Jason Liu

### Goals of This Hackathon

1. **Experiment with AI Coding Tools:** Explore capabilities and limitations of AI coding assistants in real-world development
2. **Test AI-Driven Development Processes:** Evaluate workflows where AI assistants contribute to architecture, implementation, and documentation
3. **Build Working Software Fast:** Create functional prototypes that demonstrate practical security tools for LLM applications
4. **Document the Journey:** Capture insights about human-AI collaboration in software engineering

### What Makes This Project Special

- **AI Pair Programming:** Core features developed in collaboration with AI coding assistants (Claude Code)
- **Living Documentation:** [`AGENTS.md`](./AGENTS.md) serves as onboarding guide for AI assistants joining the project
- **Transparent Development:** [`PLAN.md`](./PLAN.md) tracks implementation progress and decisions in real-time
- **Iterative Refinement:** Product requirements ([`PRD.md`](./PRD.md)) evolved through AI-human dialogue

> **Note:** This codebase demonstrates both the potential and practical considerations of AI-assisted development, including code quality, testing approaches, and documentation practices.

## Project Status

**Current Phase:** Active Development (See [`PLAN.md`](./PLAN.md) for detailed roadmap)

**Functional Features:**
- ✅ CLI scaffolding with Clap
- ✅ File and stdin input processing
- 🚧 Rule-based detection engine (in progress)
- ⏳ Risk scoring algorithms (planned)
- ⏳ JSON output formatting (planned)
- ⏳ LLM integration (planned Phase 6)

## Features (Target Capabilities)

- **Explainable Security:** Transparent scoring with rule hits, weights, and text excerpts
- **Fast & Lightweight:** Built in Rust with Aho-Corasick and optimized regex matching
- **CI/CD Friendly:** JSON output for automated pipelines; colorized CLI for interactive use
- **Optional LLM Analysis:** Get additional context and mitigation suggestions from language models
- **Flexible Input:** Scan files, stdin, or streaming logs

## Quick Start

### Installation

```bash
git clone https://github.com/yourname/llm-guard
cd llm-guard
cargo build --release
```

### Usage Examples

> **Note:** Command examples below represent planned functionality. See Project Status section for current implementation state.

**Basic Scanning:**

```bash
# Scan a file
./target/release/llm-guard scan --file samples/chat.txt

# Scan from stdin
echo "Ignore previous instructions" | ./target/release/llm-guard scan
```

**Output Example:**
```
Risk: 72/100  (HIGH)

Findings:
  [INSTR_OVERRIDE] "ignore previous instructions" at 0..29  (+16)
  [PROMPT_LEAK]    "reveal system prompt" at 45..65        (+14)

Synergy bonus (override+leak within 200 chars)              (+5)
```

**CI/CD Integration:**

```bash
# Generate JSON output for automated processing
./target/release/llm-guard scan --file samples/chat.txt --json > report.json

# Exit codes reflect risk level (0=low, 2=medium, 3=high, 1=error)
if [ $? -ge 2 ]; then
  echo "Security risk detected!"
fi
```

**Advanced Features:**

```bash
# Tail a log file and rescan on change (Ctrl+C to stop)
./target/release/llm-guard scan --file logs/chat.log --tail

# Request LLM-powered analysis (Phase 6)
export LLM_GUARD_PROVIDER=openai
export LLM_GUARD_API_KEY=your_key_here
export LLM_GUARD_ENDPOINT=https://api.openai.com
export LLM_GUARD_MODEL=gpt-4o-mini
./target/release/llm-guard scan --file samples/chat.txt --with-llm

# Switch provider and model via CLI overrides
./target/release/llm-guard scan --file samples/chat.txt --with-llm \
  --provider anthropic --model claude-3-haiku-20240307

# Tail a log while enriching with LLM verdicts
./target/release/llm-guard scan --file logs/chat.log --tail --with-llm

> `--with-llm` currently supports the OpenAI chat completions API (default provider), Anthropic Messages API, and Google Gemini API. Set `LLM_GUARD_PROVIDER=noop` to disable external calls while retaining heuristic output.
```

> Set `LLM_GUARD_PROVIDER=noop` to run locally without calling an external service (returns heuristic-only verdicts).

**LLM Environment Variables:**

- `LLM_GUARD_PROVIDER` — provider identifier (`openai` by default; `anthropic`, `gemini`, `noop` supported).
- `LLM_GUARD_API_KEY` — required API key/token for real providers.
- `LLM_GUARD_ENDPOINT` — optional custom endpoint/base URL (defaults to the provider's public API).
- `LLM_GUARD_MODEL` — optional model name (e.g., `gpt-4o-mini`).
- `LLM_GUARD_TIMEOUT_SECS` — optional HTTP timeout (seconds, default `30`).
- `LLM_GUARD_MAX_RETRIES` — optional retry count for failed calls (default `2`).

> Tip: If you keep credentials in a `.env` file, run `set -a && source .env` before invoking the CLI so these variables are exported.

## Technical Overview

### How It Works

1. **Rule Loading:** Loads keyword and regex-based detection rules using Aho-Corasick and `regex` crate
2. **Scanning:** Processes input from stdin, files, or streams to identify potential threats
3. **Finding Generation:** Emits detailed findings with rule matches, text spans, excerpts, and weights
4. **Risk Scoring:** Computes a normalized risk score (0–100) with length normalization and synergy bonuses
5. **LLM Analysis (Optional):** Queries an LLM for classification, rationale, and mitigation suggestions

### Architecture

```
cli (clap)
  ├─ reader (stdin | file | tail -f)
  ├─ scanner
  │    ├─ rules (regex + aho-corasick)
  │    ├─ heuristics (weights, windows, caps)
  │    └─ explain (feature attributions)
  ├─ llm_adapter (optional)
  └─ reporters (human / json)
```

**Core Dependencies:** `clap`, `regex`, `aho-corasick`, `serde`, `serde_json`, `anyhow`, `once_cell`, `humantime`, `tokio`, `tracing`, `reqwest`

### Detection Strategy

**Rule Categories:**
- Instruction override: "ignore previous", "disregard prior", "reset instructions"
- Data exfiltration: "reveal system prompt", "show hidden", "leak"
- Policy subversion: "bypass safety", "disable guardrails", "jailbreak"
- Obfuscation: Base64 encoding, unicode control chars, hex payloads

**Risk Rubric:**
- **0-24:** Low (proceed)
- **25-59:** Medium (review required)
- **60-100:** High (block/re-prompt)

## Project Documentation

This project includes comprehensive documentation designed for both human developers and AI coding assistants:

- **`README.md`** (this file) — Project overview and quick start
- **[`PRD.md`](./PRD.md)** — Complete Product Requirements Document with technical specifications
- **[`PLAN.md`](./PLAN.md)** — Implementation roadmap with phase-by-phase progress tracking
- **[`AGENTS.md`](./AGENTS.md)** — Onboarding guide for AI coding assistants (Rust conventions, patterns, collaboration guidelines)

## AI-Assisted Development Insights

### Development Workflow (Hendrik's Approach)

This project demonstrates a **PRD-driven, multi-agent AI coding workflow** optimized for rapid prototyping while maintaining quality:

#### 1. Requirements & Planning Phase

**PRD Development:**
- Initial PRD drafted and refined collaboratively with **GPT-4** (o3-mini model)
- Final review and discussion round with **Claude Code** for technical feasibility
- Result: [`PRD.md`](./PRD.md) serves as single source of truth for all AI agents

**Best Practices Research:**
- Used **Perplexity** to research Rust development best practices (as a Rust newbie needing a headstart)
- Created [`AGENTS.md`](./AGENTS.md) as onboarding document based on research findings
- This document enables any AI agent to understand project conventions quickly

#### 2. Development Environment Setup

**IDE Configuration:**
- **Primary IDE:** Cursor (with Codex CLI integration)
- **Secondary terminal:** Claude Code running in separate terminal within Cursor
- **Use case split:** Cursor for code review and monitoring while agents work

**MCP Server Integration:**
- **RepoPrompt MCP:** Heavily used by Codex CLI for repository-aware code generation
- **Context7 MCP:** Configured but minimal usage observed by agents
- **Observation:** Different agents leverage different context sources based on their architectures

#### 3. Coding & Implementation

**Primary Agent: GPT-4 Codex (via Codex CLI)**
- Main workhorse for feature implementation
- Leverages RepoPrompt MCP for repository context
- Handles majority of Rust code generation

**Secondary Agent: Claude Code (Claude 3.5 Sonnet)**
- Used for complex architectural decisions
- Documentation refinement and technical reviews
- Handles nuanced discussions about design trade-offs

**Workflow Pattern:**
```
PRD task → Codex CLI implementation → Claude Code review → Human review → Commit
```

#### 4. Review & Version Control

**Code Review Process:**
- Review AI-generated code and documentation in **Tower** (Git client)
- Separate review context from coding environment
- Enables focused evaluation of changes before committing

**Quality Gates:**
- All AI output reviewed before commit
- Tests run locally before pushing
- Documentation verified for accuracy

### What Worked Well

- **PRD-first approach:** Clear requirements enabled consistent AI contributions across multiple agents
- **Multi-agent collaboration:** GPT-4 Codex for implementation, Claude Code for architecture discussions
- **Separated concerns:** Cursor for review, dedicated terminals for coding agents
- **MCP integration:** RepoPrompt provided strong repository context for Codex CLI
- **Rust as learning project:** AI assistants accelerated learning curve for Rust newcomer
- **Documentation as agent onboarding:** `AGENTS.md` successfully guided AI agents on conventions

### Challenges & Learnings

**Context Management:**
- Different agents used different context sources (RepoPrompt vs manual context)
- Context7 MCP underutilized—agents didn't fully leverage this tool
- Large codebases require explicit context window management strategies

**Agent Specialization:**
- Codex CLI excelled at implementation but needed guidance on architecture
- Claude Code better at high-level design discussions and documentation
- Combining agents created better outcomes than using either alone

**Testing & Validation:**
- AI-generated tests needed human review for edge case coverage
- Security-critical code required extra scrutiny (domain expertise matters)
- Property-based test suggestions from AI were hit-or-miss

**Workflow Friction:**
- Context switching between multiple tools (Cursor, terminals, Tower) added overhead
- MCP server configuration not transparent—hard to know when/how agents use them
- Async agent work (waiting for completions) created workflow gaps

### Recommendations for AI-Assisted Projects

**Setup & Tooling:**
1. **Start with clear requirements:** Detailed PRDs (like [`PRD.md`](./PRD.md)) enable better AI contributions
2. **Use structured documentation:** Files like [`AGENTS.md`](./AGENTS.md) help AI assistants onboard quickly
3. **Configure MCP servers:** RepoPrompt and Context7 provide valuable context, but monitor usage
4. **Separate review environment:** Use dedicated tool (like Tower) for code review away from coding environment

**Workflow Optimization:**
5. **Multi-agent approach:** Use different AI models for their strengths (implementation vs architecture)
6. **Iterate incrementally:** Small, testable changes work better than large refactors
7. **Review critically:** Treat AI output as thoughtful first drafts, not final code
8. **Maintain human agency:** Keep humans in the loop for architecture and security decisions

**For Rust Newcomers:**
9. **Leverage AI for learning:** AI agents can teach language idioms while implementing features
10. **Research best practices first:** Use Perplexity/search to understand ecosystem before coding
11. **Create onboarding docs:** Document conventions so AI agents stay consistent across sessions

## Contributing

This is a hackathon project exploring AI-assisted development workflows. Contributions that further this experiment are welcome:

- **Code contributions:** Follow conventions in [`AGENTS.md`](./AGENTS.md)
- **Documentation:** Help document AI collaboration patterns
- **Detection rules:** Contribute new prompt injection patterns
- **Testing:** Add test cases for edge scenarios

## License & Disclaimer

**License:** MIT — see `LICENSE`

**Security Disclaimer:** This tool uses heuristic rules and optional LLM analysis for threat detection. It **does not guarantee** prevention of all prompt-injection attempts. Treat risk scores as decision support, not absolute truth. Use as part of a defense-in-depth security strategy.

**AI Development Notice:** This codebase was developed with significant AI assistance. While efforts have been made to ensure quality, users should conduct their own security reviews before production use.

---

**About the AI Coding Accelerator:** This hackathon is part of [Maven's AI Coding Accelerator](https://maven.com/nila/ai-coding-accelerator) course, taught by Vignesh Mohankumar and Jason Liu. The course explores practical applications of AI coding tools in modern software development.
