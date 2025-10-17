# LLM-Guard — Prompt Injection Firewall

[![Build Status](https://img.shields.io/github/actions/workflow/status/HendrikReh/llm-guard/ci.yml?branch=main)](https://github.com/HendrikReh/llm-guard/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](./LICENSE)
[![Rust Version](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://www.rust-lang.org)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](https://github.com/HendrikReh/llm-guard/pulls)
[![AI Coding Hackathon](https://img.shields.io/badge/AI%20Coding-Hackathon-purple)](https://maven.com/nila/ai-coding-accelerator)

**LLM Provider Support:**
[![OpenAI](https://img.shields.io/badge/OpenAI-412991?logo=openai&logoColor=white)](https://openai.com)
[![Azure OpenAI](https://img.shields.io/badge/Azure_OpenAI-0078D4?logo=microsoft-azure&logoColor=white)](https://azure.microsoft.com/en-us/products/ai-services/openai-service)
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

## Hackathon Context

This project was developed during the **[AI Coding Accelerator](https://maven.com/nila/ai-coding-accelerator)** hackathon (Maven) as an experiment in **AI-assisted software development**.

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
- ✅ **Phase 6:** LLM integration implemented (OpenAI, Anthropic, Gemini support)
- 🚧 **Phase 7:** Quality engineering in progress (comprehensive testing, CI/CD)
- ⏳ **Phase 8:** Documentation and release preparation (planned)

## Features

### Core Capabilities

- **Explainable Security:** Transparent risk scoring with detailed rule attribution and text excerpts
- **Fast & Lightweight:** Efficient pattern matching using Aho-Corasick and compiled regex
- **Multiple Input Sources:** Scan files, stdin, or streaming logs with tail mode
- **Flexible Output Formats:** Human-readable CLI output or JSON for automation
- **LLM-Enhanced Analysis:** Optional verdicts from OpenAI, Anthropic, or Google Gemini

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
| `LLM_GUARD_TIMEOUT_SECS` | HTTP timeout in seconds | `30` |
| `LLM_GUARD_MAX_RETRIES` | Retry count for failed calls | `2` |
| `LLM_GUARD_API_VERSION` | API version (Azure OpenAI) | Provider default |

**Loading from `.env` file:**

```bash
# Export all variables from .env file
set -a && source .env && set +a
./target/release/llm-guard scan --file samples/chat.txt --with-llm
```

**Using a configuration file:**

```toml
# llm-config.toml
[llm]
provider = "anthropic"
model = "claude-3-haiku-20240307"
endpoint = "https://api.anthropic.com"
timeout_secs = 45
max_retries = 3
```

```bash
./target/release/llm-guard --config llm-config.toml scan --with-llm --file prompt.txt
```

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

This project includes comprehensive documentation designed for both human developers and AI coding assistants:

- **`README.md`** (this file) — Project overview and quick start
- **[`PRD.md`](./PRD.md)** — Complete Product Requirements Document with technical specifications
- **[`PLAN.md`](./PLAN.md)** — Implementation roadmap with phase-by-phase progress tracking
- **[`AGENTS.md`](./AGENTS.md)** — Onboarding guide for AI coding assistants (Rust conventions, patterns, collaboration guidelines)
- **[`docs/ADR/`](./docs/ADR/)** — Architecture Decision Records documenting key design choices

## AI-Assisted Development Insights

### Development Workflow (Hendrik's Approach)

This project demonstrates a **PRD-driven, multi-agent AI coding workflow** optimized for rapid prototyping while maintaining quality:

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
- **Rust Learning Accelerator:** AI assistants dramatically shortened learning curve for Rust newcomer (zero to functional CLI in days)
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

**Before You Start:**
1. **Write a detailed PRD first:** Invest time upfront in requirements—it's your AI agents' north star
2. **Research domain best practices:** Use Perplexity/search to understand conventions before coding (especially for new languages)
3. **Create agent onboarding docs:** Document conventions early (like [`AGENTS.md`](./AGENTS.md)) so agents stay consistent

**Tool Configuration:**
4. **Set up MCP servers:** RepoPrompt provides excellent repository context—worth the setup effort
5. **Use multiple agents strategically:** Primary for implementation (GPT-5 Codex) + secondary for review (Claude Code)
6. **Separate review environment:** Dedicated git client (Tower) helps critical evaluation away from coding context
7. **Configure IDE thoughtfully:** Separate terminals for different agents reduces confusion

**Development Process:**
8. **Iterate incrementally:** Small, testable changes work better than large refactors with AI assistance
9. **Review everything critically:** Treat AI output as thoughtful first drafts, not production-ready code
10. **Test AI-generated code:** Happy path tests are often good; edge cases need human attention
11. **Keep humans in architectural loop:** AI great for implementation; humans essential for design decisions

**For Language Newcomers:**
12. **Lean into AI for learning:** Went from Rust zero to functional CLI in days—AI accelerates language learning
13. **Ask "why" questions:** Don't just accept code; understand the idioms and patterns being used
14. **Build real projects:** Learning by doing with AI assistance beats tutorial hell

**Productivity Hacks:**
15. **Fill agent wait time:** Use IDE (Cursor) for code review while waiting for agent completions
16. **Document what works:** Track which agents excel at which tasks for future reference
17. **Expect tool friction:** Multiple tools = cognitive overhead but also specialized capabilities

## Contributing

This is a hackathon project exploring AI-assisted development workflows. Contributions that further this experiment are welcome!

### How to Contribute

**Code Contributions:**
- Follow Rust conventions documented in [`AGENTS.md`](./AGENTS.md)
- Include tests for new features
- Run `cargo fmt` and `cargo clippy` before submitting
- Document your AI-assisted workflow if applicable

**Detection Rules:**
- Add new prompt injection patterns to `rules/keywords.txt` or `rules/patterns.json`
- Include test cases demonstrating the pattern
- Document the attack vector and real-world examples

**Documentation:**
- Help document AI collaboration patterns and workflows
- Share insights from your own AI-assisted development experience
- Improve existing documentation for clarity

**Testing:**
- Add test cases for edge scenarios and corner cases
- Contribute property-based tests for core logic
- Help expand the test fixture corpus

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
- **Instructors:** Vignesh Mohankumar and Jason Liu
- **Focus:** Practical applications of AI coding tools in modern software development

**Tools & Technologies**
- **AI Agents:** GPT-5 Codex (via Codex CLI), Claude Code (Anthropic)
- **MCP Servers:** RepoPrompt, Context7
- **IDE:** Cursor
- **Research:** Perplexity
- **Git Client:** Tower

**Community**
- Thanks to the Rust community for excellent documentation and tooling
- OWASP LLM Top 10 project for threat taxonomy
- All contributors and testers who help improve this tool
