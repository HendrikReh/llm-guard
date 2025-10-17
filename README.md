# LLM-Guard — Prompt Injection Firewall

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
- **Living Documentation:** `AGENTS.md` serves as onboarding guide for AI assistants joining the project
- **Transparent Development:** `PLAN.md` tracks implementation progress and decisions in real-time
- **Iterative Refinement:** Product requirements (`PRD.md`) evolved through AI-human dialogue

> **Note:** This codebase demonstrates both the potential and practical considerations of AI-assisted development, including code quality, testing approaches, and documentation practices.

## Project Status

**Current Phase:** Active Development (See `PLAN.md` for detailed roadmap)

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
export LLM_GUARD_API_KEY=your_key_here
./target/release/llm-guard scan --file samples/chat.txt --with-llm
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
cli (clap)
  ├─ reader (stdin | file | tail -f)
  ├─ scanner
  │    ├─ rules (regex + aho-corasick)
  │    ├─ heuristics (weights, windows, caps)
  │    └─ explain (feature attributions)
  ├─ llm_adapter (optional)
  └─ reporters (human / json)
```

**Core Dependencies:** `clap`, `regex`, `aho-corasick`, `serde`, `serde_json`, `anyhow`, `once_cell`, `humantime`, `tokio`, `tracing`

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
- **`PRD.md`** — Complete Product Requirements Document with technical specifications
- **`PLAN.md`** — Implementation roadmap with phase-by-phase progress tracking
- **`AGENTS.md`** — Onboarding guide for AI coding assistants (Rust conventions, patterns, collaboration guidelines)

## AI-Assisted Development Insights

### What Worked Well

- **Rapid prototyping:** AI assistants accelerated initial scaffolding and boilerplate
- **Documentation generation:** PRD and technical docs created collaboratively with high quality
- **Pattern application:** AI effectively applied Rust idioms and best practices
- **Iterative refinement:** Fast feedback loops for architecture and design decisions

### Challenges & Learnings

- **Context management:** Large codebases require careful context window management
- **Testing rigor:** AI-generated tests need human review for edge case coverage
- **Architectural consistency:** Human oversight crucial for maintaining coherent system design
- **Domain knowledge:** Security-specific logic benefits from human expertise

### Recommendations for AI-Assisted Projects

1. **Start with clear requirements:** Detailed PRDs (like `PRD.md`) enable better AI contributions
2. **Use structured documentation:** Files like `AGENTS.md` help AI assistants onboard quickly
3. **Iterate incrementally:** Small, testable changes work better than large refactors
4. **Review critically:** Treat AI output as thoughtful first drafts, not final code
5. **Maintain human agency:** Keep humans in the loop for architecture and security decisions

## Contributing

This is a hackathon project exploring AI-assisted development workflows. Contributions that further this experiment are welcome:

- **Code contributions:** Follow conventions in `AGENTS.md`
- **Documentation:** Help document AI collaboration patterns
- **Detection rules:** Contribute new prompt injection patterns
- **Testing:** Add test cases for edge scenarios

## License & Disclaimer

**License:** MIT — see `LICENSE`

**Security Disclaimer:** This tool uses heuristic rules and optional LLM analysis for threat detection. It **does not guarantee** prevention of all prompt-injection attempts. Treat risk scores as decision support, not absolute truth. Use as part of a defense-in-depth security strategy.

**AI Development Notice:** This codebase was developed with significant AI assistance. While efforts have been made to ensure quality, users should conduct their own security reviews before production use.

---

**About the AI Coding Accelerator:** This hackathon is part of [Maven's AI Coding Accelerator](https://maven.com/nila/ai-coding-accelerator) course, taught by Vignesh Mohankumar and Jason Liu. The course explores practical applications of AI coding tools in modern software development.
