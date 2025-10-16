# LLM-Guard — Prompt Injection Firewall

A fast, explainable **Rust** CLI that scans prompts and logs for **prompt-injection & jailbreak indicators**, scores the risk (0–100), and optionally asks an LLM for a short verdict and mitigation tip.

> **Note:** Any descriptions of LLM behavior are based on observed patterns and may vary.

## Status

LLM-Guard is under active development. See `PLAN.md` for the detailed implementation roadmap and progress tracking.

## Features

- **Explainable Security:** Transparent scoring with rule hits, weights, and text excerpts
- **Fast & Lightweight:** Built in Rust with Aho-Corasick and optimized regex matching
- **CI/CD Friendly:** JSON output for automated pipelines; colorized CLI for interactive use
- **Optional LLM Analysis:** Get additional context and mitigation suggestions from language models
- **Flexible Input:** Scan files, stdin, or streaming logs


## Installation

```bash
git clone https://github.com/yourname/llm-guard
cd llm-guard
cargo build --release
```

## Usage

> The CLI interface is still a stub while Phase 1–5 items in `PLAN.md` are in progress.

### Basic Scanning

```bash
# Scan a file
./target/release/llm-guard scan --file samples/chat.txt

# Scan from stdin
echo "Ignore previous instructions" | ./target/release/llm-guard scan
```

### CI/CD Integration

```bash
# Generate JSON output for automated processing
./target/release/llm-guard scan --file samples/chat.txt --json > report.json
```

### With LLM Analysis

```bash
# Set your API key
export OPENAI_API_KEY=your_key_here

# Get additional LLM-powered analysis
./target/release/llm-guard scan --file samples/chat.txt --with-llm
```

## How It Works

1. **Rule Loading:** Loads keyword and regex-based detection rules using Aho-Corasick and `regex` crate
2. **Scanning:** Processes input from stdin, files, or streams to identify potential threats
3. **Finding Generation:** Emits detailed findings with rule matches, text spans, excerpts, and weights
4. **Risk Scoring:** Computes a normalized risk score (0–100) with length normalization and synergy bonuses
5. **LLM Analysis (Optional):** Queries an LLM for classification, rationale, and mitigation suggestions

## Architecture

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

### Dependencies

**Core:** `clap`, `regex`, `aho-corasick`, `serde`, `serde_json`, `anyhow`, `colored`, `ansi_term`, `once_cell`, `humantime`
**Optional:** `notify`, `comfy-table`, `tokio`

## About

Developed during an **AI Coding Hackathon** as part of the [AI Coding Accelerator (Maven)](https://maven.com/nila/ai-coding-accelerator) course.

**Instructors:** Vignesh Mohankumar and Jason Liu

## License

MIT — see `LICENSE`.

## Disclaimer

This tool uses heuristic rules and optional LLM analysis for threat detection. It **does not guarantee** prevention of all prompt-injection attempts. Treat risk scores as decision support, not absolute truth. Use as part of a defense-in-depth security strategy.
