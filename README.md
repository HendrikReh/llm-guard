# LLM-Guard — Prompt Injection Firewall

[![Build Status](https://img.shields.io/github/actions/workflow/status/HendrikReh/llm-guard/ci.yml?branch=main)](https://github.com/HendrikReh/llm-guard/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](./LICENSE)
[![Rust Version](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://www.rust-lang.org)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](https://github.com/HendrikReh/llm-guard/pulls)
[![AI Coding Hackathon](https://img.shields.io/badge/AI%20Coding-Hackathon-purple)](https://maven.com/nila/ai-coding-accelerator)

**LLM Provider Support:**
[![OpenAI](https://img.shields.io/badge/OpenAI-412991?logo=openai&logoColor=white)](https://openai.com)
[![Anthropic](https://img.shields.io/badge/Anthropic-191919?logo=anthropic&logoColor=white)](https://anthropic.com)
[![Google Gemini](https://img.shields.io/badge/Google_Gemini-4285F4?logo=google&logoColor=white)](https://ai.google.dev)

**Built With AI Tools:**
[![Cursor](https://img.shields.io/badge/Cursor-000000?logo=visual-studio-code&logoColor=white)](https://cursor.sh)
[![Claude Code](https://img.shields.io/badge/Claude_Code-191919?logo=anthropic&logoColor=white)](https://claude.ai)
[![Codex CLI](https://img.shields.io/badge/Codex_CLI-412991?logo=openai&logoColor=white)](https://github.com/openai/codex-cli)
[![RepoPrompt MCP](https://img.shields.io/badge/RepoPrompt-MCP-orange)](https://repoprompt.com/)
[![Context7 MCP](https://img.shields.io/badge/Context7-MCP-orange)](https://context7.com/)

> **AI Coding Hackathon Project** | Experimenting with AI-assisted development workflows

A fast, explainable **Rust** CLI that scans prompts and logs for **prompt-injection & jailbreak indicators**, scores the risk (0–100), and optionally asks an LLM for a short verdict and mitigation tip.

## Table of Contents

- [Hackathon Context](#hackathon-context)
  - [Goals of This Hackathon](#goals-of-this-hackathon)
  - [What Makes This Project Special](#what-makes-this-project-special)
- [Project Status](#project-status)
- [Features](#features)
  - [Core Capabilities](#core-capabilities)
  - [Detection Coverage](#detection-coverage)
- [Quick Start](#quick-start)
  - [Installation](#installation)
  - [Usage Examples](#usage-examples)
  - [Configuration](#configuration)
- [Technical Overview](#technical-overview)
  - [How It Works](#how-it-works)
  - [Architecture](#architecture)
  - [Detection Strategy](#detection-strategy)
- [Project Documentation](#project-documentation)
- [AI-Assisted Development Insights](#ai-assisted-development-insights)
  - [Development Workflow](#development-workflow)
    - [1. Requirements & Planning Phase](#1-requirements--planning-phase)
    - [2. Development Environment Setup](#2-development-environment-setup)
    - [3. Coding & Implementation](#3-coding--implementation)
    - [4. Review & Version Control](#4-review--version-control)
  - [What Worked Well](#what-worked-well)
  - [Challenges & Learnings](#challenges--learnings)
- [Recommendations for AI-Assisted Projects](#recommendations-for-ai-assisted-projects)
- [Provider Integration Pitfalls & Fixes](#provider-integration-pitfalls--fixes)
- [Contributing](#contributing)
  - [How to Contribute](#how-to-contribute)
- [License & Disclaimer](#license--disclaimer)
  - [License](#license)
  - [Security Disclaimer](#security-disclaimer)
  - [AI Development Notice](#ai-development-notice)
- [Acknowledgments](#acknowledgments)

## Hackathon Context

This project was developed during the **[AI Coding Accelerator](https://maven.com/nila/ai-coding-accelerator)** hackathon (Maven) as an experiment in **AI-assisted software development**. The entire project was built in a **single day (~7 hours)** using AI coding assistants.

**Instructors:** [Vignesh Mohankumar](https://x.com/vig_xyz) and [Jason Liu](https://x.com/jxnlco)

### Goals of This Hackathon

1. **Experiment with AI Coding Tools:** Explore capabilities and limitations of AI coding assistants in real-world development
2. **Test AI-Driven Development Processes:** Evaluate workflows where AI assistants contribute to architecture, implementation, and documentation
3. **Build Working Software Fast:** Create functional prototypes that demonstrate practical security tools for LLM applications
4. **Document the Journey:** Capture insights about human-AI collaboration in software engineering

### What Makes This Project Special

- **Multi-Agent Collaboration:** Core features developed using GPT-5 Codex (primary) and Claude Code (reviews/docs)
- **Living Documentation:** [`AGENTS.md`](./AGENTS.md) serves as onboarding guide enabling any AI assistant to join the project
- **Transparent Development:** [`PLAN.md`](./PLAN.md) tracks implementation progress and decisions in real-time
- **PRD-Driven Workflow:** Product requirements ([`PRD.md`](./PRD.md)) evolved through collaborative refinement with multiple AI models
- **MCP Integration:** RepoPrompt and Context7 MCP servers provide repository context to coding agents

> **Note:** This codebase demonstrates both the potential and practical considerations of AI-assisted development, including code quality, testing approaches, and documentation practices.

## Project Status

**Current Phase:** Active Development (See [`PLAN.md`](./PLAN.md) for detailed roadmap)

**Implementation Status:**
- ✅ **Phase 0-5:** Core functionality complete (CLI, scanning, scoring, reporting)
- ✅ **Phase 6:** LLM integration implemented (OpenAI, Anthropic, Gemini, Azure OpenAI support)
- ✅ **Phase 9:** Migration to `rig.rs` complete (unified multi-provider orchestration)
- 🚧 **Phase 7:** Quality engineering in progress (comprehensive testing, CI/CD)
- 📝 **Phase 8:** Documentation and release preparation (ongoing)

**Test Coverage:** 40 tests (30 active, 10 ignored) | See [`docs/TESTING_GUIDE.md`](./docs/TESTING_GUIDE.md)

## Features

### Core Capabilities

- **Explainable Security:** Transparent risk scoring (0-100) with detailed rule attribution and text excerpts
- **Fast & Lightweight:** Efficient pattern matching using Aho-Corasick and compiled regex (<100ms for typical prompts)
- **Multiple Input Sources:** Scan files, stdin, or streaming logs with tail mode
- **Flexible Output Formats:** Human-readable CLI output or JSON for automation and CI/CD integration
- **Multi-Provider LLM Analysis:** Optional verdicts from OpenAI, Anthropic, Google Gemini, or Azure OpenAI via `rig.rs`
- **Provider Health Checks:** Built-in diagnostics to validate LLM provider connectivity and configuration

### Detection Coverage

- **Instruction Override:** Detects attempts to manipulate system prompts
- **Data Exfiltration:** Identifies prompt leak and secret revelation attempts
- **Policy Subversion:** Catches jailbreak and safety bypass patterns
- **Obfuscation Techniques:** Recognizes encoded payloads and Unicode tricks

## Quick Start

### Installation

```bash
git clone https://github.com/HendrikReh/llm-guard
cd llm-guard
cargo build --release
```

### Usage Examples

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

**LLM-Enhanced Analysis:**

```bash
# Request LLM verdict using OpenAI (default)
export LLM_GUARD_PROVIDER=openai
export LLM_GUARD_API_KEY=your_key_here
export LLM_GUARD_MODEL=gpt-4o-mini
./target/release/llm-guard scan --file samples/chat.txt --with-llm

# Switch to Anthropic via CLI overrides
./target/release/llm-guard scan --file samples/chat.txt --with-llm \
  --provider anthropic --model claude-3-haiku-20240307

# Or use Google Gemini
./target/release/llm-guard scan --file samples/chat.txt --with-llm \
  --provider gemini --model gemini-1.5-flash

# Azure OpenAI
export LLM_GUARD_PROVIDER=azure
export LLM_GUARD_API_KEY=your_azure_key
export LLM_GUARD_ENDPOINT=https://your-resource.openai.azure.com
export LLM_GUARD_MODEL=your-deployment
export LLM_GUARD_API_VERSION=2024-02-15-preview
./target/release/llm-guard scan --file samples/chat.txt --with-llm

# Dry-run mode (no external API calls)
./target/release/llm-guard scan --file samples/chat.txt --with-llm \
  --provider noop

# Debug mode (dump raw provider verdict payloads on parse errors)
./target/release/llm-guard scan --file samples/chat.txt --with-llm \
  --provider anthropic --debug

# Health check against configured providers (uses llm_providers.yaml if present)
./target/release/llm-guard health --providers-config llm_providers.yaml
```

**Streaming Mode:**

```bash
# Tail a log file and scan new content as it arrives
./target/release/llm-guard scan --file logs/chat.log --tail

# Combine tail with LLM analysis
./target/release/llm-guard scan --file logs/chat.log --tail --with-llm
```

### Configuration

**LLM Environment Variables:**

| Variable | Description | Default |
|----------|-------------|---------|
| `LLM_GUARD_PROVIDER` | Provider identifier (`openai`, `anthropic`, `gemini`, `noop`) | `openai` |
| `LLM_GUARD_API_KEY` | API key/token (required for real providers) | - |
| `LLM_GUARD_ENDPOINT` | Custom endpoint/base URL | Provider default |
| `LLM_GUARD_MODEL` | Model name (e.g., `gpt-4o-mini`) | Provider default |
| `LLM_GUARD_DEPLOYMENT` | Deployment identifier for rig-managed providers (e.g., Azure) | - |
| `LLM_GUARD_PROJECT` | Provider project or tenant (Gemini, Anthropic) | - |
| `LLM_GUARD_WORKSPACE` | Provider workspace identifier (if required) | - |
| `LLM_GUARD_TIMEOUT_SECS` | HTTP timeout in seconds | `30` |
| `LLM_GUARD_MAX_RETRIES` | Retry count for failed calls | `2` |
| `LLM_GUARD_API_VERSION` | API version (Azure OpenAI) | Provider default |

**CLI overrides:** Use `--provider`, `--model`, `--endpoint`, `--deployment`, `--project`, and `--workspace` to override these values for a single run without touching environment variables.

**Provider Profiles (`llm_providers.yaml`):**

The CLI supports an optional `llm_providers.yaml` configuration file (override with `--providers-config`) for managing multiple provider credentials and defaults side-by-side. Example:

```yaml
providers:
  - name: "openai"
    api_key: "OPENAI_API_KEY"
    model: "gpt-4o-mini"
  - name: "azure"
    api_key: "AZURE_OPENAI_KEY"
    endpoint: "https://your-resource.openai.azure.com"
    deployment: "gpt-4o-production"
    api_version: "2024-02-15-preview"
    timeout_secs: 60
    max_retries: 3
```

**Configuration Precedence:** CLI flags → Environment variables → Provider profile

**Quick Start:** Copy `llm_providers.example.yaml` to `llm_providers.yaml` and update with your API keys.

**Loading from `.env` file:**

```bash
# Export all variables from .env file
set -a && source .env && set +a
./target/release/llm-guard scan --file samples/chat.txt --with-llm
```

**Alternative: TOML Configuration File:**

```toml
# llm-config.toml
[llm]
provider = "anthropic"
model = "claude-3-haiku-20240307"
timeout_secs = 45
max_retries = 3
```

```bash
./target/release/llm-guard --config llm-config.toml scan --with-llm --file prompt.txt
```

> **Recommendation:** Use `llm_providers.yaml` for multi-provider setups or TOML for single-provider configurations.

## Technical Overview

### How It Works

1. **Rule Loading:** Loads keyword and regex-based detection rules using Aho-Corasick and `regex` crate
2. **Scanning:** Processes input from stdin, files, or streams to identify potential threats
3. **Finding Generation:** Emits detailed findings with rule matches, text spans, excerpts, and weights
4. **Risk Scoring:** Computes a normalized risk score (0–100) with length normalization and synergy bonuses
5. **LLM Analysis (Optional):** Queries an LLM for classification, rationale, and mitigation suggestions

### Architecture

```
Workspace Structure:
├─ llm-guard-core (library)
│  ├─ scanner (rule loading, pattern matching)
│  ├─ scoring (heuristics, risk calculation)
│  └─ types (Rule, Finding, ScanReport)
└─ llm-guard-cli (binary)
   ├─ cli (clap argument parsing)
   ├─ input (stdin, file, tail readers)
   ├─ output (human/JSON formatters)
   └─ llm (provider clients via rig.rs)

Detection Pipeline:
stdin/file → scanner → findings → scoring → optional LLM → report
```

**Core Dependencies:** `clap`, `regex`, `aho-corasick`, `serde`, `serde_json`, `anyhow`, `once_cell`, `humantime`, `tokio`, `tracing`, `reqwest`, `rig-core`

**Key Architectural Decisions:**
- [ADR-0001](./docs/ADR/0001-heuristic-risk-scoring.md): Heuristic-based scoring for transparency
- [ADR-0002](./docs/ADR/0002-workspace-architecture.md): Core/CLI separation for reusability
- [ADR-0003](./docs/ADR/0003-optional-llm-integration.md): Multi-provider LLM support
- [ADR-0004](./docs/ADR/0004-aho-corasick-regex-detection.md): Dual pattern-matching engines

See [`docs/ADR/`](./docs/ADR/) for complete architecture decision records.

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

Comprehensive documentation designed for both human developers and AI coding assistants:

| Document | Purpose | Audience |
|----------|---------|----------|
| **[`README.md`](./README.md)** | Project overview, quick start, and AI workflow insights | Everyone |
| **[`docs/USAGE.md`](./docs/USAGE.md)** | Complete CLI reference with all commands and flags | Users, operators |
| **[`PRD.md`](./PRD.md)** | Complete Product Requirements Document | Developers, AI agents |
| **[`PLAN.md`](./PLAN.md)** | Implementation roadmap with phase-by-phase tracking | Project contributors |
| **[`AGENTS.md`](./AGENTS.md)** | Onboarding guide for AI coding assistants | AI agents, developers |
| **[`docs/TESTING_GUIDE.md`](./docs/TESTING_GUIDE.md)** | Testing strategy, commands, and troubleshooting | Developers, QA |
| **[`docs/ADR/`](./docs/ADR/)** | Architecture Decision Records | Technical stakeholders |

## AI-Assisted Development Insights

### Development Workflow

This project demonstrates a **PRD-driven, multi-agent AI coding workflow** that achieved a functional Rust CLI with comprehensive features in ~7 hours:

#### 1. Requirements & Planning Phase

**PRD Development:**
- Initial PRD drafted and refined collaboratively with **GPT-5**
- Final review and discussion round with **Claude Code** for technical feasibility
- Result: [`PRD.md`](./PRD.md) serves as single source of truth for all AI agents

**Best Practices Research:**
- Used **Perplexity** to research Rust development best practices (as a Rust newbie needing a headstart)
- Created [`AGENTS.md`](./AGENTS.md) as onboarding document based on research findings
- This document enables any AI agent to understand project conventions quickly

#### 2. Development Environment Setup

**IDE Configuration:**
- **IDE:** Cursor
- **Terminal #1:** Codex CLI running in separate terminal within Cursor
- **Terminal #2:** Claude Code running in separate terminal within Cursor
- **Use case split:** Cursor for code review and monitoring while agents work

**MCP Server Integration:**
- **RepoPrompt MCP:** Heavily used by Codex CLI for repository-aware code generation
- **Context7 MCP:** Configured but minimal usage observed by agents
- **Observation:** Different agents leverage different context sources based on their architectures

**RepoPrompt Usage:**

RepoPrompt is a macOS MCP server that converts selected files from local repositories into structured prompts for AI assistants, enabling repository-aware code generation and large-scale refactoring.

**Key Features:**
- **Selective Context:** Choose specific files/directories to include in prompts
- **Structured Prompts:** Converts repository code into optimized format for AI consumption
- **Reviewable Diffs:** Apply model-generated edits with diffs before writing to disk
- **Token Estimation:** Monitors context size to stay within model limits
- **Code Maps:** Generates repository structure summaries
- **Multi-Provider:** Works with OpenAI, Anthropic, and other LLM providers

**How It Works:**
1. Select relevant files from your repository (e.g., `crates/llm-guard-core/src/scanner/`)
2. RepoPrompt packages them into a structured prompt with file tree and content
3. AI assistant receives full repository context for informed code generation
4. Review proposed changes as diffs before accepting

**Benefits in This Project:**
- **Repository Awareness:** Codex CLI leveraged RepoPrompt to understand workspace structure
- **Cross-File Refactoring:** Made large-scale changes (e.g., rig.rs migration) with context
- **Consistent Patterns:** AI generated code matching existing conventions by seeing similar files
- **Reduced Context Management:** Automatic selection of relevant files reduced manual copying

**Observation:**
During this hackathon, GPT-5 Codex via Codex CLI **heavily used RepoPrompt** for repository-wide context. This was a key factor in the rapid development speed—AI assistants had comprehensive awareness of project structure, naming conventions, and implementation patterns without explicit prompting.

**Context7 Usage:**

Context7 is an MCP server that injects up-to-date, version-specific documentation for referenced libraries directly into your prompt context. When working with AI assistants, append **`use context7`** to your prompts to trigger automatic documentation fetching:

```
# Example prompts that leverage Context7
Show me how to use tokio runtime in Rust. use context7

Create a clap CLI with subcommands and environment variables. use context7

Implement retry logic with reqwest. use context7 for tokio, reqwest

Debug serde_json deserialization errors. use context7 for serde, serde_json
```

**Benefits:**
- Access current, version-specific documentation without manual lookups
- Reduce hallucinations and outdated code suggestions
- Especially useful for rapidly evolving crates like `tokio`, `reqwest`, `rig-core`

**Configuration:**
- Works with Claude Desktop, Cursor, Windsurf, and other MCP clients
- Configured in MCP client settings (see [Context7 docs](https://context7.com/))
- Can be auto-triggered for relevant prompts using client rules

**When to Use:**
- Working with unfamiliar Rust crates or libraries
- Debugging dependencies or upgrading versions
- Needing reliable, current API examples during implementation

#### 3. Coding & Implementation

**Primary Agent: GPT-5 Codex (via Codex CLI)**
- Main workhorse for feature implementation
- Leverages RepoPrompt MCP for repository context
- Handles majority of Rust code generation
- **Observation:** Demonstrated exceptionally strong Rust capabilities—possibly because Codex CLI itself is implemented in Rust, creating a feedback loop where the tool's implementation informs the model's training data

**Secondary Agent: Claude Code (Claude 4.5 Sonnet)**
- Used for additional code reviews (second opinion)
- Handles nuanced discussions about design trade-offs
- Documentation creation and refinement

**Typical Workflow Pattern:**
```
1. PRD task selection
2. Codex CLI implementation (with RepoPrompt context)
3. Claude Code review (second opinion on design/quality)
4. Human review in Tower (Git client)
5. Commit and iterate
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

- **PRD-First Approach:** Having [`PRD.md`](./PRD.md) as single source of truth enabled consistent AI contributions across sessions
- **Multi-Agent Specialization:** GPT-5 Codex for implementation + Claude Code for reviews = better outcomes than single agent
- **Separated Tool Contexts:** Cursor (review) + separate terminals (coding) + Tower (git) created clear mental boundaries
- **MCP Context Servers:** RepoPrompt provided excellent repository-wide context for Codex CLI
- **Rust Learning Accelerator:** AI assistants dramatically shortened learning curve for Rust newcomer (zero to functional CLI in ~7 hours)
- **Living Documentation:** [`AGENTS.md`](./AGENTS.md) successfully onboarded AI agents with consistent conventions across sessions
- **Perplexity for Research:** Quick ramp-up on Rust best practices through targeted research queries

### Challenges & Learnings

**Context Management:**
- **Agent differences:** Codex CLI heavily used RepoPrompt MCP; Claude Code relied more on explicit context
- **Context7 underutilization:** Configured but agents didn't leverage it—unclear why
- **Manual context still needed:** Large codebases require explicit context window curation despite MCP servers

**Agent Specialization & Trade-offs:**
- **Codex CLI strengths:** Fast implementation, good Rust idioms, leveraged MCP context effectively
- **Codex CLI weaknesses:** Sometimes missed architectural nuances, needed guidance on design decisions
- **Claude Code strengths:** Better at design discussions, documentation quality, catching architectural issues
- **Claude Code weaknesses:** Less aggressive with RepoPrompt usage, slower for pure implementation
- **Key insight:** Different agents for different phases worked better than single agent for everything

**Testing & Validation:**
- **AI-generated tests:** Covered happy paths well but needed human review for edge cases
- **Security code:** Required extra scrutiny—AI suggestions good starting point, not sufficient alone
- **Property-based tests:** Hit-or-miss quality, often needed significant human refinement

**Workflow & Tool Friction:**
- **Context switching overhead:** Moving between Cursor, terminals, and Tower added cognitive load
- **MCP transparency gap:** Hard to know when/how agents used context servers—debug experience poor
- **Async agent delays:** Waiting for completions created workflow gaps—used Cursor for code review to fill time
- **Tool proliferation:** Multiple terminals, IDE, git client = powerful but complex setup

### Recommendations for AI-Assisted Projects

Based on this ~7-hour hackathon experience building a production-ready Rust CLI:

#### Before You Start

1. **Write a detailed PRD first** — Invest time upfront in requirements; it becomes your AI agents' north star
2. **Research domain best practices** — Use Perplexity/search to understand conventions before coding (critical for new languages)
3. **Create agent onboarding docs** — Document conventions early (like [`AGENTS.md`](./AGENTS.md)) to maintain consistency

#### Tool Configuration

4. **Set up MCP servers** — RepoPrompt provides excellent repository context; worth the 30-minute setup
5. **Use multiple agents strategically** — Primary for implementation (GPT-5 Codex) + secondary for review (Claude Code)
6. **Separate review environment** — Dedicated git client (Tower) enables critical evaluation away from coding context
7. **Configure IDE thoughtfully** — Separate terminals for different agents reduces confusion and context bleeding

#### Development Process

8. **Iterate incrementally** — Small, testable changes work better than large refactors with AI assistance
9. **Review everything critically** — Treat AI output as intelligent first drafts, not production-ready code
10. **Test AI-generated code thoroughly** — Happy path tests often good; edge cases require human attention
11. **Keep humans in architectural loop** — AI excels at implementation; humans essential for design decisions

#### For Language Newcomers

12. **Lean into AI for accelerated learning** — Went from Rust zero to functional CLI in ~7 hours
13. **Ask "why" questions consistently** — Don't just accept code; understand the idioms and patterns
14. **Build real projects** — Learning by doing with AI assistance beats tutorial-based learning

#### Productivity Hacks

15. **Fill agent wait time** — Use IDE (Cursor) for code review while waiting for agent completions
16. **Document what works** — Track which agents excel at which tasks for future workflows
17. **Expect tool friction** — Multiple tools = cognitive overhead but also specialized capabilities and redundancy

## Provider Integration Pitfalls & Fixes

While wiring rig.rs into real LLM providers we hit a few repeat offenders. The highlights:

- **Anthropic truncation & malformed JSON** — Responses frequently dropped closing quotes/braces and embedded raw newlines inside strings. We added newline sanitisation, automatic quote/brace repair, a JSON5 fallback, and eventually a fallback verdict so scans never abort.
- **OpenAI reasoning-only replies** — GPT‑5 reasoning models returned only reasoning traces without textual content when using `json_schema` response format. We now capture tool-call arguments and use simpler `json_object` response format (instead of strict `json_schema`) for better compatibility with reasoning models. Falls back to an "unknown" verdict when the model withholds textual content.
- **Gemini rig.rs incompatibility** — Rig's Gemini implementation has deserialization issues with the current Gemini API (missing `generationConfig` field errors). The Gemini API also rejects requests combining forced function calling (ANY mode) with `responseMimeType: 'application/json'`. Solution: Bypassed rig entirely for Gemini; implemented standalone HTTP client using Gemini's native REST API with prompt-based JSON formatting.
- **Gemini empty responses** — Successful calls can still return empty candidates. Health checks now treat empty responses as warnings instead of hard failures, surfacing an "unknown" verdict with guidance.
- **Debugging provider quirks** — The global `--debug` flag flips `LLM_GUARD_DEBUG=1`, causing the adapter to log the raw upstream payload whenever parsing fails, making it obvious when prompt/schema updates are needed.

These guardrails keep the CLI resilient even when upstream providers change response contracts mid-flight.

## Contributing

This is a hackathon project exploring AI-assisted development workflows. Contributions that further this experiment are welcome!

### How to Contribute

**Code Contributions:**
- Follow Rust conventions in [`AGENTS.md`](./AGENTS.md)
- Include tests for new features (see [`docs/TESTING_GUIDE.md`](./docs/TESTING_GUIDE.md))
- Run `cargo fmt && cargo clippy` before submitting
- Document your AI-assisted workflow if applicable

**Detection Rules:**
- Add new patterns to `rules/keywords.txt` or `rules/patterns.json`
- Include test cases with expected risk scores
- Document attack vectors with real-world examples

**Documentation:**
- Document AI collaboration patterns and workflows
- Share insights from AI-assisted development experiences
- Improve clarity and add practical examples

**Testing:**
- Add edge cases and corner case tests
- Expand test fixture corpus with real-world samples
- Contribute property-based tests for core logic

## License & Disclaimer

### License

MIT License — see [`LICENSE`](./LICENSE) file for details.

### Security Disclaimer

⚠️ **Important:** This tool uses heuristic rules and optional LLM analysis for threat detection. It **does not guarantee** prevention of all prompt-injection attempts.

**Key Limitations:**
- Heuristic-based detection can produce false positives and false negatives
- Novel attack patterns not in rule set will be missed
- LLM verdicts subject to model limitations and biases
- Not a substitute for proper input validation and security architecture

**Recommended Usage:**
- Treat risk scores as **decision support**, not absolute truth
- Use as **one layer** in a defense-in-depth security strategy
- Combine with other security measures (input sanitization, rate limiting, monitoring)
- Regularly update detection rules based on emerging threats

### AI Development Notice

🤖 **This codebase was developed with significant AI assistance** as part of an experimental workflow exploring AI-driven development.

**What this means:**
- Most code generated by GPT-5 Codex via Codex CLI
- Documentation and architectural decisions involved Claude Code
- All AI output reviewed and validated by human developer
- Test coverage and security-critical paths received extra scrutiny

**Before Production Use:**
- Conduct your own security review and testing
- Validate detection rules against your specific threat model
- Monitor false positive/negative rates in your environment
- Consider customizing rules and thresholds for your use case

---

## Acknowledgments

**AI Coding Accelerator Hackathon**
- **Course:** [Maven's AI Coding Accelerator](https://maven.com/nila/ai-coding-accelerator)
- **Instructors:** [Vignesh Mohankumar](https://x.com/vig_xyz) and [Jason Liu](https://x.com/jxnlco)
- **Focus:** Practical applications of AI coding tools in modern software development

**Tools & Technologies**
- **AI Agents:**  [Codex CLI (OpenAI)](https://github.com/openai/codex-cli), [Claude Code (Anthropic)](https://claude.ai)
- **MCP Servers:** [RepoPrompt](https://repoprompt.com/), [Context7](https://context7.com/)
- **IDE:** [Cursor](https://cursor.sh)
- **Research:** [Perplexity](https://www.perplexity.ai/)
- **Git Client:** [Tower](https://www.git-tower.com/mac)
